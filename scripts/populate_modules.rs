use anyhow::{bail, Context};
use reqwest::blocking::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::thread::sleep;
use std::time::Duration;

#[derive(Clone)]
struct Cfg {
    endpoint: String,
    project: String,
    key: String,
    database: String,
    modules_col: String,
    versions_col: String,
}

fn cfg_from_env() -> anyhow::Result<Cfg> {
    let get = |k: &str| std::env::var(k).with_context(|| format!("missing env {k}"));
    Ok(Cfg {
        endpoint: get("APPWRITE_ENDPOINT")?,
        project: get("APPWRITE_PROJECT_ID")?,
        key: get("APPWRITE_API_KEY")?,
        database: get("APPWRITE_DATABASE_ID")?,
        modules_col: get("APPWRITE_MODULES_COLLECTION_ID")?,
        versions_col: get("APPWRITE_MODULEVERSIONS_COLLECTION_ID")?,
    })
}

fn base_api(e: &str) -> String {
    let e = e.trim_end_matches('/');
    if e.ends_with("/v1") { e.to_string() } else { format!("{}/v1", e) }
}

fn headers<'a>(rb: RequestBuilder, cfg: &Cfg) -> RequestBuilder {
    rb.header("X-Appwrite-Project", &cfg.project)
        .header("X-Appwrite-Key", &cfg.key)
        .header("X-Appwrite-Response-Format", "1.7.0")
        .header("Content-Type", "application/json")
}

#[derive(Serialize)]
struct ModuleData {
    ownerUserId: String,
    ownerUsername: String,
    name: String,
    latestVersion: String,
    visibility: String,
}

#[derive(Serialize)]
struct CreateDoc<T> { documentId: String, data: T }

#[derive(Deserialize)]
struct Created { #[serde(rename = "$id")] id: String }

#[derive(Serialize)]
struct VersionData { moduleId: String, version: String, sha256: String, hcl: String, createdAt: String }

#[derive(Deserialize)]
struct Attr { key: String, status: String }

#[derive(Deserialize)]
struct AttrList { attributes: Vec<Attr> }

#[derive(Serialize)]
struct CreateStringAttr { key: String, size: i32, required: bool, #[serde(skip_serializing_if = "Option::is_none")] default: Option<String>, #[serde(default)] array: bool }

#[derive(Serialize)]
struct CreateEnumAttr { key: String, elements: Vec<String>, required: bool, #[serde(skip_serializing_if = "Option::is_none")] default: Option<String>, #[serde(default)] array: bool }

#[derive(Serialize)]
struct CreateDateTimeAttr { key: String, required: bool, #[serde(skip_serializing_if = "Option::is_none")] default: Option<String>, #[serde(default)] array: bool }

fn ensure_string_attr(http: &Client, api: &str, cfg: &Cfg, col: &str, key: &str, size: i32, required: bool) -> anyhow::Result<()> {
    let url = format!("{}/databases/{}/collections/{}/attributes/string", api, cfg.database, col);
    let body = CreateStringAttr { key: key.to_string(), size, required, default: None, array: false };
    let resp = headers(http.post(&url), cfg).json(&body).send()?;
    if resp.status().as_u16() == 409 { return Ok(()); }
    resp.error_for_status()?; Ok(())
}

fn ensure_enum_attr(http: &Client, api: &str, cfg: &Cfg, col: &str, key: &str, elements: Vec<String>, required: bool) -> anyhow::Result<()> {
    let url = format!("{}/databases/{}/collections/{}/attributes/enum", api, cfg.database, col);
    let body = CreateEnumAttr { key: key.to_string(), elements, required, default: None, array: false };
    let resp = headers(http.post(&url), cfg).json(&body).send()?;
    if resp.status().as_u16() == 409 { return Ok(()); }
    resp.error_for_status()?; Ok(())
}

fn ensure_datetime_attr(http: &Client, api: &str, cfg: &Cfg, col: &str, key: &str, required: bool) -> anyhow::Result<()> {
    let url = format!("{}/databases/{}/collections/{}/attributes/datetime", api, cfg.database, col);
    let body = CreateDateTimeAttr { key: key.to_string(), required, default: None, array: false };
    let resp = headers(http.post(&url), cfg).json(&body).send()?;
    if resp.status().as_u16() == 409 { return Ok(()); }
    resp.error_for_status()?; Ok(())
}

fn wait_attrs_available(http: &Client, api: &str, cfg: &Cfg, col: &str, keys: &[&str]) -> anyhow::Result<()> {
    let url = format!("{}/databases/{}/collections/{}/attributes", api, cfg.database, col);
    for _ in 0..30 {
        let list: AttrList = headers(http.get(&url), cfg).send()?.error_for_status()?.json()?;
        let mut ok = true;
        for k in keys {
            if let Some(a) = list.attributes.iter().find(|a| &a.key == k) {
                if a.status != "available" { ok = false; break; }
            } else { ok = false; break; }
        }
        if ok { return Ok(()); }
        sleep(Duration::from_millis(500));
    }
    bail!("attributes not available in time for collection {}", col)
}

fn sha256(s: &str) -> String { let mut hasher = sha2::Sha256::new(); use sha2::Digest; hasher.update(s.as_bytes()); format!("{:x}", hasher.finalize()) }

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let cfg = cfg_from_env()?;
    let http = Client::builder().build()?;
    let api = base_api(&cfg.endpoint);

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 { eprintln!("Usage: cargo run --bin populate_modules -- <owner_username> <module_name> <version> <hcl_path>"); std::process::exit(1); }
    let owner_username = &args[1];
    let module_name = &args[2];
    let version = &args[3];
    let hcl_path = &args[4];
    let hcl = fs::read_to_string(hcl_path).context("read hcl file")?;
    let digest = sha256(&hcl);

