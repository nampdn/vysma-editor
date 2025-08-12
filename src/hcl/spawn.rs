use crate::hcl::{
    loader::HclSceneAsset,
    registry::{ApplyCtx, ComponentApplier, ComponentRegistry, EntityScratch},
    schema::{AssetsBlock, EntityDecl, MeshKind, SceneDoc},
};
use ahash::AHashMap as HashMap;
use bevy::prelude::*;
#[cfg(feature = "remote_assets")]
use reqwest::blocking as http;
use log::info; // Add logging dependency
use super::types::{HclTags, HclPersistent};

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
            if spawner.spawned_roots.contains_key(h) { return; }
            if let Some(asset) = assets.get(h) {
                let Some(doc) = merge_includes(&asset.doc, &asset_server, &assets) else { return; };
                let root = spawn_from_doc(
                    &mut commands,
                    &registry,
                    &mut ctx,
                    &asset_server,
                    &mut meshes,
                    &mut materials,
                    &doc,
                );
                spawner.spawned_roots.insert(h.clone(), root);
            }
        }
    }
}

pub fn apply_persisted_state(
    mut store: ResMut<super::types::HclPersistStore>,
    mut q: Query<(&super::types::HclPersistent, &mut Transform)>,
) {
    if store.0.is_empty() { return; }
    for (key, mut tf) in q.iter_mut() {
        if let Some(saved) = store.0.get(&key.0) { *tf = *saved; }
    }
    store.0.clear();
}

pub fn hot_reload(
    mut commands: Commands,
    mut events: EventReader<bevy::asset::AssetEvent<HclSceneAsset>>,
    mut spawner: ResMut<SceneSpawner>,
    assets: Res<Assets<HclSceneAsset>>,
    registry: Res<ComponentRegistry>,
    mut ctx: ResMut<ApplyCtx>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut persist: ResMut<super::types::HclPersistStore>,
    q_persist: Query<(&super::types::HclPersistent, Option<&Transform>)>,
) {
    for ev in events.read() {
        if let bevy::asset::AssetEvent::Modified { id } = *ev {
            println!("hot reload: {:?}", id);
            if let Some(handle) = asset_server.get_id_handle(id) {
                if let Some(root) = spawner.spawned_roots.get(&handle) {
                    persist.0.clear();
                    for (tag, t) in q_persist.iter() { if let Some(tf) = t { persist.0.insert(tag.0.clone(), *tf); } }
                    commands.entity(*root).despawn_recursive();
                }
                if let Some(asset) = assets.get(&handle) {
                    let Some(doc) = merge_includes(&asset.doc, &asset_server, &assets) else { continue; };
                    let new_root = spawn_from_doc(
                        &mut commands,
                        &registry,
                        &mut ctx,
                        &asset_server,
                        &mut meshes,
                        &mut materials,
                        &doc,
                    );
                    spawner.spawned_roots.insert(handle.clone(), new_root);
                }
            }
        }
    }
}

fn spawn_from_doc(
    commands: &mut Commands,
    registry: &Res<ComponentRegistry>,
    ctx: &mut ResMut<ApplyCtx>,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    doc: &SceneDoc,
) -> Entity {
    ctx.meshes.clear();
    ctx.materials.clear();
    ctx.images.clear();
    ctx.scenes.clear();

    if let Some(assets) = &doc.assets { load_assets(assets, ctx, asset_server, meshes, materials); }

    // Build prefab map for includes
    let mut prefabs: HashMap<String, serde_json::Value> = HashMap::default();
    for p in &doc.prefab { prefabs.insert(p.name.clone(), p.components.clone()); }

    let root = commands
        .spawn((
            Name::new("HCL Root"),
            HclTags::default(),
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
        ))
        .id();

    let mut scratch = EntityScratch::default();
    for ent in &doc.entity {
        spawn_entity(commands, registry, ctx, &mut scratch, &prefabs, &ent, Some(root));
    }

    root
}

