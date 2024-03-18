use libp2p::futures::StreamExt;
use libp2p::{
    core::multiaddr::Multiaddr,
    identify::Event as IdentifyEvent,
    identity::Keypair,
    kad::{self, Mode},
    noise,
    swarm::SwarmEvent,
    tcp, yamux, PeerId, Swarm, SwarmBuilder,
};
use libp2p_gossipsub::{Event as GossipsubEvent, IdentTopic};
use std::env::args;
use std::error::Error;
use tokio::{io, io::AsyncBufReadExt, select};
use tracing::{debug, error, info, warn};

use crate::core::behaviour::ArchBehaviourEvent;

use super::{behaviour::ArchBehaviour, error::ArchError};

const GOSSIP_TOPIC: &str = "gossip_topic";
pub const BOOT_NODE: &str = "/ip4/127.0.0.1/tcp/8080";

#[allow(dead_code)]
pub struct Node {
    peer_id: PeerId,
    key: Keypair,
    swarm: Swarm<ArchBehaviour>,
    addr: Multiaddr,
}

impl Node {
    pub async fn new(addr: Multiaddr) -> Result<Self, ArchError> {
        let key = Keypair::generate_ed25519();
        let peer_id = PeerId::from(key.public());
        let behaviour = ArchBehaviour::new(&key, &peer_id)?;

        let swarm = Self::init_swarm(&key, behaviour)?;

        Ok(Self {
            peer_id,
            key,
            addr,
            swarm,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let topic = IdentTopic::new(GOSSIP_TOPIC);
        let _subscibed = self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

        info!("Subscribed to topic {topic}");
        info!("See args: {:?}", args().nth(1));
        // Connect to the boot node.
        // If an argument is passed to the program, assume it's the address of the boot node.
        if let Some(boot_addr) = args().nth(1) {
            let _listener_id = self.swarm.listen_on(self.addr.clone())?;

            let boot_node: Multiaddr = boot_addr.parse()?;
            self.swarm.dial(boot_node.clone())?;
            debug!("Dialed to boot node: {boot_node}");
        } else {
            info!("Acting as the bootstrap node.");
            self.swarm.listen_on(BOOT_NODE.parse()?)?;
        }

        self.event_listener(topic).await;
        Ok(())
    }

    async fn event_listener(&mut self, topic: IdentTopic) {
        let mut stdin = io::BufReader::new(io::stdin()).lines();

        info!("Enter messages via STDIN and they will be sent to connected peers using Gossipsub");

        loop {
            select! {
                    Ok(Some(line)) = stdin.next_line() => {
                      if let Err(e) = self.swarm
                          .behaviour_mut().gossipsub
                          .publish(topic.clone(), line.as_bytes()) {
                          info!("Publish error: {e:?}");
                      } else {
                        info!("Message {:?} published", line.as_bytes());
                      }
                  }

            // Handle swarm events
            event = self.swarm.select_next_some() => match event {
              SwarmEvent::NewListenAddr { address, .. } => info!("Listening on {address:?}"),

              SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("Connected to {}", peer_id);
              }

               // Identify events
               SwarmEvent::Behaviour(ArchBehaviourEvent::Identify(event)) => match event {
                IdentifyEvent::Sent { peer_id } => info!("IdentifyEvent:Sent: {peer_id}"),

                IdentifyEvent::Received { peer_id, info } => {
                    info!("IdentifyEvent:Received {peer_id} => {info:?}");

                    let addr = info.listen_addrs[0].clone();

                    match self.swarm
                        .behaviour_mut()
                        .kad
                        .add_address(&peer_id, addr.clone())
                    {
                        kad::RoutingUpdate::Failed => {
                            error!("IdentifyReceived: Failed to register address to Kademlia")
                        }

                        kad::RoutingUpdate::Success => {
                            info!("IdentifyReceived: {addr}: Success register address")
                        }

                        kad::RoutingUpdate::Pending => {
                            warn!("IdentifyReceived: Register address pending")
                        }
                    }
                }

                _ => {}
               }

                // Kad events
               SwarmEvent::Behaviour(ArchBehaviourEvent::Kad(event)) => match event {
                kad::Event::ModeChanged { new_mode } => info!("KadEvent:ModeChanged: {new_mode}"),

                kad::Event::RoutablePeer { peer, address } => {
                    info!("KadEvent:RoutablePeer: {peer} | {address}")
                }

                kad::Event::PendingRoutablePeer { peer, address } => {
                    info!("KadEvent:PendingRoutablePeer: {peer} | {address}")
                }

                kad::Event::InboundRequest { request } => {
                    info!("KadEvent:InboundRequest: {request:?}")
                }

                kad::Event::RoutingUpdated {
                  peer,
                  is_new_peer,
                  addresses,
                  bucket_range,
                  old_peer,
              } => {
                  info!("KadEvent:RoutingUpdated: {peer} | IsNewPeer? {is_new_peer} | {addresses:?} | {bucket_range:?} | OldPeer: {old_peer:?}");
              }

              kad::Event::OutboundQueryProgressed {
                  id,
                  result,
                  stats,
                  step,
              } => {
                  info!("KadEvent:OutboundQueryProgressed: ID: {id:?} | Result: {result:?} | Stats: {stats:?} | Step: {step:?}")
              }

              _ => {}
               },

               SwarmEvent::Behaviour(ArchBehaviourEvent::Gossipsub(GossipsubEvent::Message {
                propagation_source: peer_id,
                message_id: id,
                message,
            })) => info!(
                    "Got message: '{}' with id: {id} from peer: {peer_id}",
                    String::from_utf8_lossy(&message.data),
                ),


               _ => {}
            }

              }
        }
    }

    fn init_swarm(
        key: &Keypair,
        behaviour: ArchBehaviour,
    ) -> Result<Swarm<ArchBehaviour>, ArchError> {
        let mut swarm = SwarmBuilder::with_existing_identity(key.clone())
            .with_async_std()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|err| {
                error!("Failed to build swarm {err}");
                ArchError::SwarmBuilderError
            })?
            .with_behaviour(|_| behaviour)
            .map_err(|err| {
                error!("Failed to build swarm {err}");
                ArchError::SwarmBuilderError
            })?
            .build();

        swarm.behaviour_mut().kad.set_mode(Some(Mode::Server));
        Ok(swarm)
    }
}
