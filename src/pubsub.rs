use once_cell::sync::Lazy;
use tokio::sync::broadcast;

static BROADCAST_TX: Lazy<broadcast::Sender<String>> = Lazy::new(|| {
  let (tx, _) = broadcast::channel(32);
  tx
});

pub fn tx() -> broadcast::Sender<String> {
  BROADCAST_TX.clone()
}
