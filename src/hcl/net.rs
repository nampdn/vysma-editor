use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::hcl::loader::{parse_hcl_to_asset, HclLoader, HclSceneAsset};
use lightyear::prelude::*;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
#[reflect(Component)]
pub struct HclSceneBlob {
    pub path: String,
    pub sha256: String,
    pub content: Option<String>,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
#[reflect(Component)]
pub struct HclUpdateRequest {
    pub path: Option<String>,
    pub sha256: String,
    pub content: String,
}

#[derive(Component)]
struct HclNetMarker;

#[derive(Resource, Default)]
struct LastAppliedHash(Option<String>);

/// Optional server-side source cache for the currently active HCL
#[derive(Resource, Default, Clone)]
pub struct HclSource {
    pub path: Option<String>,
    pub content: Option<String>,
    pub sha256: Option<String>,
}

#[cfg(feature = "client")]
#[derive(Resource, Default)]
struct FileWatchState {
    last_path: Option<String>,
    last_mtime: Option<SystemTime>,
    last_sha: Option<String>,
}

pub struct HclNetPlugin;

impl Plugin for HclNetPlugin {
    fn build(&self, app: &mut App) {
        // Ensure the HCL asset type and loader exist even on headless server
        app.init_asset::<HclSceneAsset>();
        app.init_asset_loader::<HclLoader>();

        app.init_resource::<LastAppliedHash>();
        app.init_resource::<HclSource>();
        app.register_type::<HclSceneBlob>();
        app.register_type::<HclUpdateRequest>();
        app.add_systems(Startup, ensure_net_singleton);
        app.add_systems(Update, apply_hcl_scene_from_net);
        app.add_systems(Update, publish_hcl_scene_if_changed);
        #[cfg(feature = "client")]
        {
            app.init_resource::<FileWatchState>();
            app.add_systems(Update, (editor_demo_send, watch_local_hcl_file_and_publish));
        }
        #[cfg(feature = "server")]
        app.add_observer(handle_hcl_update_request);
    }
}

fn ensure_net_singleton(mut commands: Commands, q: Query<Entity, With<HclNetMarker>>) {
    if q.is_empty() {
        commands.spawn((HclNetMarker, Replicate::to_clients(NetworkTarget::All)));
    }
}

fn apply_hcl_scene_from_net(
    mut commands: Commands,
    net: Query<&HclSceneBlob, Changed<HclSceneBlob>>,
    mut last: ResMut<LastAppliedHash>,
    assets: Res<AssetServer>,
    mut hcl_assets: ResMut<Assets<HclSceneAsset>>,
) {
    if let Some(blob) = net.iter().next() {
        if last.0.as_deref() == Some(blob.sha256.as_str()) {
            return;
        }
        last.0 = Some(blob.sha256.clone());

        if let Some(src) = &blob.content {
            // In-memory parse and register into Assets
            match parse_hcl_to_asset(src) {
                Ok(asset) => {
                    let handle = hcl_assets.add(asset);
                    commands.insert_resource(crate::hcl::HclEntry(Some(handle)));
                }
                Err(err) => {
                    warn!("Failed to parse HCL content from net: {err:?}. Falling back to path load");
                    commands.insert_resource(crate::hcl::HclEntry(Some(
                        assets.load::<HclSceneAsset>(blob.path.as_str()),
                    )));
                }
            }
        } else {
            // Fallback to path-based load
            commands.insert_resource(crate::hcl::HclEntry(Some(
                assets.load::<HclSceneAsset>(blob.path.as_str()),
            )));
        }
    }
}

#[cfg(feature = "server")]
fn publish_hcl_scene_if_changed(
    mut commands: Commands,
    mut q: Query<(Entity, &mut HclSceneBlob)>,
    entry: Option<Res<crate::hcl::HclEntry>>,
    src: Option<Res<HclSource>>,
) {
    // Prefer server-side source cache if content exists
    if let Some(src) = src {
        if let Some(content) = &src.content {
            let sha = src.sha256.clone().unwrap_or_else(|| sha256_str(content));
            let path = src.path.clone().unwrap_or_else(|| "mem://active.hcl".to_string());
            if let Some((_e, mut blob)) = q.iter_mut().next() {
                if blob.sha256 != sha || blob.path != path || blob.content.as_deref() != Some(content.as_str()) {
                    blob.path = path;
                    blob.sha256 = sha;
                    blob.content = Some(content.clone());
                }
            } else {
                commands.spawn((
                    HclNetMarker,
                    Replicate::to_clients(NetworkTarget::All),
                    HclSceneBlob { path, sha256: sha, content: Some(content.clone()) },
                ));
            }
            return;
        }
    }

    // Otherwise, fall back to asset path read
    let Some(entry) = entry else { return };
    let Some(path_handle) = &entry.0 else { return };
    let path = path_handle.path().map(|p| p.to_string()).unwrap_or_default();
    if path.is_empty() { return; }

    let (sha, content_opt) = match std::fs::read_to_string(std::path::Path::new("assets").join(&path)) {
        Ok(s) => (sha256_str(&s), Some(s)),
        Err(_) => (sha256_str(&path), None),
    };

    if let Some((_e, mut blob)) = q.iter_mut().next() {
        if blob.sha256 != sha || blob.path != path {
            blob.path = path;
            blob.sha256 = sha;
            blob.content = content_opt;
        }
    } else {
        commands.spawn((
            HclNetMarker,
            Replicate::to_clients(NetworkTarget::All),
            HclSceneBlob { path, sha256: sha, content: content_opt },
        ));
    }
}

#[cfg(feature = "server")]
pub(crate) fn handle_hcl_update_request(
    _trigger: Trigger<OnAdd, HclUpdateRequest>,
    mut commands: Commands,
    mut hcl_assets: ResMut<Assets<HclSceneAsset>>,
    mut src: ResMut<HclSource>,
    updates: Query<&HclUpdateRequest>,
) {
    // There might be multiple; handle all
    for upd in updates.iter() {
        match parse_hcl_to_asset(&upd.content) {
            Ok(asset) => {
                let handle = hcl_assets.add(asset);
                commands.insert_resource(crate::hcl::HclEntry(Some(handle)));
                src.path = upd.path.clone().or(Some("mem://active.hcl".into()));
                src.content = Some(upd.content.clone());
                src.sha256 = Some(upd.sha256.clone());
                info!("Applied HCL update from client; sha={}", upd.sha256);
            }
            Err(err) => {
                warn!("Failed to parse HCL update from client: {err:?}");
            }
        }
        // Despawn the update entity to avoid reprocessing
        // We cannot access the entity id here without another query; leave GC to client replication cleanup.
    }
}

#[cfg(feature = "client")]
fn editor_demo_send(
    keys: Res<ButtonInput<KeyCode>>,
    mode: Option<Res<crate::hcl::EditorState>>,
    mut commands: Commands,
) {
    let Some(mode) = mode else { return; };
    if !matches!(mode.0, crate::hcl::EditorMode::Edit) { return; }

    // F6 sends a minimal demo scene; Key1/Key2 toggle color
    let mut content: Option<String> = None;
    if keys.just_pressed(KeyCode::F6) || keys.just_pressed(KeyCode::Digit1) {
        content = Some(mini_hcl_with_color("#3aa7ff"));
    } else if keys.just_pressed(KeyCode::Digit2) {
        content = Some(mini_hcl_with_color("#ff7a3a"));
    }
    if let Some(s) = content {
        let sha = sha256_str(&s);
        commands.spawn((
            HclUpdateRequest { path: Some("mem://active.hcl".into()), sha256: sha, content: s },
            Replicate::to_server(),
            Name::new("HclUpdateRequest"),
        ));
        info!("Sent HCL update request to server");
    }
}

#[cfg(feature = "client")]
fn watch_local_hcl_file_and_publish(
    entry: Option<Res<crate::hcl::HclEntry>>,
    mut state: ResMut<FileWatchState>,
    mut commands: Commands,
    mut hcl_assets: ResMut<Assets<HclSceneAsset>>,
    mode: Option<Res<crate::hcl::EditorState>>,
) {
    let Some(entry) = entry else { return; };
    // Prefer current handle path; if none (memory asset), fall back to last known path
    let rel_path = if let Some(handle) = &entry.0 {
        if let Some(p) = handle.path().map(|p| p.to_string()) {
            // record latest path for future memory-only updates
            state.last_path = Some(p.clone());
            p
        } else if let Some(p) = state.last_path.clone() {
            p
        } else {
            return;
        }
    } else if let Some(p) = state.last_path.clone() {
        p
    } else {
        return;
    };

    // Build absolute path under assets/
    let mut abs_path = PathBuf::from("assets");
    abs_path.push(&rel_path);

    let Ok(md) = fs::metadata(&abs_path) else { return; };
    let Ok(modified) = md.modified() else { return; };

    // If path changed, reset state
    if state.last_path.as_deref() != Some(rel_path.as_str()) {
        state.last_path = Some(rel_path.clone());
        state.last_mtime = None;
        state.last_sha = None;
    }

    if Some(modified) == state.last_mtime { return; }

    let Ok(content) = fs::read_to_string(&abs_path) else { return; };
    let sha = sha256_str(&content);
    if state.last_sha.as_deref() == Some(sha.as_str()) {
        state.last_mtime = Some(modified);
        return;
    }

    // Parse locally and apply
    match parse_hcl_to_asset(&content) {
        Ok(asset) => {
            let handle = hcl_assets.add(asset);
            commands.insert_resource(crate::hcl::HclEntry(Some(handle)));
            // If in Edit mode, publish to server so other clients update
            if let Some(mode) = mode {
                if matches!(mode.0, crate::hcl::EditorMode::Edit) {
                    commands.spawn((
                        HclUpdateRequest { path: Some(rel_path.clone()), sha256: sha.clone(), content: content.clone() },
                        Replicate::to_server(),
                        Name::new("HclUpdateRequest"),
                    ));
                }
            }
            state.last_mtime = Some(modified);
            state.last_sha = Some(sha);
        }
        Err(err) => {
            warn!("Local HCL parse failed on change: {err:?}");
        }
    }
}

#[cfg(feature = "client")]
fn mini_hcl_with_color(color: &str) -> String {
    format!(
        "assets = {{ mesh = [ {{ name = \"cube\", builtin = \"cube\" }} ], material = [ {{ name = \"hero\", pbr = {{ base_color = \"{}\" }} }} ] }}\nvars = {{ speed = 6.0 }}\nprefab \"Hero\" {{ components = {{ MeshRef = {{ mesh = \"cube\" }}, StandardMaterialRef = {{ material = \"hero\" }}, Transform = {{ s = [1,1,1] }} }} }}\nentity \"root\" {{ children = [ {{ name = \"Player\", include = [\"Hero\"], components = {{ Transform = {{ t = [0,1,0] }} }} }} ] }}\ntriggers {{ trigger \"move_w\" {{ on = {{ key_held = \"KeyW\" }} target = {{ name = \"Player\" }} actions = [ {{ translate_axis = {{ vec = [0,0,-1], speed_var = \"speed\" }} }} ] }} }}",
        color
    )
}

fn sha256_str(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}
