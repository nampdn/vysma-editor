use anyhow::Context;
use bevy::prelude::App;
use serde::Deserialize;
use std::collections::HashMap;

pub struct AppwriteConfig {
    pub endpoint: String,
    pub project_id: String,
    pub api_key: String,
    pub database_id: String,
    pub modules_collection_id: String,
    pub module_versions_collection_id: String,
}

impl AppwriteConfig {
    pub fn from_env() -> Option<Self> {
        let get = |k: &str| std::env::var(k).ok();
        Some(Self {
            endpoint: get("APPWRITE_ENDPOINT")?,
            project_id: get("APPWRITE_PROJECT_ID")?,
            api_key: get("APPWRITE_API_KEY")?,
            database_id: get("APPWRITE_DATABASE_ID")?,
            modules_collection_id: get("APPWRITE_MODULES_COLLECTION_ID")?,
            module_versions_collection_id: get("APPWRITE_MODULEVERSIONS_COLLECTION_ID")?,
        })
    }
}

fn normalize_endpoint(ep: &str) -> String {
    let e = ep.trim_end_matches('/');
    if e.ends_with("/v1") { e.to_string() } else { format!("{}/v1", e) }
}

pub struct AppwriteClient {
    cfg: AppwriteConfig,
    // Lazily created runtime for async SDK calls
    rt: tokio::runtime::Runtime,
    sdk: unofficial_appwrite::client::Client,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ManifestEntry {
    pub original_path: String,
    pub sha256: String,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub content_type: Option<String>,
    pub url_path: String,
}

#[derive(Clone, Debug)]
pub struct ModuleVersionData {
    pub version: String,
    pub hcl: String,
    pub manifest: Option<Vec<ManifestEntry>>,
}

impl AppwriteClient {
    pub fn new(cfg: AppwriteConfig) -> Self {
        let endpoint = normalize_endpoint(&cfg.endpoint);
        let mut builder = unofficial_appwrite::client::ClientBuilder::default();
        let sdk = builder
            .set_endpoint(&endpoint).expect("endpoint")
            .set_project(&cfg.project_id).expect("project")
            .set_key(&cfg.api_key).expect("key")
            .build()
            .expect("build appwrite client");
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("tokio rt");
        Self { cfg, rt, sdk }
    }

    pub fn get_module_id(&self, owner_username: &str, name: &str) -> anyhow::Result<String> {
        let client = self.sdk.clone();
        let db = self.cfg.database_id.clone();
        let col = self.cfg.modules_collection_id.clone();
        let owner = owner_username.to_string();
        let mod_name = name.to_string();
        let fut = async move {
            use unofficial_appwrite::{query::Query, services::server::databases::Databases};
            use serde_json::Value;
            let mut args: HashMap<String, Value> = HashMap::new();
            let queries = vec![
                Query::equal("ownerUsername".into(), vec![owner].into()),
                Query::equal("name".into(), vec![mod_name].into()),
                Query::limit(1.into()),
            ];
            args.insert("queries".into(), Value::Array(queries.into_iter().map(Value::String).collect()));
            let list = Databases::list_documents(&client, &db, &col, args).await?;
            let doc = list.documents.into_iter().next().ok_or_else(|| unofficial_appwrite::error::Error::Custom("module not found".into()))?;
            Ok::<String, unofficial_appwrite::error::Error>(doc.id)
        };
        let id = self.rt.block_on(fut)?;
        Ok(id)
    }

