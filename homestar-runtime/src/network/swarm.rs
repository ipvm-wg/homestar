//! Sets up a libp2p [Swarm], containing the state of the network and the way
//! it should behave.
//!
//! [Swarm]: libp2p::Swarm

use crate::{
    network::{error::PubSubError, pubsub},
    settings, Receipt, RECEIPT_TAG, WORKFLOW_TAG,
};
use anyhow::{Context, Result};
use const_format::formatcp;
use enum_assoc::Assoc;
use faststr::FastStr;
use futures::future::Either;
use libp2p::{
    autonat,
    core::{
        muxing::StreamMuxerBox,
        transport::{self, OptionalTransport},
        upgrade,
    },
    dns,
    gossipsub::{self, MessageId, TopicHash},
    identify,
    identity::Keypair,
    kad::{
        self,
        store::{MemoryStore, MemoryStoreConfig},
    },
    mdns,
    multiaddr::Protocol,
    noise, quic, rendezvous,
    request_response::{self, ProtocolSupport},
    swarm::{self, behaviour::toggle::Toggle, NetworkBehaviour, Swarm},
    yamux, PeerId, StreamProtocol, Transport,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{info, warn};

/// Homestar protocol version, shared among peers, tied to the homestar version.
pub(crate) const HOMESTAR_PROTOCOL_VER: &str = formatcp!("homestar/{VERSION}");

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build a new [Swarm] with a given transport and a tokio executor.
pub(crate) async fn new(settings: &settings::Network) -> Result<Swarm<ComposedBehaviour>> {
    let keypair = settings
        .keypair_config
        .keypair()
        .with_context(|| "failed to generate/import keypair for libp2p".to_string())?;

    let peer_id = keypair.public().to_peer_id();
    info!(
        subject = "swarm.init",
        category = "libp2p.swarm",
        peer_id = peer_id.to_string(),
        "local peer ID generated"
    );

    let transport = build_transport(settings, keypair.clone())?;

    let mut swarm = Swarm::new(
        transport,
        ComposedBehaviour {
            autonat: autonat::Behaviour::new(
                keypair.clone().public().to_peer_id(),
                autonat::Config {
                    boot_delay: settings.libp2p().autonat().boot_delay,
                    retry_interval: settings.libp2p().autonat().retry_interval,
                    throttle_server_period: settings.libp2p().autonat().throttle_server_period,
                    only_global_ips: settings.libp2p().autonat().only_public_ips,
                    ..Default::default()
                },
            ),
            gossipsub: Toggle::from(if settings.libp2p.pubsub.enable {
                Some(pubsub::new(keypair.clone(), settings.libp2p().pubsub())?)
            } else {
                None
            }),
            kademlia: kad::Behaviour::with_config(
                peer_id,
                MemoryStore::with_config(
                    peer_id,
                    MemoryStoreConfig {
                        // TODO: if below a better max, rely on cache-store or
                        // blockstore to fetch result if requested directly.
                        // 2gb at the moment
                        max_value_bytes: 10 * 1024 * 1024,
                        ..Default::default()
                    },
                ),
                {
                    let mut cfg = kad::Config::default();
                    // Set max packet size for records put to the DHT.
                    // Currently set to 2gb.
                    cfg.set_max_packet_size(10 * 1024 * 1024);
                    // Only add peers to the routing table manually.
                    cfg.set_kbucket_inserts(kad::BucketInserts::Manual);
                    cfg
                },
            ),
            request_response: request_response::cbor::Behaviour::new(
                [(
                    StreamProtocol::new("/homestar-exchange/1.0.0"),
                    ProtocolSupport::Full,
                )],
                request_response::Config::default(),
            ),
            mdns: Toggle::from(if settings.libp2p.mdns.enable {
                Some(mdns::Behaviour::new(
                    mdns::Config {
                        ttl: settings.libp2p.mdns.ttl,
                        query_interval: settings.libp2p.mdns.query_interval,
                        enable_ipv6: settings.libp2p.mdns.enable_ipv6,
                    },
                    peer_id,
                )?)
            } else {
                None
            }),
            rendezvous_client: Toggle::from(if settings.libp2p.rendezvous.enable_client {
                Some(rendezvous::client::Behaviour::new(keypair.clone()))
            } else {
                None
            }),
            rendezvous_server: Toggle::from(if settings.libp2p.rendezvous.enable_server {
                Some(rendezvous::server::Behaviour::new(
                    rendezvous::server::Config::with_min_ttl(
                        rendezvous::server::Config::default(),
                        1,
                    ),
                ))
            } else {
                None
            }),
            identify: identify::Behaviour::new(
                identify::Config::new(HOMESTAR_PROTOCOL_VER.to_string(), keypair.public())
                    .with_agent_version(format!("homestar-runtime/{}", env!("CARGO_PKG_VERSION"))),
            ),
        },
        peer_id,
        swarm::Config::with_tokio_executor()
            .with_idle_connection_timeout(settings.libp2p.idle_connection_timeout),
    );

    init(&mut swarm, settings)?;

    Ok(swarm)
}

/// Initialize a [Swarm] with given [settings::Network].
///
/// Steps includes:
/// - Listen on given address.
/// - Dial nodes specified in configuration and add them to kademlia.
/// - Subscribe to `receipts` topic for [gossipsub].
///
/// [gossipsub]: libp2p::gossipsub
pub(crate) fn init(
    swarm: &mut Swarm<ComposedBehaviour>,
    settings: &settings::Network,
) -> Result<()> {
    // Listen-on given address
    swarm.listen_on(settings.libp2p.listen_address.to_string().parse()?)?;

    // Set Kademlia server mode
    swarm
        .behaviour_mut()
        .kademlia
        .set_mode(Some(kad::Mode::Server));

    // add external addresses from settings
    if !settings.libp2p.announce_addresses.is_empty() {
        for addr in settings.libp2p.announce_addresses.iter() {
            swarm.add_external_address(addr.clone());
        }
    } else {
        info!(
            subject = "swarm.init",
            category = "libp2p.swarm",
            "no addresses to announce to peers defined in settings: node may be unreachable to external peers"
        )
    }

    // Dial nodes specified in settings. Failure here shouldn't halt node startup.
    for (index, addr) in settings.libp2p.node_addresses.iter().enumerate() {
        if index < settings.libp2p.max_connected_peers as usize {
            let _ = swarm
                .dial(addr.clone())
                // log dial failure and continue
                .map_err(|e| {
                    warn!(subject = "swarm.init.err",
                          category = "libp2p.swarm",
                          err=?e, "failed to dial configured node")
                });

            // add node to kademlia routing table
            if let Some(Protocol::P2p(peer_id)) =
                addr.iter().find(|proto| matches!(proto, Protocol::P2p(_)))
            {
                info!(subject = "swarm.init",
                      category = "libp2p.swarm",
                      addr=?addr,
                      "added configured node to kademlia routing table");
                swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, addr.clone());
            } else {
                warn!(subject = "swarm.init.err",
                      category = "libp2p.swarm",
                      addr=?addr,
                      err="configured node address did not include a peer ID",
                      "node not added to kademlia routing table")
            }
        } else {
            warn!(subject = "swarm.init.err",
                  category = "libp2p.swarm",
                  addr=?addr,
                  "address not dialed because node addresses count exceeds max connected peers configuration")
        }
    }

    if settings.libp2p.pubsub.enable {
        // join `receipts` topic
        swarm
            .behaviour_mut()
            .gossip_subscribe(pubsub::RECEIPTS_TOPIC)?;
    }

    Ok(())
}

