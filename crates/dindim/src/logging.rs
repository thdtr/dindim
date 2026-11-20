use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, fmt};

const KEEP_FILES: usize = 7;

/// Writes JSON logs to a daily rotating file and, in debug builds, readable
/// logs to stderr. `DINDIM_LOG` overrides the level filter.
pub fn init(dir: &Path) -> Option<WorkerGuard> {
    let filter = || {
        EnvFilter::try_from_env("DINDIM_LOG").unwrap_or_else(|_| {
            EnvFilter::new("warn,dindim=info,dindim_store=info,dindim_core=info")
        })
    };
    let stderr = cfg!(debug_assertions).then(|| {
        fmt::layer()
            .with_writer(std::io::stderr)
            .with_filter(filter())
    });

    let appender = std::fs::create_dir_all(dir).ok().and_then(|()| {
        Builder::new()
            .rotation(Rotation::DAILY)
            .filename_prefix("dindim")
            .filename_suffix("log")
            .max_log_files(KEEP_FILES)
            .build(dir)
            .ok()
    });
    let (file, guard) = match appender {
        Some(appender) => {
            let (writer, guard) = tracing_appender::non_blocking(appender);
            let layer = fmt::layer()
                .json()
                .with_current_span(false)
                .with_writer(writer)
                .with_filter(filter());
            (Some(layer), Some(guard))
        }
        None => (None, None),
    };

    let _ = tracing_subscriber::registry()
        .with(file)
        .with(stderr)
        .try_init();
    install_panic_hook();
    guard
}

fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_default();
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_default();
        tracing::error!(%location, %message, "panic");
        previous(info);
    }));
}
