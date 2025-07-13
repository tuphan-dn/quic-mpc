use crate::{
  model::{event_model::EventModel, ping_model::PingModel},
  pubsub::Topic,
};
use libp2p::{
  Swarm, autonat, gossipsub, identify, kad,
  swarm::{NetworkBehaviour, SwarmEvent},
};
use tracing::{debug, info};

#[derive(NetworkBehaviour)]
pub struct Behaviour {
  pub identify: identify::Behaviour,
  pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
  pub autonat: autonat::Behaviour,
  pub gossipsub: gossipsub::Behaviour,
}

pub fn handle_events(swarm: &mut Swarm<Behaviour>, event: SwarmEvent<BehaviourEvent>) -> () {
  match event {
    SwarmEvent::NewListenAddr { address, .. } => {
      let addr = format!(
        "{}/p2p/{}",
        &address.to_string(),
        &swarm.local_peer_id().to_string()
      );
      info!("✅ Local node is listening on {addr}");
    }
    SwarmEvent::IncomingConnection { send_back_addr, .. } => {
      info!("⏳ Connecting to {send_back_addr}");
    }
    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
      info!("🔗 Connected to {peer_id}");
    }
    SwarmEvent::ConnectionClosed { peer_id, .. } => {
      info!("💔 Disconnected to {peer_id}");
    }
    // Identify
    SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received {
      peer_id,
      info,
      ..
    })) => {
      info!("👤 Identify new peer: {peer_id}");
      if info.protocols.iter().any(|p| *p == kad::PROTOCOL_NAME) {
        for addr in info.listen_addrs {
          swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
        }
      }
    }
    // Kademlia
    SwarmEvent::Behaviour(BehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed {
      result: kad::QueryResult::Bootstrap(Ok(kad::BootstrapOk { peer: peer_id, .. })),
      ..
    })) => {
      if peer_id != *swarm.local_peer_id() {
        info!("🚀 Kademlia bootstrapped completely: {peer_id:?}");
      }
    }
    SwarmEvent::Behaviour(BehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed {
      result: kad::QueryResult::GetClosestPeers(Ok(kad::GetClosestPeersOk { peers, .. })),
      ..
    })) => {
      info!("🔍 Kademlia discovered new peers: {peers:?}");
    }
    // Gossipsub
    SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(gossipsub::Event::Message {
      propagation_source: _peer_id,
      message,
      ..
    })) => {
      let event_model: EventModel = bincode::deserialize(&message.data).unwrap();
      let ping_topic = Topic::<PingModel>::new(PingModel::get_topic());
      if PingModel::is_topic(&event_model.topic) {
        let data: PingModel = bincode::deserialize(&event_model.data).unwrap();
        ping_topic.publish(&data).unwrap();
      }
    }
    // Others
    _ => {
      debug!("❓ Other Behaviour events {event:?}");
    }
  }
}
