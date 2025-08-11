use ahash::AHashMap as HashMap;
use bevy::prelude::*;
use bevy::scene::Scene;
use serde::de::DeserializeOwned;

use crate::hcl::schema::{ColorDef, TransformDef};

pub type Json = serde_json::Value;

/// Per‑scene caches for asset handles. Shared across entities during a spawn pass.
#[derive(Resource, Default)]
pub struct ApplyCtx {
    pub meshes: HashMap<String, Handle<Mesh>>,
    pub materials: HashMap<String, Handle<StandardMaterial>>,
    pub images: HashMap<String, Handle<Image>>,
    pub scenes: HashMap<String, Handle<Scene>>,
}

/// Tiny per‑entity scratch so appliers can communicate without reading the world.
#[derive(Default)]
pub struct EntityScratch {
    pub desired_material: Option<Handle<StandardMaterial>>, // consumed by MeshRef
}

/// Component appliers are stateless and fast.
pub trait ComponentApplier: Send + Sync + 'static {
    /// Unique key in HCL map, e.g. "Transform".
    fn key(&self) -> &'static str;
    /// Priority for ordering. Lower runs earlier. (e.g. material before mesh)
    fn priority(&self) -> u8 {
        100
    }
    /// Apply the component or write data into `scratch`.
    fn apply(
        &self,
        payload: &Json,
        entity: &mut EntityCommands,
        scratch: &mut EntityScratch,
        ctx: &mut ApplyCtx,
    ) -> anyhow::Result<()>;
}

#[derive(Resource, Default)]
pub struct ComponentRegistry {
    map: HashMap<&'static str, Box<dyn ComponentApplier>>,
}

impl ComponentRegistry {
    pub fn register(&mut self, applier: impl ComponentApplier) {
        self.map.insert(applier.key(), Box::new(applier));
    }
    pub fn get(&self, key: &str) -> Option<&Box<dyn ComponentApplier>> {
        self.map.get(key)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &Box<dyn ComponentApplier>)> {
        self.map.iter().map(|(k, v)| (*k, v))
    }
}

/// Install stock appliers.
pub struct DefaultStdComponents;

// ---- helpers ----
pub fn from_json<T: DeserializeOwned>(v: &Json) -> anyhow::Result<T> {
    Ok(serde_json::from_value(v.clone())?)
}
pub fn color_from_def(c: &ColorDef) -> Color {
    match c {
        ColorDef::Hex(h) => parse_hex_color(h).unwrap_or(Color::WHITE),
        ColorDef::Rgba { r, g, b, a } => Color::srgba(*r, *g, *b, a.unwrap_or(1.0)),
    }
}

fn parse_hex_color(s: &str) -> Option<Color> {
    let s = s.trim();
    let hex = s.strip_prefix('#').unwrap_or(s);
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    let bytes = match hex.len() {
        6 => u32::from_str_radix(hex, 16).ok().map(|v| (v << 8) | 0xFF)?,
        8 => u32::from_str_radix(hex, 16).ok()?,
        _ => return None,
    };
    let r = ((bytes >> 24) & 0xFF) as u8;
    let g = ((bytes >> 16) & 0xFF) as u8;
    let b = ((bytes >> 8) & 0xFF) as u8;
    let a = (bytes & 0xFF) as u8;
    Some(Color::srgba_u8(r, g, b, a))
}

// ---- stock appliers ----
pub struct TransformApplier;
impl ComponentApplier for TransformApplier {
    fn key(&self) -> &'static str {
        "Transform"
    }
    fn apply(
        &self,
        payload: &Json,
        entity: &mut EntityCommands,
        _scratch: &mut EntityScratch,
        _ctx: &mut ApplyCtx,
    ) -> anyhow::Result<()> {
        let td: TransformDef = from_json(payload)?;
        let mut t = Transform::IDENTITY;
        if let Some([x, y, z]) = td.t {
            t.translation = Vec3::new(x, y, z);
        }
        if let Some([x, y, z]) = td.s {
            t.scale = Vec3::new(x, y, z);
        }
        if let Some([x, y, z, w]) = td.r {
            t.rotation = Quat::from_xyzw(x, y, z, w);
        }
        if let Some(e) = td.euler {
            t.rotation = Quat::from_euler(
                EulerRot::YXZ,
                e.y.to_radians(),
                e.x.to_radians(),
                e.z.to_radians(),
            );
        }
        if let Some([lx, ly, lz]) = td.look_at {
            t = Transform::from_translation(t.translation)
                .looking_at(Vec3::new(lx, ly, lz), Vec3::Y);
        }
        entity.insert(t);
        Ok(())
    }
}

pub struct NameApplier;
impl ComponentApplier for NameApplier {
    fn key(&self) -> &'static str { "Name" }
    fn apply(
        &self,
        payload: &Json,
        entity: &mut EntityCommands,
        _scratch: &mut EntityScratch,
        _ctx: &mut ApplyCtx,
    ) -> anyhow::Result<()> {
        if let Some(s) = payload.as_str() { entity.insert(Name::new(s.to_owned())); }
        Ok(())
    }
}

pub struct VisibilityApplier;
impl ComponentApplier for VisibilityApplier {
    fn key(&self) -> &'static str { "Visibility" }
    fn apply(
        &self,
        payload: &Json,
        entity: &mut EntityCommands,
        _scratch: &mut EntityScratch,
        _ctx: &mut ApplyCtx,
    ) -> anyhow::Result<()> {
        match payload.as_str() {
            Some("Visible") => entity.insert(Visibility::Visible),
            Some("Hidden") => entity.insert(Visibility::Hidden),
            _ => entity.insert(Visibility::Inherited),
        };
        Ok(())
    }
}

