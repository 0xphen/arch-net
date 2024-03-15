use super::{common::address_to_multiaddr, error::NodeError, types::NodeInfo};
use async_std::stream::StreamExt;
use libp2p::{
    identity, mdns, noise, swarm::NetworkBehaviour, swarm::SwarmEvent, tcp, yamux, Multiaddr,
    PeerId, Swarm, SwarmBuilder,
};
use libp2p_gossipsub::*;
use log::{debug, error, info};
use serde_json;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::io;

#[derive(NetworkBehaviour)]
struct PeerRouterBehaviour {
    gossipsub: Behaviour,
    mdns: mdns::async_io::Behaviour,
}

pub struct PeerRouter {
    pub swarm: Swarm<PeerRouterBehaviour>,
}

impl PeerRouter {
    pub fn new(
        local_peer_id: &PeerId,
        key_pair: &identity::Keypair,
    ) -> Result<Self, Box<dyn Error>> {
        let gossipsub_config = Self::gossipsub_config()?;

        let swarm = Self::create_swarm(local_peer_id, gossipsub_config, key_pair)?;
        Ok(Self { swarm })
    }

    fn gossipsub_config() -> Result<Config, Box<dyn Error>> {
        let message_id_fn = |message: &Message| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            MessageId::from(s.finish().to_string())
        };

        let gossipsub_config = ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .build()
            .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?;

        Ok(gossipsub_config)
    }

    fn create_swarm(
        local_peer_id: &PeerId,
        gossipsub_config: Config,
        key_pair: &identity::Keypair,
    ) -> Result<Swarm<PeerRouterBehaviour>, Box<dyn Error>> {
        let mut swarm = SwarmBuilder::with_new_identity()
            .with_async_std()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|key| {
                let gossipsub: Behaviour<IdentityTransform, AllowAllSubscriptionFilter> =
                    Behaviour::new(
                        MessageAuthenticity::Signed(key_pair.clone()),
                        gossipsub_config,
                    )?;

                let mdns =
                    mdns::async_io::Behaviour::new(mdns::Config::default(), local_peer_id.clone())?;

                Ok(PeerRouterBehaviour { gossipsub, mdns })
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        Ok(swarm)
    }

    pub async fn run_swarm(
        &mut self,
        gossip_topic: &str,
        peers: &Vec<Multiaddr>,
        node_info: &NodeInfo,
    ) -> Result<(), Box<dyn Error>> {
        let topic = IdentTopic::new(gossip_topic);
        let _ = self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

        let local_addr = match address_to_multiaddr(node_info.addr) {
            Some(local_addr) => local_addr,
            None => return Err("Failed to convert address".into()),
        };

        let listener_id = self.swarm.listen_on(local_addr.clone())?;
        debug!(
            "Swarm #{} listening on address {:?}",
            listener_id, local_addr
        );

        let node_info_str = serde_json::to_string(&node_info).unwrap();
        let data = node_info_str.into_bytes();

        for addr in peers {
            self.swarm.dial(addr.clone()).unwrap();

            // let x = self
            //     .swarm
            //     .behaviour_mut()
            //     .gossipsub
            //     .publish(topic.clone(), data.clone())
            //     .unwrap();
        }

        println!(
            "PEERS: {:?}",
            self.swarm.connected_peers().collect::<Vec<&PeerId>>()
        );

        self.listen_on_events().await;

        Ok(())
    }

    async fn listen_on_events(&mut self) {
        while let Some(event) = self.swarm.next().await {
            match event {
                SwarmEvent::Behaviour(PeerRouterBehaviourEvent::Gossipsub(Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                })) => {
                    info!("New Message: {:?}", message);
                }
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Local node is listening on address {}", address);
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    info!("New connection established to remote peer {}", peer_id);
                }
                SwarmEvent::Behaviour(PeerRouterBehaviourEvent::Mdns(mdns::Event::Discovered(
                    list,
                ))) => {
                    for (peer_id, _multiaddr) in list {
                        info!("mDNS discovered a new peer: {peer_id}");
                        self.swarm
                            .behaviour_mut()
                            .gossipsub
                            .add_explicit_peer(&peer_id);
                    }
                }
                _ => {}
            }
        }
    }
}
