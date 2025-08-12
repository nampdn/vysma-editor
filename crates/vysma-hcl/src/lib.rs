pub mod hcl {
    use bevy::prelude::*;

    // Minimal shared types expected by included modules
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum EditorMode { Edit, Preview }

    #[derive(Resource, Debug, Clone, Copy)]
    pub struct EditorState(pub EditorMode);
    impl Default for EditorState { fn default() -> Self { Self(EditorMode::Preview) } }

    #[derive(Resource, Default)]
    pub struct HclEntry(pub Option<Handle<loader::HclSceneAsset>>);

    pub mod loader;
    pub mod registry;
    pub mod schema;
    pub mod spawn;
    pub mod runtime;
    pub mod types;
    pub mod net;
    pub mod module_loader;
    pub mod module_registry;
    #[cfg(feature = "hcl_overlay_ui")]
    pub mod overlay;
    pub mod remote;

    // Public plugin to install all HCL capabilities
    pub struct HclPlugin;
    #[derive(Resource)]
    struct HclOverlayLogTimer(Timer);

    impl Plugin for HclPlugin {
        fn build(&self, app: &mut App) {
            app.init_asset::<loader::HclSceneAsset>();
            app.init_asset_loader::<loader::HclLoader>();
            app.init_resource::<registry::ComponentRegistry>();
            app.init_resource::<registry::ApplyCtx>();
            app.init_resource::<spawn::SceneSpawner>();
            app.init_resource::<types::HclPersistStore>();
            app.init_resource::<runtime::HclRuntime>();
            app.init_resource::<EditorState>();
            app.insert_resource(HclOverlayLogTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
            app.add_event::<spawn::RespawnRequest>();
            app.add_systems(PreUpdate, spawn::hot_reload);
            app.add_systems(Update, (spawn::spawn_ready, runtime::process_triggers, spawn::apply_persisted_state));
            // Add simple overlay log gate
            fn log_overlay(mut timer: ResMut<HclOverlayLogTimer>, time: Res<Time>, rt: Option<Res<runtime::HclRuntime>>) {
                if !timer.0.tick(time.delta()).just_finished() { return; }
                if let Some(rt) = rt {
                    if let Some(v) = rt.get_var("debug_overlay_log") { if v <= 0.0 { return; } }
                    let line = rt.overlay_line(8, 5);
                    if !line.is_empty() { info!("HCL: {line}"); }
                }
            }
            fn toggle_editor_mode(keys: Res<ButtonInput<KeyCode>>, mut mode: ResMut<EditorState>) {
                if keys.just_pressed(KeyCode::F5) {
                    mode.0 = match mode.0 { EditorMode::Edit => EditorMode::Preview, EditorMode::Preview => EditorMode::Edit };
                    info!("HCL EditorMode -> {:?}", mode.0);
                }
            }
            app.add_systems(Update, (log_overlay, toggle_editor_mode));
            app.add_plugins(registry::DefaultStdComponents);
            app.add_plugins(net::HclNetPlugin);
            app.add_plugins(module_registry::ModuleRegistryPlugin);
            app.add_plugins(module_loader::ModuleLoaderPlugin);
            #[cfg(feature = "hcl_overlay_ui")]
            app.add_plugins(overlay::HclOverlayPlugin);
        }
    }

    // Convenience: load an HCL scene at startup and spawn when ready.
    pub fn load_scene_at_startup(path: &str) -> impl FnMut(Commands, Res<AssetServer>) + 'static {
        let path = path.to_owned();
        move |mut commands: Commands, assets: Res<AssetServer>| {
            commands.insert_resource(HclEntry(Some(assets.load::<loader::HclSceneAsset>(path.as_str()))));
        }
    }

    // Convenience: load a module path at startup; registers with module loader
    pub fn load_module_at_startup(
        module_name: String,
        path: String,
    ) -> impl FnMut(Res<AssetServer>, ResMut<module_registry::ModuleRegistry>, ResMut<module_loader::ModuleLoader>) + 'static {
        move |assets: Res<AssetServer>, _registry: ResMut<module_registry::ModuleRegistry>, mut loader: ResMut<module_loader::ModuleLoader>| {
            let _handle = assets.load::<loader::HclSceneAsset>(path.as_str());
            loader.register_module_path(module_name.clone(), path.clone());
        }
    }

    // Helper accessors used internally by spawn to access resources safely
    pub(crate) fn app_provider(world: &World) -> Option<&remote::RemoteModuleProviderResource> {
        world.get_resource::<remote::RemoteModuleProviderResource>()
    }
    pub(crate) fn app_loader(world: &World) -> &module_loader::ModuleLoader {
        world.get_resource::<module_loader::ModuleLoader>().expect("ModuleLoader not initialized")
    }
} 