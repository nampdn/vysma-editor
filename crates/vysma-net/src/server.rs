#![allow(unused_imports)]
#![allow(unused_variables)]
use core::net::{Ipv4Addr, SocketAddr};
use bevy::prelude::*;
use core::time::Duration;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
#[cfg(not(target_family = "wasm"))]
use bevy::tasks::IoTaskPool;
use lightyear::netcode::{NetcodeServer, PRIVATE_KEY_BYTES};
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::shared::SharedSettings;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ServerTransports {
    Udp { local_port: u16 },
    WebTransport { local_port: u16, certificate: WebTransportCertificateSettings },
    WebSocket { local_port: u16 },
    Steam { local_port: u16 },
}

#[derive(Component, Debug)]
#[component(on_add = ServerNetwork::on_add)]
pub struct ServerNetwork {
    /// Possibly add a conditioner to simulate network conditions
    pub conditioner: Option<RecvLinkConditioner>,
    /// Which transport to use
    pub transport: ServerTransports,
    pub shared: SharedSettings,
}

impl ServerNetwork {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;
        world.commands().queue(move |world: &mut World| -> Result {
            let mut entity_mut = world.entity_mut(entity);
            let settings = entity_mut.take::<ServerNetwork>().unwrap();
            entity_mut.insert((Name::from("Server"),));

            let add_netcode = |entity_mut: &mut EntityWorldMut| {
                let private_key = settings.shared.private_key;
                entity_mut.insert(NetcodeServer::new(NetcodeConfig { protocol_id: settings.shared.protocol_id, private_key, ..Default::default() }));
            };

            match settings.transport {
                ServerTransports::Udp { local_port } => {
                    add_netcode(&mut entity_mut);
                    let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), local_port);
                    entity_mut.insert((LocalAddr(server_addr), ServerUdpIo::default()));
                }
                ServerTransports::WebTransport { local_port, certificate } => {
                    add_netcode(&mut entity_mut);
                    #[cfg(feature = "webtransport")]
                    {
                        let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), local_port);
                        entity_mut.insert((LocalAddr(server_addr), WebTransportServerIo { certificate: (&certificate).into() }));
                    }
                }
                ServerTransports::WebSocket { local_port: _ } => {
                    add_netcode(&mut entity_mut);
                    // TODO: implement WebSocket server IO when feature is added
                }
                ServerTransports::Steam { local_port: _ } => {
                    add_netcode(&mut entity_mut);
                    // TODO: implement Steam server IO when feature is added
                }
            };
            Ok(())
        });
    }
}

pub fn start(mut commands: Commands, server: Single<Entity, With<Server>>) {
    commands.trigger_targets(Start, server.into_inner());
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WebTransportCertificateSettings {
    AutoSelfSigned(Vec<String>),
    FromFile { cert: String, key: String },
}

impl Default for WebTransportCertificateSettings {
    fn default() -> Self {
        let sans = vec!["localhost".to_string(), "127.0.0.1".to_string(), "::1".to_string()];
        WebTransportCertificateSettings::AutoSelfSigned(sans)
    }
}

#[cfg(feature = "webtransport")]
impl From<&WebTransportCertificateSettings> for Identity {
    fn from(wt: &WebTransportCertificateSettings) -> Identity {
        match wt {
            WebTransportCertificateSettings::AutoSelfSigned(sans) => {
                let mut sans = sans.clone();
                if let Ok(public_ip) = std::env::var("ARBITRIUM_PUBLIC_IP") {
                    println!("🔐 SAN += ARBITRIUM_PUBLIC_IP: {public_ip}");
                    sans.push(public_ip);
                    sans.push("*.pr.edgegap.net".to_string());
                }
                if let Ok(san) = std::env::var("SELF_SIGNED_SANS") {
                    println!("🔐 SAN += SELF_SIGNED_SANS: {san}");
                    sans.extend(san.split(',').map(|s| s.to_string()));
                }
                println!("🔐 Generating self-signed certificate with SANs: {sans:?}");
                let identity = Identity::self_signed(sans).unwrap();
                let digest = identity.certificate_chain().as_slice()[0].hash();
                println!("🔐 Certificate digest: {digest}");
                identity
            }
            WebTransportCertificateSettings::FromFile { cert: cert_pem_path, key: private_key_pem_path } => {
                println!("Reading certificate PEM files:\n * cert: {cert_pem_path}\n * key: {private_key_pem_path}",);
                let identity = bevy::tasks::IoTaskPool::get()
                    .scope(|s| {
                        s.spawn(async move {
                            Identity::load_pemfiles(cert_pem_path, private_key_pem_path).await.unwrap()
                        });
                    })
                    .pop()
                    .unwrap();
                let digest = identity.certificate_chain().as_slice()[0].hash();
                println!("🔐 Certificate digest: {digest}");
                identity
            }
        }
    }
} 