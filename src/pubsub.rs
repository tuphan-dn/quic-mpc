use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::error::Error;
use tokio::sync::broadcast;

static BROADCAST_TX: Lazy<broadcast::Sender<Vec<u8>>> = Lazy::new(|| {
  let (tx, _) = broadcast::channel(32);
  tx
});

#[derive(Serialize, Deserialize, Debug)]
pub struct Topic<T> {
  pub topic: String,
  pub data: Option<T>,
}

impl<T: Serialize + DeserializeOwned + Send> Topic<T> {
  pub fn new(topic: String) -> Self {
    Topic { topic, data: None }
  }

  fn tx(&self) -> broadcast::Sender<Vec<u8>> {
    BROADCAST_TX.clone()
  }

  pub async fn subscribe(&self) -> Result<T, Box<dyn Error>> {
    let mut rx = self.tx().subscribe();
    if let Ok(msg) = rx.recv().await {
      let model: Topic<T> = bincode::deserialize(&msg)?;
      if model.topic == self.topic
        && let Some(data) = model.data
      {
        return Ok(data);
      }
      return Err("Irrelevant topic".into());
    }
    Err("Channel closed".into())
  }

  pub fn publish(&self, data: &T) -> Result<(), Box<dyn Error>> {
    let msg = bincode::serialize(data)?;
    self.tx().send(msg)?;
    Ok(())
  }
}
