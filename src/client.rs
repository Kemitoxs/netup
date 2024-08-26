use std::{
    io::{self, Error, ErrorKind, Result},
    net::{IpAddr, SocketAddr, UdpSocket},
    sync::mpsc,
};

use tracing::{debug, error, info, trace};

use crate::{gui::MessageSentEvent, utils, ClientArgs};
use crate::{
    gui::{MessageReceivedEvent, NetupEvent},
    utils::RawMessage,
};

fn get_socket() -> Option<UdpSocket> {
    // Incrementally try to bind to a port starting from 10000
    info!("Incrementally trying to bind to a port between 10000 and 65535");
    for port in 10000..65535 {
        let addr = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port);
        match UdpSocket::bind(addr) {
            Ok(udp) => {
                info!("Bound to {}", addr);
                return Some(udp);
            }
            Err(e) => match e.kind() {
                ErrorKind::AddrInUse => {
                    trace!("Port {} is in use", port);
                    continue;
                }
                _ => {
                    error!("Failed to bind to port {}: {}", port, e);
                    info!("Incremental port binding is only available when `ErrorKind::AddrInUse` is returned... Terminating application");
                    return None;
                }
            },
        }
    }

    error!("All ports between 10000 and 65535 are in use... Terminating application");
    None
}

pub fn run_client(args: &ClientArgs, channel: mpsc::Sender<NetupEvent>) -> Result<()> {
    info!("Running as client");
    let Some(udp) = get_socket() else {
        return Err(Error::new(
            io::ErrorKind::AddrInUse,
            "Failed to bind to any port",
        ));
    };
    udp.set_nonblocking(true)?;

    let mut idx = 0;
    let mut buffer = [0; 1024];
    let mut next_send = utils::get_timestamp();
    const INTERVAL: u128 = 50;

    loop {
        if utils::get_timestamp() >= next_send {
            let timestamp = utils::get_timestamp();
            let msg = RawMessage::build(idx, timestamp);
            let serialized = msg.to_bytes();
            udp.send_to(&serialized, &args.remote_addr)?;
            _ = channel.send(NetupEvent::MessageSent(MessageSentEvent::new(
                idx, timestamp,
            )));
            trace!("Sent message {:?}", msg);

            next_send += INTERVAL;
            idx += 1;
        }

        let res = udp.recv_from(&mut buffer);
        match &res {
            Ok(_) => {}
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    error!("Failed to receive data: {}", e);
                }
                continue;
            }
        }

        let (amt, _) = res.unwrap();
        let Some(msg) = RawMessage::from_bytes(buffer[..amt].to_vec()) else {
            error!("Failed to deserialize message");
            continue;
        };

        if !msg.check_hash() {
            error!("Hash check failed for presumed idx: {}", msg.idx);
            continue;
        }

        let now = utils::get_timestamp();
        _ = channel.send(NetupEvent::MessageReceived(MessageReceivedEvent::new(
            msg.idx, now,
        )));

        debug!(
            "Received idx #{} with delta {}ms",
            msg.idx,
            now - msg.timestamp
        );
    }
}
