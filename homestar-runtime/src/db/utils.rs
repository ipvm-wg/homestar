//! Utility functions Database interaction.

use chrono::NaiveDateTime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Trait for converting nanoseconds to a timestamp.
pub(crate) trait Timestamp {
    fn timestamp_from_nanos(&self) -> Option<NaiveDateTime>;
}

impl Timestamp for i64 {
    fn timestamp_from_nanos(&self) -> Option<NaiveDateTime> {
        let nanos = self % 1_000_000_000;
        let seconds = (self - nanos) / 1_000_000_000;
        NaiveDateTime::from_timestamp_opt(seconds, nanos as u32)
    }
}

/// Health status of the server and database connection.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Health {
    /// Health status.
    pub healthy: bool,
}
