use crate::db::schema::receipts;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
pub struct Receipt {
    pub id: String,
    pub closure_cid: String, // FIXME Cid,
    pub val: i32,            // FIXME Ipld,
}
