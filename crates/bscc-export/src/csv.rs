use bscc_core::{Exporter, Report};
use std::io::{self, Write};

/// CSV: one row per file with every metric field. Optional tree-sitter-tier
/// fields render as the empty string when absent.
#[derive(Default)]
pub struct CsvExporter;

const HEADERS: &[&str] = &[
    "path",
    "language",
    "bytes",
    "lines",
    "code",
    "comments",
    "blanks",
    "functions",
    "cyclomatic_total",
    "cyclomatic_max",
    "cognitive",
    "max_nesting_depth",
    "longest_function_lines",
    "todo_comments",
    "imports",
];

impl Exporter for CsvExporter {
    fn write(&self, report: &Report, sink: &mut dyn Write) -> io::Result<()> {
        let mut first = true;
        for h in HEADERS {
            if !first {
                sink.write_all(b",")?;
            }
            sink.write_all(h.as_bytes())?;
            first = false;
        }
        sink.write_all(b"\n")?;

        for f in &report.files {
            write_csv_field(sink, f.path.to_string_lossy().as_ref(), true)?;
            write_csv_field(sink, &f.language, false)?;
            write_num(sink, f.bytes, false)?;
            write_num(sink, u64::from(f.lines), false)?;
            write_num(sink, u64::from(f.code), false)?;
            write_num(sink, u64::from(f.comments), false)?;
            write_num(sink, u64::from(f.blanks), false)?;
            write_opt(sink, f.functions)?;
            write_opt(sink, f.cyclomatic_total)?;
            write_opt(sink, f.cyclomatic_max)?;
            write_opt(sink, f.cognitive)?;
            write_opt(sink, f.max_nesting_depth)?;
            write_opt(sink, f.longest_function_lines)?;
            write_opt(sink, f.todo_comments)?;
            write_opt(sink, f.imports)?;
            sink.write_all(b"\n")?;
        }
        Ok(())
    }
}

fn write_csv_field(sink: &mut dyn Write, value: &str, first: bool) -> io::Result<()> {
    if !first {
        sink.write_all(b",")?;
    }
    let needs_quote = value.contains([',', '"', '\n']);
    if needs_quote {
        sink.write_all(b"\"")?;
        for ch in value.chars() {
            if ch == '"' {
                sink.write_all(b"\"\"")?;
            } else {
                let mut buf = [0u8; 4];
                let s = ch.encode_utf8(&mut buf);
                sink.write_all(s.as_bytes())?;
            }
        }
        sink.write_all(b"\"")?;
    } else {
        sink.write_all(value.as_bytes())?;
    }
    Ok(())
}

fn write_num(sink: &mut dyn Write, n: u64, first: bool) -> io::Result<()> {
    if !first {
        sink.write_all(b",")?;
    }
    write!(sink, "{n}")
}

fn write_opt(sink: &mut dyn Write, n: Option<u32>) -> io::Result<()> {
    sink.write_all(b",")?;
    if let Some(v) = n {
        write!(sink, "{v}")?;
    }
    Ok(())
}
