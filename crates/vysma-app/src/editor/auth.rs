use anyhow::Result;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::protocol::HclUpdateRequest;

#[derive(Resource, Debug)]
pub struct EditorAuth {
    pub enabled: bool,
    pub project_id: Option<String>,
    pub jwks: Option<vysma_cloud::jwt::Jwks>,
    pub endpoint: Option<String>,
}

impl Default for EditorAuth {
    fn default() -> Self {
        Self {
            enabled: false,
            project_id: None,
            jwks: None,
            endpoint: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthProfile {
    pub endpoint: String,
    pub project: String,
    pub key: String,
    pub jwt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub profiles: HashMap<String, AuthProfile>,
}

impl AuthConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self {
                profiles: HashMap::new(),
            })
        }
    }
    
    pub fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".vysma").join("config.toml"))
    }
    
    pub fn get_profile(&self, name: &str) -> Option<&AuthProfile> {
        self.profiles.get(name)
    }
}

pub struct EditorAuthPlugin;

impl Plugin for EditorAuthPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorAuth>()
            .add_systems(Startup, setup_editor_auth)
            .add_systems(Update, verify_editor_updates);
    }
}

fn setup_editor_auth(mut auth: ResMut<EditorAuth>) {
    // Try to load auth config and enable JWT verification if available
    if let Ok(config) = AuthConfig::load() {
        // For now, use the first available profile
        if let Some((_, profile)) = config.profiles.iter().next() {
            info!("Editor auth enabled for project: {}", profile.project);
            auth.enabled = true;
            auth.project_id = Some(profile.project.clone());
            auth.endpoint = Some(profile.endpoint.clone());
            
            // Fetch JWKS in background
            let endpoint = profile.endpoint.clone();
            tokio::spawn(async move {
                if let Ok(jwks) = vysma_cloud::jwt::fetch_jwks(&endpoint).await {
                    info!("JWKS fetched successfully");
                    // TODO: Store JWKS in resource
                } else {
                    warn!("Failed to fetch JWKS");
                }
            });
        }
    }
    
    if !auth.enabled {
        info!("Editor auth disabled - no profiles configured");
    }
}

fn verify_editor_updates(
    auth: Res<EditorAuth>,
    mut update_events: EventReader<HclUpdateRequest>,
) {
    if !auth.enabled {
        return; // Skip verification if auth is disabled
    }
    
    for update in update_events.read() {
        // Extract JWT from Authorization header if present
        if let Some(auth_header) = &update.authorization {
            if auth_header.starts_with("Bearer ") {
                let token = &auth_header[7..]; // Remove "Bearer " prefix
                
                // TODO: Verify JWT using stored JWKS
                info!("JWT token received: {}...", &token[..token.len().min(20)]);
                
                // For now, just log the token
                // In a real implementation, we would:
                // 1. Decode the JWT header to get the key ID
                // 2. Find the corresponding key in JWKS
                // 3. Verify the signature and claims
                // 4. Check project access
            } else {
                warn!("Invalid Authorization header format");
            }
        } else {
            warn!("No Authorization header - update rejected");
            // TODO: Reject the update
        }
    }
}
