use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use chrono::Utc;
use clap::{Parser, Subcommand};
use reqwest::blocking::{Client, RequestBuilder};
use reqwest::blocking::multipart;
use serde::{Deserialize, Serialize};
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
		#[arg(long)] owner: String,
		#[arg(long)] name: String,
		#[arg(long)] version: String,
		#[arg(long)] hcl: PathBuf,
		#[arg(long)] assets: Option<PathBuf>,
		#[arg(long, default_value = "public")] visibility: String,
		#[arg(long)] desc: Option<String>,
		#[arg(long, default_value_t = true)] set_latest: bool,
	},
	/// Ensure Appwrite collections have required attributes (Modules, ModuleVersions, and Asset Index if configured)
	EnsureSchema,
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

// Asset index
#[derive(Serialize)]
struct AssetIndexData { moduleVersionId: String, path: String, storageFileId: String, sha256: String, size: i64, original_path: String }

#[derive(Deserialize)]
struct CreatedFile { #[serde(rename = "$id")] id: String, #[serde(default)] sizeOriginal: Option<i64> }

#[derive(Deserialize)]
struct Attr { key: String, status: String }
#[derive(Deserialize)]
struct AttrList { attributes: Vec<Attr> }
#[derive(Serialize)]
struct CreateStringAttr { key: String, size: i32, required: bool, #[serde(skip_serializing_if = "Option::is_none")] default: Option<String>, #[serde(default)] array: bool }
#[derive(Serialize)]
struct CreateEnumAttr { key: String, elements: Vec<String>, required: bool, #[serde(default)] array: bool }
#[derive(Serialize)]
struct CreateDatetimeAttr { key: String, required: bool, #[serde(default)] array: bool }

fn ensure_string_attr(http: &Client, api: &str, cfg: &AppwriteCfg, col: &str, key: &str, size: i32, required: bool) -> anyhow::Result<()> {
	let url = format!("{}/databases/{}/collections/{}/attributes/string", api, cfg.database, col);
	let body = CreateStringAttr { key: key.into(), size, required, default: None, array: false };
	let _ = headers(http.post(&url), cfg).json(&body).send()?;
	Ok(())
}
fn ensure_enum_attr(http: &Client, api: &str, cfg: &AppwriteCfg, col: &str, key: &str, values: Vec<String>, required: bool) -> anyhow::Result<()> {
	let url = format!("{}/databases/{}/collections/{}/attributes/enum", api, cfg.database, col);
	let body = CreateEnumAttr { key: key.into(), elements: values, required, array: false };
	let _ = headers(http.post(&url), cfg).json(&body).send()?;
	Ok(())
}
fn ensure_datetime_attr(http: &Client, api: &str, cfg: &AppwriteCfg, col: &str, key: &str, required: bool) -> anyhow::Result<()> {
	let url = format!("{}/databases/{}/collections/{}/attributes/datetime", api, cfg.database, col);
	let body = CreateDatetimeAttr { key: key.into(), required, array: false };
	let _ = headers(http.post(&url), cfg).json(&body).send()?;
	Ok(())
}
fn wait_attrs_available(http: &Client, api: &str, cfg: &AppwriteCfg, col: &str, keys: &[&str]) -> anyhow::Result<()> {
	let url = format!("{}/databases/{}/collections/{}/attributes", api, cfg.database, col);
	for _ in 0..20 {
		let resp: AttrList = headers(http.get(&url), cfg).send()?.error_for_status()?.json()?;
		let ready = keys.iter().all(|k| resp.attributes.iter().any(|a| a.key == *k && a.status == "available"));
		if ready { return Ok(()); }
		std::thread::sleep(std::time::Duration::from_millis(300));
	}
	bail!("attributes not available in time for collection {}", col)
}

fn update_latest_version(http: &Client, api: &str, cfg: &AppwriteCfg, module_id: &str, latest: &str) -> anyhow::Result<()> {
	let url = format!("{}/databases/{}/collections/{}/documents/{}", api, cfg.database, cfg.modules_col, module_id);
	let body = serde_json::json!({ "latestVersion": latest });
	let resp = headers(http.patch(&url), cfg).json(&body).send()?;
	if !resp.status().is_success() {
		let status = resp.status();
		let txt = resp.text().unwrap_or_default();
		bail!("Update latestVersion failed {}: {}", status, txt);
	}
	Ok(())
}

fn sha256_hex_str(content: &[u8]) -> String { let mut h=Sha256::new(); h.update(content); format!("{:x}", h.finalize()) }
fn sha256_hex(s: &str) -> String { sha256_hex_str(s.as_bytes()) }

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
		return Ok(module_id);
	}
	bail!("Module create failed {}: {}", status, txt)
}

fn create_version(http: &Client, api: &str, cfg: &AppwriteCfg, module_id: &str, version: &str, sha: &str, hcl: &str) -> anyhow::Result<String> {
	let url = format!("{}/databases/{}/collections/{}/documents", api, cfg.database, cfg.versions_col);
	let version_doc_id = format!("{}__{}", module_id, version);
	let created_at = Utc::now().to_rfc3339();
	let vbody = CreateDoc { documentId: version_doc_id.clone(), data: VersionData { moduleId: module_id.to_string(), version: version.to_string(), sha256: sha.to_string(), hcl: hcl.to_string(), createdAt: created_at } };
	let resp = headers(http.post(&url), cfg).json(&vbody).send()?;
	let status = resp.status();
	if !status.is_success() {
		let txt = resp.text().unwrap_or_default();
		bail!("Version create failed {}: {}", status, txt);
	}
	Ok(version_doc_id)
}

fn create_asset_index(http: &Client, api: &str, cfg: &AppwriteCfg, version_doc_id: &str, path: &str, file_id: &str, sha: &str, size: i64, original_path: &str) -> anyhow::Result<()> {
	let Some(index_col) = &cfg.assets_index_col else { return Ok(()); };
	let url = format!("{}/databases/{}/collections/{}/documents", api, cfg.database, index_col);
	let doc_id = format!("{}__{}", version_doc_id, sha);
	let body = CreateDoc { documentId: doc_id, data: AssetIndexData { moduleVersionId: version_doc_id.into(), path: path.into(), storageFileId: file_id.into(), sha256: sha.into(), size, original_path: original_path.into() } };
	let resp = headers(http.post(&url), cfg).json(&body).send()?;
	if !resp.status().is_success() {
		let status = resp.status();
		let txt = resp.text().unwrap_or_default();
		bail!("Asset index create failed {}: {}", status, txt);
	}
	Ok(())
}

fn upload_assets(http: &Client, api: &str, cfg: &AppwriteCfg, owner: &str, name: &str, version: &str, version_doc_id: &str, dir: &Path) -> anyhow::Result<usize> {
	let mut count = 0usize;
	for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok).filter(|e| e.file_type().is_file()) {
		let full_path = entry.path().to_path_buf();
		let rel = full_path.strip_prefix(dir).unwrap();
		let rel_str = rel.to_string_lossy();
		let file_bytes = fs::read(&full_path)?;
		let file_sha = sha256_hex_str(&file_bytes);
		// namespaced hashed id and key
		let file_id = format!("{}__{}__{}", owner, name, file_sha);
		let key_path = format!("{}/{}/{}", owner, name, file_sha);
		let url = format!("{}/storage/buckets/{}/files", api, cfg.assets_bucket_id);
		let file_part = multipart::Part::bytes(file_bytes.clone()).file_name(rel_str.to_string());
		let form = multipart::Form::new()
			.text("fileId", file_id.clone())
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
			// If already exists (409), continue; else error
			if resp.status().as_u16() != 409 {
				let status = resp.status();
				let txt = resp.text().unwrap_or_default();
				bail!("Asset upload failed {}: {}", status, txt);
			}
		}
		let created: Option<CreatedFile> = if resp.status().is_success() { Some(resp.json()?) } else { None };
		let size = created.as_ref().and_then(|c| c.sizeOriginal).unwrap_or(0);
		create_asset_index(http, api, cfg, version_doc_id, &key_path, &file_id, &file_sha, size, &rel_str)?;
		count += 1;
	}
	Ok(count)
}

