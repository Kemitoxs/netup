use std::{io::Result, net::UdpSocket};

use tracing::{info, trace};

pub fn run_server(addr: &String) -> Result<()> {
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
        let res = udp.send_to(&buf[..amt], src);
        match res {
            Ok(_) => {
                trace!("Sent {} bytes to {}", amt, src);
            }
            Err(e) => {
                trace!("Failed to send to {}: {}", src, e);
            }
        }
    }
}
