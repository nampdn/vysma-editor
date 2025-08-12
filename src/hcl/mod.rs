mod loader;
mod registry;
mod schema;
mod spawn;
mod runtime;
mod types;
pub mod net;
mod module_registry;
mod module_loader;
#[cfg(feature = "hcl_overlay_ui")]
mod overlay;

use bevy::prelude::*;
use loader::HclSceneAsset;
use registry::{ApplyCtx, ComponentRegistry, DefaultStdComponents};
use spawn::SceneSpawner;
use runtime::{HclRuntime, process_triggers};
use module_registry::{ModuleRegistry, ModuleRegistryPlugin};
use module_loader::{ModuleLoader, ModuleLoaderPlugin};

pub struct HclPlugin;

#[derive(Resource)]
struct HclOverlayLogTimer(Timer);

fn log_overlay(mut timer: ResMut<HclOverlayLogTimer>, time: Res<Time>, rt: Option<Res<HclRuntime>>) {
    if !timer.0.tick(time.delta()).just_finished() { return; }
    if let Some(rt) = rt {
        // Toggle via var debug_overlay_log > 0 to enable
        if rt.overlay_line(0, 0).is_empty() { /* avoid unused warnings */ }
        if let Some(v) = rt.get_var("debug_overlay_log") { if v <= 0.0 { return; } }
        let line = rt.overlay_line(8, 5);
        if !line.is_empty() { info!("HCL: {line}"); }
    }
}

impl Plugin for HclPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<HclSceneAsset>();
        app.init_asset_loader::<loader::HclLoader>();
        app.init_resource::<ComponentRegistry>();
        app.init_resource::<ApplyCtx>();
        app.init_resource::<SceneSpawner>();
        app.init_resource::<types::HclPersistStore>();
        app.init_resource::<HclRuntime>();
        app.insert_resource(HclOverlayLogTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
        app.add_event::<spawn::RespawnRequest>();
        app.add_systems(PreUpdate, spawn::hot_reload);
        app.add_systems(Update, (spawn::spawn_ready, process_triggers, spawn::apply_persisted_state));
        app.add_systems(Update, log_overlay);
        app.add_plugins(DefaultStdComponents);
        app.add_plugins(net::HclNetPlugin);
        app.add_plugins(ModuleRegistryPlugin);
        app.add_plugins(ModuleLoaderPlugin);
        #[cfg(feature = "hcl_overlay_ui")]
        app.add_plugins(overlay::HclOverlayPlugin);
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

/// Load a module and register it with the module registry
pub fn load_module_at_startup(
    module_name: String,
    path: String,
) -> impl FnMut(Res<AssetServer>, ResMut<ModuleRegistry>, ResMut<ModuleLoader>) + 'static {
    move |assets: Res<AssetServer>, _registry: ResMut<ModuleRegistry>, mut loader: ResMut<ModuleLoader>| {
        let _handle = assets.load::<HclSceneAsset>(path.as_str());
        loader.register_module_path(module_name.clone(), path.clone());
    }
}

// # examples/minimal.rs
// use bevy::prelude::*;
// use bevy_hcl_plugin::{HclPlugin, load_scene_at_startup};

// fn main() { App::new().add_plugins(DefaultPlugins.set(WindowPlugin{ primary_window: Some(Window{ title: "HCL Bevy Plugin".into(), ..default() }), ..default() })).add_plugins(HclPlugin).add_systems(Startup, load_scene_at_startup("scene.hcl")).run(); }