/// Discovery information for peers discovered through rendezvous.
#[derive(Debug, Clone)]
pub(crate) struct PeerDiscoveryInfo {
    pub(crate) rendezvous_point: PeerId,
}

impl PeerDiscoveryInfo {
    /// Create a new [PeerDiscoveryInfo] with the rendezvous point [PeerId] where
    /// a peer was discovered.
    ///
    /// [PeerId]: libp2p::PeerId
    pub(crate) fn new(rendezvous_point: PeerId) -> Self {
        Self { rendezvous_point }
    }
}

/// Key data structure for [request_response::Event] messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RequestResponseKey {
    pub(crate) cid: FastStr,
    pub(crate) capsule_tag: CapsuleTag,
}

impl RequestResponseKey {
    /// Create a new [RequestResponseKey] with a given Cid string and capsule tag.
    pub(crate) fn new(cid: FastStr, capsule_tag: CapsuleTag) -> Self {
        Self { cid, capsule_tag }
    }
}

/// Tag for [RequestResponseKey] to indicate the type of capsule wrapping.
#[derive(Debug, Clone, Assoc, Serialize, Deserialize)]
#[func(pub(crate) fn tag(&self) -> &'static str)]
#[func(pub(crate) fn capsule_type(s: &str) -> Option<Self>)]
pub(crate) enum CapsuleTag {
    /// Receipt capsule-tag-wrapper: [RECEIPT_TAG].
    #[assoc(tag = RECEIPT_TAG)]
    #[assoc(capsule_type = RECEIPT_TAG)]
    Receipt,
    /// Workflow/workflow-info capsule-tag-wrapper: [WORKFLOW_TAG].
    #[assoc(tag = WORKFLOW_TAG)]
    #[assoc(capsule_type = WORKFLOW_TAG)]
    Workflow,
}

