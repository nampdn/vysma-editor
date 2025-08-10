use crate::hcl::schema::{AssetsBlock, EntityDecl, GltfAsset, ImageAsset, MaterialAsset, MeshAsset, MeshKind, PbrMat, Prefab, SceneDoc};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
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
        let s = String::from_utf8_lossy(&bytes);
        let body: hcl::Body = hcl::from_str(&s)?;
        let doc = normalize_hcl_to_scene(body)?;
        Ok(HclSceneAsset { doc })
    }
    fn extensions(&self) -> &[&str] {
        &["hcl", "hclscene"]
    }
}

fn normalize_hcl_to_scene(body: hcl::Body) -> Result<SceneDoc, HclLoaderError> {
    use hcl::{Attribute, Block};
    let mut assets_block: Option<AssetsBlock> = None;
    let mut prefabs: Vec<Prefab> = Vec::new();
    let mut entities: Vec<EntityDecl> = Vec::new();

    // 1) Handle attribute-style values if present
    if let Some(attr) = find_attr(&body, "assets") {
        if let Ok(a) = serde_json::from_value::<AssetsBlock>(expr_to_json(attr.expr())) {
            assets_block.get_or_insert(a);
        }
    }
    if let Some(attr) = find_attr(&body, "prefab") {
        if let Ok(p) = serde_json::from_value::<Vec<Prefab>>(expr_to_json(attr.expr())) {
            prefabs = p;
        }
    }
    if let Some(attr) = find_attr(&body, "entity") {
        if let Ok(e) = serde_json::from_value::<Vec<EntityDecl>>(expr_to_json(attr.expr())) {
            entities = e;
        }
    }

    // 2) Handle block-style declarations
    for b in body.blocks() {
        match b.identifier() {
            "assets" => {
                let mut a = assets_block.take().unwrap_or_default();
                collect_assets_from_assets_block(&mut a, b)?;
                assets_block = Some(a);
            }
            "prefab" => {
                prefabs.push(prefab_from_block(b)?);
            }
            "entity" => {
                entities.push(entity_from_block(b)?);
            }
            _ => {}
        }
    }

    Ok(SceneDoc {
        assets: assets_block,
        prefab: prefabs,
        entity: entities,
    })
}

fn find_attr<'a>(body: &'a hcl::Body, name: &str) -> Option<&'a hcl::Attribute> {
    body.attributes().find(|a| a.key() == name)
}

fn value_to_json(v: &hcl::Value) -> serde_json::Value {
    serde_json::to_value(v).unwrap_or(serde_json::Value::Null)
}

fn collect_assets_from_assets_block(dst: &mut AssetsBlock, block: &hcl::Block) -> Result<(), HclLoaderError> {
    // Look for nested blocks: mesh/material/image/gltf
    for b in block.body().blocks() {
        match b.identifier() {
            "mesh" => dst.mesh.push(mesh_from_block(b)?),
            "material" => dst.material.push(material_from_block(b)?),
            "image" => dst.image.push(image_from_block(b)?),
            "gltf" => dst.gltf.push(gltf_from_block(b)?),
            _ => {}
        }
    }
    // Also support attribute-style arrays inside assets { mesh = [...], ... }
    if let Some(attr) = find_attr(block.body(), "mesh") {
        if let Ok(v) = serde_json::from_value::<Vec<MeshAsset>>(expr_to_json(attr.expr())) {
            dst.mesh.extend(v);
        }
    }
    if let Some(attr) = find_attr(block.body(), "material") {
        if let Ok(v) = serde_json::from_value::<Vec<MaterialAsset>>(expr_to_json(attr.expr())) {
            dst.material.extend(v);
        }
    }
    if let Some(attr) = find_attr(block.body(), "image") {
        if let Ok(v) = serde_json::from_value::<Vec<ImageAsset>>(expr_to_json(attr.expr())) {
            dst.image.extend(v);
        }
    }
    if let Some(attr) = find_attr(block.body(), "gltf") {
        if let Ok(v) = serde_json::from_value::<Vec<GltfAsset>>(expr_to_json(attr.expr())) {
            dst.gltf.extend(v);
        }
    }
    Ok(())
}

