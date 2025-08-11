use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::hcl::module_registry::{ModuleRegistry, ModuleMetadata};

/// Module marketplace for discovering and installing published MOBA modules
#[derive(Resource, Default)]
pub struct ModuleMarketplace {
    available_modules: HashMap<String, PublishedModule>,
    installed_modules: HashMap<String, InstalledModule>,
    categories: HashMap<String, Vec<String>>,
    search_index: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishedModule {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub version: String,
    pub downloads: u32,
    pub rating: f32,
    pub price: Option<f32>,
    pub download_url: String,
    pub thumbnail_url: Option<String>,
    pub dependencies: Vec<String>,
    pub compatibility: CompatibilityInfo,
    pub changelog: Vec<ChangelogEntry>,
    pub reviews: Vec<ModuleReview>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledModule {
    pub id: String,
    pub name: String,
    pub version: String,
    pub install_date: String,
    pub update_available: bool,
    pub latest_version: String,
    pub enabled: bool,
    pub auto_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    pub min_version: String,
    pub max_version: Option<String>,
    pub required_modules: Vec<String>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    pub version: String,
    pub date: String,
    pub changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleReview {
    pub author: String,
    pub rating: u8,
    pub comment: String,
    pub date: String,
}

impl ModuleMarketplace {
    pub fn new() -> Self {
        Self {
            available_modules: HashMap::new(),
            installed_modules: HashMap::new(),
            categories: HashMap::new(),
            search_index: HashMap::new(),
        }
    }

    /// Discover available modules from remote sources
    pub async fn discover_modules(&mut self, remote_url: &str) -> anyhow::Result<Vec<PublishedModule>> {
        // This would typically make HTTP requests to module repositories
        // For now, we'll simulate with some example modules
        
        let example_modules = vec![
            PublishedModule {
                id: "axe_hero_v1".to_string(),
                name: "Axe Hero".to_string(),
                author: "MOBA_Dev".to_string(),
                description: "Axe the Axe - Strength hero with tanking abilities".to_string(),
                category: "hero".to_string(),
                tags: vec!["strength", "tank", "initiator"].into_iter().map(|s| s.to_string()).collect(),
                version: "1.0.0".to_string(),
                downloads: 1250,
                rating: 4.8,
                price: None,
                download_url: "https://modules.moba.dev/axe_hero_v1.hcl".to_string(),
                thumbnail_url: Some("https://modules.moba.dev/thumbnails/axe.png".to_string()),
                dependencies: vec!["moba_core_v1".to_string()],
                compatibility: CompatibilityInfo {
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    required_modules: vec!["moba_core".to_string()],
                    conflicts: vec![],
                },
                changelog: vec![
                    ChangelogEntry {
                        version: "1.0.0".to_string(),
                        date: "2024-01-15".to_string(),
                        changes: vec!["Initial release".to_string(), "All abilities implemented".to_string()],
                    }
                ],
                reviews: vec![
                    ModuleReview {
                        author: "Player123".to_string(),
                        rating: 5,
                        comment: "Great hero implementation!".to_string(),
                        date: "2024-01-16".to_string(),
                    }
                ],
            },
            PublishedModule {
                id: "moba_core_v1".to_string(),
                name: "MOBA Core".to_string(),
                author: "MOBA_Dev".to_string(),
                description: "Core MOBA game mechanics and systems".to_string(),
                category: "core".to_string(),
                tags: vec!["core", "mechanics", "systems"].into_iter().map(|s| s.to_string()).collect(),
                version: "1.0.0".to_string(),
                downloads: 5000,
                rating: 4.9,
                price: None,
                download_url: "https://modules.moba.dev/moba_core_v1.hcl".to_string(),
                thumbnail_url: Some("https://modules.moba.dev/thumbnails/core.png".to_string()),
                dependencies: vec![],
                compatibility: CompatibilityInfo {
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    required_modules: vec![],
                    conflicts: vec![],
                },
                changelog: vec![
                    ChangelogEntry {
                        version: "1.0.0".to_string(),
                        date: "2024-01-10".to_string(),
                        changes: vec!["Initial release".to_string(), "Basic MOBA systems".to_string()],
                    }
                ],
                reviews: vec![
                    ModuleReview {
                        author: "Dev456".to_string(),
                        rating: 5,
                        comment: "Essential foundation for MOBA games".to_string(),
                        date: "2024-01-12".to_string(),
                    }
                ],
            }
        ];

        for module in &example_modules {
            self.available_modules.insert(module.id.clone(), module.clone());
            self.index_module(module);
        }

        Ok(example_modules)
    }

    /// Index a module for search functionality
    fn index_module(&mut self, module: &PublishedModule) {
        // Index by category
        self.categories.entry(module.category.clone())
            .or_insert_with(Vec::new)
            .push(module.id.clone());

        // Index by tags
        for tag in &module.tags {
            self.search_index.entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(module.id.clone());
        }

        // Index by name
        let words: Vec<&str> = module.name.split_whitespace().collect();
        for word in words {
            self.search_index.entry(word.to_lowercase())
                .or_insert_with(Vec::new)
                .push(module.id.clone());
        }
    }

    /// Search for modules
    pub fn search_modules(&self, query: &str, category: Option<&str>, tags: Option<&[String]>) -> Vec<&PublishedModule> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for (id, module) in &self.available_modules {
            let mut matches = false;

            // Text search
            if query.is_empty() || 
               module.name.to_lowercase().contains(&query_lower) ||
               module.description.to_lowercase().contains(&query_lower) ||
               module.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower)) {
                matches = true;
            }

            // Category filter
            if let Some(cat) = category {
                if module.category != cat {
                    matches = false;
                }
            }

            // Tags filter
            if let Some(required_tags) = tags {
                if !required_tags.iter().all(|tag| module.tags.contains(tag)) {
                    matches = false;
                }
            }

            if matches {
                results.push(module);
            }
        }

        // Sort by rating and downloads
        results.sort_by(|a, b| {
            b.rating.partial_cmp(&a.rating)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.downloads.cmp(&a.downloads))
        });

