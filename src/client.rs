use std::{
    io::Result,
    net::{IpAddr, SocketAddr, UdpSocket},
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{error, info, trace};

use crate::utils;
use crate::utils::Message;

/*
The message format

First 2 bytes: Port of the receiver
Next 8 bytes: Index of the message
Next 16 bytes: Timestamp of the message
Last 64 bytes: SHA-256 hash of the message

*/

pub fn run_client(addr: String) -> Result<()> {
    info!("Running as client... Starting TX & RX threads");

    let tx_addr = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), 56701);
    let tx_udp = utils::bind_addr(tx_addr, true)?;
    info!("TX bound to {}", tx_addr);

    let rx_addr = SocketAddr::new(IpAddr::from([0, 0, 0, 0]), 56702);
    let rx_udp = utils::bind_addr(rx_addr, true)?;
    let rx_port = rx_udp.local_addr().unwrap().port();
    info!("RX bound to {}", rx_addr);

    let target_addr = addr.parse::<SocketAddr>().expect("Failed to parse address");
    let tx = thread::spawn(move || send(tx_udp, target_addr, rx_port));
    let rx = thread::spawn(move || receive(rx_udp));

    tx.join().expect("Failed to join TX thread")?;
    rx.join().expect("Failed to join RX thread")?;

    Ok(())
}

fn send(socket: UdpSocket, target_addr: SocketAddr, rx_port: u16) -> Result<()> {
    let mut idx: u64 = 0;

    info!("TX ready... Transmitting from {:?}", socket);
    loop {
        let msg = Message::build(rx_port, idx, utils::get_timestamp());
        let serialized = msg.to_bytes();
        socket.send_to(&serialized, target_addr)?;
        trace!("Sent message {:?}", msg);

        idx += 1;
        thread::sleep(Duration::from_millis(10));
    }
}

fn receive(socket: UdpSocket) -> Result<()> {
    let mut buf = [0; 1024];

    info!("RX ready... Listening on {:?}", socket);
    loop {
        let (amt, src) = socket.recv_from(&mut buf).expect("Failed to receive data");
        trace!("Received {} bytes from {}", amt, src);

        let Some(opt) = Message::from_bytes(buf[..amt].to_vec()) else {
            error!("Failed to deserialize message");
            continue;
        };

        if !opt.check_hash() {
            error!("Hash check failed for presumed idx: {}", opt.idx);
        }

        info!(
            "Received message with delta: {}",
            utils::get_timestamp() - opt.timestamp
        );
    }
}