fn mesh_from_block(b: &hcl::Block) -> Result<MeshAsset, HclLoaderError> {
    let name = b
        .labels()
        .get(0)
        .map(|l| l.as_str().to_string())
        .unwrap_or_else(|| "mesh".into());
    let builtin = get_string(b.body(), "builtin").unwrap_or_else(|| "cube".into());
    Ok(MeshAsset { name, kind: MeshKind::Builtin { builtin } })
}

fn material_from_block(b: &hcl::Block) -> Result<MaterialAsset, HclLoaderError> {
    let name = b
        .labels()
        .get(0)
        .map(|l| l.as_str().to_string())
        .unwrap_or_else(|| "material".into());
    let mut pbr: Option<PbrMat> = None;
    if let Some(attr) = find_attr(b.body(), "pbr") {
        if let Ok(v) = serde_json::from_value::<PbrMat>(expr_to_json(attr.expr())) {
            pbr = Some(v);
        }
    }
    Ok(MaterialAsset { name, pbr })
}

fn image_from_block(b: &hcl::Block) -> Result<ImageAsset, HclLoaderError> {
    let name = b
        .labels()
        .get(0)
        .map(|l| l.as_str().to_string())
        .unwrap_or_else(|| "image".into());
    // Support either `file` or `url`; store into `file`
    let file = get_string(b.body(), "file").or_else(|| get_string(b.body(), "url")).unwrap_or_default();
    Ok(ImageAsset { name, file })
}

fn gltf_from_block(b: &hcl::Block) -> Result<GltfAsset, HclLoaderError> {
    let name = b
        .labels()
        .get(0)
        .map(|l| l.as_str().to_string())
        .unwrap_or_else(|| "gltf".into());
    let file = get_string(b.body(), "file").or_else(|| get_string(b.body(), "url")).unwrap_or_default();
    let node = get_string(b.body(), "node");
    Ok(GltfAsset { name, file, node })
}

fn prefab_from_block(b: &hcl::Block) -> Result<Prefab, HclLoaderError> {
    let name = b
        .labels()
        .get(0)
        .map(|l| l.as_str().to_string())
        .unwrap_or_else(|| "Prefab".into());
    let components = if let Some(attr) = find_attr(b.body(), "components") {
        expr_to_json(attr.expr())
    } else {
        serde_json::json!({})
    };
    Ok(Prefab { name, components })
}

fn entity_from_block(b: &hcl::Block) -> Result<EntityDecl, HclLoaderError> {
    let mut ent = EntityDecl::default();
    ent.name = b.labels().get(0).map(|l| l.as_str().to_string());
    if let Some(attr) = find_attr(b.body(), "include") {
        if let Ok(v) = serde_json::from_value::<Vec<String>>(expr_to_json(attr.expr())) {
            ent.include = v;
        }
    }
    if let Some(attr) = find_attr(b.body(), "components") {
        ent.components = expr_to_json(attr.expr());
    }
    if let Some(attr) = find_attr(b.body(), "children") {
        if let Ok(v) = serde_json::from_value::<Vec<EntityDecl>>(expr_to_json(attr.expr())) {
            ent.children = v;
        }
    }
    // Also support nested entity blocks as children
    for cb in b.body().blocks().filter(|x| x.identifier() == "entity") {
        ent.children.push(entity_from_block(cb)?);
    }
    Ok(ent)
}

fn get_string(body: &hcl::Body, key: &str) -> Option<String> {
    find_attr(body, key).and_then(|a| match a.expr().clone().into() {
        hcl::Value::String(s) => Some(s),
        _ => None,
    })
}

fn expr_to_json(e: &hcl::Expression) -> serde_json::Value {
    let v: hcl::Value = e.clone().into();
    value_to_json(&v)
}
