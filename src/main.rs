use crate::{
  behaviour::Behaviour,
  cli::{Args, Parser},
  model::{event_model::EventModel, ping_model::PingModel},
  pubsub::Topic,
  utils::{
    keypair::{parse_peer_id, read_keypair},
    msg::message_id,
  },
};
use futures::stream::StreamExt;
use libp2p::{SwarmBuilder, autonat, gossipsub, identify, kad};
use std::{error::Error, time::Duration};
use tokio::{select, spawn, time::sleep};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

pub mod behaviour;
pub mod cli;
pub mod model;
pub mod pubsub;
pub mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let Args {
    bootstrap, port, ..
  } = Args::parse();
  let _ = tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
    .try_init();

  let keypair = read_keypair()?;
  let mut swarm = SwarmBuilder::with_existing_identity(keypair.clone())
    .with_tokio()
    .with_quic()
    .with_dns()?
    .with_behaviour(|key| {
      // Create a Identify behaviour.
      let identify = identify::Behaviour::new(identify::Config::new(
        "/ipfs/id/1.0.0".to_string(),
        keypair.public(),
      ));
      // Create a Kademlia behaviour.
      let kademlia = kad::Behaviour::new(
        key.public().to_peer_id(),
        kad::store::MemoryStore::new(key.public().to_peer_id()),
      );
      // Create a AutoNAT behaviour.
      let autonat = autonat::Behaviour::new(key.public().to_peer_id(), Default::default());
      // Create a Gossipsub behaviour.
      let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(key.clone()),
        gossipsub::ConfigBuilder::default()
          .heartbeat_interval(Duration::from_secs(10))
          .validation_mode(gossipsub::ValidationMode::Strict)
          .message_id_fn(message_id)
          .build()?,
      )?;
      // Return my behavour
      Ok(Behaviour {
        identify,
        kademlia,
        autonat,
        gossipsub,
      })
    })?
    .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(3600))) // Disconnected after 1 hour idle
    .build();

  // Peer node: Listen on all interfaces and whatever port the OS assigns
  swarm
    .behaviour_mut()
    .kademlia
    .set_mode(Some(kad::Mode::Server));
  swarm.listen_on(format!("/ip4/0.0.0.0/udp/{port}/quic-v1").parse()?)?;

  if let Some(bootstrap_addr) = bootstrap {
    // Add peers to the DHT
    swarm
      .behaviour_mut()
      .kademlia
      .add_address(&parse_peer_id(&bootstrap_addr)?, bootstrap_addr.parse()?);
    // Bootstrap the connection
    if let Err(e) = swarm.behaviour_mut().kademlia.bootstrap() {
      error!("Failed to run Kademlia bootstrap: {e:?}");
    } else {
      // Manual ping
      spawn(async {
        let event_topic = Topic::<EventModel>::new(EventModel::get_topic());
        for i in 0..10 {
          sleep(Duration::from_secs(10)).await;
          event_topic
            .publish(&EventModel {
              topic: "ping".into(),
              data: bincode::serialize(&PingModel { rand: i }).unwrap(),
            })
            .unwrap();
        }
      });
    }
  }

  // General events
  spawn(async {
    let ping_topic = Topic::<PingModel>::new(PingModel::get_topic());
    while let Ok(msg) = ping_topic.subscribe().await {
      println!("Event: {:?}", msg);
    }
  });

  // Read full lines from stdin
  let room = gossipsub::IdentTopic::new("quic-the-room");
  swarm.behaviour_mut().gossipsub.subscribe(&room)?;

  // Kick it off
  let event_topic = Topic::<EventModel>::new(EventModel::get_topic());
  loop {
    select! {
      Ok(msg) = event_topic.subscribe() => {
        // Publish messages
        if let Err(er) = swarm.behaviour_mut().gossipsub.publish(room.clone(), msg.data) {
          error!("Failed to publish the message: {er}");
        } else {
          info!("🛫 .................. Sent");
        }
      }
      event = swarm.select_next_some() => behaviour::handle_events(&mut swarm, event)
    }
  }
}
