use core::num;
use std::{io::Result, net::UdpSocket, rc};

use tracing::{info, trace};

pub fn run_server(addr: String) -> Result<()> {
    info!("Running as server... Binding to {}", addr);
    let udp = UdpSocket::bind(addr)?;
    let mut buf = [0; 1024];

    info!("Server is ready... Listening for incoming packets");
    loop {
        let rcv_res = udp.recv_from(&mut buf);
        match &rcv_res {
            Ok((amt, src)) => {
                trace!("Received {} bytes from {}", amt, src);
            }
            Err(e) => {
                trace!("Failed to receive data: {}", e);
                continue;
            }
        }

        let (amt, src) = rcv_res.unwrap();

        let mut port_buf = [0; 2];
        port_buf.copy_from_slice(&buf[..2]);
        let port_num = u16::from_ne_bytes(port_buf);
        let mut rx_addr = src.clone();
        rx_addr.set_port(port_num);

        let res = udp.send_to(&buf[..amt], rx_addr);
        match res {
            Ok(_) => {
                trace!("Sent {} bytes to {}", amt, rx_addr);
            }
            Err(e) => {
                trace!("Failed to send to {}: {}", rx_addr, e);
            }
        }
    }
}
