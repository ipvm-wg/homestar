//! Internal libp2p [SwarmEvent] handling and [Handler] implementation.

use super::EventHandler;
#[cfg(feature = "websocket-notify")]
use crate::event_handler::notification::{self, NetworkNotification};
#[cfg(feature = "ipfs")]
use crate::network::IpfsCli;
use crate::{
    db::Database,
    event_handler::{
        cache::{self, CacheData, CacheValue},
        Event, Handler, RequestResponseError,
    },
    libp2p::multiaddr::MultiaddrExt,
    network::{
        pubsub,
        swarm::{
            CapsuleTag, ComposedEvent, PeerDiscoveryInfo, RequestResponseKey, HOMESTAR_PROTOCOL_VER,
        },
    },
    workflow,
    workflow::WORKFLOW_TAG,
    Db, Receipt,
};
use anyhow::{anyhow, Result};
use libipld::Cid;
#[cfg(feature = "websocket-notify")]
use libp2p::Multiaddr;
use libp2p::{
    autonat::{self, NatStatus},
    gossipsub, identify, kad,
    kad::{AddProviderOk, BootstrapOk, GetProvidersOk, GetRecordOk, PutRecordOk, QueryResult},
    mdns,
    multiaddr::Protocol,
    rendezvous::{self, Namespace, Registration},
    request_response,
    swarm::{dial_opts::DialOpts, SwarmEvent},
    PeerId, StreamProtocol,
};
#[cfg(feature = "websocket-notify")]
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};
use tracing::{debug, error, info, warn};

pub(crate) mod record;
pub(crate) use record::*;

const RENDEZVOUS_PROTOCOL_NAME: StreamProtocol = StreamProtocol::new("/rendezvous/1.0.0");
const RENDEZVOUS_NAMESPACE: &str = "homestar";

/// Internal events within the [SwarmEvent] context related to finding results
/// on the DHT.
#[derive(Debug)]
pub(crate) enum ResponseEvent {
    /// Found [libp2p::kad::PeerRecord] on the DHT.
    Found(Result<FoundEvent>),
    /// Found providers/[PeerId]s on the DHT.
    #[allow(dead_code)]
    Providers(Result<HashSet<PeerId>>),
}

/// Internal events within the [SwarmEvent] context related to finding specific
/// data items on the DHT.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FoundEvent {
    /// Found [Receipt] on the DHT.
    Receipt(ReceiptEvent),
    /// Found [workflow::Info] on the DHT.
    Workflow(WorkflowInfoEvent),
}

/// [FoundEvent] variant for receipts found on the DHT.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ReceiptEvent {
    pub(crate) peer_id: Option<PeerId>,
    pub(crate) receipt: Receipt,
}

/// [FoundEvent] variant for workflow info found on the DHT.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct WorkflowInfoEvent {
    pub(crate) peer_id: Option<PeerId>,
    pub(crate) workflow_info: workflow::Info,
    #[cfg(feature = "websocket-notify")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket-notify")))]
    pub(crate) workflow_source: notification::WorkflowInfoSource,
}

impl<DB> Handler<DB> for SwarmEvent<ComposedEvent>
where
    DB: Database + Sync,
{
    #[cfg(feature = "ipfs")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ipfs")))]
    async fn handle_event(self, event_handler: &mut EventHandler<DB>, _ipfs: IpfsCli) {
        handle_swarm_event(self, event_handler).await
    }

    #[cfg(not(feature = "ipfs"))]
    async fn handle_event(self, event_handler: &mut EventHandler<DB>) {
        handle_swarm_event(self, event_handler).await
    }
}

