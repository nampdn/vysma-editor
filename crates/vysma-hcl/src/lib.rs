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
            // New: manifest and base URL resources
            app.init_resource::<remote::ManifestMap>();
            app.init_resource::<remote::AssetBaseUrl>();
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
            // Add automatic HCL scene discovery
            app.add_plugins(HclDiscoveryPlugin);
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

    // Smart HCL discovery: automatically find and load HCL files from current working directory
    pub fn auto_discover_hcl_scenes() -> impl FnMut(Commands, Res<AssetServer>) + 'static {
        move |mut commands: Commands, _assets: Res<AssetServer>| {
            // This will be handled by the HclDiscoveryPlugin
            info!("HCL auto-discovery enabled - will load scenes from current working directory");
        }
    }

    // Plugin for automatic HCL scene discovery and loading
    pub struct HclDiscoveryPlugin;

    impl Plugin for HclDiscoveryPlugin {
        fn build(&self, app: &mut App) {
            app.init_resource::<HclDiscoveryState>();
            app.add_systems(Startup, discover_and_load_hcl_scenes);
            app.add_systems(Update, watch_hcl_changes);
        }
    }

    #[derive(Resource)]
    pub struct HclDiscoveryState {
        pub discovered_scenes: Vec<String>,
        pub current_scene: Option<String>,
        pub last_check: std::time::Instant,
    }

    impl Default for HclDiscoveryState {
        fn default() -> Self {
            Self {
                discovered_scenes: Vec::new(),
                current_scene: None,
                last_check: std::time::Instant::now(),
            }
        }
    }

    // System to discover HCL files at startup
    fn discover_and_load_hcl_scenes(
        mut commands: Commands,
        mut discovery: ResMut<HclDiscoveryState>,
        assets: Res<AssetServer>,
    ) {
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let assets_dir = cwd.join("assets");
        
        if !assets_dir.exists() {
            warn!("No assets/ directory found in {:?}", cwd);
            return;
        }

        // Look for HCL files in common locations
        let hcl_paths = [
            "main.hcl",
            "scene.hcl", 
            "game.hcl",
            "scenes/main.hcl",
            "scenes/scene.hcl",
            "scenes/game.hcl",
            "scenes/example.hcl",
        ];

        let mut found_scenes = Vec::new();
        
        // First, try to find main.hcl or scene.hcl in assets root
        for path in &hcl_paths {
            let full_path = assets_dir.join(path);
            if full_path.exists() {
                found_scenes.push(path.to_string());
                info!("Found HCL scene: assets/{}", path);
            }
        }

        // If no main scenes found, scan for any .hcl files
        if found_scenes.is_empty() {
            if let Ok(entries) = std::fs::read_dir(&assets_dir) {
                for entry in entries.filter_map(Result::ok) {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "hcl" {
                            let full_path = entry.path();
                            let rel_path = full_path.strip_prefix(&assets_dir).unwrap();
                            found_scenes.push(rel_path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        // Also scan scenes/ subdirectory
        let scenes_dir = assets_dir.join("scenes");
        if scenes_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&scenes_dir) {
                for entry in entries.filter_map(Result::ok) {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "hcl" {
                            let full_path = entry.path();
                            let rel_path = full_path.strip_prefix(&assets_dir).unwrap();
                            found_scenes.push(rel_path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        discovery.discovered_scenes = found_scenes.clone();
        
        // Load the first available scene
        if let Some(first_scene) = found_scenes.first() {
            // Use just the filename, let Bevy resolve it relative to assets root
            let scene_path = first_scene.clone();
            info!("Loading primary scene: {}", scene_path);
            discovery.current_scene = Some(scene_path.clone());
            
            let handle = assets.load::<loader::HclSceneAsset>(&scene_path);
            commands.insert_resource(HclEntry(Some(handle)));
        } else {
            warn!("No HCL scenes found in {:?}/assets", cwd);
        }

        discovery.last_check = std::time::Instant::now();
    }

    // System to watch for HCL file changes
    fn watch_hcl_changes(
        mut commands: Commands,
        mut discovery: ResMut<HclDiscoveryState>,
        assets: Res<AssetServer>,
        time: Res<Time>,
    ) {
        // Check every 2 seconds for new HCL files
        let now = std::time::Instant::now();
        let time_since_last = now.duration_since(discovery.last_check);
        if time_since_last.as_secs_f32() < 2.0 {
            return;
        }

        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let assets_dir = cwd.join("assets");
        
        if !assets_dir.exists() {
            return;
        }

        let mut current_scenes = Vec::new();
        
        // Scan for current HCL files
        if let Ok(entries) = std::fs::read_dir(&assets_dir) {
            for entry in entries.filter_map(Result::ok) {
                if let Some(ext) = entry.path().extension() {
                    if ext == "hcl" {
                        let full_path = entry.path();
                        let rel_path = full_path.strip_prefix(&assets_dir).unwrap();
                        current_scenes.push(rel_path.to_string_lossy().to_string());
                    }
                }
            }
        }

        let scenes_dir = assets_dir.join("scenes");
        if scenes_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&scenes_dir) {
                for entry in entries.filter_map(Result::ok) {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "hcl" {
                            let full_path = entry.path();
                            let rel_path = full_path.strip_prefix(&assets_dir).unwrap();
                            current_scenes.push(rel_path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        // Check if we have new scenes
        let new_scenes: Vec<_> = current_scenes.iter()
            .filter(|s| !discovery.discovered_scenes.contains(s))
            .collect();

        if !new_scenes.is_empty() {
            info!("New HCL scenes detected: {:?}", new_scenes);
            discovery.discovered_scenes = current_scenes;
            
            // Reload the primary scene if it changed
            if let Some(first_scene) = discovery.discovered_scenes.first() {
                // Use just the filename, let Bevy resolve it relative to assets root
                let scene_path = first_scene.clone();
                if discovery.current_scene.as_deref() != Some(&scene_path) {
                    info!("Switching to new primary scene: {}", scene_path);
                    discovery.current_scene = Some(scene_path.clone());
                    
                    let handle = assets.load::<loader::HclSceneAsset>(&scene_path);
                    commands.insert_resource(HclEntry(Some(handle)));
                }
            }
        }

        discovery.last_check = now;
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