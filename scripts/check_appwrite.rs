use anyhow::Context;
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Clone)]
struct Cfg {
    endpoint: String,
    project: String,
    key: String,
    database: String,
    modules_col: String,
}

fn cfg_from_env() -> anyhow::Result<Cfg> {
    let get = |k: &str| std::env::var(k).with_context(|| format!("missing env {k}"));
    Ok(Cfg {
        endpoint: get("APPWRITE_ENDPOINT")?,
        project: get("APPWRITE_PROJECT_ID")?,
        key: get("APPWRITE_API_KEY")?,
        database: get("APPWRITE_DATABASE_ID")?,
        modules_col: get("APPWRITE_MODULES_COLLECTION_ID")?,
    })
}

#[derive(Deserialize)]
struct Health { status: String }

#[derive(Deserialize)]
struct ListResp<T> { documents: Vec<T> }

#[derive(Deserialize)]
struct DocId { #[serde(rename = "$id")] id: String }

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let cfg = cfg_from_env()?;
    let http = Client::builder().build()?;

    // // Health
    // let health_url = format!("{}/v1/health", cfg.endpoint);
    // println!("Health URL: {}", health_url);
    // let health: Health = http.get(&health_url).send()?.error_for_status()?.json()?;
    // println!("Health: {}", health.status);

    // Auth check: list first doc (or empty)
    let url = format!(
        "{}/databases/{}/collections/{}/documents",
        cfg.endpoint, cfg.database, cfg.modules_col
    );
    let resp: ListResp<DocId> = http
        .get(&url)
        .header("X-Appwrite-Project", &cfg.project)
        .header("X-Appwrite-Key", &cfg.key)
        .header("X-Appwrite-Response-Format", "1.7.0")
        .header("Content-Type", "application/json")
        .query(&[("limit", "1".to_string())])
        .send()?
        .error_for_status()?
        .json()?;
    println!("Modules collection reachable. count(sample): {}", resp.documents.len());
    Ok(())
} 