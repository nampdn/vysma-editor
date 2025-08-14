use bevy::prelude::*;

#[cfg(any(target_os = "android", target_os = "ios"))]
use bevy::ecs::{
    entity::Entity,
    system::{Commands, Query, SystemState},
};
#[cfg(any(target_os = "android", target_os = "ios"))]
use bevy::input::{
    ButtonState,
    keyboard::{Key, KeyboardInput},
};
use core::time::Duration;

#[cfg(any(target_os = "android", target_os = "ios"))]
use vysma_platform::app_view as app_view;

#[cfg(any(target_os = "android", target_os = "ios"))]
use vysma_platform::ffi as ffi;
#[cfg(any(target_os = "android", target_os = "ios"))]
pub use ffi::*;

pub use vysma_app::input_binding::*;

#[cfg(target_os = "android")]
use vysma_platform::android_asset_io;

pub use vysma_app::client;
pub use vysma_app::common;
mod protocol { pub use vysma_app::protocol::*; }
pub use vysma_app::renderer;
pub use vysma_app::server;
pub use vysma_app::shared;

// Re-export HCL from workspace crate during migration
pub use vysma_hcl::hcl as hcl;

#[allow(unused_variables)]
pub fn create_breakout_app(
    #[cfg(target_os = "android")] android_asset_manager: android_asset_io::AndroidAssetManager,
) -> App {
    #[allow(unused_imports)]
    use bevy::winit::WinitPlugin;

    let mut bevy_app = App::new();

    #[allow(unused_mut)]
    let mut default_plugins = DefaultPlugins.build();

    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        default_plugins = default_plugins
            .disable::<WinitPlugin>()
            .set(WindowPlugin::default());
    }

    #[cfg(target_os = "android")]
    {
        bevy_app.insert_non_send_resource(android_asset_manager);

        use bevy::render::{
            RenderPlugin,
            settings::{RenderCreation, WgpuSettings},
        };
        default_plugins = default_plugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings { backends: Some(wgpu::Backends::VULKAN), ..default() }),
            ..default()
        });

        default_plugins = default_plugins
            .add_before::<bevy::asset::AssetPlugin>(android_asset_io::AndroidAssetIoPlugin);
    }
    bevy_app
        .insert_resource(ClearColor(Color::srgb(0.8, 0.4, 0.6)))
        .add_plugins(default_plugins);

    #[cfg(any(target_os = "android", target_os = "ios"))]
    bevy_app.add_plugins(app_view::AppViewPlugin);

    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        use bevy::app::PluginsState;
        if bevy_app.plugins_state() == PluginsState::Ready {}
        bevy_app.finish();
        bevy_app.cleanup();
    }

    bevy_app
}

pub fn create_client_app() -> App {
    use crate::client::VysmaClientPlugin;
    use crate::common::{
        cli::Cli,
        shared::{FIXED_TIMESTEP_HZ, SEND_INTERVAL},
    };
    use crate::renderer::RendererPlugin;
    use crate::shared::SharedPlugin;
    use lightyear::prelude::{Client, ReplicationSender, SendUpdatesMode};

    let cli = Cli::default();
    let mut app = cli.build_app(Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ), false);

    cli.spawn_connections(&mut app);

    #[cfg(any(target_os = "android", target_os = "ios"))]
    app.add_plugins(app_view::AppViewPlugin);

    app.add_plugins(SharedPlugin);
    app.add_plugins(VysmaClientPlugin);

    let client_entity = app
        .world_mut()
        .query_filtered::<Entity, With<Client>>()
        .single(app.world_mut())
        .unwrap();
    app.world_mut()
        .entity_mut(client_entity)
        .insert(ReplicationSender::new(SEND_INTERVAL, SendUpdatesMode::SinceLastAck, false));

    app.add_plugins(RendererPlugin);

    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        use bevy::app::PluginsState;
        if app.plugins_state() == PluginsState::Ready {}
        app.finish();
        app.cleanup();
    }

    app
}

#[cfg(any(target_os = "android", target_os = "ios"))]
pub(crate) fn change_input(app: &mut App, key_code: KeyCode, state: ButtonState) {
    let mut windows_system_state: SystemState<Query<(Entity, &mut Window)>> = SystemState::from_world(app.world_mut());
    let windows = windows_system_state.get_mut(app.world_mut());
    if let Ok((entity, _)) = windows.single() {
        let input = KeyboardInput {
            logical_key: if key_code == KeyCode::ArrowLeft { Key::ArrowLeft } else { Key::ArrowRight },
            state,
            key_code: key_code,
            window: entity,
            repeat: false,
            text: None,
        };
        app.world_mut().send_event(input);
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[allow(clippy::type_complexity)]
pub(crate) fn close_bevy_window(mut app: Box<App>) {
    let mut windows_state: SystemState<(Commands, Query<(Entity, &mut Window)>, EventWriter<AppExit>)> = SystemState::from_world(app.world_mut());
    let (mut commands, windows, mut app_exit_events) = windows_state.get_mut(app.world_mut());
    for (window, _focus) in windows.iter() { commands.entity(window).despawn(); }
    app_exit_events.write(AppExit::Success);
    windows_state.apply(app.world_mut());
    app.update();
}
