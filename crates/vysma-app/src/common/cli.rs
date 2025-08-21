use std::time::Duration;

use bevy::diagnostic::DiagnosticsPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use clap::{Parser, Subcommand};
#[cfg(any(target_family = "wasm", target_os = "ios"))]
use rand::random;

#[cfg(all(feature = "gui", feature = "client"))]
use crate::common::renderer::ClientRendererPlugin;

mod window;
mod log;
mod spawn;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub mode: Option<Mode>,
}

impl Cli {
    pub fn client_id(&self) -> Option<u64> {
        match &self.mode {
            #[cfg(feature = "client")]
            Some(Mode::Client { client_id }) => *client_id,
            #[cfg(all(feature = "client", feature = "server"))]
            Some(Mode::Separate { client_id }) => *client_id,
            #[cfg(all(feature = "client", feature = "server"))]
            Some(Mode::HostClient { client_id }) => *client_id,
            _ => None,
        }
    }

    pub fn build_app(&self, tick_duration: Duration, add_inspector: bool) -> App {
        match self.mode {
            #[cfg(feature = "client")]
            Some(Mode::Client { client_id }) => {
                let mut app = new_gui_app(add_inspector);
                #[cfg(feature = "steam")]
                app.add_steam_resources(STEAM_APP_ID);
                app.add_plugins((
                    lightyear::prelude::client::ClientPlugins { tick_duration },
                    #[cfg(feature = "gui")]
                    ClientRendererPlugin::new(format!("Client {client_id:?}")),
                ));
                #[cfg(all(feature = "gui", feature = "client"))]
                {
                    use crate::EditorUiPlugin;
                    app.add_plugins(EditorUiPlugin);
                }
                // set up networking entities
                spawn::spawn_connections(&mut app, &Mode::Client { client_id });
                app
            }
            #[cfg(feature = "server")]
            Some(Mode::Server) => {
                cfg_if::cfg_if! {
                    if #[cfg(feature = "gui")] {
                        let mut app = new_gui_app(add_inspector);
                    } else {
                        let mut app = new_headless_app();
                    }
                }
                #[cfg(feature = "steam")]
                app.add_steam_resources(STEAM_APP_ID);
                app.add_plugins((
                    lightyear::prelude::server::ServerPlugins { tick_duration },
                ));
                // set up networking entities
                spawn::spawn_connections(&mut app, &Mode::Server);
                app
            }
            #[cfg(all(feature = "client", feature = "server"))]
            Some(Mode::HostClient { client_id: _ }) => {
                let mut app = new_gui_app(add_inspector);
                #[cfg(feature = "steam")]
                app.add_steam_resources(STEAM_APP_ID);
                app.add_plugins((
                    lightyear::prelude::client::ClientPlugins { tick_duration },
                    lightyear::prelude::server::ServerPlugins { tick_duration },
                ));
                // set up networking entities
                spawn::spawn_connections(&mut app, &Mode::HostClient { client_id: None });
                app
            }
            None => {
                panic!("Mode is required");
            }
            _ => {
                todo!()
            }
        }
    }

    pub fn spawn_connections(&self, app: &mut App) {
        if let Some(mode) = &self.mode { spawn::spawn_connections(app, mode); }
    }
}

#[derive(Subcommand, Debug)]
pub enum Mode {
    #[cfg(feature = "client")]
    /// Runs the app in client mode
    Client {
        #[arg(short, long, default_value = None)]
        client_id: Option<u64>,
    },
    #[cfg(feature = "server")]
    /// Runs the app in server mode
    Server,
    #[cfg(all(feature = "client", feature = "server"))]
    /// Creates two bevy apps: a client app and a server app.
    /// Data gets passed between the two via channels.
    Separate {
        #[arg(short, long, default_value = None)]
        client_id: Option<u64>,
    },
    #[cfg(all(feature = "client", feature = "server"))]
    /// Run the app in host-client mode.
    /// The client and the server will run inside the same app. The peer acts both as a client and a server.
    HostClient {
        #[arg(short, long, default_value = None)]
        client_id: Option<u64>,
    },
}

impl Default for Mode {
    fn default() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "client", feature = "server"))] {
                Mode::HostClient { client_id: None }
            } else if #[cfg(feature = "server")] {
                Mode::Server
            } else {
                Mode::Client { client_id: None }
            }
        }
    }
}

struct SendApp(App);
unsafe impl Send for SendApp {}
impl SendApp {
    fn run(&mut self) {
        self.0.run();
    }
}

impl Default for Cli {
    fn default() -> Self {
        cli()
    }
}

pub fn cli() -> Cli {
    cfg_if::cfg_if! {
        if #[cfg(any(target_family= "wasm", target_os="ios"))] {
            let client_id = random::<u64>();
            Cli {
                mode: Some(Mode::Client {
                    client_id: Some(client_id),
                })
            }
        } else {
            Cli::parse()
        }
    }
}

#[cfg(feature = "gui")]
pub fn new_gui_app(add_inspector: bool) -> App {
    #[allow(unused_imports)]
    use bevy::winit::WinitPlugin;
    use bevy::window::WindowPlugin;

    #[cfg(target_os = "ios")]
    use std::default;

    let mut app = App::new();

    #[cfg(not(target_os = "ios"))]
    app.add_plugins(
        DefaultPlugins
            .build()
            .set(AssetPlugin {
                // https://github.com/bevyengine/bevy/issues/10157
                meta_check: bevy::asset::AssetMetaCheck::Never,
                watch_for_changes_override: Some(true),
                file_path: detect_assets_root().to_string_lossy().into_owned(),
                ..default()
            })
            .set(log::log_plugin())
            .set(window::window_plugin()),
    );

    #[cfg(feature = "http_assets")]
    {
        use vysma_hcl::hcl::net::HttpAssetIoPlugin;
        app.add_plugins(HttpAssetIoPlugin);
    }

    #[allow(unused_mut)]
    let mut _default_plugins = DefaultPlugins.build();

    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        _default_plugins = _default_plugins
            .disable::<WinitPlugin>()
            .set(WindowPlugin::default());

        app.insert_resource(ClearColor(Color::srgb(0.8, 0.4, 0.6)))
            .add_plugins(_default_plugins);
    }

    #[cfg(feature = "visualizer")]
    {
        app.add_plugins(bevy_metrics_dashboard::RegistryPlugin::default());
        app.add_plugins(bevy_metrics_dashboard::DashboardPlugin);
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(bevy_metrics_dashboard::DashboardWindow::new("Metrics"));
        });
    }

    // Always add Egui so editor UI can render
    app.add_plugins(bevy_inspector_egui::bevy_egui::EguiPlugin::default());
    if add_inspector {
        app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
    }
    // Ensure there is a camera for UI/rendering
    fn ensure_camera(mut commands: Commands, existing: Query<Entity, With<Camera>>) {
        if existing.is_empty() {
            commands.spawn(Camera2d);
        }
    }
    app.add_systems(Startup, ensure_camera);
    app
}

#[cfg(not(target_os = "ios"))]
fn detect_assets_root() -> std::path::PathBuf {
    use std::path::PathBuf;
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let proj_assets = cwd.join("assets");
    if proj_assets.exists() { return proj_assets; }
    // Fallback to binary dir/assets
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join("assets");
            if p.exists() { return p; }
        }
    }
    cwd.join("assets")
}

pub fn new_headless_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        log::log_plugin(),
        StatesPlugin,
        DiagnosticsPlugin,
    ));
    app
}
