use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt as _};

mod subcommand;

#[derive(clap::Parser)]
struct Args {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    Server,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let args = <Args as clap::Parser>::parse();
    match args.subcommand {
        Subcommand::Server => subcommand::server::run().await,
    }
}
