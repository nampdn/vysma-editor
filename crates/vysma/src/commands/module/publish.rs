use std::path::Path;

use anyhow::{bail, Context};
use chrono::Utc;
use reqwest::blocking::Client;
use reqwest::blocking::multipart;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use super::common::{AppwriteCfg, base_api, cfg_from_env, headers};

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

#[derive(Serialize)]
struct AssetIndexData { moduleVersionId: String, path: String, storageFileId: String, sha256: String, size: i64, original_path: String }

#[derive(Deserialize)]
struct CreatedFile { #[serde(rename = "$id")] id: String, #[serde(default)] sizeOriginal: Option<i64> }

pub fn sha256_hex_str(content: &[u8]) -> String { let mut h=Sha256::new(); h.update(content); format!("{:x}", h.finalize()) }
pub fn sha256_hex(s: &str) -> String { sha256_hex_str(s.as_bytes()) }

pub fn read_hcl(path: &Path) -> anyhow::Result<String> {
	Ok(std::fs::read_to_string(path).with_context(|| format!("read hcl file: {}", path.display()))?)
}

pub fn run(owner: String, name: String, version: String, hcl: &Path, assets: Option<&Path>, visibility: String, desc: Option<String>, set_latest: bool, dry_run: bool, parallel: usize, retries: usize) -> anyhow::Result<()> {
	let hcl_content = read_hcl(hcl)?;
	let digest = sha256_hex(&hcl_content);

	let mut manifest: Option<Vec<ManifestRow>> = None;
	if let Some(dir) = assets {
		if dir.exists() {
			let rows = upload_assets_and_manifest(&Client::builder().build()?, "", &AppwriteCfg {
				endpoint: String::new(), project: String::new(), key: String::new(), database: String::new(), modules_col: String::new(), versions_col: String::new(), assets_bucket_id: String::new(), assets_index_col: None
			}, &owner, &name, &version, "", dir, parallel, 0, true)?;
			manifest = Some(rows);
		}
	}
	if dry_run {
		println!("[dry-run] Module {}::{} v{} (sha={})", owner, name, version, digest);
		println!("Import with: modules = [{{ name = \"{}::{}\", alias = \"{}\", version = \"{}\" }}]", owner, name, name, version);
		if let Some(rows) = &manifest { println!("Manifest ({} entries):", rows.len()); for r in rows { println!("  {} -> {} ({} bytes)", r.original_path, r.url_path, r.size); } }
		return Ok(());
	}

	let cfg = cfg_from_env()?;
	let http = Client::builder().build()?;
	let api = base_api(&cfg.endpoint);
	let module_id = create_or_update_module(&http, &api, &cfg, &owner, &name, &version, &visibility, &desc)?;

	if let Some(dir) = assets { if dir.exists() { let rows = upload_assets_and_manifest(&http, &api, &cfg, &owner, &name, &version, &format!("{}__{}", module_id, version), dir, parallel, retries, false)?; manifest = Some(rows); } }

	let _version_doc_id = create_version(&http, &api, &cfg, &module_id, &version, &digest, &hcl_content, manifest.clone())?;
	if set_latest { let _ = update_latest_version(&http, &api, &cfg, &module_id, &version); }

	println!("Published module {}::{} v{}", owner, name, version);
	if let Some(rows) = manifest { println!("Manifest ({} entries):", rows.len()); for r in rows { println!("  {} -> {} ({} bytes)", r.original_path, r.url_path, r.size); } }
	println!("Import with: modules = [{{ name = \"{}::{}\", alias = \"{}\", version = \"{}\" }}]", owner, name, name, version);
	Ok(())
}

fn create_or_update_module(http: &Client, api: &str, cfg: &AppwriteCfg, owner: &str, name: &str, version: &str, visibility: &str, desc: &Option<String>) -> anyhow::Result<String> {
	let modules_url = format!("{}/databases/{}/collections/{}/documents", api, cfg.database, cfg.modules_col);
	let module_id = format!("{}__{}", owner, name);
	let body = CreateDoc { documentId: module_id.clone(), data: ModuleData { ownerUserId: owner.to_string(), ownerUsername: owner.to_string(), name: name.to_string(), latestVersion: version.to_string(), visibility: visibility.to_string(), description: desc.clone() } };
	let resp = headers(http.post(&modules_url), cfg).json(&body).send()?;
	if resp.status().is_success() { return Ok(module_id); }
	let status = resp.status();
	let txt = resp.text().unwrap_or_default();
	if status.as_u16() == 409 { return Ok(module_id); }
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

fn content_type_for(path: &Path) -> Option<String> {
	let s = path.to_string_lossy();
	if s.ends_with(".png") { Some("image/png".into()) }
	else if s.ends_with(".jpg") || s.ends_with(".jpeg") { Some("image/jpeg".into()) }
	else if s.ends_with(".glb") { Some("model/gltf-binary".into()) }
	else { None }
}

fn upload_assets_and_manifest(http: &Client, api: &str, cfg: &AppwriteCfg, owner: &str, name: &str, _version: &str, version_doc_id: &str, dir: &Path, parallel: usize, _retries: usize, dry_run: bool) -> anyhow::Result<Vec<ManifestRow>> {
	let mut rows: Vec<ManifestRow> = Vec::new();
	let mut tasks: Vec<(std::path::PathBuf, std::path::PathBuf)> = Vec::new();
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


