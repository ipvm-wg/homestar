//! Workflow settings for a worker's run/execution.

/// Workflow settings.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub(crate) retries: u32,
    pub(crate) retry_backoff_strategy: BackoffStrategy,
    pub(crate) retry_max_delay_secs: u64,
    pub(crate) retry_initial_delay_ms: u64,
    pub(crate) p2p_timeout_secs: u64,
}

#[cfg(not(test))]
impl Default for Settings {
    fn default() -> Self {
        Self {
            retries: 10,
            retry_backoff_strategy: BackoffStrategy::Exponential,
            retry_max_delay_secs: 60,
            retry_initial_delay_ms: 500,
            p2p_timeout_secs: 60,
        }
    }
}

#[cfg(test)]
impl Default for Settings {
    fn default() -> Self {
        Self {
            retries: 1,
            retry_backoff_strategy: BackoffStrategy::Exponential,
            retry_max_delay_secs: 1,
            retry_initial_delay_ms: 50,
            p2p_timeout_secs: 1,
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
