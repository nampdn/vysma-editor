use core::time::Duration;

use bevy::prelude::*;
use bevy_in_app::common::{
    cli::{Cli, Mode},
    shared::SEND_INTERVAL,
};
use lightyear::prelude::{ReplicationSender, SendUpdatesMode};

#[cfg(feature = "client")]
use bevy_in_app::client::VysmaClientPlugin;
#[cfg(feature = "server")]
use bevy_in_app::server::VysmaServerPlugin;

use bevy_in_app::renderer::RendererPlugin;
use bevy_in_app::shared::SharedPlugin;

#[cfg(any(target_os = "android", target_os = "ios"))]
fn main() {}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn main() {
    use bevy_in_app::common::shared::FIXED_TIMESTEP_HZ;

    let cli = Cli::default();

    let mut app = cli.build_app(Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ), true);
    app.add_plugins(SharedPlugin);

    match cli.mode {
        #[cfg(feature = "client")]
        Some(Mode::Client { .. }) => {
            // Client: run full HCL authoring/runtime and load example project scene
            app.add_plugins(bevy_in_app::hcl::HclPlugin);
            app.add_systems(Startup, bevy_in_app::hcl::load_scene_at_startup("moba_hcl/moba_game.hcl"));

            use lightyear::prelude::Client;
            app.add_plugins(VysmaClientPlugin);
            let client = app
                .world_mut()
                .query_filtered::<Entity, With<Client>>()
                .single(app.world_mut())
                .unwrap();
            // We are doing client->server replication so we need to include a ReplicationSender for the client
            app.world_mut()
                .entity_mut(client)
                .insert(ReplicationSender::new(
                    SEND_INTERVAL,
                    SendUpdatesMode::SinceLastAck,
                    false,
                ));
        }
        #[cfg(feature = "server")]
        Some(Mode::Server { .. }) => {
            // Server: only handle HCL networking (updates + publishing). Do not run HCL runtime/spawn.
            app.add_plugins(bevy_in_app::hcl::net::HclNetPlugin);
            app.add_plugins(VysmaServerPlugin);
        }
        #[cfg(all(feature = "client", feature = "server"))]
        Some(Mode::HostClient { client_id: _ }) => {
            // Host-client: run full HCL (for preview/authoring) and both client/server plugins
            app.add_plugins(bevy_in_app::hcl::HclPlugin);
            app.add_systems(Startup, bevy_in_app::hcl::load_scene_at_startup("moba_hcl/moba_game.hcl"));
            app.add_plugins(VysmaClientPlugin);
            app.add_plugins(VysmaServerPlugin);
        }
        _ => {}
    }

    #[cfg(feature = "gui")]
    app.add_plugins(RendererPlugin);

    app.run();
}