pub struct StandardMaterialRefApplier;
impl ComponentApplier for StandardMaterialRefApplier {
    fn key(&self) -> &'static str { "StandardMaterialRef" }
    fn priority(&self) -> u8 { 10 }
    fn apply(
        &self,
        payload: &Json,
        _entity: &mut EntityCommands,
        scratch: &mut EntityScratch,
        ctx: &mut ApplyCtx,
    ) -> anyhow::Result<()> {
        if let Some(name) = payload.get("material").and_then(|s| s.as_str()) {
            scratch.desired_material = ctx.materials.get(name).cloned().or_else(|| ctx.materials.get("__default").cloned());
        }
        Ok(())
    }
}

pub struct MeshRefApplier;
impl ComponentApplier for MeshRefApplier {
    fn key(&self) -> &'static str { "MeshRef" }
    fn priority(&self) -> u8 { 20 }
    fn apply(
        &self,
        payload: &Json,
        entity: &mut EntityCommands,
        scratch: &mut EntityScratch,
        ctx: &mut ApplyCtx,
    ) -> anyhow::Result<()> {
        let mesh_name = payload.get("mesh").and_then(|s| s.as_str()).ok_or_else(|| anyhow::anyhow!("MeshRef.mesh missing"))?;
        let mesh = ctx.meshes.get(mesh_name).cloned().ok_or_else(|| anyhow::anyhow!("Unknown mesh {mesh_name}"))?;
        let material = scratch.desired_material.clone().or_else(|| ctx.materials.get("__default").cloned()).unwrap_or_default();
        entity.insert((
            Transform::default(),
            GlobalTransform::default(),
            bevy::prelude::Mesh3d(mesh),
            bevy::pbr::MeshMaterial3d::<StandardMaterial>(material),
            Visibility::Visible,
            InheritedVisibility::default(),
        ));
        Ok(())
    }
}

pub struct Camera3dApplier;
impl ComponentApplier for Camera3dApplier {
    fn key(&self) -> &'static str { "Camera3d" }
    fn apply(
        &self,
        payload: &Json,
        entity: &mut EntityCommands,
        _scratch: &mut EntityScratch,
        _ctx: &mut ApplyCtx,
    ) -> anyhow::Result<()> {
        // Insert Camera3d marker and Projection so we can see large maps
        entity.insert(Camera3d::default());
        let mut cam = bevy::render::camera::Camera::default();
        if let Some(hdr) = payload.get("hdr").and_then(|v| v.as_bool()) { cam.hdr = hdr; }
        entity.insert(cam);
        // Set a far plane large enough to view 10k+ units
        entity.insert(bevy::render::camera::Projection::Perspective(
            bevy::render::camera::PerspectiveProjection { far: 50_000.0, ..Default::default() }
        ));
        Ok(())
    }
}

pub struct DirectionalLightApplier;
impl ComponentApplier for DirectionalLightApplier {
    fn key(&self) -> &'static str { "DirectionalLight" }
    fn apply(
        &self,
        payload: &Json,
        entity: &mut EntityCommands,
        _scratch: &mut EntityScratch,
        _ctx: &mut ApplyCtx,
    ) -> anyhow::Result<()> {
        let mut light = DirectionalLight::default();
        if let Some(i) = payload.get("illuminance").and_then(|v| v.as_f64()) { light.illuminance = i as f32; }
        if let Some(s) = payload.get("shadows").and_then(|v| v.as_bool()) { light.shadows_enabled = s; }
        entity.insert(light);
        Ok(())
    }
}

pub struct PointLightApplier;
impl ComponentApplier for PointLightApplier {
    fn key(&self) -> &'static str { "PointLight" }
    fn apply(
        &self,
        payload: &Json,
        entity: &mut EntityCommands,
        _scratch: &mut EntityScratch,
        _ctx: &mut ApplyCtx,
    ) -> anyhow::Result<()> {
        let mut light = PointLight::default();
        if let Some(i) = payload.get("intensity").and_then(|v| v.as_f64()) { light.intensity = i as f32; }
        if let Some(r) = payload.get("range").and_then(|v| v.as_f64()) { light.range = r as f32; }
        entity.insert(light);
        Ok(())
    }
}

pub struct SceneRefApplier;
impl ComponentApplier for SceneRefApplier {
    fn key(&self) -> &'static str { "SceneRef" }
    fn apply(
        &self,
        payload: &Json,
        entity: &mut EntityCommands,
        _scratch: &mut EntityScratch,
        ctx: &mut ApplyCtx,
    ) -> anyhow::Result<()> {
        let scene_name = payload.get("scene").and_then(|s| s.as_str()).ok_or_else(|| anyhow::anyhow!("SceneRef.scene missing"))?;
        let scene = ctx.scenes.get(scene_name).cloned().ok_or_else(|| anyhow::anyhow!("Unknown scene {scene_name}"))?;
        entity.insert(bevy::scene::SceneRoot(scene));
        Ok(())
    }
}

// Register defaults
impl DefaultStdComponents {
    pub fn register(reg: &mut ComponentRegistry) {
        reg.register(StandardMaterialRefApplier);
        reg.register(MeshRefApplier);
        reg.register(SceneRefApplier);
        reg.register(TransformApplier);
        reg.register(NameApplier);
        reg.register(VisibilityApplier);
        reg.register(Camera3dApplier);
        reg.register(DirectionalLightApplier);
        reg.register(PointLightApplier);
    }
}

impl Plugin for DefaultStdComponents {
    fn build(&self, app: &mut App) {
        let mut reg = app.world_mut().resource_mut::<ComponentRegistry>();
        DefaultStdComponents::register(&mut reg);
    }
}
