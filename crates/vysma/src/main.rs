use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use chrono::Utc;
use clap::{Parser, Subcommand};
use reqwest::blocking::{Client, RequestBuilder};
use reqwest::blocking::multipart;
use serde::Serialize;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(version, about = "Vysma CLI")] 
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
	/// Module-related commands
	Module {
		#[command(subcommand)]
		cmd: ModuleCmd,
	},
}

#[derive(Subcommand, Debug)]
enum ModuleCmd {
	/// Publish or update a module version in Appwrite
	Publish {
		/// Owner username (namespace)
		#[arg(long)]
		owner: String,
		/// Module name
		#[arg(long)]
		name: String,
		/// Version string (e.g. 0.1.0)
		#[arg(long)]
		version: String,
		/// HCL file path
		#[arg(long)]
		hcl: PathBuf,
		/// Optional directory of assets to upload
		#[arg(long)]
		assets: Option<PathBuf>,
		/// Visibility: public|private
		#[arg(long, default_value = "public")]
		visibility: String,
		/// Optional description
		#[arg(long)]
		desc: Option<String>,
		/// If set, update module.latestVersion to this version
		#[arg(long, default_value_t = true)]
		set_latest: bool,
	},
}

#[derive(Clone)]
struct AppwriteCfg {
	endpoint: String,
	project: String,
	key: String,
	database: String,
	modules_col: String,
	versions_col: String,
	assets_bucket_id: String,
	assets_index_col: Option<String>,
}

fn cfg_from_env() -> anyhow::Result<AppwriteCfg> {
	let get = |k: &str| std::env::var(k).with_context(|| format!("missing env {k}"));
	Ok(AppwriteCfg {
		endpoint: get("APPWRITE_ENDPOINT")?,
		project: get("APPWRITE_PROJECT_ID")?,
		key: get("APPWRITE_API_KEY")?,
		database: get("APPWRITE_DATABASE_ID")?,
		modules_col: get("APPWRITE_MODULES_COLLECTION_ID")?,
		versions_col: get("APPWRITE_MODULEVERSIONS_COLLECTION_ID")?,
		assets_bucket_id: std::env::var("APPWRITE_MODULE_ASSETS_BUCKET_ID").unwrap_or_else(|_| "module-assets".to_string()),
		assets_index_col: std::env::var("APPWRITE_MODULE_ASSETS_INDEX_COLLECTION_ID").ok(),
	})
}

fn base_api(e: &str) -> String {
	let e = e.trim_end_matches('/');
	if e.ends_with("/v1") { e.to_string() } else { format!("{}/v1", e) }
}

fn headers(rb: RequestBuilder, cfg: &AppwriteCfg) -> RequestBuilder {
	rb.header("X-Appwrite-Project", &cfg.project)
		.header("X-Appwrite-Key", &cfg.key)
		.header("X-Appwrite-Response-Format", "1.7.0")
		.header("Content-Type", "application/json")
}

#[derive(Serialize)]
struct CreateDoc<T> { documentId: String, data: T }

#[derive(Serialize)]
struct ModuleData {
	ownerUserId: String,
	ownerUsername: String,
	name: String,
	latestVersion: String,
	visibility: String,
	#[serde(skip_serializing_if = "Option::is_none")] description: Option<String>,
}

#[derive(Serialize)]
struct VersionData { moduleId: String, version: String, sha256: String, hcl: String, createdAt: String }

fn sha256_hex(s: &str) -> String {
	let mut hasher = Sha256::new();
	hasher.update(s.as_bytes());
	format!("{:x}", hasher.finalize())
}

fn read_hcl(path: &Path) -> anyhow::Result<String> {
	Ok(fs::read_to_string(path).with_context(|| format!("read hcl file: {}", path.display()))?)
}

