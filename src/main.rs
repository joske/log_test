use std::{
    io::stdout,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crossterm::tty::IsTty;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

mod dynamic_format;

use dynamic_format::DynamicFormatter;

fn main() {
    let dim = Arc::new(AtomicBool::new(false));

    let _ = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::Layer::default()
                .with_ansi(stdout().is_tty())
                .event_format(DynamicFormatter::new(dim.clone()))
                .with_filter(EnvFilter::from_default_env()),
        )
        .try_init();

    info!("NORMAL");
    dim.store(true, Ordering::Relaxed);
    info!("DIM");
    dim.store(false, Ordering::Relaxed);
    info!("NORMAL");
}
