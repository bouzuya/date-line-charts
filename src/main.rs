#[derive(clap::Parser)]
struct Args {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    Server,
}

fn main() {
    let args = <Args as clap::Parser>::parse();
    match args.subcommand {
        Subcommand::Server => {
            println!("FIXME: run server");
        }
    }
}
