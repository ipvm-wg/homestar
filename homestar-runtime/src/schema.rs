// @generated automatically by Diesel CLI.

diesel::table! {
    receipts (cid) {
        cid -> Text,
        ran -> Text,
        out -> Binary,
        meta -> Binary,
        iss -> Nullable<Text>,
        prf -> Binary,
    }
}
