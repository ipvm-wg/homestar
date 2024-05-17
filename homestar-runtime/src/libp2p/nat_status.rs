/// NatStatus extension methods.
use libp2p::{autonat::NatStatus, Multiaddr};

/// [NatStatus] extension trait.
pub(crate) trait NatStatusExt {
    fn to_tuple(&self) -> (String, Option<Multiaddr>);
}

impl NatStatusExt for NatStatus {
    fn to_tuple(&self) -> (String, Option<Multiaddr>) {
        match &self {
            NatStatus::Public(address) => ("Public".to_string(), Some(address.to_owned())),
            NatStatus::Private => ("Private".to_string(), None),
            NatStatus::Unknown => ("Unknown".to_string(), None),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn converts_nat_status_to_tuple() {
        let address: Multiaddr = "/ip4/127.0.0.1/tcp/7001".parse().unwrap();
        let public_status = NatStatus::Public(address.clone());
        let private_status = NatStatus::Private;
        let unknown_status = NatStatus::Unknown;

        assert_eq!(
            public_status.to_tuple(),
            ("Public".to_string(), Some(address))
        );
        assert_eq!(private_status.to_tuple(), ("Private".to_string(), None));
        assert_eq!(unknown_status.to_tuple(), ("Unknown".to_string(), None));
    }
}
