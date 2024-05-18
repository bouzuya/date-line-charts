mod command_use_case;
mod query_use_case;
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
    let args = <Args as clap::Parser>::parse();
    match args.subcommand {
        Subcommand::Server => subcommand::server::run().await,
    }
}