    // Ensure attributes for modules collection
    ensure_string_attr(&http, &api, &cfg, &cfg.modules_col, "ownerUserId", 256, true)?;
    ensure_string_attr(&http, &api, &cfg, &cfg.modules_col, "ownerUsername", 256, true)?;
    ensure_string_attr(&http, &api, &cfg, &cfg.modules_col, "name", 256, true)?;
    ensure_string_attr(&http, &api, &cfg, &cfg.modules_col, "latestVersion", 64, true)?;
    ensure_enum_attr(&http, &api, &cfg, &cfg.modules_col, "visibility", vec!["public".into(), "private".into()], true)?;
    wait_attrs_available(&http, &api, &cfg, &cfg.modules_col, &["ownerUserId", "ownerUsername", "name", "latestVersion", "visibility"])?;

    // Ensure attributes for versions collection
    ensure_string_attr(&http, &api, &cfg, &cfg.versions_col, "moduleId", 256, true)?;
    ensure_string_attr(&http, &api, &cfg, &cfg.versions_col, "version", 64, true)?;
    ensure_string_attr(&http, &api, &cfg, &cfg.versions_col, "sha256", 256, true)?;
    ensure_string_attr(&http, &api, &cfg, &cfg.versions_col, "hcl", 65535, true)?;
    ensure_datetime_attr(&http, &api, &cfg, &cfg.versions_col, "createdAt", true)?;
    wait_attrs_available(&http, &api, &cfg, &cfg.versions_col, &["moduleId", "version", "sha256", "hcl", "createdAt"])?;

    // Create module document
    let mod_url = format!("{}/databases/{}/collections/{}/documents", api, cfg.database, cfg.modules_col);
    let module_id = format!("{}__{}", owner_username, module_name);
    let body = CreateDoc { documentId: module_id.clone(), data: ModuleData { ownerUserId: owner_username.clone(), ownerUsername: owner_username.clone(), name: module_name.clone(), latestVersion: version.clone(), visibility: "public".into() } };
    let resp = headers(http.post(&mod_url), &cfg).json(&body).send()?;
    if !resp.status().is_success() {
        let status = resp.status();
        let txt = resp.text().unwrap_or_default();
        bail!("Module create failed {}: {}", status, txt);
    }
    let created: Created = resp.json()?;

    // Create version document
    let ver_url = format!("{}/databases/{}/collections/{}/documents", api, cfg.database, cfg.versions_col);
    let version_doc_id = format!("{}__{}", created.id, version);
    let created_at = chrono::Utc::now().to_rfc3339();
    let vbody = CreateDoc { documentId: version_doc_id, data: VersionData { moduleId: created.id, version: version.clone(), sha256: digest, hcl, createdAt: created_at } };
    let vresp = headers(http.post(&ver_url), &cfg).json(&vbody).send()?;
    let status = vresp.status();
    if !status.is_success() {
        let txt = vresp.text().unwrap_or_default();
        bail!("Version create failed {}: {}", status, txt);
    }

    println!("✅ Published module {}::{} version {}", owner_username, module_name, version);
    Ok(())
} 