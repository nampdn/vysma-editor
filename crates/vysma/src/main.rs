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
	/// Bootstrap a new Vysma project
	New { name: String },
	/// Run the authoritative server with hot-reload
	Serve {
		#[arg(long, default_value_t = String::from("assets/moba_hcl/moba_game.hcl"))] scene: String,
		#[arg(long, default_value_t = false)] gui: bool,
	},
	/// Run a client connected to the local server
	Client {
		#[arg(short, long, default_value = None)] client_id: Option<u64>,
		#[arg(long, default_value_t = false)] gui: bool,
	},
	/// Module-related commands
	Module {
		#[command(subcommand)]
		cmd: ModuleCmd,
	},
	/// Ensure Appwrite schema
	EnsureSchema,
	/// Verify config/env
	Verify,
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
		#[arg(long, default_value_t = false)] dry_run: bool,
		#[arg(long, default_value_t = 4)] parallel: usize,
		#[arg(long, default_value_t = 3)] retries: usize,
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
struct VersionData { moduleId: String, version: String, sha256: String, hcl: String, createdAt: String, #[serde(skip_serializing_if = "Option::is_none")] manifest: Option<Vec<ManifestRow>> }

#[derive(Serialize, Clone, Debug)]
struct ManifestRow { original_path: String, sha256: String, size: i64, #[serde(skip_serializing_if = "Option::is_none")] content_type: Option<String>, url_path: String }

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

fn create_version(http: &Client, api: &str, cfg: &AppwriteCfg, module_id: &str, version: &str, sha: &str, hcl: &str, manifest: Option<Vec<ManifestRow>>) -> anyhow::Result<String> {
	let url = format!("{}/databases/{}/collections/{}/documents", api, cfg.database, cfg.versions_col);
	let version_doc_id = format!("{}__{}", module_id, version);
	let created_at = Utc::now().to_rfc3339();
	let vbody = CreateDoc { documentId: version_doc_id.clone(), data: VersionData { moduleId: module_id.to_string(), version: version.to_string(), sha256: sha.to_string(), hcl: hcl.to_string(), createdAt: created_at, manifest } };
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

fn content_type_for(path: &Path) -> Option<String> {
	let s = path.to_string_lossy();
	if s.ends_with(".png") { Some("image/png".into()) }
	else if s.ends_with(".jpg") || s.ends_with(".jpeg") { Some("image/jpeg".into()) }
	else if s.ends_with(".glb") { Some("model/gltf-binary".into()) }
	else { None }
}

fn upload_assets_and_manifest(http: &Client, api: &str, cfg: &AppwriteCfg, owner: &str, name: &str, _version: &str, version_doc_id: &str, dir: &Path, parallel: usize, _retries: usize, dry_run: bool) -> anyhow::Result<Vec<ManifestRow>> {
	let mut rows: Vec<ManifestRow> = Vec::new();
	let mut tasks: Vec<(PathBuf, PathBuf)> = Vec::new();
	for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok).filter(|e| e.file_type().is_file()) {
		let full_path = entry.path().to_path_buf();
		let rel = full_path.strip_prefix(dir).unwrap().to_path_buf();
		tasks.push((full_path, rel));
	}
	let mut i = 0;
	while i < tasks.len() {
		let end = usize::min(i + parallel.max(1), tasks.len());
		let slice = &tasks[i..end];
		let mut handles = Vec::new();
		for (full_path, rel) in slice.iter().cloned() {
			let http = http.clone();
			let api = api.to_string();
			let cfg = cfg.clone();
			let owner = owner.to_string();
			let name = name.to_string();
			let version_doc_id = version_doc_id.to_string();
			handles.push(std::thread::spawn(move || -> anyhow::Result<ManifestRow> {
				let rel_str = rel.to_string_lossy().to_string();
				let file_bytes = std::fs::read(&full_path)?;
				let file_sha = sha256_hex_str(&file_bytes);
				let file_id = file_sha.chars().take(32).collect::<String>();
				let key_path = format!("{}/{}/{}", owner, name, file_sha);
				let size = file_bytes.len() as i64;
				if !dry_run {
					let url = format!("{}/storage/buckets/{}/files", api, cfg.assets_bucket_id);
					let file_part = multipart::Part::bytes(file_bytes.clone()).file_name(rel_str.clone());
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
					if !resp.status().is_success() && resp.status().as_u16() != 409 {
						let status = resp.status();
						let txt = resp.text().unwrap_or_default();
						anyhow::bail!("Asset upload failed {}: {}", status, txt);
					}
					let created: Option<CreatedFile> = if resp.status().is_success() { Some(resp.json()?) } else { None };
					let size = created.as_ref().and_then(|c| c.sizeOriginal).unwrap_or(size);
					let _ = create_asset_index(&http, &api, &cfg, &version_doc_id, &key_path, &file_id, &file_sha, size, &rel_str);
				}
				Ok(ManifestRow { original_path: rel_str, sha256: file_sha, size, content_type: content_type_for(&full_path), url_path: key_path })
			}));
		}
		for h in handles {
			match h.join().unwrap() {
				Ok(row) => rows.push(row),
				Err(err) => eprintln!("upload error: {}", err),
			}
		}
		i = end;
	}
	rows.sort_by(|a,b| a.original_path.cmp(&b.original_path));
	Ok(rows)
}

fn write_scaffold(name: &str) -> anyhow::Result<()> {
	let root = PathBuf::from(name);
	if root.exists() { bail!("directory '{}' already exists", name); }
	fs::create_dir_all(root.join("assets/scenes"))?;
	fs::create_dir_all(root.join("assets/mesh"))?;
	fs::create_dir_all(root.join("assets/textures"))?;
	let hcl = r##"assets { mesh "cube" { builtin = "cube" } material "hero" { pbr = { base_color = "#3aa7ff" } } }
vars = { speed = 6.0 }
prefab "Hero" { components = { MeshRef = { mesh = "cube" }, StandardMaterialRef = { material = "hero" }, Transform = { s = [1,1,1] } } }
entity "root" { children = [ { name = "Player", include = ["Hero"], components = { Transform = { t = [0,1,0] } } } ] }
triggers { trigger "move_w" { on = { key_held = "KeyW" } target = { name = "Player" } actions = [ { translate_axis = { vec = [0,0,-1], speed_var = "speed" } } ] } }
"##;
	fs::write(root.join("assets/scenes/example.hcl"), hcl)?;
	fs::write(root.join("Cargo.toml"), format!("[workspace]\nmembers=[\n    \"crates/vysma-hcl\",\n    \"crates/vysma-cloud\",\n    \"crates/vysma\"\n]\nresolver=\"2\"\n"))?;
	fs::create_dir_all(root.join(".git"))?; // don't run git init; just create .gitignore
	fs::write(root.join(".gitignore"), "target/\n.DS_Store\n")?;
	Ok(())
}

fn run_server(_scene: &str, gui: bool) -> anyhow::Result<()> {
	let mut cmd = std::process::Command::new("cargo");
	cmd.arg("run").arg("--");
	cmd.arg("server");
	if gui { cmd.arg("--features").arg("gui"); }
	cmd.env("RUST_LOG", "info");
	let status = cmd.status().context("run server")?;
	if !status.success() { bail!("server exited with status {:?}", status.code()); }
	Ok(())
}

fn run_client(client_id: Option<u64>, gui: bool) -> anyhow::Result<()> {
	let mut cmd = std::process::Command::new("cargo");
	cmd.arg("run").arg("--");
	cmd.arg("client");
	if let Some(id) = client_id { cmd.arg("-c").arg(id.to_string()); }
	if gui { cmd.arg("--features").arg("gui"); }
	cmd.env("RUST_LOG", "info");
	let status = cmd.status().context("run client")?;
	if !status.success() { bail!("client exited with status {:?}", status.code()); }
	Ok(())
}

fn main() -> anyhow::Result<()> {
	dotenvy::dotenv().ok();
	let cli = Cli::parse();
	match cli.command {
		Commands::New { name } => {
			write_scaffold(&name)?;
			println!("Scaffolded '{}'\n- assets/\n- Cargo.toml (workspace)\n- .gitignore", name);
			Ok(())
		}
		Commands::Serve { scene: _, gui } => { run_server("assets/moba_hcl/moba_game.hcl", gui) }
		Commands::Client { client_id, gui } => { run_client(client_id, gui) }
		Commands::Module { cmd } => match cmd {
			ModuleCmd::Publish { owner, name, version, hcl, assets, visibility, desc, set_latest, dry_run, parallel, retries } => {
				let cfg = cfg_from_env()?;
				let http = Client::builder().build()?;
				let api = base_api(&cfg.endpoint);

				let hcl_content = read_hcl(&hcl)?;
				let digest = sha256_hex(&hcl_content);
				let module_id = create_or_update_module(&http, &api, &cfg, &owner, &name, &version, &visibility, &desc)?;

				let mut manifest: Option<Vec<ManifestRow>> = None;
				if let Some(dir) = assets.as_ref() { if dir.exists() { let rows = upload_assets_and_manifest(&http, &api, &cfg, &owner, &name, &version, &format!("{}__{}", module_id, version), dir, parallel, retries, dry_run)?; manifest = Some(rows); } }

				let _version_doc_id = create_version(&http, &api, &cfg, &module_id, &version, &digest, &hcl_content, manifest.clone())?;
				if set_latest { let _ = update_latest_version(&http, &api, &cfg, &module_id, &version); }

				println!("Published module {}::{} v{}", owner, name, version);
				if let Some(rows) = manifest { println!("Manifest ({} entries):", rows.len()); for r in rows { println!("  {} -> {} ({} bytes)", r.original_path, r.url_path, r.size); } }
				println!("Import with: modules = [{{ name = \"{}::{}\", alias = \"{}\", version = \"{}\" }}]", owner, name, name, version);
				Ok(())
			}
		},
		Commands::EnsureSchema => {
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
			// manifest (json) stored inline; Appwrite accepts any JSON; no schema attr required
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
		Commands::Verify => {
			let _ = cfg_from_env()?; println!("Env OK"); Ok(())
		}
	}
} 