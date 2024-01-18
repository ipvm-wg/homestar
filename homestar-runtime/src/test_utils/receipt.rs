//! Test utilities for working with [receipts].
//!
//! [receipts]: crate::Receipt

use crate::Receipt;
use homestar_invocation::{
    authority::UcanPrf, receipt::Receipt as InvocationReceipt, task, test_utils, Pointer,
};
use libipld::{
    cid::Cid,
    multihash::{Code, MultihashDigest},
    Ipld, Link,
};

#[allow(dead_code)]
const RAW: u64 = 0x55;

/// Return both a `mocked` [Ucan Invocation Receipt] and a runtime [Receipt]
///
/// [UCAN Invocation Receipt]: homestar_invocation::Receipt
#[allow(dead_code)]
pub(crate) fn receipts() -> (InvocationReceipt<Ipld>, Receipt) {
    let h = Code::Blake3_256.digest(b"beep boop");
    let cid = Cid::new_v1(RAW, h);
    let link: Link<Cid> = Link::new(cid);
    let local = InvocationReceipt::new(
        Pointer::new_from_link(link),
        task::Result::Ok(Ipld::Bool(true)),
        Ipld::Null,
        None,
        UcanPrf::default(),
    );
    let receipt = Receipt::try_with(
        test_utils::instruction::<Ipld>().try_into().unwrap(),
        &local,
    )
    .unwrap();

    (local, receipt)
}
