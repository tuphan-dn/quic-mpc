use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PingModel {
  pub rand: u8,
}

impl PingModel {
  pub fn get_topic() -> String {
    "ping".into()
  }
  pub fn is_topic(topic: &String) -> bool {
    *topic == Self::get_topic()
  }
}
