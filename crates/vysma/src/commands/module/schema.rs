use anyhow::bail;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use super::common::{AppwriteCfg, base_api, cfg_from_env, headers};

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

pub fn run() -> anyhow::Result<()> {
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


