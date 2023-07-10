//! [Workflow] settings for a worker's run/execution.
//!
//! [Workflow]: homestar_core::Workflow

use std::time::Duration;

/// Workflow settings.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub(crate) retries: u32,
    pub(crate) retry_backoff_strategy: BackoffStrategy,
    pub(crate) retry_max_delay: Duration,
    pub(crate) retry_initial_delay: Duration,
    pub(crate) p2p_check_timeout: Duration,
    pub(crate) p2p_timeout: Duration,
}

#[cfg(not(test))]
impl Default for Settings {
    fn default() -> Self {
        Self {
            retries: 10,
            retry_backoff_strategy: BackoffStrategy::Exponential,
            retry_max_delay: Duration::new(60, 0),
            retry_initial_delay: Duration::from_millis(500),
            p2p_check_timeout: Duration::new(5, 0),
            p2p_timeout: Duration::new(60, 0),
        }
    }
}

#[cfg(test)]
impl Default for Settings {
    fn default() -> Self {
        Self {
            retries: 1,
            retry_backoff_strategy: BackoffStrategy::Exponential,
            retry_max_delay: Duration::new(1, 0),
            retry_initial_delay: Duration::from_millis(50),
            p2p_check_timeout: Duration::new(1, 0),
            p2p_timeout: Duration::new(1, 0),
        }
    }
}

/// Backoff strategies supported for workflows.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BackoffStrategy {
    /// Exponential backoff: the delay will double each time.
    Exponential,
    /// Fixed backoff: the delay wont change between attempts.
    Fixed,
    /// Linear backoff: the delay will scale linearly with the number of attempts.
    Linear,
    /// No backoff: forcing just leveraging retries.
    None,
}
