use bevy::prelude::*;

use crate::hcl::{
	loader::parse_hcl_to_asset,
	module_loader::ModuleLoader,
	schema::{ModuleImport, SceneDoc},
};

#[derive(Clone, Debug)]
pub struct ManifestEntry {
	pub original_path: String,
	pub sha256: String,
	pub size: i64,
	pub content_type: Option<String>,
	pub url_path: String,
}

#[derive(Clone, Debug)]
pub struct RemoteModule {
	pub hcl: String,
	pub manifest: Option<Vec<ManifestEntry>>,
}

#[derive(Resource, Default)]
pub struct ManifestMap(pub std::collections::HashMap<String, String>);

#[derive(Resource, Default)]
pub struct AssetBaseUrl(pub String);

pub trait RemoteModuleProvider: Send + Sync + 'static {
	fn get_module_hcl(&self, username: &str, name: &str, version: Option<&str>) -> anyhow::Result<String> { let m = self.get_module(username, name, version)?; Ok(m.hcl) }
	fn get_module(&self, username: &str, name: &str, version: Option<&str>) -> anyhow::Result<RemoteModule>;
}

#[derive(Resource)]
pub struct RemoteModuleProviderResource(pub Box<dyn RemoteModuleProvider>);

/// Merge remote modules (username::module[@version]) into the working document.
/// Only processes imports with empty `path` and `name` containing `::`.
pub fn merge_remote_modules(
	target: &mut SceneDoc,
	loader: &ModuleLoader,
	provider: Option<&RemoteModuleProviderResource>,
	manifest_map: &mut ManifestMap,
	base_url: Option<&AssetBaseUrl>,
) -> anyhow::Result<()> {
	let Some(provider) = provider else { return Ok(()); };
	let base = base_url.map(|b| b.0.clone()).unwrap_or_default();
	// Take a snapshot of imports; we'll append resolved pieces into target
	let imports: Vec<ModuleImport> = target.modules.clone();
	for import in imports.iter() {
		if !import.path.is_empty() { continue; }
		if !import.name.contains("::") { continue; }
		let mut parts = import.name.split("::");
		let username = parts.next().unwrap_or("");
		let module = parts.next().unwrap_or("");
		if username.is_empty() || module.is_empty() { continue; }
		let remote = provider.0.get_module(username, module, import.version.as_deref())?;
		let dep = parse_hcl_to_asset(&remote.hcl)?.doc;
		// Populate manifest map if provided
		if let Some(mani) = remote.manifest.as_ref() {
			for e in mani {
				let url = if base.is_empty() { e.url_path.clone() } else { format!("{}/{}", base.trim_end_matches('/'), e.url_path) };
				manifest_map.0.insert(e.original_path.clone(), url);
			}
		}
		// Reuse existing namespacing/merge rules from the loader
		loader.merge_module_import(target, &dep, import)?;
	}
	Ok(())
} 