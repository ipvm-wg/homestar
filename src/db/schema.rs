// @generated automatically by Diesel CLI.

diesel::table! {
    receipts (id) {
        id -> Text,
        closure_cid -> Text,
        val -> Integer,
    }
}