impl fmt::Display for CapsuleTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag())
    }
}

/// Custom event types to listen for and respond to.
#[derive(Debug)]
pub(crate) enum ComposedEvent {
    /// [autonat::Event] event.
    Autonat(autonat::Event),
    /// [gossipsub::Event] event.
    Gossipsub(Box<gossipsub::Event>),
    /// [kad::Event] event.
    Kademlia(kad::Event),
    /// [request_response::Event] event.
    RequestResponse(request_response::Event<RequestResponseKey, Vec<u8>>),
    /// [mdns::Event] event.
    Mdns(mdns::Event),
    /// [rendezvous::client::Event] event.
    RendezvousClient(rendezvous::client::Event),
    /// [rendezvous::server::Event] event.
    RendezvousServer(rendezvous::server::Event),
    /// [identify::Event] event.
    Identify(identify::Event),
}

/// Message types to deliver on a topic.
#[derive(Debug)]
pub(crate) enum TopicMessage {
    /// Receipt topic, wrapping [Receipt].
    CapturedReceipt(pubsub::Message<Receipt>),
}

/// Custom behaviours for [Swarm].
#[allow(missing_debug_implementations)]
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "ComposedEvent")]
pub(crate) struct ComposedBehaviour {
    /// [autonat::Behaviour] behaviour.
    pub(crate) autonat: autonat::Behaviour,
    /// [gossipsub::Behaviour] behaviour.
    pub(crate) gossipsub: Toggle<gossipsub::Behaviour>,
    /// In-memory [kademlia: kad::Behaviour] behaviour.
    pub(crate) kademlia: kad::Behaviour<MemoryStore>,
    /// [request_response::Behaviour] CBOR-flavored behaviour.
    pub(crate) request_response: request_response::cbor::Behaviour<RequestResponseKey, Vec<u8>>,
    /// [mdns::tokio::Behaviour] behaviour.
    pub(crate) mdns: Toggle<mdns::tokio::Behaviour>,
    /// [rendezvous::client::Behaviour] behaviour.
    pub(crate) rendezvous_client: Toggle<rendezvous::client::Behaviour>,
    /// [rendezvous::server::Behaviour] behaviour.
    pub(crate) rendezvous_server: Toggle<rendezvous::server::Behaviour>,
    /// [identify::Behaviour] behaviour.
    pub(crate) identify: identify::Behaviour,
}

impl ComposedBehaviour {
    /// Subscribe to [gossipsub] topic.
    pub(crate) fn gossip_subscribe(&mut self, topic: &str) -> Result<bool, PubSubError> {
        if let Some(gossipsub) = self.gossipsub.as_mut() {
            let topic = gossipsub::IdentTopic::new(topic);
            let subscribed = gossipsub.subscribe(&topic)?;

            Ok(subscribed)
        } else {
            Err(PubSubError::NotEnabled)
        }
    }

