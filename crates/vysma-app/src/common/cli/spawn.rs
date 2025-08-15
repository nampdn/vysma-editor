use bevy::prelude::*;
use lightyear::link::RecvLinkConditioner;
use lightyear::prelude::LinkConditionerConfig;
use crate::common::shared::{CLIENT_PORT, SERVER_ADDR, SERVER_PORT, SHARED_SETTINGS};
use super::Mode;

pub fn spawn_connections(app: &mut App, mode: &Mode) {
	let conditioner = LinkConditionerConfig::average_condition();
	match mode {
		#[cfg(feature = "client")]
		Mode::Client { client_id } => {
			let _client = app
				.world_mut()
				.spawn(vysma_net::client::ClientNetwork {
					client_id: client_id.expect("You need to specify a client_id via `-c ID`") ,
					client_port: CLIENT_PORT,
					server_addr: SERVER_ADDR,
					conditioner: Some(RecvLinkConditioner::new(conditioner.clone())),
					transport: vysma_net::client::ClientTransports::Udp,
					shared: SHARED_SETTINGS,
				})
				.id();
			app.add_systems(Startup, vysma_net::client::connect);
		}
		#[cfg(feature = "server")]
		Mode::Server => {
			use vysma_net::server::{start, ServerNetwork, ServerTransports};

			let _server = app
				.world_mut()
				.spawn(ServerNetwork {
					conditioner: None,
					transport: ServerTransports::Udp { local_port: SERVER_PORT },
					shared: SHARED_SETTINGS,
				})
				.id();
			app.add_systems(Startup, start);
		}
		#[cfg(all(feature = "client", feature = "server"))]
		Mode::HostClient { client_id: _ } => {
			// See commented example for spawning both server and client
		}
		#[allow(unreachable_patterns)]
		_ => {}
	}
} 