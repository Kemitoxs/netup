use clap::{arg, Command};
use tracing::{info, Level};

mod client;
mod server;
mod utils;

fn parse_args() -> Command {
    clap::Command::new("netup")
        .subcommand_required(true)
        .subcommand(
            Command::new("server")
                .about("Run as the server")
                .arg(arg!(<ADDR> "The address to receive on")),
        )
        .subcommand(
            Command::new("client")
                .about("Run as the client")
                .arg(arg!(<ADDR> "The address to connect to (Eg. localhost:56701)")),
        )
}

fn main() {
    let cmd = parse_args().get_matches();
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();
    info!("Starting application");

    match cmd.subcommand() {
        Some(("server", args)) => {
            let addr = args.get_one::<String>("ADDR").unwrap();
            server::run_server(addr).unwrap();
        }
        Some(("client", args)) => {
            let addr = args.get_one::<String>("ADDR").unwrap();
            client::run_client(addr).unwrap();
        }
        _ => {}
    }
}