    /// Serialize [TopicMessage] and publish to [gossipsub] topic.
    pub(crate) fn gossip_publish(
        &mut self,
        topic: &str,
        msg: TopicMessage,
    ) -> Result<MessageId, PubSubError> {
        if let Some(gossipsub) = self.gossipsub.as_mut() {
            let id_topic = gossipsub::IdentTopic::new(topic);
            // Make this a match once we have other topics.
            let TopicMessage::CapturedReceipt(message) = msg;
            let msg_bytes: Vec<u8> = message.try_into()?;

            if gossipsub
                .mesh_peers(&TopicHash::from_raw(topic))
                .peekable()
                .peek()
                .is_some()
            {
                let msg_id = gossipsub.publish(id_topic, msg_bytes)?;
                Ok(msg_id)
            } else {
                Err(PubSubError::InsufficientPeers(topic.to_owned()))
            }
        } else {
            Err(PubSubError::NotEnabled)
        }
    }
}

impl From<autonat::Event> for ComposedEvent {
    fn from(event: autonat::Event) -> Self {
        ComposedEvent::Autonat(event)
    }
}

impl From<gossipsub::Event> for ComposedEvent {
    fn from(event: gossipsub::Event) -> Self {
        ComposedEvent::Gossipsub(Box::new(event))
    }
}

impl From<kad::Event> for ComposedEvent {
    fn from(event: kad::Event) -> Self {
        ComposedEvent::Kademlia(event)
    }
}

impl From<request_response::Event<RequestResponseKey, Vec<u8>>> for ComposedEvent {
    fn from(event: request_response::Event<RequestResponseKey, Vec<u8>>) -> Self {
        ComposedEvent::RequestResponse(event)
    }
}

impl From<mdns::Event> for ComposedEvent {
    fn from(event: mdns::Event) -> Self {
        ComposedEvent::Mdns(event)
    }
}

impl From<rendezvous::client::Event> for ComposedEvent {
    fn from(event: rendezvous::client::Event) -> Self {
        ComposedEvent::RendezvousClient(event)
    }
}

impl From<rendezvous::server::Event> for ComposedEvent {
    fn from(event: rendezvous::server::Event) -> Self {
        ComposedEvent::RendezvousServer(event)
    }
}

impl From<identify::Event> for ComposedEvent {
    fn from(event: identify::Event) -> Self {
        ComposedEvent::Identify(event)
    }
}

fn build_transport(
    settings: &settings::Network,
    keypair: Keypair,
) -> Result<transport::Boxed<(PeerId, StreamMuxerBox)>> {
    let build_tcp = || libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::new().nodelay(true));
    let build_quic = if settings.libp2p.quic.enable {
        OptionalTransport::some(quic::tokio::Transport::new(quic::Config::new(&keypair)))
    } else {
        OptionalTransport::none()
    };

    let dns_transport = || {
        if let Ok((conf, opts)) = hickory_resolver::system_conf::read_system_conf() {
            info!(
                subject = "swarm.init",
                category = "libp2p.swarm",
                "using system DNS configuration from /etc/resolv.conf"
            );
            dns::tokio::Transport::custom(build_tcp(), conf, opts)
        } else {
            info!(
                subject = "swarm.init",
                category = "libp2p.swarm",
                "using cloudflare DNS configuration as a fallback"
            );
            dns::tokio::Transport::custom(
                build_tcp(),
                dns::ResolverConfig::cloudflare(),
                dns::ResolverOpts::default(),
            )
        }
    };

    // ws + wss transport or dns + tcp transport or ws + tcp
    let transport = libp2p::websocket::WsConfig::new(dns_transport())
        .or_transport(dns_transport())
        .or_transport(libp2p::websocket::WsConfig::new(build_tcp()))
        .upgrade(upgrade::Version::V1Lazy)
        .authenticate(noise::Config::new(&keypair)?)
        .multiplex(yamux::Config::default())
        .timeout(settings.libp2p.transport_connection_timeout)
        .or_transport(build_quic)
        .map(|either_output, _| match either_output {
            Either::Left((peer_id, muxer)) => (peer_id, StreamMuxerBox::new(muxer)),
            Either::Right((peer_id, muxer)) => (peer_id, StreamMuxerBox::new(muxer)),
        })
        .boxed();

    Ok(transport)
}
