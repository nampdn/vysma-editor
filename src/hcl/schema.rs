// use serde::Deserialize;
// use indexmap::IndexMap;

// #[derive(Debug, Deserialize, Clone)]
// pub struct SceneDoc {
//     #[serde(default)] pub assets: Option<AssetsBlock>,
//     #[serde(default)] pub prefab: Vec<Prefab>,
//     #[serde(default)] pub entity: Vec<EntityDecl>,
// }

// #[derive(Debug, Deserialize, Clone, Default)]
// pub struct AssetsBlock {
//     #[serde(default)] pub mesh: Vec<MeshAsset>,
//     #[serde(default)] pub material: Vec<MaterialAsset>,
//     #[serde(default)] pub image: Vec<ImageAsset>,
//     #[serde(default)] pub gltf: Vec<GltfAsset>,
// }

// #[derive(Debug, Deserialize, Clone)]
// pub struct MeshAsset { pub name: String, #[serde(flatten)] pub kind: MeshKind }

// #[derive(Debug, Deserialize, Clone)]
// #[serde(untagged)]
// pub enum MeshKind { Builtin { builtin: String }, }

// #[derive(Debug, Deserialize, Clone)]
// pub struct MaterialAsset { pub name: String, #[serde(default)] pub pbr: Option<PbrMat> }

// #[derive(Debug, Deserialize, Clone, Default)]
// pub struct PbrMat {
//     pub base_color: Option<ColorDef>,
//     pub metallic: Option<f32>,
//     pub roughness: Option<f32>,
//     pub emissive: Option<ColorDef>,
// }

// #[derive(Debug, Deserialize, Clone)]
// pub struct ImageAsset { pub name: String, pub file: String }

// #[derive(Debug, Deserialize, Clone)]
// pub struct GltfAsset { pub name: String, pub file: String, pub node: Option<String> }

// #[derive(Debug, Deserialize, Clone, Default)]
// pub struct Prefab { pub name: String, #[serde(default)] pub components: serde_json::Value }

// #[derive(Debug, Deserialize, Clone, Default)]
// pub struct EntityDecl {
//     pub name: Option<String>,
//     #[serde(default)] pub include: Vec<String>,
//     #[serde(default)] pub components: serde_json::Value,
//     #[serde(default)] pub children: Vec<EntityDecl>,
//     #[serde(default)] pub tags: Vec<String>,
// }

// #[derive(Debug, Deserialize, Clone, Default)]
// pub struct TransformDef {
//     #[serde(default)] pub t: Option<[f32; 3]>,
//     #[serde(default)] pub s: Option<[f32; 3]>,
//     #[serde(default)] pub r: Option<[f32; 4]>,
//     #[serde(default)] pub euler: Option<EulerDef>,
//     #[serde(default)] pub look_at: Option<[f32; 3]>,
// }

// #[derive(Debug, Deserialize, Clone, Default)]
// pub struct EulerDef { pub x: f32, pub y: f32, pub z: f32 }

// #[derive(Debug, Deserialize, Clone)]
// #[serde(untagged)]
// pub enum ColorDef { Hex(String), Rgba { r: f32, g: f32, b: f32, #[serde(default)] a: Option<f32> } }

// impl Default for ColorDef { fn default() -> Self { ColorDef::Hex("#ffffff".into()) } }

use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct SceneDoc {
    #[serde(default)]
    pub assets: Option<AssetsBlock>,
    #[serde(default)]
    pub prefab: Vec<Prefab>,
    #[serde(default)]
    pub entity: Vec<EntityDecl>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AssetsBlock {
    #[serde(default)]
    pub mesh: Vec<MeshAsset>,
    #[serde(default)]
    pub material: Vec<MaterialAsset>,
    #[serde(default)]
    pub image: Vec<ImageAsset>,
    #[serde(default)]
    pub gltf: Vec<GltfAsset>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MeshAsset {
    pub name: String,
    #[serde(flatten)]
    pub kind: MeshKind,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum MeshKind {
    Builtin { builtin: String },
}

#[derive(Debug, Deserialize, Clone)]
pub struct MaterialAsset {
    pub name: String,
    #[serde(default)]
    pub pbr: Option<PbrMat>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct PbrMat {
    pub base_color: Option<ColorDef>,
    pub metallic: Option<f32>,
    pub roughness: Option<f32>,
    pub emissive: Option<ColorDef>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ImageAsset {
    pub name: String,
    pub file: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GltfAsset {
    pub name: String,
    pub file: String,
    pub node: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Prefab {
    pub name: String,
    #[serde(default)]
    pub components: serde_json::Value,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct EntityDecl {
    pub name: Option<String>,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub components: serde_json::Value,
    #[serde(default)]
    pub children: Vec<EntityDecl>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TransformDef {
    #[serde(default)]
    pub t: Option<[f32; 3]>,
    #[serde(default)]
    pub s: Option<[f32; 3]>,
    #[serde(default)]
    pub r: Option<[f32; 4]>,
    #[serde(default)]
    pub euler: Option<EulerDef>,
    #[serde(default)]
    pub look_at: Option<[f32; 3]>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct EulerDef {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ColorDef {
    Hex(String),
    Rgba {
        r: f32,
        g: f32,
        b: f32,
        #[serde(default)]
        a: Option<f32>,
    },
}

impl Default for ColorDef {
    fn default() -> Self {
        ColorDef::Hex("#ffffff".into())
    }
}
