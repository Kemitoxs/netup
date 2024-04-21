use std::{
    io::Result,
    net::{IpAddr, SocketAddr, UdpSocket},
    sync::mpsc,
};

use tracing::{debug, error, info, trace};

use crate::{gui::MessageSentEvent, utils};
use crate::{
    gui::{MessageReceivedEvent, NetupEvent},
    utils::Message,
};

/*
The message format

First 2 bytes: Port of the receiver
Next 8 bytes: Index of the message
Next 16 bytes: Timestamp of the message
Last 64 bytes: SHA-256 hash of the message

*/

pub fn run_client(remote_addr: String, channel: mpsc::Sender<NetupEvent>) -> Result<()> {
    info!("Running as client... Starting TX & RX threads");
    let addr = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), 56701);
    let udp = UdpSocket::bind(addr)?;
    udp.set_nonblocking(true)?;

    let mut idx = 0;
    let mut buffer = [0; 1024];
    let mut next_send = utils::get_timestamp();
    const INTERVAL: u128 = 20;

    loop {
        if utils::get_timestamp() >= next_send {
            let timestamp = utils::get_timestamp();
            let msg = Message::build(idx, timestamp);
            let serialized = msg.to_bytes();
            udp.send_to(&serialized, &remote_addr)?;
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
        let Some(msg) = Message::from_bytes(buffer[..amt].to_vec()) else {
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
