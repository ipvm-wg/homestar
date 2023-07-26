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

diesel::table! {
    workflows (cid) {
        cid -> Text,
        num_tasks -> Integer,
        resources -> Binary,
        created_at -> Timestamp,
        completed_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    workflows_receipts (workflow_cid, receipt_cid) {
        workflow_cid -> Text,
        receipt_cid -> Text,
    }
}

diesel::joinable!(workflows_receipts -> receipts (receipt_cid));
diesel::joinable!(workflows_receipts -> workflows (workflow_cid));

diesel::allow_tables_to_appear_in_same_query!(receipts, workflows, workflows_receipts,);
