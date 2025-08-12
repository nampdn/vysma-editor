use ahash::AHashMap as HashMap;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::hcl::schema::{ModuleExport, SceneDoc};

/// Registry for managing cross-file modules and their exports
#[derive(Resource, Default)]
pub struct ModuleRegistry {
    modules: HashMap<String, RegisteredModule>,
    module_cache: HashMap<String, SceneDoc>,
    dependency_graph: HashMap<String, HashSet<String>>,
}

#[derive(Debug, Clone)]
pub struct RegisteredModule {
    pub name: String,
    pub path: String,
    pub version: Option<String>,
    pub exports: ModuleExport,
    pub doc: SceneDoc,
    pub dependencies: HashSet<String>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self { modules: HashMap::new(), module_cache: HashMap::new(), dependency_graph: HashMap::new() }
    }

    /// Register a module with its exports
    pub fn register_module(&mut self, name: String, path: String, doc: SceneDoc) -> anyhow::Result<()> {
        let exports = doc.exports.iter().find(|e| e.name == name).cloned().unwrap_or_else(|| ModuleExport {
            name: name.clone(), prefabs: vec![], entities: vec![], triggers: vec![], vars: vec![], public: true, category: None, description: None, version: None,
        });
        let dependencies = doc.modules.iter().map(|m| m.name.clone()).collect();
        let module = RegisteredModule { name: name.clone(), path, version: None, exports, doc, dependencies };
        self.modules.insert(name.clone(), module);
        self.build_dependency_graph();
        Ok(())
    }

    /// Get a module by name
    pub fn get_module(&self, name: &str) -> Option<&RegisteredModule> { self.modules.get(name) }
    /// Get all public modules
    pub fn get_public_modules(&self) -> Vec<&RegisteredModule> { self.modules.values().filter(|m| m.exports.public).collect() }

    /// Check if a module has a specific export
    pub fn has_export(&self, module_name: &str, export_type: &str, export_name: &str) -> bool {
        if let Some(module) = self.modules.get(module_name) {
            match export_type {
                "prefab" => module.exports.prefabs.contains(&export_name.to_string()),
                "entity" => module.exports.entities.contains(&export_name.to_string()),
                "trigger" => module.exports.triggers.contains(&export_name.to_string()),
                "var" => module.exports.vars.contains(&export_name.to_string()),
                _ => false,
            }
        } else { false }
    }

    /// Resolve module dependencies and return them in dependency order
    pub fn resolve_dependencies(&self, module_name: &str) -> anyhow::Result<Vec<String>> {
        let mut resolved = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();
        self.visit_module(module_name, &mut resolved, &mut visited, &mut temp_visited)?;
        Ok(resolved)
    }

    fn visit_module(&self, module_name: &str, resolved: &mut Vec<String>, visited: &mut HashSet<String>, temp_visited: &mut HashSet<String>) -> anyhow::Result<()> {
        if temp_visited.contains(module_name) { return Err(anyhow::anyhow!("Circular dependency detected: {}", module_name)); }
        if visited.contains(module_name) { return Ok(()); }
        temp_visited.insert(module_name.to_string());
        if let Some(module) = self.modules.get(module_name) { for dep in &module.dependencies { self.visit_module(dep, resolved, visited, temp_visited)?; } }
        temp_visited.remove(module_name);
        visited.insert(module_name.to_string());
        resolved.push(module_name.to_string());
        Ok(())
    }

    /// Build dependency graph for cycle detection
    fn build_dependency_graph(&mut self) {
        self.dependency_graph.clear();
        for (name, module) in &self.modules { self.dependency_graph.insert(name.clone(), module.dependencies.clone()); }
    }

    /// Get module metadata for publishing ecosystem
    pub fn get_module_metadata(&self) -> Vec<ModuleMetadata> {
        self.modules.values().map(|m| ModuleMetadata {
            name: m.name.clone(), path: m.path.clone(), version: m.version.clone(), description: None, category: None, public: m.exports.public, export_counts: ExportCounts {
                prefabs: m.exports.prefabs.len(), entities: m.exports.entities.len(), triggers: m.exports.triggers.len(), vars: m.exports.vars.len(),
            },
        }).collect()
    }

    /// Search modules by category or tags
    pub fn search_modules(&self, query: &str, _category: Option<&str>) -> Vec<&RegisteredModule> {
        self.modules.values().filter(|m| {
            let matches_query = query.is_empty() || m.name.to_lowercase().contains(&query.to_lowercase());
            matches_query
        }).collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleMetadata {
    pub name: String,
    pub path: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub public: bool,
    pub export_counts: ExportCounts,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportCounts { pub prefabs: usize, pub entities: usize, pub triggers: usize, pub vars: usize }

/// Plugin for the module registry system
pub struct ModuleRegistryPlugin;

impl Plugin for ModuleRegistryPlugin { fn build(&self, app: &mut App) { app.init_resource::<ModuleRegistry>(); } } 