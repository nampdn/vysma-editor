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
    #[serde(default)]
    pub triggers: Vec<TriggerDecl>,
    #[serde(default)]
    pub vars: IndexMap<String, f64>,
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
    #[serde(default)]
    pub persist_key: Option<String>,
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

// ---- Triggers / Events / Conditions / Actions ----

#[derive(Debug, Deserialize, Clone)]
pub struct TriggerDecl {
    #[serde(default)]
    pub name: Option<String>,
    pub on: EventDef,
    #[serde(default)]
    pub when: Vec<ConditionDef>,
    #[serde(default)]
    pub actions: Vec<ActionDef>,
    #[serde(default)]
    pub target: Option<Selector>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum EventDef {
    KeyPressed { key_pressed: String },
    KeyHeld { key_held: String },
    Tick { tick: TickDef },
    Startup { startup: bool },
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TickDef { pub every: f32 }

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ConditionDef {
    Any { any_visible: Selector },
    All { all_visible: Selector },
    Not { not: Box<ConditionDef> },
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Selector {
    Name { name: String },
    Tag { tag: String },
    All { all: bool },
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ActionDef {
    ToggleVisibility { toggle_visibility: ActionTarget },
    SetVisibility { set_visibility: VisibilitySet },
    Translate { translate: TranslateDef },
    RotateEuler { rotate_euler: RotateDef },
    SetMaterial { set_material: MaterialSet },
    Spawn { spawn: SpawnDef },
    Despawn { despawn: ActionTarget },
    SetVar { set_var: VarSet },
    AddVar { add_var: VarDelta },
    MulVar { mul_var: VarDelta },
    TranslateAxis { translate_axis: TranslateAxisDef },
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ActionTarget { pub targets: Option<Selector> }

#[derive(Debug, Deserialize, Clone, Default)]
pub struct VisibilitySet { pub targets: Option<Selector>, pub value: Option<String> }

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TranslateDef { pub targets: Option<Selector>, pub by: [f32; 3] }

#[derive(Debug, Deserialize, Clone, Default)]
pub struct RotateDef { pub targets: Option<Selector>, pub by: EulerDef }

#[derive(Debug, Deserialize, Clone, Default)]
pub struct MaterialSet { pub targets: Option<Selector>, pub material: String }

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SpawnDef {
    pub prefab: Option<String>,
    #[serde(default)]
    pub components: serde_json::Value,
    pub parent: Option<Selector>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct VarSet { pub name: String, pub value: f64 }

#[derive(Debug, Deserialize, Clone, Default)]
pub struct VarDelta { pub name: String, pub by: f64 }

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TranslateAxisDef {
    pub targets: Option<Selector>,
    pub vec: [f32; 3],
    pub speed_var: String,
    #[serde(default = "default_true")]
    pub use_dt: bool,
}

fn default_true() -> bool { true }
