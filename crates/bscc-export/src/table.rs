use crate::fmt::{FAMILIES, fmt_int};
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

        // Organize into display blocks. A block is either a single
        // standalone language or a family header followed by indented
        // members. Families only activate when ≥2 of their members are
        // present in the report.
        let blocks = build_blocks(&rows);

        let total_row = Row {
            name: "Total".into(),
            files: grand.files,
            lines: grand.lines,
            code: grand.code,
            comments: grand.comments,
            blanks: grand.blanks,
        };

        let widths = compute_widths(&blocks, &total_row);
        let sep = "─".repeat(widths.total_width());

        writeln!(sink, "{sep}")?;
        write_header(sink, &widths)?;
        writeln!(sink, "{sep}")?;
        for block in &blocks {
            write_row(sink, &block.head, 0, &widths)?;
            for member in &block.indented {
                write_row(sink, member, INDENT, &widths)?;
            }
        }
        writeln!(sink, "{sep}")?;
        write_row(sink, &total_row, 0, &widths)?;
        writeln!(sink, "{sep}")?;
        Ok(())
    }
}

const INDENT: usize = 2;

#[derive(Default, Clone)]
struct Row {
    name: String,
    files: u32,
    lines: u32,
    code: u32,
    comments: u32,
    blanks: u32,
}

struct Block {
    head: Row,
    indented: Vec<Row>,
}

/// Turn the per-language rows into display blocks. Active families (≥2
/// members present) become a head row with indented members; everything else
/// is a single-row block. The block list is sorted by head.lines descending.
fn build_blocks(rows: &[Row]) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();
    let mut consumed: Vec<&str> = Vec::new();

    for fam in FAMILIES {
        let members: Vec<&Row> = fam
            .members
            .iter()
            .filter_map(|m| rows.iter().find(|r| r.name == *m))
            .collect();
        if members.len() < 2 {
            continue;
        }
        let mut head = Row {
            name: fam.display.into(),
            ..Row::default()
        };
        let mut indented: Vec<Row> = members.iter().map(|&r| r.clone()).collect();
        for m in &indented {
            head.files += m.files;
            head.lines += m.lines;
            head.code += m.code;
            head.comments += m.comments;
            head.blanks += m.blanks;
        }
        indented.sort_by(|a, b| b.lines.cmp(&a.lines).then(a.name.cmp(&b.name)));
        for m in fam.members {
            consumed.push(m);
        }
        blocks.push(Block { head, indented });
    }

    for r in rows {
        if !consumed.iter().any(|c| *c == r.name) {
            blocks.push(Block {
                head: r.clone(),
                indented: Vec::new(),
            });
        }
    }

    blocks.sort_by(|a, b| {
        b.head
            .lines
            .cmp(&a.head.lines)
            .then(a.head.name.cmp(&b.head.name))
    });
    blocks
}

struct Widths {
    lang: usize,
    num: usize,
}

impl Widths {
    fn total_width(&self) -> usize {
        // 5 numeric columns + lang column + 6 single-space gutters
        self.lang + 5 * self.num + 6
    }
}

fn compute_widths(blocks: &[Block], total: &Row) -> Widths {
    let lang_header = "Language".len();
    let mut lang_max = total.name.chars().count();
    let mut num_rows: Vec<&Row> = vec![total];
    for b in blocks {
        lang_max = lang_max.max(b.head.name.chars().count());
        num_rows.push(&b.head);
        for m in &b.indented {
            lang_max = lang_max.max(m.name.chars().count() + INDENT);
            num_rows.push(m);
        }
    }
    let num_max = num_rows
        .iter()
        .flat_map(|r| [r.files, r.lines, r.code, r.comments, r.blanks])
        .map(|n| fmt_int(n).chars().count())
        .max()
        .unwrap_or(1);
    Widths {
        lang: lang_header.max(lang_max),
        num: num_max.max("Comments".len()),
    }
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

fn write_row(sink: &mut dyn Write, r: &Row, indent: usize, w: &Widths) -> io::Result<()> {
    let padded_name = format!("{:indent$}{}", "", r.name, indent = indent);
    writeln!(
        sink,
        "{name:<lang_w$} {files:>num_w$} {lines:>num_w$} {code:>num_w$} {comments:>num_w$} {blanks:>num_w$}",
        name = padded_name,
        files = fmt_int(r.files),
        lines = fmt_int(r.lines),
        code = fmt_int(r.code),
        comments = fmt_int(r.comments),
        blanks = fmt_int(r.blanks),
        lang_w = w.lang,
        num_w = w.num,
    )
}
