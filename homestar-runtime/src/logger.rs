//! Logger initialization.

use anyhow::Result;
#[cfg(not(feature = "logfmt"))]
use tracing_subscriber::prelude::*;
#[cfg(feature = "logfmt")]
use tracing_subscriber::{layer::SubscriberExt as _, prelude::*, EnvFilter};

/// Initialize a [tracing_subscriber::Registry] with a [logfmt] layer.
///
/// [logfmt]: <https://brandur.org/logfmt>
#[cfg(feature = "logfmt")]
pub fn init() -> Result<()> {
    let registry = tracing_subscriber::Registry::default()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_logfmt::layer());

    #[cfg(all(feature = "console", tokio_unstable))]
    #[cfg_attr(docsrs, doc(cfg(feature = "console")))]
    {
        let console_layer = console_subscriber::ConsoleLayer::builder()
            .retention(Duration::from_secs(60))
            .spawn();

        registry.with(console_layer).init();
    }

    #[cfg(any(not(feature = "console"), not(tokio_unstable)))]
    {
        registry.init();
    }

    Ok(())
}

/// Initialize a default [tracing_subscriber::FmtSubscriber].
#[cfg(not(feature = "logfmt"))]
pub fn init() -> Result<()> {
    let registry = tracing_subscriber::FmtSubscriber::builder()
        .with_target(false)
        .finish();

    #[cfg(all(feature = "console", tokio_unstable))]
    #[cfg_attr(docsrs, doc(cfg(feature = "console")))]
    {
        let console_layer = console_subscriber::ConsoleLayer::builder()
            .retention(Duration::from_secs(60))
            .spawn();

        registry.with(console_layer).init();
    }

    #[cfg(any(not(feature = "console"), not(tokio_unstable)))]
    {
        registry.init();
    }

    Ok(())
}
