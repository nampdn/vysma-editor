use crate::hcl::schema::SceneDoc;
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
};
use thiserror::Error;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct HclSceneAsset {
    pub doc: SceneDoc,
}

#[derive(Default)]
pub struct HclLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum HclLoaderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HCL parse error: {0}")]
    Hcl(#[from] hcl::error::Error),
}

impl AssetLoader for HclLoader {
    type Asset = HclSceneAsset;
    type Settings = ();
    type Error = HclLoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _ctx: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let doc: SceneDoc = hcl::from_slice(&bytes)?;
        Ok(HclSceneAsset { doc })
    }
    fn extensions(&self) -> &[&str] {
        &["hcl", "hclscene"]
    }
}
