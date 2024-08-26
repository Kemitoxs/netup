use std::{net::SocketAddr, path::PathBuf, sync::mpsc, thread};

use clap::{Parser, Subcommand};
use tracing::{info, trace, Level};

mod client;
mod gui;
mod server;
mod utils;

/*
TODO:
 - Use clap_derivce instead of manual stuff
 - Always ensure you can run without GUI
 - Load / Save from file with CSV
 - GUI:
   - Add two modes:

Arguments:
 - server <ADDR>
 - client <REMOTE_ADDR>
  - "--no-gui": run without GUI
  - "--file": set with file to use
  - "--port": from which port to send (if null incrementelly try from 10000)
*/

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Prints all log message with `Trace` level or higher
    #[clap(short, long, action)]
    pub verbose: bool,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Server(ServerArgs),
    Client(ClientArgs),
}

/// Arguments for the server command.
#[derive(Parser, Debug)]
pub struct ServerArgs {
    /// The address to receive on
    #[clap(value_parser)]
    pub addr: SocketAddr,
}

#[derive(Parser, Debug)]
pub struct ClientArgs {
    /// The remote address to connect to
    #[clap(value_parser)]
    remote_addr: SocketAddr,

    /// The address to bind to (if null incrementally try from 10000)
    #[clap(short, long, value_parser)]
    local_addr: Option<SocketAddr>,

    /// The file to write to / read from
    #[clap(short, long, value_parser)]
    file: Option<PathBuf>,

    /// Run the client without GUI (requires --file to be set)
    #[clap(short, long, action, requires_if("true", "file"))]
    no_gui: bool,
}

fn main() {
    let cmd = Args::parse();

    // Setup logging
    let mut fmt = tracing_subscriber::fmt();
    if cmd.verbose {
        fmt = fmt.with_max_level(Level::TRACE);
    } else {
        fmt = fmt.with_max_level(Level::INFO);
    }
    fmt.init();

    info!("Starting application");
    trace!("Verbose flag: {}", cmd.verbose);

    match cmd.command {
        Commands::Server(args) => {
            server::run_server(&args).unwrap();
        }
        Commands::Client(args) => {
            let (tx, rx) = mpsc::channel();
            if args.no_gui {
                info!("Running without GUI");
                client::run_client(&args, tx).unwrap();
                return;
            }

            info!("Running with GUI");
            let client_thread = thread::spawn(move || client::run_client(&args, tx));
            gui::run_gui(rx);
            client_thread.join().unwrap().unwrap();
        }
    }
}