fn load_assets(
    assets: &AssetsBlock,
    ctx: &mut ApplyCtx,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    for m in &assets.mesh {
        if let MeshKind::Builtin { builtin } = &m.kind {
            let mesh = match builtin.as_str() {
                "cube" => Mesh::from(Cuboid::default()),
                "plane" => Mesh::from(Plane3d::default()),
                _ => Mesh::from(Cuboid::default()),
            };
            ctx.meshes.insert(m.name.clone(), meshes.add(mesh));
        }
    }
    for mat in &assets.material {
        let color = crate::hcl::registry::color_from_def(&mat.pbr.as_ref().and_then(|p| p.base_color.clone()).unwrap_or_default());
        let mut m = StandardMaterial::from(color);
        if let Some(p) = &mat.pbr {
            if let Some(rough) = p.roughness { m.perceptual_roughness = rough; }
            if let Some(metal) = p.metallic { m.metallic = metal; }
            if let Some(em) = &p.emissive { m.emissive = crate::hcl::registry::color_from_def(em).into(); }
        }
        ctx.materials.insert(mat.name.clone(), materials.add(m));
    }
    if !ctx.materials.contains_key("__default") {
        let default_handle = materials.add(StandardMaterial::default());
        ctx.materials.insert("__default".to_string(), default_handle);
    }

    // Load GLTF scenes into ctx.scenes
    if let Some(gl) = &assets.gltf.first() {} // keep compiler happy if empty access
    for g in &assets.gltf {
        let key = if let Some(node) = &g.node { format!("{}#{}", g.file, node) } else { format!("{}#Scene0", g.file) };
        let handle: Handle<bevy::scene::Scene> = asset_server.load(key);
        ctx.scenes.insert(g.name.clone(), handle);
    }
}

fn build_appliers_ordered<'a>(registry: &'a Res<ComponentRegistry>) -> Vec<(&'static str, &'a Box<dyn ComponentApplier>)> {
    let mut items: Vec<(&'static str, &Box<dyn ComponentApplier>)> = registry.iter().collect();
    items.sort_by_key(|(_, a)| a.priority());
    items
}

fn merge_json(dst: &mut serde_json::Value, src: &serde_json::Value) {
    match (dst, src) {
        (serde_json::Value::Object(d), serde_json::Value::Object(s)) => {
            for (k, v) in s { merge_json(d.entry(k.clone()).or_insert(serde_json::Value::Null), v); }
        }
        (d, s) => *d = s.clone(),
    }
}

fn spawn_entity(
    commands: &mut Commands,
    registry: &Res<ComponentRegistry>,
    ctx: &mut ResMut<ApplyCtx>,
    scratch: &mut EntityScratch,
    prefabs: &HashMap<String, serde_json::Value>,
    ent: &EntityDecl,
    parent: Option<Entity>,
) -> Entity {
    let mut ec = commands.spawn((
        Name::new(ent.name.clone().unwrap_or("Unnamed".into())),
        HclTags(ent.tags.clone()),
        Transform::default(),
        GlobalTransform::default(),
        Visibility::Visible,
        InheritedVisibility::default(),
    ));
    if let Some(p) = parent { ec.insert(ChildOf(p)); }

    // Merge components from includes and overrides
    let mut merged = serde_json::json!({});
    for inc in &ent.include {
        if let Some(p) = prefabs.get(inc) { merge_json(&mut merged, p); }
    }
    merge_json(&mut merged, &ent.components);

    // Apply components in registry priority order
    let appliers = build_appliers_ordered(registry);
    if let Some(obj) = merged.as_object() {
        for (key, applier) in appliers {
            if let Some(payload) = obj.get(key) {
                let _ = applier.apply(payload, &mut ec, scratch, ctx);
            }
        }
    }

    let id = ec.id();
    for c in &ent.children { spawn_entity(commands, registry, ctx, scratch, prefabs, c, Some(id)); }
    id
}

pub fn merge_includes(doc: &SceneDoc, asset_server: &AssetServer, assets: &Assets<HclSceneAsset>) -> Option<SceneDoc> {
    if doc.includes.is_empty() { return Some(doc.clone()); }
    let mut merged = SceneDoc { assets: doc.assets.clone(), prefab: doc.prefab.clone(), entity: doc.entity.clone(), triggers: doc.triggers.clone(), vars: doc.vars.clone(), includes: doc.includes.clone(), modules: doc.modules.clone(), exports: doc.exports.clone(), functions: doc.functions.clone() };
    for inc in &doc.includes {
        let h = asset_server.load::<HclSceneAsset>(inc.as_str());
        if let Some(dep) = assets.get(&h) { merge_doc_into(&mut merged, &dep.doc); } else { return None; }
    }
    Some(merged)
}

pub fn merge_doc_into(dst: &mut SceneDoc, src: &SceneDoc) {
    if let Some(a) = &src.assets {
        let dst_assets = dst.assets.get_or_insert(AssetsBlock::default());
        dst_assets.mesh.extend_from_slice(&a.mesh);
        dst_assets.material.extend_from_slice(&a.material);
        dst_assets.image.extend_from_slice(&a.image);
        dst_assets.gltf.extend_from_slice(&a.gltf);
    }
    dst.prefab.extend_from_slice(&src.prefab);
    dst.entity.extend_from_slice(&src.entity);
    dst.triggers.extend_from_slice(&src.triggers);
    for (k, v) in &src.vars { dst.vars.insert(k.clone(), *v); }
}
