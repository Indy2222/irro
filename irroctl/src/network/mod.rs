//! This module implements tools for discovery and probing of Irro on LAN.

use log::info;
use std::net::{IpAddr, Ipv4Addr, SocketAddrV4, UdpSocket};
use std::time::Duration;

const BROADCAST_PORT: u16 = 34254;

/// Listen on broadcast UDP packets for Irro and return source IP address once
/// such a packet is received. Packet recv is setup with 60 seconds timeout.
///
/// See https://irro.cz/api.html for more information about Irro discovery.
pub fn discover_irro() -> std::io::Result<IpAddr> {
    info!(
        "Trying to receive broadcast packet on port {}...",
        BROADCAST_PORT
    );

    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, BROADCAST_PORT))?;
    socket.join_multicast_v4(&Ipv4Addr::new(224, 0, 0, 0), &Ipv4Addr::UNSPECIFIED)?;
    socket.set_read_timeout(Some(Duration::from_secs(60)))?;

    let mut buf = [0; 50];
    let (_, src) = socket.recv_from(&mut buf)?;
    Ok(src.ip())
}
