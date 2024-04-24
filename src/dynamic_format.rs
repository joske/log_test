use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::SystemTime,
};

use chrono::{DateTime, Utc};
use tracing::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{format::Writer, FmtContext, FormatEvent, FormatFields},
    registry::LookupSpan,
};

pub struct DynamicFormatter {
    dim_format: DimFormat,
    default_format: tracing_subscriber::fmt::format::Format,
    dim: Arc<AtomicBool>,
}

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
        if self.dim.load(Ordering::Relaxed) {
            self.dim_format.format_event(ctx, writer, event)
        } else {
            self.default_format.format_event(ctx, writer, event)
        }
    }
}

impl DynamicFormatter {
    pub fn new(dim: Arc<AtomicBool>) -> Self {
        let dim_format = DimFormat::new();
        let default_format = tracing_subscriber::fmt::format::Format::default();
        Self {
            dim_format,
            default_format,
            dim,
        }
    }
}

struct DimFormat {}

impl DimFormat {
    fn new() -> Self {
        Self {}
    }
}

impl<S, N> FormatEvent<S, N> for DimFormat
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
        if writer.has_ansi_escapes() {
            write!(writer, "\x1b[2m")?;
        }

        let system_time = SystemTime::now();
        let date_time: DateTime<Utc> = system_time.into();
        write!(writer, "{}  ", date_time.format("%Y-%m-%dT%H:%M:%S%.6fZ"))?;

        let meta = event.metadata();
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
