use crate::{
  behaviour::Behaviour,
  cli::{Args, Parser},
  utils::{
    keypair::{parse_peer_id, read_keypair},
    msg::message_id,
  },
};
use futures::stream::StreamExt;
use libp2p::{SwarmBuilder, autonat, gossipsub, identify, kad};
use std::{error::Error, time::Duration};
use tokio::{select, spawn, sync::broadcast, time::sleep};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

pub mod behaviour;
pub mod cli;
pub mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let Args {
    bootstrap, port, ..
  } = Args::parse();
  let _ = tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
    .try_init();
  let (tx, mut rx) = broadcast::channel(32);

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
      let rand_channel = tx.clone();
      spawn(async move {
        for i in 0..10 {
          sleep(Duration::from_secs(10)).await;
          rand_channel.send(format!("ping {}", i)).unwrap();
        }
      });
      // General events
      let mut event_channel = tx.subscribe();
      spawn(async move {
        while let Ok(msg) = event_channel.recv().await {
          println!("Event: {}", msg);
        }
      });
    }
  }

  // Read full lines from stdin
  let topic = gossipsub::IdentTopic::new("quic-the-room");
  swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

  // Kick it off
  loop {
    select! {
      Ok(msg) = rx.recv() => {
        // Publish messages
        let t: Vec<&str> = msg.split(" ").collect();
        if t[0] != "ping" {
          // Skip
        } else if let Err(er) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), msg.as_bytes()) {
          error!("Failed to publish the message: {er}");
        } else {
          info!("🛫 .................. Sent");
        }
      }
      event = swarm.select_next_some() => behaviour::handle_events(&mut swarm, event, tx.clone())
    }
  }
}