async fn handle_swarm_event<DB: Database>(
    event: SwarmEvent<ComposedEvent>,
    event_handler: &mut EventHandler<DB>,
) {
    match event {
        SwarmEvent::Behaviour(ComposedEvent::Autonat(autonat_event)) => {
            match autonat_event {
                autonat::Event::InboundProbe(event) => match event {
                    autonat::InboundProbeEvent::Request {
                        peer, addresses, ..
                    } => {
                        debug!(
                            subject = "libp2p.autonat.inbound_probe",
                            category = "handle_swarm_event",
                            peer_id = peer.to_string(),
                            addresses = ?addresses,
                            "received a probe request",
                        );
                    }
                    autonat::InboundProbeEvent::Response { peer, address, .. } => {
                        debug!(
                            subject = "libp2p.autonat.inbound_probe",
                            category = "handle_swarm_event",
                            peer_id = peer.to_string(),
                            address = address.to_string(),
                            "successfully probed an external address for a peer",
                        );
                    }
                    autonat::InboundProbeEvent::Error { peer, error, .. } => {
                        debug!(
                            subject = "libp2p.autonat.inbound_probe",
                            category = "handle_swarm_event",
                            peer_id = peer.to_string(),
                            error = ?error,
                            "unable to probe a peer",
                        );
                    }
                },
                autonat::Event::OutboundProbe(event) => match event {
                    autonat::OutboundProbeEvent::Request { peer, .. } => {
                        debug!(
                            subject = "libp2p.autonat.outbound_probe",
                            category = "handle_swarm_event",
                            peer_id = peer.to_string(),
                            "requested a probe from a peer",
                        );
                    }
                    autonat::OutboundProbeEvent::Response { peer, address, .. } => {
                        debug!(
                            subject = "libp2p.autonat.outbound_probe",
                            category = "handle_swarm_event",
                            peer_id = peer.to_string(),
                            address = address.to_string(),
                            "peer successfully probed an external address",
                        );
                    }
                    autonat::OutboundProbeEvent::Error { peer, error, .. } => {
                        debug!(
                            subject = "libp2p.autonat.outbound_probe",
                            category = "handle_swarm_event",
                            peer_id = peer.map(|p| p.to_string()).unwrap_or("<none>".to_string()),
                            error = ?error,
                            "requested probe failed",
                        );
                    }
                },
                autonat::Event::StatusChanged { old, new } => {
                    match &new {
                        NatStatus::Public(address) => {
                            event_handler.swarm.add_external_address(address.clone());

                            info!(
                                subject = "libp2p.autonat.status_change",
                                category = "handle_swarm_event",
                                address = address.to_string(),
                                "confirmed a public address",
                            );
                        }
                        _ => {
                            if let NatStatus::Public(address) = old {
                                // Announce addresses are configured and should not be removed
                                if !event_handler.announce_addresses.contains(&address) {
                                    event_handler.swarm.remove_external_address(&address);

                                    info!(
                                        subject = "libp2p.autonat.status_change",
                                        category = "handle_swarm_event",
                                        address = address.to_string(),
                                        "removed an address that is no longer public",
                                    );
                                }
                            }
                        }
                    }

                    #[cfg(feature = "websocket-notify")]
                    notification::emit_network_event(
                        event_handler.ws_evt_sender(),
                        NetworkNotification::StatusChangedAutonat(
                            notification::StatusChangedAutonat::new(new),
                        ),
                    );
                }
            }
        }
        SwarmEvent::Behaviour(ComposedEvent::Identify(identify_event)) => {
            match identify_event {
                identify::Event::Error { peer_id, error } => {
                    warn!(subject = "libp2p.identify.err",
                          category = "handle_swarm_event",
                          peer_id=peer_id.to_string(),
                          err=?error,
                          "error while attempting to identify the remote")
                }
                identify::Event::Sent { peer_id } => {
                    debug!(
                        subject = "libp2p.identify.sent",
                        category = "handle_swarm_event",
                        peer_id = peer_id.to_string(),
                        "sent identify info to peer"
                    )
                }
                identify::Event::Received { peer_id, info } => {
                    debug!(subject = "libp2p.identify.recv",
                           category = "handle_swarm_event",
                           peer_id=peer_id.to_string(),
                           info=?info,
                           "identify info received from peer");

                    // Ignore peers that do not use the Homestar protocol
                    if info.protocol_version != HOMESTAR_PROTOCOL_VER {
                        debug!(subject ="libp2p.identify.recv",
                               category="handle_swarm_event",
                               protocol_version=info.protocol_version,
                               "peer was not using our homestar protocol version: {HOMESTAR_PROTOCOL_VER}");
                        return;
                    }

                    let num_addresses = event_handler.swarm.external_addresses().count();

                    // Probe observed address as an external address if we are identifying ourselves
                    if &peer_id == event_handler.swarm.local_peer_id()
                        && num_addresses < event_handler.external_address_limit as usize
                    {
                        info.observed_addr
                            .iter()
                            // If any part of the multiaddr includes a private IP, don't add it to our external address list
                            .filter_map(|proto| match proto {
                                Protocol::Ip4(ip) => Some(ip),
                                _ => None,
                            })
                            .all(|proto| !proto.is_private())
                            // We have observed a potentially valid external address that we weren't aware of.
                            // Probe it with AutoNAT to confirm it and on confirmation add it to addresses we announce to peers.
                            .then(|| {
                                event_handler
                                    .swarm
                                    .behaviour_mut()
                                    .autonat
                                    .probe_address(info.observed_addr)
                            });
                    }

                    let behavior = event_handler.swarm.behaviour_mut();

                    // Add listen addresses to kademlia routing table
                    if info.protocols.contains(&kad::PROTOCOL_NAME) {
                        for addr in info.listen_addrs {
                            behavior.kademlia.add_address(&peer_id, addr);
                            debug!(
                                subject = "libp2p.identify.recv",
                                category = "handle_swarm_event",
                                peer_id = peer_id.to_string(),
                                "added identified node to kademlia routing table"
                            );
                        }
                    }

                    // Register and discover with nodes running the rendezvous protocol
                    if info.protocols.contains(&RENDEZVOUS_PROTOCOL_NAME) {
                        if let Some(rendezvous_client) = event_handler
                            .swarm
                            .behaviour_mut()
                            .rendezvous_client
                            .as_mut()
                        {
                            // register self with remote
                            if let Err(err) = rendezvous_client.register(
                                Namespace::from_static(RENDEZVOUS_NAMESPACE),
                                peer_id,
                                Some(event_handler.rendezvous.registration_ttl.as_secs()),
                            ) {
                                warn!(
                                    subject = "libp2p.identify.recv",
                                    category = "handle_swarm_event",
                                    peer_id = peer_id.to_string(),
                                    err = format!("{err}"),
                                    "failed to register with rendezvous peer"
                                )
                            }

                            // Discover other nodes
                            rendezvous_client.discover(
                                Some(Namespace::from_static(RENDEZVOUS_NAMESPACE)),
                                None,
                                None,
                                peer_id,
                            );
                        }
                    }
                }
                identify::Event::Pushed { peer_id, .. } => debug!(
                    subject = "libp2p.identify.pushed",
                    category = "handle_swarm_event",
                    peer_id = peer_id.to_string(),
                    "pushed identify info to peer"
                ),
            }
        }
        SwarmEvent::Behaviour(ComposedEvent::RendezvousClient(rendezvous_client_event)) => {
            match rendezvous_client_event {
                rendezvous::client::Event::Discovered {
                    rendezvous_node,
                    registrations,
                    cookie,
                } => {
                    if cookie.namespace() == Some(&Namespace::from_static(RENDEZVOUS_NAMESPACE)) {
                        debug!(
                            subject = "libp2p.rendezvous.client.discovered",
                            category = "handle_swarm_event",
                            peer_id = rendezvous_node.to_string(),
                            "received discovery from rendezvous server"
                        );

                        // Store cookie
                        event_handler
                            .rendezvous
                            .cookies
                            .insert(rendezvous_node, cookie);

                        let connected_peers_count = event_handler.connections.peers.len();

                        // Skip dialing peers if at connected peers limit
                        if connected_peers_count >= event_handler.connections.max_peers as usize {
                            debug!(
                                subject = "libp2p.rendezvous.client.discovered.err",
                                category = "handle_swarm_event",
                                "peers discovered not dialed because max connected peers limit reached"
                            );
                            return;
                        }

                        // Filter out already connected peers
                        let new_registrations: Vec<&Registration> = registrations
                            .iter()
                            .filter(|registration| {
                                !event_handler
                                    .connections
                                    .peers
                                    .contains_key(&registration.record.peer_id())
                            })
                            .collect();

                        // Dial newly discovered peers
                        for (index, registration) in new_registrations.iter().enumerate() {
                            let self_registration = &registration.record.peer_id()
                                == event_handler.swarm.local_peer_id();

                            // Dial discovered peer if not us and not at connected peers limit
                            if !self_registration
                                && connected_peers_count + index
                                    < event_handler.connections.max_peers as usize
                            {
                                let peer_id = registration.record.peer_id();
                                let opts = DialOpts::peer_id(peer_id)
                                    .addresses(registration.record.addresses().to_vec())
                                    .condition(
                                        libp2p::swarm::dial_opts::PeerCondition::Disconnected,
                                    )
                                    .build();

                                match event_handler.swarm.dial(opts) {
                                    Ok(_) => {
                                        event_handler.rendezvous.discovered_peers.insert(
                                            peer_id,
                                            PeerDiscoveryInfo::new(rendezvous_node),
                                        );
                                    }
                                    Err(err) => {
                                        warn!(subject = "libp2p.rendezvous.client.discovered.err",
                                              category = "handle_swarm_event",
                                              peer_id=peer_id.to_string(),
                                              err=?err,
                                              "failed to dial discovered peer");
                                    }
                                };
                            } else if !self_registration {
                                debug!(subject = "libp2p.rendezvous.client.discovered.err",
                                       category = "handle_swarm_event",
                                       peer_id=registration.record.peer_id().to_string(),
                                       "peer discovered not dialed because the max connected peers limit was reached")
                            }
                        }

                        #[cfg(feature = "websocket-notify")]
                        notification::emit_network_event(
                            event_handler.ws_evt_sender(),
                            NetworkNotification::DiscoveredRendezvous(
                                notification::DiscoveredRendezvous::new(
                                    rendezvous_node,
                                    registrations
                                        .iter()
                                        .map(|registration| {
                                            (
                                                registration.record.peer_id(),
                                                registration.record.addresses().to_owned(),
                                            )
                                        })
                                        .collect::<BTreeMap<PeerId, Vec<Multiaddr>>>(),
                                ),
                            ),
                        );

                        // Discover peers again at discovery interval
                        event_handler
                            .cache
                            .insert(
                                format!("{}-discover", rendezvous_node),
                                CacheValue::new(
                                    event_handler.rendezvous.discovery_interval,
                                    HashMap::from([
                                        (
                                            "on_expiration".to_string(),
                                            CacheData::OnExpiration(
                                                cache::DispatchEvent::DiscoverPeers,
                                            ),
                                        ),
                                        (
                                            "rendezvous_node".to_string(),
                                            CacheData::Peer(rendezvous_node),
                                        ),
                                    ]),
                                ),
                            )
                            .await;
                    } else {
                        // Do not dial peers that are not using our namespace
                        debug!(subject = "libp2p.rendezvous.client.discovered.err",
                               category = "handle_swarm_event",
                               peer_id=rendezvous_node.to_string(),
                               namespace=?cookie.namespace(),
                               "rendezvous peer gave records from an unexpected namespace");
                    }
                }
                rendezvous::client::Event::DiscoverFailed {
                    rendezvous_node,
                    error,
                    ..
                } => warn!(subject = "libp2p.rendezvous.client.discovered.err",
                           category = "handle_swarm_event",
                           peer_id=rendezvous_node.to_string(),
                           err=?error,
                           "failed to discover peers"),

                rendezvous::client::Event::Registered {
                    rendezvous_node,
                    ttl,
                    ..
                } => {
                    debug!(
                        subject = "libp2p.rendezvous.client.registered",
                        category = "handle_swarm_event",
                        peer_id = rendezvous_node.to_string(),
                        ttl = ttl,
                        "registered self with rendezvous node"
                    );

                    #[cfg(feature = "websocket-notify")]
                    notification::emit_network_event(
                        event_handler.ws_evt_sender(),
                        NetworkNotification::RegisteredRendezvous(
                            notification::RegisteredRendezvous::new(rendezvous_node),
                        ),
                    );

                    event_handler
                        .cache
                        .insert(
                            format!("{}-register", rendezvous_node),
                            CacheValue::new(
                                event_handler.rendezvous.registration_ttl,
                                HashMap::from([
                                    (
                                        "on_expiration".to_string(),
                                        CacheData::OnExpiration(cache::DispatchEvent::RegisterPeer),
                                    ),
                                    (
                                        "rendezvous_node".to_string(),
                                        CacheData::Peer(rendezvous_node),
                                    ),
                                ]),
                            ),
                        )
                        .await;
                }
                rendezvous::client::Event::RegisterFailed {
                    rendezvous_node,
                    error,
                    ..
                } => {
                    warn!(subject = "libp2p.rendezvous.client.registered.err",
                          category = "handle_swarm_event",
                          peer_id=rendezvous_node.to_string(),
                          err=?error,
                          "failed to register self with rendezvous peer")
                }
                rendezvous::client::Event::Expired { peer } => {
                    // re-discover records from peer
                    if let Some(rendezvous_client) = event_handler
                        .swarm
                        .behaviour_mut()
                        .rendezvous_client
                        .as_mut()
                    {
                        let cookie = event_handler.rendezvous.cookies.get(&peer).cloned();

                        if let Some(discovery_info) =
                            event_handler.rendezvous.discovered_peers.remove(&peer)
                        {
                            rendezvous_client.discover(
                                Some(Namespace::from_static(RENDEZVOUS_NAMESPACE)),
                                cookie,
                                None,
                                discovery_info.rendezvous_point,
                            );
                        }
                    }
                }
            }
        }
        SwarmEvent::Behaviour(ComposedEvent::RendezvousServer(rendezvous_server_event)) => {
            match rendezvous_server_event {
                rendezvous::server::Event::DiscoverServed { enquirer, .. } => {
                    debug!(
                        subject = "libp2p.rendezvous.server.discover",
                        category = "handle_swarm_event",
                        peer_id = enquirer.to_string(),
                        "served rendezvous discover request to peer"
                    );

                    #[cfg(feature = "websocket-notify")]
                    notification::emit_network_event(
                        event_handler.ws_evt_sender(),
                        NetworkNotification::DiscoverServedRendezvous(
                            notification::DiscoverServedRendezvous::new(enquirer),
                        ),
                    );
                }
                rendezvous::server::Event::DiscoverNotServed { enquirer, error } => {
                    warn!(subject = "libp2p.rendezvous.server.discover.err",
                          category = "handle_swarm_event",
                          peer_id=enquirer.to_string(),
                          err=?error,
                          "did not serve rendezvous discover request")
                }
                rendezvous::server::Event::PeerRegistered { peer, registration } => {
                    debug!(
                        subject = "libp2p.rendezvous.server.peer_registered",
                        category = "handle_swarm_event",
                        peer_id = peer.to_string(),
                        addresses = ?registration.record.addresses(),
                        "registered peer through rendezvous"
                    );

                    #[cfg(feature = "websocket-notify")]
                    notification::emit_network_event(
                        event_handler.ws_evt_sender(),
                        NetworkNotification::PeerRegisteredRendezvous(
                            notification::PeerRegisteredRendezvous::new(
                                peer,
                                registration.record.addresses().to_owned(),
                            ),
                        ),
                    );
                }
                rendezvous::server::Event::PeerNotRegistered {
                    peer,
                    namespace,
                    error,
                } => {
                    debug!(subject = "libp2p.rendezvous.server.peer_registered.err",
                           category = "handle_swarm_event",
                           peer_id=peer.to_string(),
                           err=?error,
                           namespace=?namespace,
                           "did not register peer with rendezvous")
                }
                rendezvous::server::Event::RegistrationExpired(registration) => {
                    debug!(
                        subject = "libp2p.rendezvous.server.registration_expired",
                        category = "handle_swarm_event",
                        peer_id = registration.record.peer_id().to_string(),
                        "rendezvous peer registration expired on server"
                    )
                }
                _ => (),
            }
        }
        SwarmEvent::Behaviour(ComposedEvent::Gossipsub(gossip_event)) => match *gossip_event {
            gossipsub::Event::Message {
                message,
                propagation_source,
                message_id,
            } => {
                let bytes: Vec<u8> = message.data;
                match pubsub::Message::<Receipt>::try_from(bytes) {
                    Ok(msg) => {
                        let receipt = msg.payload;
                        info!(
                            subject = "libp2p.gossipsub.recv",
                            category = "handle_swarm_event",
                            peer_id = propagation_source.to_string(),
                            message_id = message_id.to_string(),
                            "message received on receipts topic: {}",
                            receipt.cid()
                        );

                        // Store gossiped receipt.
                        let _ = event_handler
                            .db
                            .conn()
                            .as_mut()
                            .map(|conn| Db::store_receipt(receipt.clone(), conn));

                        #[cfg(feature = "websocket-notify")]
                        notification::emit_network_event(
                            event_handler.ws_evt_sender(),
                            NetworkNotification::ReceivedReceiptPubsub(
                                notification::ReceivedReceiptPubsub::new(
                                    propagation_source,
                                    receipt.cid(),
                                    receipt.ran(),
                                ),
                            ),
                        )
                    }
                    Err(err) => debug!(subject = "libp2p.gossipsub.err",
                                       category = "handle_swarm_event",
                                       err=?err,
                                       "cannot handle incoming gossipsub message"),
                }
            }
            gossipsub::Event::Subscribed { peer_id, topic } => debug!(
                subject = "libp2p.gossipsub.subscribed",
                category = "handle_swarm_event",
                peer_id = peer_id.to_string(),
                topic = topic.to_string(),
                "subscribed to topic over gossipsub"
            ),
            _ => {}
        },

        SwarmEvent::Behaviour(ComposedEvent::Kademlia(kad::Event::OutboundQueryProgressed {
            id,
            result,
            ..
        })) => {
            match result {
                QueryResult::Bootstrap(Ok(BootstrapOk { peer, .. })) => debug!(
                    subject = "libp2p.kad.bootstrap",
                    category = "handle_swarm_event",
                    "successfully bootstrapped node: {peer}"
                ),
                QueryResult::GetProviders(Ok(GetProvidersOk::FoundProviders {
                    key: _,
                    mut providers,
                })) => {
                    let _ = providers.remove(event_handler.swarm.local_peer_id());

                    if providers.is_empty() {
                        return;
                    }

                    debug!(
                        subject = "libp2p.kad.get_providers",
                        category = "handle_swarm_event",
                        providers = ?providers,
                        "got workflow info providers"
                    );

                    let Some((key, sender)) = event_handler.query_senders.remove(&id) else {
                        return;
                    };

                    if let Some(sender) = sender {
                        let ev_sender = event_handler.sender();
                        let _ = ev_sender
                            .send_async(Event::Providers(Ok((providers, key, sender))))
                            .await;
                    }

                    // Finish the query. We are only interested in the first
                    // result from a provider.
                    let _ = event_handler
                        .swarm
                        .behaviour_mut()
                        .kademlia
                        .query_mut(&id)
                        .map(|mut query| query.finish());
                }

                QueryResult::GetProviders(Err(err)) => {
                    warn!(subject = "libp2p.kad.get_providers.err",
                          category = "handle_swarm_event",
                          err=?err,
                          "error retrieving outbound query providers");

                    let Some((_, sender)) = event_handler.query_senders.remove(&id) else {
                        return;
                    };

                    if let Some(sender) = sender {
                        let _ = sender
                            .send_async(ResponseEvent::Providers(Err(err.into())))
                            .await;
                    }
                }
                QueryResult::GetRecord(Ok(GetRecordOk::FoundRecord(peer_record))) => {
                    match peer_record.found_record() {
                        Ok(decoded_record) => {
                            let Some((_, sender)) = event_handler.query_senders.remove(&id) else {
                                return;
                            };

                            match decoded_record {
                                DecodedRecord::Receipt(ReceiptRecord { peer_id, receipt }) => {
                                    let response_event = ResponseEvent::Found(Ok(
                                        FoundEvent::Receipt(ReceiptEvent {
                                            peer_id,
                                            receipt: receipt.clone(),
                                        }),
                                    ));

                                    if let Some(sender) = sender {
                                        let _ = sender.send_async(response_event).await;
                                    }

                                    debug!(
                                        subject = "libp2p.kad.get_record",
                                        category = "handle_swarm_event",
                                        cid = receipt.cid().to_string(),
                                        instruction_cid = receipt.instruction().cid().to_string(),
                                        "found receipt record published by {}",
                                        match peer_id {
                                            Some(peer) => peer.to_string(),
                                            None => "unknown peer".to_string(),
                                        }
                                    );
                                }
                                DecodedRecord::Workflow(WorkflowInfoRecord {
                                    peer_id,
                                    workflow_info,
                                }) => {
                                    let response_event = ResponseEvent::Found(Ok(
                                        FoundEvent::Workflow(WorkflowInfoEvent {
                                            peer_id,
                                            workflow_info: workflow_info.clone(),
                                            #[cfg(feature = "websocket-notify")]
                                            workflow_source: notification::WorkflowInfoSource::Dht,
                                        }),
                                    ));

                                    if let Some(sender) = sender {
                                        let _ = sender.send_async(response_event).await;
                                    }

                                    debug!(
                                        subject = "libp2p.kad.get_record",
                                        category = "handle_swarm_event",
                                        cid = workflow_info.cid().to_string(),
                                        "found workflow info record published by {}",
                                        match peer_id {
                                            Some(peer) => peer.to_string(),
                                            None => "unknown peer".to_string(),
                                        }
                                    );
                                }
                            }
                        }
                        Err(err) => {
                            warn!(subject = "libp2p.kad.get_record.err",
                                  category = "handle_swarm_event",
                                  err=?err,
                                  "error retrieving record");
                            let Some((_, sender)) = event_handler.query_senders.remove(&id) else {
                                return;
                            };

                            if let Some(sender) = sender {
                                let _ = sender.send_async(ResponseEvent::Found(Err(err))).await;
                            }
                        }
                    }
                }
                QueryResult::GetRecord(Ok(_)) => {}
                QueryResult::GetRecord(Err(err)) => {
                    warn!(subject = "libp2p.kad.get_record.err",
                          category = "handle_swarm_event",
                          err=?err,
                          "error retrieving record");

                    // Upon an error, attempt to find the record on the DHT via
                    // a provider if it's a Workflow/Info one.
                    match event_handler.query_senders.remove(&id) {
                        Some((
                            RequestResponseKey {
                                capsule_tag: CapsuleTag::Workflow,
                                ..
                            },
                            sender,
                        )) => {
                            if let Some(sender) = sender {
                                let _ = sender
                                    .send_async(ResponseEvent::Found(Err(err.into())))
                                    .await;
                            }
                        }
                        Some((RequestResponseKey { capsule_tag, .. }, sender)) => {
                            if let Some(sender) = sender {
                                let _ = sender
                                    .send_async(ResponseEvent::Found(Err(anyhow!(
                                        "not a valid provider record tag: {capsule_tag}"
                                    ))))
                                    .await;
                            } else {
                                warn!(
                                    subject = "libp2p.kad.req_resp.err",
                                    category = "handle_swarm_event",
                                    "not a valid provider record tag: {capsule_tag}",
                                )
                            }
                        }
                        None => debug!(
                            subject = "libp2p.kad.req_resp.err",
                            category = "handle_swarm_event",
                            "No provider found for outbound query {id:?}"
                        ),
                    }
                }
                QueryResult::PutRecord(Ok(PutRecordOk { .. })) => {
                    let Some((key, _)) = event_handler.query_senders.remove(&id) else {
                        return;
                    };

                    debug!(
                        subject = "libp2p.kad.put_record",
                        category = "handle_swarm_event",
                        cid = key.cid.to_string(),
                        "quorum success for {} record",
                        match key.capsule_tag {
                            CapsuleTag::Receipt => "receipt",
                            CapsuleTag::Workflow => "workflow info",
                        }
                    );

                    #[cfg(feature = "websocket-notify")]
                    match key.capsule_tag {
                        CapsuleTag::Receipt => notification::emit_network_event(
                            event_handler.ws_evt_sender(),
                            NetworkNotification::ReceiptQuorumSuccessDht(
                                notification::ReceiptQuorumSuccessDht::new(
                                    key.cid,
                                    event_handler.quorum.receipt,
                                ),
                            ),
                        ),
                        CapsuleTag::Workflow => notification::emit_network_event(
                            event_handler.ws_evt_sender(),
                            NetworkNotification::WorkflowInfoQuorumSuccessDht(
                                notification::WorkflowInfoQuorumSuccessDht::new(
                                    key.cid,
                                    event_handler.quorum.workflow,
                                ),
                            ),
                        ),
                    }
                }
                QueryResult::PutRecord(Err(err)) => {
                    let Some((key, _)) = event_handler.query_senders.remove(&id) else {
                        return;
                    };

                    warn!(
                      subject = "libp2p.kad.put_record.err",
                      category = "handle_swarm_event",
                      err=?err.clone(),
                      cid = key.cid.to_string(),
                      "error propagating {} record",
                      match key.capsule_tag {
                          CapsuleTag::Receipt => "receipt",
                          CapsuleTag::Workflow => "workflow info",
                      }
                    );

                    #[cfg(feature = "websocket-notify")]
                    if let kad::PutRecordError::QuorumFailed { success, .. } = err {
                        match key.capsule_tag {
                            CapsuleTag::Receipt => notification::emit_network_event(
                                event_handler.ws_evt_sender(),
                                NetworkNotification::ReceiptQuorumFailureDht(
                                    notification::ReceiptQuorumFailureDht::new(
                                        key.cid,
                                        event_handler.quorum.receipt,
                                        event_handler.connections.peers.len(),
                                        success,
                                    ),
                                ),
                            ),
                            CapsuleTag::Workflow => notification::emit_network_event(
                                event_handler.ws_evt_sender(),
                                NetworkNotification::WorkflowInfoQuorumFailureDht(
                                    notification::WorkflowInfoQuorumFailureDht::new(
                                        key.cid,
                                        event_handler.quorum.workflow,
                                        event_handler.connections.peers.len(),
                                        success,
                                    ),
                                ),
                            ),
                        }
                    }
                }
                QueryResult::StartProviding(Ok(AddProviderOk { key })) => {
                    // Currently, we don't send anything to the <worker> channel,
                    // once they key is provided.
                    if let Some((
                        RequestResponseKey {
                            cid: ref cid_str,
                            capsule_tag: CapsuleTag::Workflow,
                        },
                        _,
                    )) = event_handler.query_senders.remove(&id)
                    {
                        debug!(
                            subject = "libp2p.kad.provide_record",
                            category = "handle_swarm_event",
                            cid=%cid_str,
                            "successfully providing {key:#?}"
                        );
                    }
                }
                QueryResult::StartProviding(Err(err)) => {
                    // Currently, we don't send anything to the <worker> channel,
                    // once they key is provided.
                    if let Some((
                        RequestResponseKey {
                            cid: ref cid_str,
                            capsule_tag: CapsuleTag::Workflow,
                        },
                        _,
                    )) = event_handler.query_senders.remove(&id)
                    {
                        warn!(
                            subject = "libp2p.kad.provide_record.err",
                            category = "handle_swarm_event",
                            cid=%cid_str,
                            "error providing key: {:#?}",
                            err.key()
                        );
                    }
                }
                _ => {}
            }
        }
        SwarmEvent::Behaviour(ComposedEvent::Kademlia(kad::Event::InboundRequest { request })) => {
            debug!(
                subject = "libp2p.kad.inbound_request",
                category = "handle_swarm_event",
                "kademlia inbound request received {request:?}"
            )
        }
        SwarmEvent::Behaviour(ComposedEvent::Kademlia(kad::Event::RoutingUpdated {
            peer, ..
        })) => {
            debug!(
                subject = "libp2p.kad.routing",
                category = "handle_swarm_event",
                peer = peer.to_string(),
                "kademlia routing table updated with peer"
            )
        }

        SwarmEvent::Behaviour(ComposedEvent::RequestResponse(
            request_response::Event::Message { message, peer },
        )) => match message {
            request_response::Message::Request {
                request, channel, ..
            } => match (
                Cid::try_from(request.cid.as_str()),
                request.capsule_tag.tag(),
            ) {
                (Ok(cid), WORKFLOW_TAG) => {
                    match workflow::Info::retrieve(
                        cid,
                        event_handler.sender.clone(),
                        event_handler.db.conn().ok(),
                        event_handler.p2p_provider_timeout,
                    )
                    .await
                    {
                        Ok(workflow_info) => {
                            if let Ok(bytes) = workflow_info.capsule() {
                                let _ = event_handler
                                    .swarm
                                    .behaviour_mut()
                                    .request_response
                                    .send_response(channel, bytes);

                                debug!(subject = "libp2p.req_resp",
                                      category = "handle_swarm_event",
                                      cid=?cid,
                                      peer_id = peer.to_string(),
                                      "sent workflow info to peer"
                                );

                                #[cfg(feature = "websocket-notify")]
                                notification::emit_network_event(
                                    event_handler.ws_evt_sender(),
                                    NetworkNotification::SentWorkflowInfo(
                                        notification::SentWorkflowInfo::new(
                                            peer,
                                            workflow_info.cid(),
                                            workflow_info.name,
                                            workflow_info.num_tasks,
                                            workflow_info.progress,
                                            workflow_info.progress_count,
                                        ),
                                    ),
                                )
                            } else {
                                let _ = event_handler
                                    .swarm
                                    .behaviour_mut()
                                    .request_response
                                    .send_response(
                                        channel,
                                        RequestResponseError::InvalidCapsule(request)
                                            .encode()
                                            .unwrap_or_default(),
                                    );
                            }
                        }
                        Err(err) => {
                            warn!(subject = "libp2p.req_resp.err",
                                  category = "handle_swarm_event",
                                  err=?err,
                                  cid=?cid,
                                  "error retrieving workflow info");

                            let _ = event_handler
                                .swarm
                                .behaviour_mut()
                                .request_response
                                .send_response(
                                    channel,
                                    RequestResponseError::Timeout(request)
                                        .encode()
                                        .unwrap_or_default(),
                                );
                        }
                    }
                }
                _ => {
                    let _ = event_handler
                        .swarm
                        .behaviour_mut()
                        .request_response
                        .send_response(
                            channel,
                            RequestResponseError::Unsupported(request)
                                .encode()
                                .unwrap_or_default(),
                        );
                }
            },
            request_response::Message::Response {
                request_id,
                response,
            } => {
                if let Some((RequestResponseKey { cid: key_cid, .. }, sender)) =
                    event_handler.request_response_senders.remove(&request_id)
                {
                    if let Ok(cid) = Cid::try_from(key_cid.as_str()) {
                        match decode_capsule(cid, Some(peer), &response) {
                            Ok(DecodedRecord::Workflow(WorkflowInfoRecord {
                                peer_id,
                                workflow_info,
                            })) => {
                                let response_event = ResponseEvent::Found(Ok(
                                    FoundEvent::Workflow(WorkflowInfoEvent {
                                        peer_id,
                                        workflow_info: workflow_info.clone(),
                                        #[cfg(feature = "websocket-notify")]
                                        workflow_source:
                                            notification::WorkflowInfoSource::RequestResponse,
                                    }),
                                ));

                                let _ = sender.send_async(response_event).await;

                                debug!(subject = "libp2p.req_resp",
                                      category = "handle_swarm_event",
                                      cid=?cid,
                                      peer_id = peer.to_string(),
                                      "received workflow info from peer"
                                );
                            }
                            Ok(DecodedRecord::Receipt(record)) => {
                                debug!(subject = "libp2p.req_resp.resp.err",
                                      category = "handle_swarm_event",
                                      cid = record.receipt.cid().to_string(),
                                      "received a receipt when workflow info was expected: {request_id}");

                                let _ = sender
                                    .send_async(ResponseEvent::Found(Err(anyhow!(
                                        "Found receipt record when workflow info was expected"
                                    ))))
                                    .await;
                            }
                            Err(err) => {
                                warn!(subject = "libp2p.req_resp.resp.err",
                                      category = "handle_swarm_event",
                                      err=?err,
                                      cid = key_cid.as_str(),
                                      "error returning capsule for request_id: {request_id}");

                                let _ = sender.send_async(ResponseEvent::Found(Err(err))).await;
                            }
                        }
                    }
                }
            }
        },
        SwarmEvent::Behaviour(ComposedEvent::RequestResponse(
            request_response::Event::ResponseSent { peer, .. },
        )) => {
            debug!(
                subject = "libp2p.req_resp.resp_sent",
                category = "handle_swarm_event",
                peer_id = peer.to_string(),
                "response sent with workflow info record"
            );
        }

        SwarmEvent::Behaviour(ComposedEvent::Mdns(mdns::Event::Discovered(list))) => {
            for (peer_id, multiaddr) in list.clone() {
                debug!(
                    subject = "libp2p.mdns.discovered",
                    category = "handle_swarm_event",
                    peer_id = peer_id.to_string(),
                    addr = multiaddr.to_string(),
                    "mDNS discovered a new peer"
                );

                if event_handler.connections.peers.len()
                    < event_handler.connections.max_peers as usize
                {
                    let _ = event_handler.swarm.dial(
                        DialOpts::peer_id(peer_id)
                            .addresses(vec![multiaddr])
                            .build(),
                    );
                } else {
                    debug!(subject = "libp2p.mdns.discovered.err",
                           category = "handle_swarm_event",
                           peer_id = peer_id.to_string(),
                           "peer discovered by mDNS not dialed because max connected peers limit reached"
                    )
                }
            }

            #[cfg(feature = "websocket-notify")]
            notification::emit_network_event(
                event_handler.ws_evt_sender(),
                NetworkNotification::DiscoveredMdns(notification::DiscoveredMdns::new(
                    list.iter()
                        .map(|peer| (peer.0, peer.1.to_owned()))
                        .collect::<BTreeMap<PeerId, Multiaddr>>(),
                )),
            )
        }
        SwarmEvent::Behaviour(ComposedEvent::Mdns(mdns::Event::Expired(list))) => {
            let behaviour = event_handler.swarm.behaviour_mut();

            if let Some(mdns) = behaviour.mdns.as_ref() {
                for (peer_id, multiaddr) in list {
                    debug!(
                        subject = "libp2p.mdns.expired",
                        category = "handle_swarm_event",
                        peer_id = peer_id.to_string(),
                        "mDNS discover peer has expired"
                    );
                    if mdns.discovered_nodes().any(|id| id == &peer_id) {
                        behaviour.kademlia.remove_address(&peer_id, &multiaddr);
                        debug!(
                            subject = "libp2p.mdns.expired",
                            category = "handle_swarm_event",
                            peer_id = peer_id.to_string(),
                            "removed peer address from kademlia table"
                        );
                    }
                }
            }
        }
        SwarmEvent::NewListenAddr { address, .. } => {
            let local_peer = *event_handler.swarm.local_peer_id();

            info!(
                subject = "libp2p.listen.addr",
                category = "handle_swarm_event",
                peer_id = local_peer.to_string(),
                "local node is listening on {}",
                address
            );

            #[cfg(feature = "websocket-notify")]
            notification::emit_network_event(
                event_handler.ws_evt_sender(),
                NetworkNotification::NewListenAddr(notification::NewListenAddr::new(
                    local_peer, address,
                )),
            );

            // Init bootstrapping of the DHT
            //
            // Bootstrapping requires at least one node of the DHT to be
            // known.
            //
            // See `libp2p::Behaviour::add_address`.
            if event_handler
                .swarm
                .connected_peers()
                .peekable()
                .peek()
                .is_some()
            {
                let _ = event_handler
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .bootstrap()
                    .map(|_| {
                        debug!(
                            subject = "libp2p.kad.bootstrap",
                            category = "handle_swarm_event",
                            "bootstrapped kademlia"
                        )
                    })
                    .map_err(|err| {
                        warn!(subject = "libp2p.kad.bootstrap.err",
                          category = "handle_swarm_event",
                          err=?err,
                          "error bootstrapping kademlia")
                    });
            }

            event_handler
                .cache
                .insert(
                    "bootstrap".to_string(),
                    CacheValue::new(
                        event_handler.bootstrap.interval,
                        HashMap::from([(
                            "on_expiration".to_string(),
                            CacheData::OnExpiration(cache::DispatchEvent::Bootstrap),
                        )]),
                    ),
                )
                .await;
        }
        SwarmEvent::IncomingConnection { .. } => {}
        SwarmEvent::ConnectionEstablished {
            peer_id, endpoint, ..
        } => {
            debug!(subject = "libp2p.conn.established",
                   category = "handle_swarm_event",
                   peer_id=peer_id.to_string(),
                   endpoint=?endpoint,
                   "peer connection established");

            // add peer to connected peers list
            event_handler
                .connections
                .peers
                .insert(peer_id, endpoint.clone());

            #[cfg(feature = "websocket-notify")]
            notification::emit_network_event(
                event_handler.ws_evt_sender(),
                NetworkNotification::ConnnectionEstablished(
                    notification::ConnectionEstablished::new(
                        peer_id,
                        endpoint.get_remote_address().to_owned(),
                    ),
                ),
            )
        }
        SwarmEvent::ConnectionClosed {
            peer_id,
            cause,
            endpoint,
            ..
        } => {
            debug!(
                subject = "libp2p.conn.closed",
                category = "handle_swarm_event",
                peer_id = peer_id.to_string(),
                "peer connection closed, cause: {cause:#?}, endpoint: {endpoint:#?}"
            );
            event_handler.connections.peers.remove_entry(&peer_id);

            // Remove peer from DHT if not in configured peers
            if event_handler.node_addresses.iter().all(|multiaddr| {
                if let Some(id) = multiaddr.peer_id() {
                    id != peer_id
                } else {
                    // TODO: We may want to check the multiadress without relying on
                    // the peer ID. This would give more flexibility when configuring nodes.
                    warn!(
                        subject = "libp2p.conn.closed",
                        category = "handle_swarm_event",
                        "Configured peer must include a peer ID: {multiaddr}"
                    );
                    true
                }
            }) {
                event_handler
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .remove_peer(&peer_id);

                debug!(
                    subject = "libp2p.kad.remove",
                    category = "handle_swarm_event",
                    peer_id = peer_id.to_string(),
                    "removed peer from kademlia table"
                );
            } else {
                debug!(
                    subject = "libp2p.conn.closed",
                    category = "handle_swarm_event",
                    peer_id = peer_id.to_string(),
                    "redialing trusted peer in {interval:?}",
                    interval = event_handler.connections.dial_interval
                );

                // Dial peers again at dial interval
                event_handler
                    .cache
                    .insert(
                        format!("{}-dial", peer_id),
                        CacheValue::new(
                            event_handler.connections.dial_interval,
                            HashMap::from([
                                (
                                    "on_expiration".to_string(),
                                    CacheData::OnExpiration(cache::DispatchEvent::DialPeer),
                                ),
                                ("node".to_string(), CacheData::Peer(peer_id)),
                            ]),
                        ),
                    )
                    .await;
            }

            #[cfg(feature = "websocket-notify")]
            notification::emit_network_event(
                event_handler.ws_evt_sender(),
                NetworkNotification::ConnnectionClosed(notification::ConnectionClosed::new(
                    peer_id,
                    endpoint.get_remote_address().to_owned(),
                )),
            )
        }
        SwarmEvent::OutgoingConnectionError {
            connection_id,
            peer_id,
            error,
        } => {
            warn!(subject = "libp2p.outgoing.err",
                  category = "handle_swarm_event",
                  peer_id=peer_id.map(|p| p.to_string()).unwrap_or_default(),
                  err=?error,
                  connection_id=?connection_id,
                  "outgoing connection error"
            );

            // Redial peer if in configured peers
            if let Some(peer_id) = peer_id {
                if event_handler.node_addresses.iter().any(|multiaddr| {
                    if let Some(id) = multiaddr.peer_id() {
                        id == peer_id
                    } else {
                        // TODO: We may want to check the multiadress without relying on
                        // the peer ID. This would give more flexibility when configuring nodes.
                        warn!(
                            subject = "libp2p.outgoing.err",
                            category = "handle_swarm_event",
                            "Configured peer must include a peer ID: {multiaddr}"
                        );
                        false
                    }
                }) {
                    debug!(
                        subject = "libp2p.outgoing.err",
                        category = "handle_swarm_event",
                        peer_id = peer_id.to_string(),
                        "redialing trusted peer in {interval:?}",
                        interval = event_handler.connections.dial_interval
                    );

                    // Dial peers again at dial interval
                    event_handler
                        .cache
                        .insert(
                            format!("{}-dial", peer_id),
                            CacheValue::new(
                                event_handler.connections.dial_interval,
                                HashMap::from([
                                    (
                                        "on_expiration".to_string(),
                                        CacheData::OnExpiration(cache::DispatchEvent::DialPeer),
                                    ),
                                    ("node".to_string(), CacheData::Peer(peer_id)),
                                ]),
                            ),
                        )
                        .await;
                }
            }

            #[cfg(feature = "websocket-notify")]
            notification::emit_network_event(
                event_handler.ws_evt_sender(),
                NetworkNotification::OutgoingConnectionError(
                    notification::OutgoingConnectionError::new(peer_id, error),
                ),
            )
        }
        SwarmEvent::IncomingConnectionError {
            connection_id,
            local_addr,
            send_back_addr,
            error,
        } => {
            warn!(subject = "libp2p.incoming.err",
                  category = "handle_swarm_event",
                  err=?error,
                  connection_id=?connection_id,
                  local_address=local_addr.to_string(),
                  remote_address=send_back_addr.to_string(),
                  "incoming connection error");

            #[cfg(feature = "websocket-notify")]
            notification::emit_network_event(
                event_handler.ws_evt_sender(),
                NetworkNotification::IncomingConnectionError(
                    notification::IncomingConnectionError::new(error),
                ),
            )
        }
        SwarmEvent::ListenerError { listener_id, error } => {
            error!(subject = "libp2p.listener.err",
                   category = "handle_swarm_event",
                   err=?error,
                   listener_id=?listener_id,
                   "listener error")
        }
        SwarmEvent::Dialing { peer_id, .. } => match peer_id {
            Some(id) => {
                debug!(
                    subject = "libp2p.dialing",
                    category = "handle_swarm_event",
                    peer_id = id.to_string(),
                    "dialing peer"
                )
            }
            None => debug!(
                subject = "libp2p.dialing",
                category = "handle_swarm_event",
                "dialing an unknown peer"
            ),
        },
        e => debug!(subject = "libp2p.event",
                    category = "handle_swarm_event",
                    e=?e,
                    "uncaught event"),
    }
}
