use bscc_core::{Exporter, Report};
use owo_colors::{OwoColorize, Stream};
use std::io::{self, Write};

#[derive(Default)]
pub struct TableExporter;

impl Exporter for TableExporter {
    fn write(&self, report: &Report, sink: &mut dyn Write) -> io::Result<()> {
        let totals = report.by_language();
        let grand = report.grand_total();

        let mut rows: Vec<Row> = totals
            .into_values()
            .map(|t| Row {
                name: t.language,
                files: t.files,
                lines: t.lines,
                code: t.code,
                comments: t.comments,
                blanks: t.blanks,
            })
            .collect();
        rows.sort_by(|a, b| b.lines.cmp(&a.lines).then(a.name.cmp(&b.name)));

        let total_row = Row {
            name: "Total".into(),
            files: grand.files,
            lines: grand.lines,
            code: grand.code,
            comments: grand.comments,
            blanks: grand.blanks,
        };

        let widths = compute_widths(&rows, &total_row);
        let sep = "─".repeat(widths.total_width());

        writeln!(sink, "{sep}")?;
        write_header(sink, &widths)?;
        writeln!(sink, "{sep}")?;
        for r in &rows {
            write_row(sink, r, &widths)?;
        }
        writeln!(sink, "{sep}")?;
        write_row(sink, &total_row, &widths)?;
        writeln!(sink, "{sep}")?;
        Ok(())
    }
}

struct Row {
    name: String,
    files: u32,
    lines: u32,
    code: u32,
    comments: u32,
    blanks: u32,
}

struct Widths {
    lang: usize,
    num: usize,
}

impl Widths {
    fn total_width(&self) -> usize {
        // 5 numeric columns + 1 lang column + 6 single-space gutters
        self.lang + 5 * self.num + 6
    }
}

fn compute_widths(rows: &[Row], total: &Row) -> Widths {
    let lang_header = "Language".len();
    let lang_max = rows
        .iter()
        .chain(std::iter::once(total))
        .map(|r| r.name.len())
        .max()
        .unwrap_or(0);
    let num_max = rows
        .iter()
        .chain(std::iter::once(total))
        .flat_map(|r| [r.files, r.lines, r.code, r.comments, r.blanks])
        .map(digits)
        .max()
        .unwrap_or(1);
    Widths {
        lang: lang_header.max(lang_max),
        num: num_max.max("Comments".len()),
    }
}

fn digits(mut n: u32) -> usize {
    if n == 0 {
        return 1;
    }
    let mut d = 0;
    while n > 0 {
        d += 1;
        n /= 10;
    }
    d
}

fn write_header(sink: &mut dyn Write, w: &Widths) -> io::Result<()> {
    writeln!(
        sink,
        "{lang:<lang_w$} {files:>num_w$} {lines:>num_w$} {code:>num_w$} {comments:>num_w$} {blanks:>num_w$}",
        lang = "Language".if_supports_color(Stream::Stdout, |s| s.bold()),
        files = "Files".if_supports_color(Stream::Stdout, |s| s.bold()),
        lines = "Lines".if_supports_color(Stream::Stdout, |s| s.bold()),
        code = "Code".if_supports_color(Stream::Stdout, |s| s.bold()),
        comments = "Comments".if_supports_color(Stream::Stdout, |s| s.bold()),
        blanks = "Blanks".if_supports_color(Stream::Stdout, |s| s.bold()),
        lang_w = w.lang,
        num_w = w.num,
    )
}

fn write_row(sink: &mut dyn Write, r: &Row, w: &Widths) -> io::Result<()> {
    writeln!(
        sink,
        "{name:<lang_w$} {files:>num_w$} {lines:>num_w$} {code:>num_w$} {comments:>num_w$} {blanks:>num_w$}",
        name = r.name,
        files = r.files,
        lines = r.lines,
        code = r.code,
        comments = r.comments,
        blanks = r.blanks,
        lang_w = w.lang,
        num_w = w.num,
    )
}
