//! Monotonic ports.

use once_cell::sync::OnceCell;
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};

static PORTS: OnceCell<AtomicUsize> = OnceCell::new();

/// Return a unique port(in runtime) for test
pub fn get_port() -> usize {
    PORTS
        .get_or_init(|| AtomicUsize::new(rand::thread_rng().gen_range(2000..6800)))
        .fetch_add(1, Ordering::Relaxed)
}
