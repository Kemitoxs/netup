use std::{
    io::ErrorKind,
    net::{SocketAddr, UdpSocket},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{debug, info};

pub fn bind_addr(mut addr: SocketAddr, iterate: bool) -> std::io::Result<UdpSocket> {
    loop {
        let res = UdpSocket::bind(addr);
        match res {
            Ok(socket) => {
                return Ok(socket);
            }

            Err(e) => match e.kind() {
                ErrorKind::AddrInUse => {
                    if !iterate {
                        return Err(e);
                    }
                    debug!("Address {} already in use... Incrementing port", addr);
                    addr.set_port(addr.port() + 1);
                }
                _ => return Err(e),
            },
        }
    }
}

pub fn get_timestamp() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

#[derive(Serialize, Deserialize, Debug, Hash)]
pub struct Message {
    pub idx: u64,
    pub timestamp: u128,
    pub hash: u128,
}

impl Message {
    pub fn build(idx: u64, timestamp: u128) -> Self {
        let mut msg = Message {
            idx,
            timestamp,
            hash: 0,
        };
        msg.set_hash();
        msg
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self).expect("Failed to serialize message")
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        bincode::deserialize(&bytes).ok()
    }

    fn calc_hash(&self) -> u128 {
        let mut hasher = Sha256::new();
        let data = format!("{}{}", self.idx, self.timestamp);
        let bytes = data.as_bytes();

        hasher.update(bytes);
        let result = hasher.finalize();
        let mut num_buf = [0 as u8; 16];
        num_buf.copy_from_slice(&result[..16]);
        let hash_num = u128::from_ne_bytes(num_buf);

        hash_num
    }

    pub fn check_hash(&self) -> bool {
        let hash_num = self.calc_hash();
        self.hash == hash_num
    }

    pub fn set_hash(&mut self) {
        self.hash = self.calc_hash();
    }
}
