use crate::hcl::{
    loader::HclSceneAsset,
    registry::{ApplyCtx, ComponentApplier, ComponentRegistry, EntityScratch},
    schema::{AssetsBlock, EntityDecl, MeshKind},
};
use ahash::AHashMap as HashMap;
use bevy::prelude::*;
#[cfg(feature = "remote_assets")]
use reqwest::blocking as http;
use log::info; // Add logging dependency
use super::types::HclTags;

#[derive(Resource, Default)]
pub struct SceneSpawner {
    spawned_roots: HashMap<Handle<HclSceneAsset>, Entity>,
}

#[derive(Event)]
pub struct RespawnRequest(pub Handle<HclSceneAsset>);

pub fn spawn_ready(
    mut commands: Commands,
    registry: Res<ComponentRegistry>,
    assets: Res<Assets<HclSceneAsset>>,
    mut ctx: ResMut<ApplyCtx>,
    entry: Option<Res<crate::hcl::HclEntry>>,
    mut spawner: ResMut<SceneSpawner>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Some(entry) = entry {
        if let Some(h) = &entry.0 {
            if spawner.spawned_roots.contains_key(h) {
                return;
            }
            if let Some(doc) = assets.get(h) {
                let root = spawn_scene(
                    &mut commands,
                    &registry,
                    &mut ctx,
                    &asset_server,
                    &mut meshes,
                    &mut materials,
                    doc,
                );
                spawner.spawned_roots.insert(h.clone(), root);
            }
        }
    }
}

pub fn hot_reload(
    _commands: Commands,
    _asset_server: Res<AssetServer>,
    _spawner: ResMut<SceneSpawner>,
    _assets: Res<Assets<HclSceneAsset>>,
    _registry: Res<ComponentRegistry>,
    _ctx: ResMut<ApplyCtx>,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
) {
    // TODO: handle AssetEvent<HclSceneAsset> for proper hot reload
}

