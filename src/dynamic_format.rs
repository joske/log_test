use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use time::{
    format_description::{self, OwnedFormatItem},
    OffsetDateTime,
};
use tracing::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{format::Writer, FmtContext, FormatEvent, FormatFields},
    registry::LookupSpan,
};

/// A dynamic formatter that can switch between the default formatter and the DIM style.
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

/// A custom format for the DIM style.
/// This formatter is quite basic and does not support all the features of the default formatter.
/// It does support all the default fields of the default formatter.
struct DimFormat {
    fmt: OwnedFormatItem,
}

impl DimFormat {
    fn new() -> Self {
        let fmt = format_description::parse_owned::<2>(
            "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:6]Z",
        )
        .expect("failed to set timestampt format");
        Self { fmt }
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
        // set the DIM style
        if writer.has_ansi_escapes() {
            write!(writer, "\x1b[2m")?;
        }

        let date_time = OffsetDateTime::now_utc();
        write!(
            writer,
            "{}  ",
            date_time.format(&self.fmt).map_err(|_| std::fmt::Error)?
        )?;

        let meta = event.metadata();
        let fmt_level = match *meta.level() {
            tracing::Level::ERROR => "ERROR",
            tracing::Level::WARN => "WARN ",
            tracing::Level::INFO => "INFO ",
            tracing::Level::DEBUG => "DEBUG",
            tracing::Level::TRACE => "TRACE",
        };
        write!(writer, "{}", fmt_level)?;

        write!(writer, "{}: ", meta.target())?;

        ctx.format_fields(writer.by_ref(), event)?;

        // reset the style
        if writer.has_ansi_escapes() {
            write!(writer, "\x1b[0m")?;
        }
        writeln!(writer)
    }
}
