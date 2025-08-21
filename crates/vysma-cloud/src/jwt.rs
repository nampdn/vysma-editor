use anyhow::Result;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtHeader {
    pub alg: String,
    pub kid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub iss: String,
    pub aud: String,
    pub exp: u64,
    pub iat: u64,
    pub sub: String,
    #[serde(rename = "https://appwrite.io/v1/user")]
    pub user: Option<JwtUser>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtUser {
    pub id: String,
    pub email: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwk {
    pub kty: String,
    #[serde(rename = "use")]
    pub r#use: String,
    pub kid: String,
    pub n: String,
    pub e: String,
}

pub struct JwtVerifier {
    jwks: Jwks,
    project_id: String,
}

impl JwtVerifier {
    pub fn new(jwks: Jwks, project_id: String) -> Self {
        Self { jwks, project_id }
    }
    
    pub fn verify_token(&self, token: &str) -> Result<JwtClaims> {
        // Decode header to get key ID
        let header = decode_header(token)?;
        let kid = header.kid.ok_or_else(|| anyhow::anyhow!("No key ID in JWT header"))?;
        
        // Find the corresponding key
        let jwk = self.jwks.keys.iter()
            .find(|k| k.kid == kid)
            .ok_or_else(|| anyhow::anyhow!("Key ID not found in JWKS"))?;
        
        // Create decoding key from JWK
        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;
        
        // Set up validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.project_id]);
        validation.set_issuer(&["https://appwrite.io/v1"]);
        
        // Decode and verify token
        let token_data = decode::<JwtClaims>(token, &decoding_key, &validation)?;
        
        Ok(token_data.claims)
    }
    
    pub fn verify_project_access(&self, claims: &JwtClaims) -> Result<bool> {
        // Verify the audience matches our project
        if claims.aud != self.project_id {
            return Ok(false);
        }
        
        // Check if token is expired
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        if claims.exp < now {
            return Ok(false);
        }
        
        Ok(true)
    }
}

pub async fn fetch_jwks(endpoint: &str) -> Result<Jwks> {
    let jwks_url = format!("{}/.well-known/jwks.json", endpoint.trim_end_matches("/v1"));
    
    let response = reqwest::get(&jwks_url).await?;
    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch JWKS: {}", response.status());
    }
    
    let jwks: Jwks = response.json().await?;
    Ok(jwks)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_jwt_verifier_creation() {
        let jwks = Jwks { keys: vec![] };
        let verifier = JwtVerifier::new(jwks, "test-project".to_string());
        assert_eq!(verifier.project_id, "test-project");
    }
}
