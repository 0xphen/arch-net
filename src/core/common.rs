use libp2p::Multiaddr;
use std::net::SocketAddr;

pub fn address_to_multiaddr(addr: SocketAddr) -> Option<Multiaddr> {
    let ip_str = addr.ip().to_string();
    let port_str = addr.port().to_string();

    let multiaddr_str = format!("/ip4/{}/tcp/{}", ip_str, port_str);
    multiaddr_str.parse::<Multiaddr>().ok()
}
