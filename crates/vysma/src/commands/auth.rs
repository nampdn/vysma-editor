use anyhow::Result;
use clap::Args;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct LoginArgs {
    /// Appwrite endpoint URL
    #[arg(long)]
    pub endpoint: String,
    
    /// Appwrite project ID
    #[arg(long)]
    pub project: String,
    
    /// Profile name (dev|prod)
    #[arg(long, default_value = "dev")]
    pub profile: String,
    
    /// API key for authentication
    #[arg(long)]
    pub key: String,
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
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }
    
    pub fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".vysma").join("config.toml"))
    }
    
    pub fn get_profile(&self, name: &str) -> Option<&AuthProfile> {
        self.profiles.get(name)
    }
    
    pub fn set_profile(&mut self, name: String, profile: AuthProfile) {
        self.profiles.insert(name, profile);
    }
}

pub fn login(args: LoginArgs) -> Result<()> {
    println!("Logging in to Appwrite...");
    println!("Endpoint: {}", args.endpoint);
    println!("Project: {}", args.project);
    println!("Profile: {}", args.profile);
    
    // Load existing config
    let mut config = AuthConfig::load()?;
    
    // Create new profile
    let profile = AuthProfile {
        endpoint: args.endpoint,
        project: args.project,
        key: args.key,
        jwt: None, // Will be obtained on first use
    };
    
    // Save profile
    config.set_profile(args.profile.clone(), profile);
    config.save()?;
    
    println!("✅ Profile '{}' saved successfully!", args.profile);
    println!("Configuration saved to: {}", AuthConfig::config_path()?.display());
    
    Ok(())
}

pub fn logout(profile: &str) -> Result<()> {
    let mut config = AuthConfig::load()?;
    
    if config.profiles.remove(profile).is_some() {
        config.save()?;
        println!("✅ Profile '{}' removed successfully!", profile);
    } else {
        println!("⚠️  Profile '{}' not found", profile);
    }
    
    Ok(())
}

pub fn list_profiles() -> Result<()> {
    let config = AuthConfig::load()?;
    
    if config.profiles.is_empty() {
        println!("No profiles configured");
        return Ok(());
    }
    
    println!("Configured profiles:");
    for (name, profile) in &config.profiles {
        println!("  {}:", name);
        println!("    Endpoint: {}", profile.endpoint);
        println!("    Project: {}", profile.project);
        println!("    JWT: {}", profile.jwt.as_deref().unwrap_or("Not set"));
    }
    
    Ok(())
}
