mod loader;
mod registry;
mod schema;
mod spawn;
mod runtime;
mod types;
pub mod net;
mod module_registry;
mod module_loader;
mod moba_components;

use bevy::prelude::*;
use loader::HclSceneAsset;
use registry::{ApplyCtx, ComponentRegistry, DefaultStdComponents};
use spawn::SceneSpawner;
use runtime::{HclRuntime, process_triggers};
use module_registry::{ModuleRegistry, ModuleRegistryPlugin};
use module_loader::{ModuleLoader, ModuleLoaderPlugin};
use moba_components::*;

pub struct HclPlugin;

impl Plugin for HclPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<HclSceneAsset>();
        app.init_asset_loader::<loader::HclLoader>();
        app.init_resource::<ComponentRegistry>();
        app.init_resource::<ApplyCtx>();
        app.init_resource::<SceneSpawner>();
        app.init_resource::<types::HclPersistStore>();
        app.init_resource::<HclRuntime>();
        app.add_event::<spawn::RespawnRequest>();
        app.add_systems(PreUpdate, spawn::hot_reload);
        app.add_systems(Update, (spawn::spawn_ready, process_triggers, spawn::apply_persisted_state));
        app.add_plugins(DefaultStdComponents);
        app.add_plugins(net::HclNetPlugin);
        app.add_plugins(ModuleRegistryPlugin);
        app.add_plugins(ModuleLoaderPlugin);
        
        // Register MOBA components
        app.add_systems(Startup, register_moba_components);
    }
}

/// Register all MOBA-specific component appliers
fn register_moba_components(mut registry: ResMut<ComponentRegistry>) {
    registry.register(HeroApplier);
    registry.register(AbilityApplier);
    registry.register(CombatApplier);
    registry.register(TeamApplier);
    registry.register(MovementApplier);
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