        results
    }

    /// Install a module
    pub async fn install_module(&mut self, module_id: &str, registry: &mut ModuleRegistry) -> anyhow::Result<()> {
        let module = self.available_modules.get(module_id)
            .ok_or_else(|| anyhow::anyhow!("Module not found: {}", module_id))?;

        // Check compatibility
        self.check_compatibility(module, registry)?;

        // Download and install
        let installed = InstalledModule {
            id: module.id.clone(),
            name: module.name.clone(),
            version: module.version.clone(),
            install_date: chrono::Utc::now().to_rfc3339(),
            update_available: false,
            latest_version: module.version.clone(),
            enabled: true,
            auto_update: true,
        };

        self.installed_modules.insert(module_id.to_string(), installed);

        // Register with module registry
        // This would typically involve downloading the actual HCL file
        // and registering it with the ModuleRegistry

        Ok(())
    }

    /// Check module compatibility
    fn check_compatibility(&self, module: &PublishedModule, registry: &ModuleRegistry) -> anyhow::Result<()> {
        // Check required modules
        for required in &module.compatibility.required_modules {
            if !registry.get_module(required).is_some() {
                return Err(anyhow::anyhow!("Required module not found: {}", required));
            }
        }

        // Check conflicts
        for conflict in &module.compatibility.conflicts {
            if registry.get_module(conflict).is_some() {
                return Err(anyhow::anyhow!("Conflicting module found: {}", conflict));
            }
        }

        Ok(())
    }

    /// Update a module
    pub async fn update_module(&mut self, module_id: &str) -> anyhow::Result<()> {
        let module = self.available_modules.get(module_id)
            .ok_or_else(|| anyhow::anyhow!("Module not found: {}", module_id))?;

        if let Some(installed) = self.installed_modules.get_mut(module_id) {
            if installed.version != module.version {
                // Perform update
                installed.version = module.version.clone();
                installed.update_available = false;
                installed.latest_version = module.version.clone();
            }
        }

        Ok(())
    }

    /// Uninstall a module
    pub fn uninstall_module(&mut self, module_id: &str) -> anyhow::Result<()> {
        self.installed_modules.remove(module_id)
            .ok_or_else(|| anyhow::anyhow!("Module not installed: {}", module_id))?;

        Ok(())
    }

    /// Get module categories
    pub fn get_categories(&self) -> Vec<&String> {
        self.categories.keys().collect()
    }

    /// Get modules by category
    pub fn get_modules_by_category(&self, category: &str) -> Vec<&PublishedModule> {
        if let Some(module_ids) = self.categories.get(category) {
            module_ids.iter()
                .filter_map(|id| self.available_modules.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get installed modules
    pub fn get_installed_modules(&self) -> Vec<&InstalledModule> {
        self.installed_modules.values().collect()
    }

    /// Check for updates
    pub fn check_for_updates(&mut self) -> Vec<&PublishedModule> {
        let mut updates = Vec::new();

        for (id, installed) in &self.installed_modules {
            if let Some(available) = self.available_modules.get(id) {
                if installed.version != available.version {
                    updates.push(available);
                }
            }
        }

        updates
    }
}

/// Plugin for the module marketplace system
pub struct ModuleMarketplacePlugin;

impl Plugin for ModuleMarketplacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModuleMarketplace>();
    }
} 