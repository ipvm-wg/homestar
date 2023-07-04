//! CID test and generation utilities.

use libipld::{Cid, Multihash};
use rand::RngCore;

fn get_random_bytes<const N: usize>(rng: &mut impl RngCore) -> [u8; N] {
    let mut bytes = [0u8; N];
    rng.fill_bytes(&mut bytes);
    bytes
}

/// Generate a random [Cid] with a `0x55` prefix.
pub fn generate_cid(rng: &mut impl RngCore) -> Cid {
    let bytes = {
        let mut tmp = [0u8; 10];
        let (a, b) = tmp.split_at_mut(2);
        a.copy_from_slice(&[0x55, 0x08]);
        b.copy_from_slice(&get_random_bytes::<8>(rng));
        tmp
    };

    Cid::new_v1(0x55, Multihash::from_bytes(&bytes).unwrap())
}
