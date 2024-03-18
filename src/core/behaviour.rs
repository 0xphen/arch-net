use libp2p::{
    identify::{Behaviour as IdentifyBehaviour, Config as IdentifyConfig},
    identity::Keypair,
    kad::{self, store::MemoryStore, Behaviour as KadBehaviour},
    swarm::NetworkBehaviour,
    PeerId,
};
use libp2p_gossipsub::{
    Behaviour as GossipsubBehaviour, ConfigBuilder as GossipsubConfigBuilder, Message,
    MessageAuthenticity, MessageId, ValidationMode,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::io;

use super::error::ArchError;

#[derive(NetworkBehaviour)]
pub struct ArchBehaviour {
    pub kad: KadBehaviour<MemoryStore>,
    pub identify: IdentifyBehaviour,
    pub gossipsub: GossipsubBehaviour,
}

impl ArchBehaviour {
    pub fn new(key: &Keypair, peer_id: &PeerId) -> Result<Self, ArchError> {
        let gossipsub = Self::gossipsub_behaviour(key)?;

        Ok(Self {
            kad: Self::kad_behaviour(peer_id),
            identify: Self::identify_beheviour(key),
            gossipsub,
        })
    }

    fn identify_beheviour(key: &Keypair) -> IdentifyBehaviour {
        IdentifyBehaviour::new(IdentifyConfig::new(
            "/arch_net/1.0.0".to_string(),
            key.public(),
        ))
    }

    fn kad_behaviour(peer_id: &PeerId) -> KadBehaviour<MemoryStore> {
        let peer_id = peer_id.clone();
        KadBehaviour::new(peer_id, kad::store::MemoryStore::new(peer_id))
    }

    fn gossipsub_behaviour(key: &Keypair) -> Result<GossipsubBehaviour, ArchError> {
        let msg_id_fn = |message: &Message| {
            let mut hasher = DefaultHasher::new();
            message.data.hash(&mut hasher);
            MessageId::from(hasher.finish().to_string())
        };

        let config = GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(ValidationMode::Strict)
            .message_id_fn(msg_id_fn)
            .build()
            .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?;

        let behaviour = GossipsubBehaviour::new(MessageAuthenticity::Signed(key.clone()), config)?;

        Ok(behaviour)
    }
}
