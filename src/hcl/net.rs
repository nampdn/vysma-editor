use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::hcl::loader::HclSceneAsset;
use lightyear::prelude::*;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect)]
#[reflect(Component)]
pub struct HclSceneBlob {
    pub path: String,
    pub sha256: String,
    pub content: Option<String>,
}

#[derive(Component)]
struct HclNetMarker;

#[derive(Resource, Default)]
struct LastAppliedHash(Option<String>);

pub struct HclNetPlugin;

impl Plugin for HclNetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastAppliedHash>();
        app.register_type::<HclSceneBlob>();
        app.add_systems(Startup, ensure_net_singleton);
        app.add_systems(Update, apply_hcl_scene_from_net);
        app.add_systems(Update, publish_hcl_scene_if_changed);
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
) {
    if let Ok(blob) = net.get_single() {
        if last.0.as_deref() == Some(blob.sha256.as_str()) {
            return;
        }
        last.0 = Some(blob.sha256.clone());
        // Prefer path-based load for now; content-based injection can be added later
        commands.insert_resource(crate::hcl::HclEntry(Some(
            assets.load::<HclSceneAsset>(blob.path.as_str()),
        )));
    }
}

#[cfg(feature = "server")]
fn publish_hcl_scene_if_changed(
    mut commands: Commands,
    mut q: Query<(Entity, &mut HclSceneBlob)>,
    entry: Option<Res<crate::hcl::HclEntry>>,
) {
    let Some(entry) = entry else { return };
    let Some(path_handle) = &entry.0 else { return };
    let path = path_handle.path().map(|p| p.to_string()).unwrap_or_default();
    if path.is_empty() { return; }

    // Compute hash of file content if accessible; if not, hash of path
    let (sha, content_opt) = match std::fs::read_to_string(std::path::Path::new("assets").join(&path)) {
        Ok(s) => (sha256_str(&s), Some(s)),
        Err(_) => (sha256_str(&path), None),
    };

    if let Ok((_e, mut blob)) = q.get_single_mut() {
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

fn sha256_str(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}
