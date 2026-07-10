use std::{
    fmt,
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

use serde::{Deserialize, Serialize};

const DISCOVERY_PORT: u16 = 47_473;
const DISCOVERY_VERSION: u8 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ManualPeerAddress {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredPeer {
    pub device_name: String,
    pub public_key: String,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscoveryPacket {
    version: u8,
    device_name: String,
    public_key: String,
    port: u16,
}

#[derive(Debug)]
pub enum DiscoveryError {
    InvalidAddress,
    Network(std::io::Error),
    Packet(serde_json::Error),
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAddress => write!(
                formatter,
                "Enter a valid IP address and port, such as 192.168.1.20:47473."
            ),
            Self::Network(error) => write!(formatter, "Local network discovery failed: {error}"),
            Self::Packet(error) => write!(formatter, "Local discovery packet was invalid: {error}"),
        }
    }
}

impl From<std::io::Error> for DiscoveryError {
    fn from(error: std::io::Error) -> Self {
        Self::Network(error)
    }
}
impl From<serde_json::Error> for DiscoveryError {
    fn from(error: serde_json::Error) -> Self {
        Self::Packet(error)
    }
}

impl ManualPeerAddress {
    pub fn parse(value: &str) -> Result<Self, DiscoveryError> {
        let address: SocketAddr = value
            .trim()
            .parse()
            .map_err(|_| DiscoveryError::InvalidAddress)?;
        if address.port() == 0 {
            return Err(DiscoveryError::InvalidAddress);
        }
        Ok(Self {
            address: address.to_string(),
        })
    }
}

pub struct DiscoveryService;

impl DiscoveryService {
    pub fn advertise(device_name: &str, public_key: &[u8; 32]) -> Result<(), DiscoveryError> {
        let packet = DiscoveryPacket {
            version: DISCOVERY_VERSION,
            device_name: device_name.to_owned(),
            public_key: public_key
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
            port: DISCOVERY_PORT,
        };
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_broadcast(true)?;
        socket.send_to(
            &serde_json::to_vec(&packet)?,
            format!("255.255.255.255:{DISCOVERY_PORT}"),
        )?;
        Ok(())
    }

    pub fn listen_once() -> Result<Vec<DiscoveredPeer>, DiscoveryError> {
        let socket = UdpSocket::bind(("0.0.0.0", DISCOVERY_PORT))?;
        socket.set_read_timeout(Some(Duration::from_millis(150)))?;
        let mut buffer = [0_u8; 1_024];
        let mut peers = Vec::new();
        while let Ok((length, source)) = socket.recv_from(&mut buffer) {
            let packet: DiscoveryPacket = serde_json::from_slice(&buffer[..length])?;
            if packet.version == DISCOVERY_VERSION && packet.port != 0 {
                peers.push(DiscoveredPeer {
                    device_name: packet.device_name,
                    public_key: packet.public_key,
                    address: format!("{}:{}", source.ip(), packet.port),
                });
            }
        }
        Ok(peers)
    }
}

#[cfg(test)]
mod tests {
    use super::ManualPeerAddress;

    #[test]
    fn accepts_ip_addresses_with_nonzero_ports() {
        assert_eq!(
            ManualPeerAddress::parse("192.168.0.12:47473")
                .unwrap()
                .address,
            "192.168.0.12:47473"
        );
    }

    #[test]
    fn rejects_invalid_or_zero_port_addresses() {
        assert!(ManualPeerAddress::parse("not a peer").is_err());
        assert!(ManualPeerAddress::parse("127.0.0.1:0").is_err());
    }
}
