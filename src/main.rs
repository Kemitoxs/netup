use std::{process::exit, sync::mpsc, thread};

use clap::{arg, Command};
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

fn parse_args() -> Command {
    clap::Command::new("netup")
        .subcommand_required(true)
        .arg(arg!(-v --verbose "Print debug information"))
        .subcommand(
            Command::new("server")
                .about("Run as the server")
                .arg(arg!(<ADDR> "The address to receive on")),
        )
        .subcommand(
            Command::new("client")
                .about("Run as the client")
                .arg(arg!(-g --gui "Run with GUI"))
                .arg(arg!(<ADDR> "The address to connect to (Eg. localhost:56701)")),
        )
}

fn main() {
    let cmd = parse_args().get_matches();
    let verbose = cmd.get_flag("verbose");
    let mut fmt = tracing_subscriber::fmt();

    if verbose {
        fmt = fmt.with_max_level(Level::TRACE);
    } else {
        fmt = fmt.with_max_level(Level::INFO);
    }

    fmt.init();

    info!("Starting application");
    trace!("Verbose flag: {}", verbose);

    match cmd.subcommand() {
        Some(("server", args)) => {
            let addr = args.get_one::<String>("ADDR").unwrap();
            server::run_server(addr).unwrap();
        }
        Some(("client", args)) => {
            let addr = args.get_one::<String>("ADDR").unwrap().to_string();
            let gui = args.get_flag("gui");

            let (tx, rx) = mpsc::channel();
            let t1 = thread::spawn(move || client::run_client(addr, tx).unwrap());

            if gui {
                info!("Running with GUI");
                gui::run_gui(rx);
                exit(0);
            }

            t1.join().unwrap();
        }
        _ => {}
    }
}
