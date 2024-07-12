use tracing::Level;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, util::SubscriberInitExt as _};

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
        .with(
            Targets::new()
                .with_target("server", Level::INFO)
                .with_target("command_use_case", Level::INFO),
        )
        .init();
    let args = <Args as clap::Parser>::parse();
    match args.subcommand {
        Subcommand::Server => subcommand::server::run().await,
    }
}
