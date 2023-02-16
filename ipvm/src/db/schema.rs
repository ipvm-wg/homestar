diesel::table! {
    receipts (cid) {
        cid -> Text,
        closure_cid -> Text,
        nonce -> Text,
        out -> Binary,
    }
}
