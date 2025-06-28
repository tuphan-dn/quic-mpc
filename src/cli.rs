pub use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  /// If seed is provided, it will be used to create the keypair.
  #[arg(short, long)]
  pub seed: Option<String>,
  /// Start with a bootstrap node. If not provided, the current node will become a bootstrap node.
  #[arg(short, long)]
  pub bootstrap: Option<String>,
  /// Port.
  #[arg(short, long, default_value_t = 0)]
  pub port: u16,
}