fn create_or_update_module(http: &Client, api: &str, cfg: &AppwriteCfg, owner: &str, name: &str, version: &str, visibility: &str, desc: &Option<String>) -> anyhow::Result<String> {
	let modules_url = format!("{}/databases/{}/collections/{}/documents", api, cfg.database, cfg.modules_col);
	let module_id = format!("{}__{}", owner, name);
	let body = CreateDoc { documentId: module_id.clone(), data: ModuleData { ownerUserId: owner.to_string(), ownerUsername: owner.to_string(), name: name.to_string(), latestVersion: version.to_string(), visibility: visibility.to_string(), description: desc.clone() } };
	let resp = headers(http.post(&modules_url), cfg).json(&body).send()?;
	if resp.status().is_success() {
		return Ok(module_id);
	}
	let status = resp.status();
	let txt = resp.text().unwrap_or_default();
	if status.as_u16() == 409 {
		// Exists: update latestVersion if requested (best-effort)
		return Ok(module_id);
	}
	bail!("Module create failed {}: {}", status, txt)
}

fn create_version(http: &Client, api: &str, cfg: &AppwriteCfg, module_id: &str, version: &str, sha: &str, hcl: &str) -> anyhow::Result<()> {
	let url = format!("{}/databases/{}/collections/{}/documents", api, cfg.database, cfg.versions_col);
	let version_doc_id = format!("{}__{}", module_id, version);
	let created_at = Utc::now().to_rfc3339();
	let vbody = CreateDoc { documentId: version_doc_id, data: VersionData { moduleId: module_id.to_string(), version: version.to_string(), sha256: sha.to_string(), hcl: hcl.to_string(), createdAt: created_at } };
	let resp = headers(http.post(&url), cfg).json(&vbody).send()?;
	let status = resp.status();
	if !status.is_success() {
		let txt = resp.text().unwrap_or_default();
		bail!("Version create failed {}: {}", status, txt);
	}
	Ok(())
}

fn upload_assets(http: &Client, api: &str, cfg: &AppwriteCfg, owner: &str, name: &str, version: &str, dir: &Path) -> anyhow::Result<usize> {
	let mut count = 0usize;
	for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok).filter(|e| e.file_type().is_file()) {
		let full_path = entry.path().to_path_buf();
		let rel = full_path.strip_prefix(dir).unwrap();
		let key_path = format!("{}/{}/{}/{}", owner, name, version, rel.to_string_lossy());
		let url = format!("{}/storage/buckets/{}/files", api, cfg.assets_bucket_id);
		let file_part = multipart::Part::file(&full_path)?;
		let form = multipart::Form::new()
			.text("fileId", "unique()")
			.text("x-appwrite-meta-key", key_path.clone())
			.part("file", file_part);
		let resp = http
			.post(&url)
			.header("X-Appwrite-Project", &cfg.project)
			.header("X-Appwrite-Key", &cfg.key)
			.header("X-Appwrite-Response-Format", "1.7.0")
			.multipart(form)
			.send()?;
		if !resp.status().is_success() {
			let status = resp.status();
			let txt = resp.text().unwrap_or_default();
			bail!("Asset upload failed {}: {}", status, txt);
		}
		count += 1;
	}
	Ok(count)
}

fn main() -> anyhow::Result<()> {
	dotenvy::dotenv().ok();
	let cli = Cli::parse();
	match cli.command {
		Commands::Module { cmd } => match cmd {
			ModuleCmd::Publish { owner, name, version, hcl, assets, visibility, desc, set_latest: _ } => {
				let cfg = cfg_from_env()?;
				let http = Client::builder().build()?;
				let api = base_api(&cfg.endpoint);

				let hcl_content = read_hcl(&hcl)?;
				let digest = sha256_hex(&hcl_content);
				let module_id = create_or_update_module(&http, &api, &cfg, &owner, &name, &version, &visibility, &desc)?;
				create_version(&http, &api, &cfg, &module_id, &version, &digest, &hcl_content)?;

				let mut uploaded = 0usize;
				if let Some(dir) = assets {
					if dir.exists() {
						uploaded = upload_assets(&http, &api, &cfg, &owner, &name, &version, &dir)?;
					}
				}
				println!("Published module {}::{} v{} (assets: {})", owner, name, version, uploaded);
				Ok(())
			}
		},
	}
} 