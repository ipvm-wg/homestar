//! [Workflow] settings for a worker's run/execution.
//!
//! [Workflow]: homestar_core::Workflow

use std::time::Duration;

/// Workflow settings.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// Number of retries for a given workflow.
    pub(crate) retries: u32,
    /// Maximum delay between retries.
    pub(crate) retry_max_delay: Duration,
    /// Initial delay between retries.
    pub(crate) retry_initial_delay: Duration,
    /// Timeout for P2P/DHT operations.
    pub(crate) p2p_timeout: Duration,
    /// Timeout for a given workflow.
    pub(crate) timeout: Duration,
}

#[cfg(all(not(test), not(feature = "test-utils")))]
impl Default for Settings {
    fn default() -> Self {
        Self {
            retries: 3,
            retry_max_delay: Duration::new(60, 0),
            retry_initial_delay: Duration::from_millis(500),
            p2p_timeout: Duration::from_millis(500),
            timeout: Duration::new(3600, 0),
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl Default for Settings {
    fn default() -> Self {
        Self {
            retries: 0,
            retry_max_delay: Duration::new(1, 0),
            retry_initial_delay: Duration::from_millis(50),
            p2p_timeout: Duration::from_millis(10),
            timeout: Duration::from_secs(3600),
        }
    }
}
