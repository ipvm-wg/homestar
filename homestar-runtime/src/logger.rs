//! Logger initialization.

use crate::settings;
use std::{io, path::PathBuf};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::{layer::SubscriberExt as _, prelude::*, EnvFilter};

const LOG_FILE: &str = "homestar.log";
const DIRECTIVE_EXPECT: &str = "Invalid tracing directive";

/// Logger interface.
#[derive(Debug)]
pub struct Logger;
/// File-logger interface.
#[derive(Debug)]
pub struct FileLogger;

impl Logger {
    /// Initialize a [tracing_subscriber::Registry] with a [logfmt] layer and
    /// write to [io::stdout].
    ///
    /// [logfmt]: <https://brandur.org/logfmt>
    pub fn init(settings: &settings::Monitoring) -> WorkerGuard {
        let (writer, guard) = tracing_appender::non_blocking(io::stdout());
        init(writer, guard, settings)
    }
}

impl FileLogger {
    /// Initialize a [tracing_subscriber::Registry] with a [logfmt] layer and
    /// write to file.
    ///
    /// [logfmt]: <https://brandur.org/logfmt>
    pub fn init(dir: PathBuf, settings: &settings::Monitoring) -> WorkerGuard {
        let file_appender = tracing_appender::rolling::daily(dir, LOG_FILE);
        let (writer, guard) = tracing_appender::non_blocking(file_appender);
        init(writer, guard, settings)
    }
}

fn init(
    writer: NonBlocking,
    guard: WorkerGuard,
    #[allow(unused_variables)] settings: &settings::Monitoring,
) -> WorkerGuard {
    // RUST_LOG ignored when EVERY_CLI is true
    let every_cli = std::env::var("EVERY_CLI").is_ok_and(|val| val == "true");

    // TODO: Add support for customizing logger(s) / specialzed formatters.
    let format_layer = if every_cli {
        tracing_logfmt::builder()
            .with_level(true)
            .with_target(false)
            .with_span_name(false)
            .with_span_path(false)
            .with_location(false)
            .with_module_path(false)
            .layer()
            .with_writer(writer)
    } else {
        tracing_logfmt::builder()
            .with_level(true)
            .with_target(true)
            .with_span_name(true)
            .with_span_path(true)
            .with_location(true)
            .with_module_path(true)
            .layer()
            .with_writer(writer)
    };

    let filter = if every_cli {
        EnvFilter::new("off")
            .add_directive(
                "homestar_runtime::worker[run]=info"
                    .parse()
                    .expect(DIRECTIVE_EXPECT),
            )
            .add_directive(
                "homestar_runtime::worker[spawn_tasks]=info"
                    .parse()
                    .expect(DIRECTIVE_EXPECT),
            )
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("info")
                .add_directive("homestar_wasm=info".parse().expect(DIRECTIVE_EXPECT))
                .add_directive("libp2p=info".parse().expect(DIRECTIVE_EXPECT))
                .add_directive(
                    "libp2p_gossipsub::behaviour=info"
                        .parse()
                        .expect(DIRECTIVE_EXPECT),
                )
                .add_directive("tarpc=info".parse().expect(DIRECTIVE_EXPECT))
                .add_directive("tower_http=info".parse().expect(DIRECTIVE_EXPECT))
                .add_directive("moka=info".parse().expect(DIRECTIVE_EXPECT))
                .add_directive("jsonrpsee=info".parse().expect(DIRECTIVE_EXPECT))
        })
    };

    #[cfg(all(
        feature = "console",
        not(test),
        not(feature = "test-utils"),
        tokio_unstable
    ))]
    let filter = filter
        .add_directive("tokio=trace".parse().expect(DIRECTIVE_EXPECT))
        .add_directive("runtime=trace".parse().expect(DIRECTIVE_EXPECT));

    let registry = tracing_subscriber::Registry::default()
        .with(filter)
        .with(format_layer);

    #[cfg(all(
        feature = "console",
        not(test),
        not(feature = "test-utils"),
        tokio_unstable
    ))]
    {
        let console_layer = console_subscriber::ConsoleLayer::builder()
            .retention(std::time::Duration::from_secs(60))
            .server_addr(([127, 0, 0, 1], settings.console_subscriber_port))
            .spawn();

        registry.with(console_layer).init();
    }

    #[cfg(any(
        not(feature = "console"),
        test,
        not(tokio_unstable),
        feature = "test-utils",
    ))]
    {
        registry.init();
    }

    guard
}
