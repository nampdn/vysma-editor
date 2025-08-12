use bevy::prelude::*;
use std::collections::HashMap;
use crate::hcl::{
    loader::HclSceneAsset,
    schema::{SceneDoc, ModuleImport},
};

/// Module loader that handles cross-file imports and dependency resolution
#[derive(Resource, Default)]
pub struct ModuleLoader {
    loaded_modules: HashMap<String, SceneDoc>,
    module_paths: HashMap<String, String>,
}

impl ModuleLoader {
    pub fn new() -> Self {
        Self { loaded_modules: HashMap::new(), module_paths: HashMap::new() }
    }

    /// Load a module and all its dependencies
    pub fn load_module(
        &mut self,
        module_name: &str,
        asset_server: &AssetServer,
        assets: &Assets<HclSceneAsset>,
    ) -> anyhow::Result<SceneDoc> {
        // Check if already loaded
        if let Some(doc) = self.loaded_modules.get(module_name) { return Ok(doc.clone()); }

        // Find the module path
        let module_path = self.module_paths.get(module_name)
            .ok_or_else(|| anyhow::anyhow!("Module path not found for: {}", module_name))?;

        // Load the module asset
        let handle = asset_server.load::<HclSceneAsset>(module_path);
        let asset = assets.get(&handle)
            .ok_or_else(|| anyhow::anyhow!("Failed to load module asset: {}", module_path))?;

        let mut doc = asset.doc.clone();

        // First, gather imported modules to avoid borrow issues
        let imports: Vec<ModuleImport> = doc.modules.clone();
        let mut dep_docs: Vec<(ModuleImport, SceneDoc)> = Vec::new();
        for import in &imports {
            let dep_doc = self.load_module(&import.name, asset_server, assets)?;
            dep_docs.push((import.clone(), dep_doc));
        }
        // Now merge dependencies
        for (import, dep_doc) in dep_docs { self.merge_module_import(&mut doc, &dep_doc, &import)?; }

        // Cache the loaded module
        self.loaded_modules.insert(module_name.to_string(), doc.clone());
        Ok(doc)
    }

    /// Merge a module import into the current document
    pub fn merge_module_import(&self, target_doc: &mut SceneDoc, source_doc: &SceneDoc, import: &ModuleImport) -> anyhow::Result<()> {
        // Merge prefabs with namespace
        let namespace = import.alias.as_ref().unwrap_or(&import.name);
        for prefab in &source_doc.prefab {
            if let Some(export) = source_doc.exports.iter().find(|e| e.name == import.name) {
                if export.prefabs.contains(&prefab.name) {
                    let mut namespaced_prefab = prefab.clone();
                    namespaced_prefab.name = format!("{}::{}", namespace, prefab.name);
                    target_doc.prefab.push(namespaced_prefab);
                }
            }
        }
        // Merge entities with namespace
        for entity in &source_doc.entity {
            if let Some(export) = source_doc.exports.iter().find(|e| e.name == import.name) {
                if export.entities.contains(&entity.name.as_ref().unwrap_or(&"".to_string())) {
                    let mut namespaced_entity = entity.clone();
                    if let Some(name) = &mut namespaced_entity.name { *name = format!("{}::{}", namespace, name); }
                    target_doc.entity.push(namespaced_entity);
                }
            }
        }
        // Merge triggers with namespace
        for trigger in &source_doc.triggers {
            if let Some(export) = source_doc.exports.iter().find(|e| e.name == import.name) {
                if export.triggers.contains(&trigger.name.as_ref().unwrap_or(&"".to_string())) {
                    let mut namespaced_trigger = trigger.clone();
                    if let Some(name) = &mut namespaced_trigger.name { *name = format!("{}::{}", namespace, name); }
                    target_doc.triggers.push(namespaced_trigger);
                }
            }
        }
        // Merge variables with namespace
        for (var_name, var_value) in &source_doc.vars {
            if let Some(export) = source_doc.exports.iter().find(|e| e.name == import.name) {
                if export.vars.contains(var_name) {
                    let namespaced_name = format!("{}::{}", namespace, var_name);
                    target_doc.vars.insert(namespaced_name, *var_value);
                }
            }
        }
        // Merge assets
        if let Some(source_assets) = &source_doc.assets {
            if target_doc.assets.is_none() {
                target_doc.assets = Some(source_assets.clone());
            } else {
                let target_assets = target_doc.assets.as_mut().unwrap();
                // Merge meshes with namespace
                for mesh in &source_assets.mesh {
                    let namespaced_name = format!("{}::{}", namespace, mesh.name);
                    let mut namespaced_mesh = mesh.clone();
                    namespaced_mesh.name = namespaced_name;
                    target_assets.mesh.push(namespaced_mesh);
                }
                // Merge materials with namespace
                for material in &source_assets.material {
                    let namespaced_name = format!("{}::{}", namespace, material.name);
                    let mut namespaced_material = material.clone();
                    namespaced_material.name = namespaced_name;
                    target_assets.material.push(namespaced_material);
                }
                // Merge other assets...
                for image in &source_assets.image {
                    let namespaced_name = format!("{}::{}", namespace, image.name);
                    let mut namespaced_image = image.clone();
                    namespaced_image.name = namespaced_name;
                    target_assets.image.push(namespaced_image);
                }
                for gltf in &source_assets.gltf {
                    let namespaced_name = format!("{}::{}", namespace, gltf.name);
                    let mut namespaced_gltf = gltf.clone();
                    namespaced_gltf.name = namespaced_name;
                    target_assets.gltf.push(namespaced_gltf);
                }
            }
        }
        Ok(())
    }

    /// Register a module path for loading
    pub fn register_module_path(&mut self, module_name: String, path: String) { self.module_paths.insert(module_name, path); }
    /// Get all loaded modules
    pub fn get_loaded_modules(&self) -> &HashMap<String, SceneDoc> { &self.loaded_modules }
    /// Clear loaded modules cache
    pub fn clear_cache(&mut self) { self.loaded_modules.clear(); }
    /// Check if a module is loaded
    pub fn is_module_loaded(&self, module_name: &str) -> bool { self.loaded_modules.contains_key(module_name) }
    /// Get module dependencies
    pub fn get_module_dependencies(&self, module_name: &str) -> Vec<String> {
        if let Some(doc) = self.loaded_modules.get(module_name) { doc.modules.iter().map(|m| m.name.clone()).collect() } else { Vec::new() }
    }
}

/// Plugin for the module loader system
pub struct ModuleLoaderPlugin;

impl Plugin for ModuleLoaderPlugin {
    fn build(&self, app: &mut App) { app.init_resource::<ModuleLoader>(); }
} 