pub mod models;
pub mod schema;

use diesel::prelude::*;
use dotenvy::dotenv;
use models::*;
use schema::{receipts, receipts::dsl::*};
use std::env;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn get_receipts(conn: &mut SqliteConnection) -> Vec<Receipt> {
    receipts
        .limit(5)
        .load::<Receipt>(conn)
        .expect("Error loading receipts")
}

pub fn get_closure_ids(conn: &mut SqliteConnection) -> Vec<String> {
    receipts
        .select(closure_cid)
        .load::<String>(conn)
        .expect("Error loading receipts")
}
