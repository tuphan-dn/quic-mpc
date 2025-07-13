use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EventModel {
  pub topic: String,
  pub data: Vec<u8>,
}

impl EventModel {
  pub fn get_topic() -> String {
    "event".into()
  }
  pub fn is_topic(topic: &String) -> bool {
    *topic == Self::get_topic()
  }
}