fn spawn_scene(
    commands: &mut Commands,
    registry: &ComponentRegistry,
    ctx: &mut ApplyCtx,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    doc: &HclSceneAsset,
) -> Entity {
    build_assets_cache(
        ctx,
        asset_server,
        meshes,
        materials,
        doc.doc.assets.as_ref(),
    );

    let mut prefab_map: HashMap<&str, &serde_json::Value> = HashMap::default();
    for p in &doc.doc.prefab {
        prefab_map.insert(p.name.as_str(), &p.components);
    }

    let root = commands
        .spawn((
            Name::new("HCLRoot"),
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();
    for e in &doc.doc.entity {
        spawn_recursive(e, commands, Some(root), registry, ctx, &prefab_map);
    }
    root
}

fn build_assets_cache(
    ctx: &mut ApplyCtx,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    assets: Option<&AssetsBlock>,
) {
    ctx.meshes.clear();
    ctx.materials.clear();
    ctx.images.clear();
    ctx.scenes.clear();
    if let Some(assets) = assets {
        use bevy::math::primitives as shape;
        for m in &assets.mesh {
            if let MeshKind::Builtin { builtin } = &m.kind {
                let mesh = match builtin.as_str() {
                    "cube" => Mesh::from(shape::Cuboid { half_size: Vec3::splat(0.5) }),
                    "plane" => Mesh::from(shape::Plane3d { normal: bevy::math::Dir3::Y, half_size: Vec2::splat(0.5) }),
                    "quad" => Mesh::from(shape::Plane3d { normal: bevy::math::Dir3::Y, half_size: Vec2::splat(0.5) }),
                    other => {
                        warn!("Unknown builtin mesh {other}, defaulting box");
                        Mesh::from(shape::Cuboid { half_size: Vec3::splat(0.5) })
                    }
                };
                let h = meshes.add(mesh);
                ctx.meshes.insert(m.name.clone(), h);
            }
        }
        for mat in &assets.material {
            let mut std = StandardMaterial::default();
            if let Some(pbr) = &mat.pbr {
                use crate::hcl::registry::color_from_def;
                if let Some(c) = &pbr.base_color {
                    std.base_color = color_from_def(c);
                }
                if let Some(m) = pbr.metallic {
                    std.metallic = m;
                }
                if let Some(r) = pbr.roughness {
                    std.perceptual_roughness = r;
                }
                if let Some(e) = &pbr.emissive {
                    std.emissive = crate::hcl::registry::color_from_def(e).to_linear();
                }
            }
            let h = materials.add(std);
            ctx.materials.insert(mat.name.clone(), h);
        }
        for img in &assets.image {
            let mut path = img.file.clone();
            if is_http(&path) {
                if let Some(local) = fetch_to_assets_cache(&path) { path = local; }
            }
            ctx.images.insert(img.name.clone(), asset_server.load(path));
        }
        for sc in &assets.gltf {
            let mut path = sc.file.clone();
            if is_http(&path) {
                if let Some(local) = fetch_to_assets_cache(&path) { path = local; }
            }
            ctx.scenes.insert(sc.name.clone(), asset_server.load(path));
        }
    }
}

pub(crate) fn spawn_recursive(
    decl: &EntityDecl,
    commands: &mut Commands,
    parent: Option<Entity>,
    registry: &ComponentRegistry,
    ctx: &mut ApplyCtx,
    prefabs: &HashMap<&str, &serde_json::Value>,
) {
    let mut ec = commands.spawn_empty();
    if let Some(p) = parent {
        ec.insert(ChildOf(p));
    }
    if let Some(n) = &decl.name {
        ec.insert(Name::new(n.clone()));
    }
    if !decl.tags.is_empty() { ec.insert(HclTags(decl.tags.clone())); }

    let mut merged = serde_json::json!({});
    for inc in &decl.include {
        if let Some(p) = prefabs.get(inc.as_str()) {
            merge_json(&mut merged, (*p).clone());
        }
    }
    merge_json(&mut merged, decl.components.clone());

    if let Some(obj) = merged.as_object() {
        let mut items: Vec<(&str, &Box<dyn ComponentApplier>)> = Vec::with_capacity(obj.len());
        for k in obj.keys() {
            if let Some(a) = registry.get(k) {
                items.push((k.as_str(), a));
            }
        }
        items.sort_by_key(|(_, a)| a.priority());

        let mut scratch = EntityScratch::default();
        for (k, a) in items {
            if let Some(v) = obj.get(k) {
                a.apply(v, &mut ec, &mut scratch, ctx)
                    .unwrap_or_else(|e| warn!("apply {k} failed: {e}"));
            }
        }
    }

    let id = ec.id();
    for c in &decl.children {
        spawn_recursive(c, commands, Some(id), registry, ctx, prefabs);
    }
}

fn merge_json(dst: &mut serde_json::Value, src: serde_json::Value) {
    match (dst, src) {
        (serde_json::Value::Object(d), serde_json::Value::Object(s)) => {
            for (k, v) in s {
                merge_json(d.entry(k).or_insert(serde_json::Value::Null), v);
            }
        }
        (d, s) => *d = s,
    }
}

fn is_http(path: &str) -> bool {
    path.starts_with("http://") || path.starts_with("https://")
}

fn fetch_to_assets_cache(url: &str) -> Option<String> {
    #[cfg(feature = "remote_assets")]
    {
        let Ok(bytes) = http::get(url).and_then(|r| r.error_for_status()).and_then(|mut r| r.bytes().map_err(|e| e.into())) else {
            warn!("Failed to fetch remote asset: {url}");
            return None;
        };
        let hash = sha256_hex(url);
        let ext = std::path::Path::new(url)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("bin");
        let rel = format!("_remote_cache/{}.{}", hash, ext);
        let out_path = std::path::Path::new("assets").join(&rel);
        if let Some(parent) = out_path.parent() { let _ = std::fs::create_dir_all(parent); }
        if std::fs::write(&out_path, &bytes).is_ok() { return Some(rel); }
        warn!("Failed to write cached asset: {}", out_path.display());
        None
    }
    #[cfg(not(feature = "remote_assets"))]
    { None }
}

fn sha256_hex(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    format!("{:x}", h.finalize())
}
