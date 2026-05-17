use crate::Report;
use std::io::{self, Write};

/// Exporters convert a `Report` into bytes on a sink (stdout, a file, …).
pub trait Exporter {
    fn write(&self, report: &Report, sink: &mut dyn Write) -> io::Result<()>;
}
