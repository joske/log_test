use std::{
    io::stdout,
    sync::atomic::{AtomicBool, Ordering},
    time::SystemTime,
};

use chrono::{DateTime, Utc};
use crossterm::tty::IsTty;
use tracing::{info, Event, Subscriber};
use tracing_subscriber::{
    fmt::{format::Writer, FmtContext, FormatEvent, FormatFields},
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

fn main() {
    let filter = EnvFilter::from_default_env();
    let _ = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::Layer::default()
                .with_ansi(stdout().is_tty())
                .event_format(DynamicFormatter)
                .with_filter(filter),
        )
        .try_init();
    info!("NORMAL");
    DIM_FORMAT.store(true, Ordering::Relaxed);
    info!("DIM");
    DIM_FORMAT.store(false, Ordering::Relaxed);
    info!("NORMAL");
}

static DIM_FORMAT: AtomicBool = AtomicBool::new(false);

struct DynamicFormatter;

impl<S, N> FormatEvent<S, N> for DynamicFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let default = tracing_subscriber::fmt::format::Format::default();
        let dim = MyFormat::new();
        if DIM_FORMAT.load(Ordering::Relaxed) {
            dim.format_event(ctx, writer, event)
        } else {
            default.format_event(ctx, writer, event)
        }
    }
}

#[derive(Debug)]
struct MyFormat {}

impl MyFormat {
    fn new() -> Self {
        Self {}
    }
}

impl<S, N> FormatEvent<S, N> for MyFormat
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let meta = event.metadata();
        let system_time = SystemTime::now();
        let date_time: DateTime<Utc> = system_time.into();
        write!(
            writer,
            "\x1b[2m{}  ",
            date_time.format("%Y-%m-%dT%H:%M:%S%.6fZ")
        )?;

        let fmt_level = match *meta.level() {
            tracing::Level::ERROR => "ERROR",
            tracing::Level::WARN => "WARN ",
            tracing::Level::INFO => "INFO ",
            tracing::Level::DEBUG => "DEBUG",
            tracing::Level::TRACE => "TRACE",
        };
        write!(writer, "{}", fmt_level)?;

        write!(writer, "{}: ", meta.target(),)?;

        ctx.format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}
