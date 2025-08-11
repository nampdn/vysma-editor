mod loader;
mod registry;
mod schema;
mod spawn;
mod runtime;
mod types;
pub mod net;

use bevy::prelude::*;
use loader::HclSceneAsset;
use registry::{ApplyCtx, ComponentRegistry, DefaultStdComponents};
use spawn::SceneSpawner;
use runtime::{HclRuntime, process_triggers};

pub struct HclPlugin;

impl Plugin for HclPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<HclSceneAsset>();
        app.init_asset_loader::<loader::HclLoader>();
        app.init_resource::<ComponentRegistry>();
        app.init_resource::<ApplyCtx>();
        app.init_resource::<SceneSpawner>();
        app.init_resource::<HclRuntime>();
        app.add_event::<spawn::RespawnRequest>();
        app.add_systems(PreUpdate, spawn::hot_reload);
        app.add_systems(Update, (spawn::spawn_ready, process_triggers));
        app.add_plugins(DefaultStdComponents);
        app.add_plugins(net::HclNetPlugin);
    }
}

/// Convenience: load an HCL scene at startup and spawn when ready.
#[derive(Resource, Default)]
pub struct HclEntry(pub Option<Handle<HclSceneAsset>>);

pub fn load_scene_at_startup(
    path: &str,
) -> impl FnMut(Commands, Res<AssetServer>) + 'static {
    let path = path.to_owned();
    move |mut commands: Commands, assets: Res<AssetServer>| {
        commands.insert_resource(HclEntry(Some(assets.load::<HclSceneAsset>(path.as_str()))));
    }
}

// # examples/minimal.rs
// use bevy::prelude::*;
// use bevy_hcl_plugin::{HclPlugin, load_scene_at_startup};

// fn main() { App::new().add_plugins(DefaultPlugins.set(WindowPlugin{ primary_window: Some(Window{ title: "HCL Bevy Plugin".into(), ..default() }), ..default() })).add_plugins(HclPlugin).add_systems(Startup, load_scene_at_startup("scene.hcl")).run(); }
