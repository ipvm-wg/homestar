// @generated automatically by Diesel CLI.

diesel::table! {
    receipts (cid) {
        cid -> Text,
        ran -> Text,
        instruction -> Text,
        out -> Binary,
        meta -> Binary,
        issuer -> Nullable<Text>,
        prf -> Binary,
        version -> Text,
    }
}
