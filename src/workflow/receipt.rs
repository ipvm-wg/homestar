use crate::db::schema::receipts;
use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Insertable)]
pub struct Receipt {
    pub id: String,
    pub closure_cid: String, // FIXME Cid,
    pub val: i32,            // FIXME Ipld,
}
