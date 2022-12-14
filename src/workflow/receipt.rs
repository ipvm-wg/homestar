use crate::db::schema::receipts;
use diesel::{Insertable, Queryable};
use libipld::{Cid, Ipld};

#[derive(Queryable, Debug)]
pub struct Receipt {
    pub id: String,
    pub closure_cid: String, // FIXME Cid,
    pub val: i32,            // FIXME Ipld,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = receipts)]
pub struct NewReceipt {
    pub closure_cid: String, // FIXME Cid,
    pub val: i32,            //FIXME Ipld,
}
