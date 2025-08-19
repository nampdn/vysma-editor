use anyhow::Context;
use reqwest::blocking::RequestBuilder;

#[derive(Clone)]
pub struct AppwriteCfg {
	pub endpoint: String,
	pub project: String,
	pub key: String,
	pub database: String,
	pub modules_col: String,
	pub versions_col: String,
	pub assets_bucket_id: String,
	pub assets_index_col: Option<String>,
}

pub fn cfg_from_env() -> anyhow::Result<AppwriteCfg> {
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

pub fn base_api(e: &str) -> String {
	let e = e.trim_end_matches('/');
	if e.ends_with("/v1") { e.to_string() } else { format!("{}/v1", e) }
}

pub fn headers(rb: RequestBuilder, cfg: &AppwriteCfg) -> RequestBuilder {
	rb.header("X-Appwrite-Project", &cfg.project)
		.header("X-Appwrite-Key", &cfg.key)
		.header("X-Appwrite-Response-Format", "1.7.0")
		.header("Content-Type", "application/json")
}


