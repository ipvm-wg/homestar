/// Multiaddr extension methods.
use libp2p::{multiaddr::Protocol, Multiaddr, PeerId};

/// [Multiaddr] extension trait.
pub(crate) trait MultiaddrExt {
    fn peer_id(&self) -> Option<PeerId>;
}

impl MultiaddrExt for Multiaddr {
    fn peer_id(&self) -> Option<PeerId> {
        match self.iter().last() {
            Some(Protocol::P2p(peer_id)) => Some(peer_id),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn contains_peer_id() {
        let peer_id = PeerId::random();
        let multiaddr: Multiaddr = format!("/ip4/127.0.0.1/tcp/7001/p2p/{}", peer_id.to_base58())
            .parse()
            .unwrap();

        assert_eq!(Multiaddr::peer_id(&multiaddr).unwrap(), peer_id)
    }

    #[test]
    fn missing_peer_id() {
        let multiaddr: Multiaddr = format!("/ip4/127.0.0.1/tcp/7001").parse().unwrap();

        assert_eq!(Multiaddr::peer_id(&multiaddr), None)
    }
}
