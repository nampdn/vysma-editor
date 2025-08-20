use std::path::Path;

use anyhow::{bail, Context};
use chrono::Utc;
use reqwest::blocking::Client;
use reqwest::blocking::multipart;
use serde::{Deserialize, Serialize};
use serde_json;
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
struct VersionData { moduleId: String, version: String, sha256: String, hcl: String, createdAt: String, #[serde(skip_serializing_if = "Option::is_none")] manifest: Option<String> }

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
		if let Some(rows) = &manifest { print_manifest_table(rows); }
		return Ok(());
	}

	let cfg = cfg_from_env()?;
	let http = Client::builder().build()?;
	let api = base_api(&cfg.endpoint);
	let module_id = create_or_update_module(&http, &api, &cfg, &owner, &name, &version, &visibility, &desc)?;

	if let Some(dir) = assets { if dir.exists() { let rows = upload_assets_and_manifest(&http, &api, &cfg, &owner, &name, &version, &format!("{}__{}", module_id, version), dir, parallel, retries, false)?; manifest = Some(rows); } }

	// Build bundler index (.toml) capturing module, version, sha, deps (from HCL), and all resources
	let deps = extract_module_names(&hcl_content);
	let mut rows_for_index = manifest.clone().unwrap_or_default();
	let bundle_toml = build_bundle_index_toml(&owner, &name, &version, &digest, &deps, &rows_for_index);
	let index_bytes = bundle_toml.as_bytes();
	let index_sha = sha256_hex_str(index_bytes);
	let index_file_id = index_sha.chars().take(32).collect::<String>();
	let rel_index_name = "index.toml".to_string();
	let created_size = try_upload_with_retries(&http, &api, &cfg, index_bytes, &rel_index_name, &index_file_id, &index_file_id, retries)?;
	let index_size = created_size.unwrap_or(index_bytes.len() as i64);
	rows_for_index.push(ManifestRow { original_path: rel_index_name.clone(), sha256: index_sha, size: index_size, content_type: Some("text/plain".into()), url_path: index_file_id.clone() });
	let manifest = Some(rows_for_index);
	let manifest_str: Option<String> = Some(serde_json::to_string(manifest.as_ref().unwrap())?);

	let _version_doc_id = create_version(&http, &api, &cfg, &module_id, &version, &digest, &hcl_content, manifest_str)?;
	if set_latest { let _ = update_latest_version(&http, &api, &cfg, &module_id, &version); }

	println!("Published module {}::{} v{}", owner, name, version);
	if let Some(rows) = manifest { print_manifest_table(&rows); }
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

fn create_version(http: &Client, api: &str, cfg: &AppwriteCfg, module_id: &str, version: &str, sha: &str, hcl: &str, manifest: Option<String>) -> anyhow::Result<String> {
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

fn upload_assets_and_manifest(http: &Client, api: &str, cfg: &AppwriteCfg, owner: &str, name: &str, _version: &str, version_doc_id: &str, dir: &Path, parallel: usize, retries: usize, dry_run: bool) -> anyhow::Result<Vec<ManifestRow>> {
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
				let ext = full_path.extension().and_then(|e| e.to_str()).map(|s| format!(".{}", s)).unwrap_or_default();
				let key_path = format!("{}/{}/{}{}", owner, name, file_sha, ext); // metadata only
				let url_token = file_id.clone(); // Appwrite is flat; use fileId as token
				let size = file_bytes.len() as i64;
				if !dry_run {
					let created_size = try_upload_with_retries(&http, &api, &cfg, &file_bytes, &rel_str, &key_path, &file_id, retries)?;
					let size = created_size.unwrap_or(size);
					let _ = create_asset_index(&http, &api, &cfg, &version_doc_id, &url_token, &file_id, &file_sha, size, &rel_str);
				}
				Ok(ManifestRow { original_path: rel_str, sha256: file_sha, size, content_type: content_type_for(&full_path), url_path: url_token })
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

fn try_upload_with_retries(
	http: &Client,
	api: &str,
	cfg: &AppwriteCfg,
	file_bytes: &[u8],
	rel_str: &str,
	key_path: &str,
	file_id: &str,
	retries: usize,
) -> anyhow::Result<Option<i64>> {
	let url = format!("{}/storage/buckets/{}/files", api, cfg.assets_bucket_id);
	let mut attempt: usize = 0;
	loop {
		let file_part = multipart::Part::bytes(file_bytes.to_vec()).file_name(rel_str.to_string());
		let form = multipart::Form::new()
			.text("fileId", file_id.to_string())
			.text("x-appwrite-meta-key", key_path.to_string())
			.part("file", file_part);
		let resp = http
			.post(&url)
			.header("X-Appwrite-Project", &cfg.project)
			.header("X-Appwrite-Key", &cfg.key)
			.header("X-Appwrite-Response-Format", "1.7.0")
			.multipart(form)
			.send()?;
		if resp.status().is_success() {
			let created: CreatedFile = resp.json()?;
			return Ok(created.sizeOriginal);
		} else if resp.status().as_u16() == 409 {
			return Ok(None);
		} else if attempt < retries {
			attempt += 1;
			std::thread::sleep(std::time::Duration::from_millis(200 * attempt as u64));
			continue;
		} else {
			let status = resp.status();
			let txt = resp.text().unwrap_or_default();
			anyhow::bail!("Asset upload failed {}: {}", status, txt);
		}
	}
}

fn print_manifest_table(rows: &[ManifestRow]) {
	println!("Manifest ({} entries):", rows.len());
	let mut w1 = 12usize;
	let mut w2 = 8usize;
	for r in rows {
		w1 = w1.max(r.original_path.len());
		w2 = w2.max(format!("{}", r.size).len());
	}
	println!("{:<w1$}  {:<w2$}  {}", "original_path", "size", "url_path", w1=w1, w2=w2);
	for r in rows {
		println!("{:<w1$}  {:<w2$}  {}", r.original_path, r.size, r.url_path, w1=w1, w2=w2);
	}
}

// Very simple dependency extractor: modules = [ { name = "alice::moba_core", ... }, ... ]
fn extract_module_names(hcl: &str) -> Vec<String> {
	let mut out = Vec::new();
	for line in hcl.lines() {
		let l = line.trim();
		if l.contains("modules") && l.contains("[") { continue; }
		if l.contains("name") && l.contains("::") {
			// crude: name = "user::module"
			if let Some(start) = l.find('"') { if let Some(end) = l[start+1..].find('"') { out.push(l[start+1..start+1+end].to_string()); } }
		}
	}
	out
}

fn build_bundle_index_toml(owner: &str, name: &str, version: &str, sha: &str, deps: &[String], manifest: &[ManifestRow]) -> String {
	use std::fmt::Write;
	let mut s = String::new();
	let _ = writeln!(s, "[module]\nowner = \"{}\"\nname = \"{}\"\nversion = \"{}\"\nsha = \"{}\"\n", owner, name, version, sha);
	if !deps.is_empty() {
		let _ = writeln!(s, "[dependencies]");
		for d in deps { let _ = writeln!(s, "dep = \"{}\"", d); }
		let _ = writeln!(s);
	}
	let _ = writeln!(s, "[[resources]]");
	for r in manifest {
		let _ = writeln!(s, "path = \"{}\"\nsha256 = \"{}\"\nsize = {}\nfileId = \"{}\"\n", r.original_path, r.sha256, r.size, r.url_path);
		let _ = writeln!(s, "[[resources]]");
	}
	s
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


