//! Logger initialization.

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt as _, prelude::*, EnvFilter};

const DIRECTIVE_EXPECT: &str = "Invalid tracing directive";

/// Initialize a [tracing_subscriber::Registry] with a [logfmt] layer.
///
/// [logfmt]: <https://brandur.org/logfmt>
pub fn init(writer: tracing_appender::non_blocking::NonBlocking) -> Result<()> {
    let format_layer = tracing_subscriber::fmt::layer()
        .event_format(tracing_logfmt::EventsFormatter::default())
        .fmt_fields(tracing_logfmt::FieldsFormatter::default())
        .with_writer(writer);

    #[cfg(all(feature = "console", tokio_unstable))]
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::new("info")
                .add_directive("libp2p=info".parse().expect(DIRECTIVE_EXPECT))
                .add_directive(
                    "libp2p_gossipsub::behaviour=debug"
                        .parse()
                        .expect(DIRECTIVE_EXPECT),
                )
        })
        .add_directive("tokio=trace".parse().expect(DIRECTIVE_EXPECT))
        .add_directive("runtime=trace".parse().expect(DIRECTIVE_EXPECT));

    #[cfg(any(not(feature = "console"), not(tokio_unstable)))]
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info")
            .add_directive("libp2p=info".parse().expect(DIRECTIVE_EXPECT))
            .add_directive(
                "libp2p_gossipsub::behaviour=debug"
                    .parse()
                    .expect(DIRECTIVE_EXPECT),
            )
    });

    let registry = tracing_subscriber::Registry::default()
        .with(filter)
        .with(format_layer);

    #[cfg(all(feature = "console", tokio_unstable))]
    {
        let console_layer = console_subscriber::ConsoleLayer::builder()
            .retention(std::time::Duration::from_secs(60))
            .spawn();

        registry.with(console_layer).init();
    }

    #[cfg(any(not(feature = "console"), not(tokio_unstable)))]
    {
        registry.init();
    }

    Ok(())
}
