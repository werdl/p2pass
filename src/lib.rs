// p2pass - a P2P file sharing program. each node, when invoked for the first time, will produce a unique identifier, based on their IP and port. This identifier is 2-way, and crucially is not stored in a DHT

use std::net::IpAddr;

// base64
use base64::{engine::general_purpose::STANDARD, Engine as _};
pub struct Node {
    ip: IpAddr,
    port: u16,
}

impl Node {
    pub fn id(&self) -> String {
        let ip = self.ip.to_string();
        let port = self.port.to_string();
        let id = format!("{}:{}", ip, port);
        STANDARD.encode(id.as_bytes())
    }

    pub fn from(id: String) -> Self {
        let id = STANDARD.decode(id).unwrap();
        let id = String::from_utf8(id).unwrap();
        let mut parts = id.split(':');
        let ip = parts.next().unwrap().parse().unwrap();
        let port = parts.next().unwrap().parse().unwrap();
        Self { ip, port }
    }
}