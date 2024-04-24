use std::{
    io::stdout,
    sync::atomic::{AtomicUsize, Ordering},
    time::SystemTime,
};

use chrono::{DateTime, Utc};
use crossterm::tty::IsTty;
use tracing::{info, Event, Subscriber};
use tracing_subscriber::{
    fmt::{format::Writer, FmtContext, FormatEvent, FormatFields},
    layer::SubscriberExt,
    registry::LookupSpan,
    EnvFilter, Layer,
};

fn main() {
    let filter = EnvFilter::from_default_env();
    let _ = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::Layer::default()
                .with_ansi(stdout().is_tty())
                .event_format(DynamicFormatter::new())
                .with_filter(filter),
        )
        .try_init();
    info!("NORMAL");
    CURRENT_FORMATTER.store(1, Ordering::Relaxed);
    info!("DIM");
    info!("NORMAL");
}

static CURRENT_FORMATTER: AtomicUsize = AtomicUsize::new(0);

struct DynamicFormatter<S, N>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    default: Box<dyn FormatEvent<S, N>>,
    dim: Box<dyn FormatEvent<S, N>>,
}

impl<S, N> FormatEvent<S, N> for DynamicFormatter<S, N>
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
        match CURRENT_FORMATTER.load(Ordering::Relaxed) {
            0 => self.default.format_event(ctx, writer, event),
            _ => self.dim.format_event(ctx, writer, event),
        }
    }
}

impl<S, N> DynamicFormatter<S, N>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    pub fn new() -> Self {
        let default = Box::new(tracing_subscriber::fmt::format::Format::default());
        let dim = Box::new(MyFormat {});
        Self { default, dim }
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
