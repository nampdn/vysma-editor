use bevy::prelude::*;

use crate::hcl::{
    loader::parse_hcl_to_asset,
    module_loader::ModuleLoader,
    schema::{ModuleImport, SceneDoc},
};

pub trait RemoteModuleProvider: Send + Sync + 'static {
    fn get_module_hcl(&self, username: &str, name: &str, version: Option<&str>) -> anyhow::Result<String>;
}

#[derive(Resource)]
pub struct RemoteModuleProviderResource(pub Box<dyn RemoteModuleProvider>);

/// Merge remote modules (username::module[@version]) into the working document.
/// Only processes imports with empty `path` and `name` containing `::`.
pub fn merge_remote_modules(
    target: &mut SceneDoc,
    loader: &ModuleLoader,
    provider: Option<&RemoteModuleProviderResource>,
) -> anyhow::Result<()> {
    let Some(provider) = provider else { return Ok(()); };
    // Take a snapshot of imports; we'll append resolved pieces into target
    let imports: Vec<ModuleImport> = target.modules.clone();
    for import in imports.iter() {
        if !import.path.is_empty() { continue; }
        if !import.name.contains("::") { continue; }
        let mut parts = import.name.split("::");
        let username = parts.next().unwrap_or("");
        let module = parts.next().unwrap_or("");
        if username.is_empty() || module.is_empty() { continue; }
        let hcl = provider.0.get_module_hcl(username, module, import.version.as_deref())?;
        let dep = parse_hcl_to_asset(&hcl)?.doc;
        // Reuse existing namespacing/merge rules from the loader
        loader.merge_module_import(target, &dep, import)?;
    }
    Ok(())
} 