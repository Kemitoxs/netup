use std::{io::Result, net::UdpSocket};

use tracing::{info, trace};

use crate::ServerArgs;

pub fn run_server(args: &ServerArgs) -> Result<()> {
    info!("Running as server... Binding to {}", args.addr);
    let udp = UdpSocket::bind(args.addr)?;
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
