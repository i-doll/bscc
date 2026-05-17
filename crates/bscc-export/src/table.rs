use crate::fmt::{FAMILIES, fmt_int, fmt_usd};
use bscc_core::{CostReport, Estimate, Exporter, Report};
use owo_colors::{OwoColorize, Stream};
use std::io::{self, Write};

#[derive(Default)]
pub struct TableExporter;

impl Exporter for TableExporter {
    fn write(&self, report: &Report, sink: &mut dyn Write) -> io::Result<()> {
        let totals = report.by_language();
        let grand = report.grand_total();
        let cost = report.cost.as_ref();

        let mut rows: Vec<Row> = totals
            .into_values()
            .map(|t| {
                let cost_usd = cost
                    .and_then(|c| c.per_language.get(&t.language))
                    .map(|e| e.cost_usd);
                Row {
                    name: t.language,
                    files: t.files,
                    lines: t.lines,
                    code: t.code,
                    comments: t.comments,
                    blanks: t.blanks,
                    complexity: t.complexity,
                    cost: cost_usd,
                }
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
            complexity: grand.complexity,
            cost: cost.map(|c| c.project.baseline.cost_usd),
        };

        let show_cost = cost.is_some();
        let widths = compute_widths(&blocks, &total_row, show_cost);
        let sep = "─".repeat(widths.total_width());

        writeln!(sink, "{sep}")?;
        write_header(sink, &widths, show_cost)?;
        writeln!(sink, "{sep}")?;
        for block in &blocks {
            write_row(sink, &block.head, 0, &widths, show_cost)?;
            for member in &block.indented {
                write_row(sink, member, INDENT, &widths, show_cost)?;
            }
        }
        writeln!(sink, "{sep}")?;
        write_row(sink, &total_row, 0, &widths, show_cost)?;
        writeln!(sink, "{sep}")?;

        if let Some(c) = cost {
            write_cost_footer(sink, c)?;
        }
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
    complexity: u32,
    cost: Option<u64>,
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
        let mut any_cost = false;
        for m in &indented {
            head.files += m.files;
            head.lines += m.lines;
            head.code += m.code;
            head.comments += m.comments;
            head.blanks += m.blanks;
            head.complexity += m.complexity;
            if let Some(c) = m.cost {
                any_cost = true;
                head.cost = Some(head.cost.unwrap_or(0) + c);
            }
        }
        if !any_cost {
            head.cost = None;
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
    cost: usize,
}

impl Widths {
    fn total_width(&self) -> usize {
        // lang col + 6 numeric cols + optional cost col + (n+1) single-space gutters
        let cost_part = if self.cost > 0 { self.cost + 1 } else { 0 };
        self.lang + 6 * self.num + cost_part + 7
    }
}

fn compute_widths(blocks: &[Block], total: &Row, show_cost: bool) -> Widths {
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
        .flat_map(|r| [r.files, r.lines, r.code, r.comments, r.blanks, r.complexity])
        .map(|n| fmt_int(n).chars().count())
        .max()
        .unwrap_or(1);
    let cost_max = if show_cost {
        let widest = num_rows
            .iter()
            .filter_map(|r| r.cost)
            .map(|c| fmt_usd(c).chars().count())
            .max()
            .unwrap_or(0);
        widest.max("Cost".len())
    } else {
        0
    };
    Widths {
        lang: lang_header.max(lang_max),
        num: num_max.max("Complexity".len()),
        cost: cost_max,
    }
}

fn write_header(sink: &mut dyn Write, w: &Widths, show_cost: bool) -> io::Result<()> {
    if show_cost {
        writeln!(
            sink,
            "{lang:<lang_w$} {files:>num_w$} {lines:>num_w$} {code:>num_w$} {comments:>num_w$} {blanks:>num_w$} {complexity:>num_w$} {cost:>cost_w$}",
            lang = "Language".if_supports_color(Stream::Stdout, |s| s.bold()),
            files = "Files".if_supports_color(Stream::Stdout, |s| s.bold()),
            lines = "Lines".if_supports_color(Stream::Stdout, |s| s.bold()),
            code = "Code".if_supports_color(Stream::Stdout, |s| s.bold()),
            comments = "Comments".if_supports_color(Stream::Stdout, |s| s.bold()),
            blanks = "Blanks".if_supports_color(Stream::Stdout, |s| s.bold()),
            complexity = "Complexity".if_supports_color(Stream::Stdout, |s| s.bold()),
            cost = "Cost".if_supports_color(Stream::Stdout, |s| s.bold()),
            lang_w = w.lang,
            num_w = w.num,
            cost_w = w.cost,
        )
    } else {
        writeln!(
            sink,
            "{lang:<lang_w$} {files:>num_w$} {lines:>num_w$} {code:>num_w$} {comments:>num_w$} {blanks:>num_w$} {complexity:>num_w$}",
            lang = "Language".if_supports_color(Stream::Stdout, |s| s.bold()),
            files = "Files".if_supports_color(Stream::Stdout, |s| s.bold()),
            lines = "Lines".if_supports_color(Stream::Stdout, |s| s.bold()),
            code = "Code".if_supports_color(Stream::Stdout, |s| s.bold()),
            comments = "Comments".if_supports_color(Stream::Stdout, |s| s.bold()),
            blanks = "Blanks".if_supports_color(Stream::Stdout, |s| s.bold()),
            complexity = "Complexity".if_supports_color(Stream::Stdout, |s| s.bold()),
            lang_w = w.lang,
            num_w = w.num,
        )
    }
}

fn write_row(
    sink: &mut dyn Write,
    r: &Row,
    indent: usize,
    w: &Widths,
    show_cost: bool,
) -> io::Result<()> {
    let padded_name = format!("{:indent$}{}", "", r.name, indent = indent);
    if show_cost {
        let cost_cell = r.cost.map(fmt_usd).unwrap_or_default();
        writeln!(
            sink,
            "{name:<lang_w$} {files:>num_w$} {lines:>num_w$} {code:>num_w$} {comments:>num_w$} {blanks:>num_w$} {complexity:>num_w$} {cost:>cost_w$}",
            name = padded_name,
            files = fmt_int(r.files),
            lines = fmt_int(r.lines),
            code = fmt_int(r.code),
            comments = fmt_int(r.comments),
            blanks = fmt_int(r.blanks),
            complexity = fmt_int(r.complexity),
            cost = cost_cell,
            lang_w = w.lang,
            num_w = w.num,
            cost_w = w.cost,
        )
    } else {
        writeln!(
            sink,
            "{name:<lang_w$} {files:>num_w$} {lines:>num_w$} {code:>num_w$} {comments:>num_w$} {blanks:>num_w$} {complexity:>num_w$}",
            name = padded_name,
            files = fmt_int(r.files),
            lines = fmt_int(r.lines),
            code = fmt_int(r.code),
            comments = fmt_int(r.comments),
            blanks = fmt_int(r.blanks),
            complexity = fmt_int(r.complexity),
            lang_w = w.lang,
            num_w = w.num,
        )
    }
}

fn write_cost_footer(sink: &mut dyn Write, c: &CostReport) -> io::Result<()> {
    let p = &c.params;
    writeln!(sink)?;
    writeln!(
        sink,
        "Cost estimate — basic COCOMO ({}, ${}/yr × {:.1} overhead)",
        p.project_type,
        fmt_int(p.avg_wage),
        p.overhead
    )?;
    write_estimate_block(sink, &c.project.baseline)?;
    writeln!(sink)?;
    writeln!(sink, "AI-assisted (×{:.1} productivity)", p.ai_multiplier)?;
    write_estimate_block(sink, &c.project.ai_assisted)?;
    writeln!(sink)?;
    writeln!(
        sink,
        "Per-language costs above sum lower than the project total; COCOMO"
    )?;
    writeln!(
        sink,
        "scales as KLOC^1.05 so a unified codebase costs more than its parts."
    )?;
    writeln!(sink, "The footer is the headline number.")?;
    Ok(())
}

fn write_estimate_block(sink: &mut dyn Write, e: &Estimate) -> io::Result<()> {
    writeln!(sink, "  Effort:    {:>10.1} person-months", e.effort_months)?;
    writeln!(
        sink,
        "  Schedule:  {:>10.1} calendar months",
        e.schedule_months
    )?;
    writeln!(sink, "  People:    {:>10.1} developers", e.people)?;
    writeln!(sink, "  Cost:      {:>10}", fmt_usd(e.cost_usd))?;
    Ok(())
}
