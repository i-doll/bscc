use bscc_core::{Exporter, Report};
use serde::Serialize;
use std::io::{self, Write};

/// JSON exporter emitting a stable, versioned envelope. Tree-sitter-only
/// fields are omitted when absent (the `FileMetrics` Option fields have
/// `skip_serializing_if = "Option::is_none"`).
#[derive(Default)]
pub struct JsonExporter {
    pub pretty: bool,
}

#[derive(Serialize)]
struct Envelope<'a> {
    schema_version: u32,
    report: &'a Report,
}

impl Exporter for JsonExporter {
    fn write(&self, report: &Report, sink: &mut dyn Write) -> io::Result<()> {
        let env = Envelope {
            schema_version: 1,
            report,
        };
        let bytes = if self.pretty {
            serde_json::to_vec_pretty(&env).map_err(io::Error::other)?
        } else {
            serde_json::to_vec(&env).map_err(io::Error::other)?
        };
        sink.write_all(&bytes)?;
        sink.write_all(b"\n")
    }
}
