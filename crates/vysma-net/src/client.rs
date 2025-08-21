#![allow(unused_imports)]
#![allow(unused_variables)]
use core::net::{Ipv4Addr, SocketAddr};
use bevy::ecs::{component::HookContext, world::DeferredWorld};
use bevy::prelude::*;
use lightyear::netcode::{client_plugin::NetcodeConfig, NetcodeClient};
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::shared::SharedSettings;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClientTransports {
    #[cfg(not(target_family = "wasm"))]
    Udp,
    WebTransport,
    #[cfg(feature = "websocket")]
    WebSocket,
    #[cfg(feature = "steam")]
    Steam,
}

/// Event that examples can trigger to spawn a client.
#[derive(Component, Clone, Debug)]
#[component(on_add = ClientNetwork::on_add)]
pub struct ClientNetwork {
    pub client_id: u64,
    /// The client port to listen on
    pub client_port: u16,
    /// The socket address of the server
    pub server_addr: SocketAddr,
    /// Possibly add a conditioner to simulate network conditions
    pub conditioner: Option<RecvLinkConditioner>,
    /// Which transport to use
    pub transport: ClientTransports,
    pub shared: SharedSettings,
}

impl ClientNetwork {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;
        world.commands().queue(move |world: &mut World| -> Result {
            let mut entity_mut = world.entity_mut(entity);
            let settings = entity_mut.take::<ClientNetwork>().unwrap();
            let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), settings.client_port);

            entity_mut.insert((
                Client::default(),
                Link::new(settings.conditioner.clone()),
                LocalAddr(client_addr),
                PeerAddr(settings.server_addr),
                ReplicationReceiver::default(),
                PredictionManager::default(),
                InterpolationManager::default(),
                Name::from("Client"),
            ));

            let add_netcode = |entity_mut: &mut EntityWorldMut| -> Result {
                let auth = Authentication::Manual {
                    server_addr: settings.server_addr,
                    client_id: settings.client_id,
                    private_key: settings.shared.private_key,
                    protocol_id: settings.shared.protocol_id,
                };
                let netcode_config = NetcodeConfig { client_timeout_secs: 3, token_expire_secs: -1, ..Default::default() };
                entity_mut.insert(NetcodeClient::new(auth, netcode_config)?);
                Ok(())
            };

            match settings.transport {
                #[cfg(not(target_family = "wasm"))]
                ClientTransports::Udp => {
                    add_netcode(&mut entity_mut)?;
                    entity_mut.insert(UdpIo::default());
                }
                ClientTransports::WebTransport => {
                    add_netcode(&mut entity_mut)?;
                    #[cfg(feature = "webtransport")]
                    {
                        let certificate_digest = {
                            #[cfg(target_family = "wasm")]
                            { include_str!("../../../certificate/digest.txt").to_string() }
                            #[cfg(not(target_family = "wasm"))]
                            { "".to_string() }
                        };
                        entity_mut.insert(WebTransportClientIo { certificate_digest });
                    }
                }
                #[cfg(feature = "steam")]
                ClientTransports::Steam => {
                    entity_mut.insert(SteamClientTo { target: COnnectTarget::Addr(settings.server_addr), config: Default::default() });
                }
            }

            Ok(())
        });
    }
}

pub fn connect(mut commands: Commands, client: Single<Entity, With<Client>>) {
    commands.trigger_targets(Connect, client.into_inner());
} 