fn main() -> anyhow::Result<()> {
	dotenvy::dotenv().ok();
	let cli = Cli::parse();
	match cli.command {
		Commands::Module { cmd } => match cmd {
			ModuleCmd::Publish { owner, name, version, hcl, assets, visibility, desc, set_latest } => {
				let cfg = cfg_from_env()?;
				let http = Client::builder().build()?;
				let api = base_api(&cfg.endpoint);

				let hcl_content = read_hcl(&hcl)?;
				let digest = sha256_hex(&hcl_content);
				let module_id = create_or_update_module(&http, &api, &cfg, &owner, &name, &version, &visibility, &desc)?;
				let version_doc_id = create_version(&http, &api, &cfg, &module_id, &version, &digest, &hcl_content)?;
				if set_latest { let _ = update_latest_version(&http, &api, &cfg, &module_id, &version); }

				let mut uploaded = 0usize;
				if let Some(dir) = assets { if dir.exists() { uploaded = upload_assets(&http, &api, &cfg, &owner, &name, &version, &version_doc_id, &dir)?; } }

				println!("Published module {}::{} v{} (assets: {})", owner, name, version, uploaded);
				println!("Import with: modules = [{{ name = \"{}::{}\", alias = \"{}\", version = \"{}\" }}]", owner, name, name, version);
				Ok(())
			}
			ModuleCmd::EnsureSchema => {
				let cfg = cfg_from_env()?;
				let http = Client::builder().build()?;
				let api = base_api(&cfg.endpoint);
				// Modules
				ensure_string_attr(&http, &api, &cfg, &cfg.modules_col, "ownerUserId", 256, true)?;
				ensure_string_attr(&http, &api, &cfg, &cfg.modules_col, "ownerUsername", 256, true)?;
				ensure_string_attr(&http, &api, &cfg, &cfg.modules_col, "name", 256, true)?;
				ensure_string_attr(&http, &api, &cfg, &cfg.modules_col, "latestVersion", 64, true)?;
				ensure_enum_attr(&http, &api, &cfg, &cfg.modules_col, "visibility", vec!["public".into(), "private".into()], true)?;
				wait_attrs_available(&http, &api, &cfg, &cfg.modules_col, &["ownerUserId", "ownerUsername", "name", "latestVersion", "visibility"])?;
				// ModuleVersions
				ensure_string_attr(&http, &api, &cfg, &cfg.versions_col, "moduleId", 256, true)?;
				ensure_string_attr(&http, &api, &cfg, &cfg.versions_col, "version", 64, true)?;
				ensure_string_attr(&http, &api, &cfg, &cfg.versions_col, "sha256", 256, true)?;
				ensure_string_attr(&http, &api, &cfg, &cfg.versions_col, "hcl", 65535, true)?;
				ensure_datetime_attr(&http, &api, &cfg, &cfg.versions_col, "createdAt", true)?;
				wait_attrs_available(&http, &api, &cfg, &cfg.versions_col, &["moduleId", "version", "sha256", "hcl", "createdAt"])?;
				// Assets index (optional)
				if let Some(index_col) = &cfg.assets_index_col {
					ensure_string_attr(&http, &api, &cfg, index_col, "moduleVersionId", 256, true)?;
					ensure_string_attr(&http, &api, &cfg, index_col, "path", 1024, true)?;
					ensure_string_attr(&http, &api, &cfg, index_col, "storageFileId", 256, true)?;
					ensure_string_attr(&http, &api, &cfg, index_col, "sha256", 256, true)?;
					ensure_string_attr(&http, &api, &cfg, index_col, "size", 64, true)?;
					wait_attrs_available(&http, &api, &cfg, index_col, &["moduleVersionId", "path", "storageFileId", "sha256", "size"])?;
				}
				println!("Schema ensured");
				Ok(())
			}
		},
	}
} 