    pub fn get_latest_version(&self, module_id: &str) -> anyhow::Result<ModuleVersionData> {
        let client = self.sdk.clone();
        let db = self.cfg.database_id.clone();
        let col = self.cfg.module_versions_collection_id.clone();
        let mid = module_id.to_string();
        let fut = async move {
            use unofficial_appwrite::{query::Query, services::server::databases::Databases};
            use serde_json::Value;
            let mut args: HashMap<String, Value> = HashMap::new();
            let queries = vec![
                Query::equal("moduleId".into(), vec![mid].into()),
                Query::order_desc("$createdAt".into()),
                Query::limit(1.into()),
            ];
            args.insert("queries".into(), Value::Array(queries.into_iter().map(Value::String).collect()));
            let list = Databases::list_documents(&client, &db, &col, args).await?;
            let doc = list.documents.into_iter().next().ok_or_else(|| unofficial_appwrite::error::Error::Custom("no versions for module".into()))?;
            let version = doc.data.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let hcl = doc.data.get("hcl").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let manifest: Option<Vec<ManifestEntry>> = doc.data.get("manifest").and_then(|v| serde_json::from_value(v.clone()).ok());
            Ok::<ModuleVersionData, unofficial_appwrite::error::Error>(ModuleVersionData { version, hcl, manifest })
        };
        let out = self.rt.block_on(fut)?;
        Ok(out)
    }

    pub fn get_specific_version(&self, module_id: &str, version: &str) -> anyhow::Result<ModuleVersionData> {
        let client = self.sdk.clone();
        let db = self.cfg.database_id.clone();
        let col = self.cfg.module_versions_collection_id.clone();
        let mid = module_id.to_string();
        let ver = version.to_string();
        let fut = async move {
            use unofficial_appwrite::{query::Query, services::server::databases::Databases};
            use serde_json::Value;
            let mut args: HashMap<String, Value> = HashMap::new();
            let queries = vec![
                Query::equal("moduleId".into(), vec![mid].into()),
                Query::equal("version".into(), vec![ver].into()),
                Query::limit(1.into()),
            ];
            args.insert("queries".into(), Value::Array(queries.into_iter().map(Value::String).collect()));
            let list = Databases::list_documents(&client, &db, &col, args).await?;
            let doc = list.documents.into_iter().next().ok_or_else(|| unofficial_appwrite::error::Error::Custom("version not found".into()))?;
            let version = doc.data.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let hcl = doc.data.get("hcl").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let manifest: Option<Vec<ManifestEntry>> = doc.data.get("manifest").and_then(|v| serde_json::from_value(v.clone()).ok());
            Ok::<ModuleVersionData, unofficial_appwrite::error::Error>(ModuleVersionData { version, hcl, manifest })
        };
        let out = self.rt.block_on(fut)?;
        Ok(out)
    }
}

pub struct AppwriteRemoteProvider { client: AppwriteClient }

impl AppwriteRemoteProvider {
    pub fn try_from_env() -> Option<Self> {
        let cfg = AppwriteConfig::from_env()?;
        Some(Self { client: AppwriteClient::new(cfg) })
    }
}

impl vysma_hcl::hcl::remote::RemoteModuleProvider for AppwriteRemoteProvider {
    fn get_module(&self, username: &str, name: &str, version: Option<&str>) -> anyhow::Result<vysma_hcl::hcl::remote::RemoteModule> {
        let module_id = self.client.get_module_id(username, name)?;
        let mv = if let Some(v) = version { self.client.get_specific_version(&module_id, v)? } else { self.client.get_latest_version(&module_id)? };
        Ok(vysma_hcl::hcl::remote::RemoteModule { hcl: mv.hcl, manifest: mv.manifest.map(|m| m.into_iter().map(|e| vysma_hcl::hcl::remote::ManifestEntry { original_path: e.original_path, sha256: e.sha256, size: e.size, content_type: e.content_type, url_path: e.url_path }).collect()) })
    }
}

pub fn install_appwrite_provider_if_env(app: &mut App) {
    if let Some(p) = AppwriteRemoteProvider::try_from_env() {
        app.insert_resource(vysma_hcl::hcl::remote::RemoteModuleProviderResource(Box::new(p)));
    }
    // Also configure remote asset base URL if provided
    if let Ok(base) = std::env::var("VYSMA_ASSET_BASE_URL") {
        if !base.is_empty() {
            app.insert_resource(vysma_hcl::hcl::remote::AssetBaseUrl(base));
        }
    }
} 