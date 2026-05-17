use crate::fmt::{FAMILIES, fmt_int, fmt_usd};
use bscc_core::{CostReport, Exporter, Report};
use std::io::{self, Write};

/// Self-contained HTML report. Inline CSS, no external assets, no JS.
#[derive(Default)]
pub struct HtmlExporter;

const CSS: &str = "\
* { box-sizing: border-box; }
body { font: 14px/1.5 -apple-system, system-ui, sans-serif; max-width: 1200px; margin: 2em auto; padding: 0 1.5em; color: #1a1a1a; background: #fff; }
header { border-bottom: 2px solid #2563eb; padding-bottom: 1em; margin-bottom: 2em; }
header h1 { margin: 0 0 .25em; font-size: 1.8em; color: #2563eb; }
header p { margin: 0; color: #555; font-size: .95em; }
h2 { margin: 2em 0 .5em; font-size: 1.2em; border-bottom: 1px solid #ddd; padding-bottom: .25em; }
table { border-collapse: collapse; width: 100%; font-size: .92em; }
th, td { padding: 6px 10px; text-align: right; border-bottom: 1px solid #eee; }
th:first-child, td:first-child { text-align: left; }
th { background: #f4f6fa; color: #333; font-weight: 600; }
tr:hover td { background: #fbfbfd; }
td.muted { color: #999; }
td.hot { color: #c2410c; font-weight: 600; }
tr.family td:first-child { font-weight: 600; }
tr.family td { background: #f4f6fa; border-top: 1px solid #ccd; }
tr.member td:first-child { padding-left: 2em; color: #555; }
code { font: 12.5px ui-monospace, SF Mono, Consolas, monospace; color: #4338ca; }
footer { margin-top: 3em; padding-top: 1em; border-top: 1px solid #eee; font-size: .85em; color: #999; }
.cost-cards { display: grid; grid-template-columns: 1fr 1fr; gap: 1em; margin: 1em 0; }
.cost-card { padding: 1em 1.25em; background: #f4f6fa; border-radius: 8px; border-left: 4px solid #2563eb; }
.cost-card.ai { border-left-color: #9333ea; }
.cost-card h3 { margin: 0 0 .5em; font-size: 1em; color: #333; }
.cost-card dl { margin: 0; display: grid; grid-template-columns: max-content 1fr; gap: 4px 1em; font-size: .92em; }
.cost-card dt { color: #666; }
.cost-card dd { margin: 0; font-variant-numeric: tabular-nums; text-align: right; }
.cost-card dd.total { font-weight: 600; color: #1a1a1a; }
.cost-note { font-size: .85em; color: #888; margin-top: .5em; }
";

impl Exporter for HtmlExporter {
    #[allow(clippy::too_many_lines)] // self-contained HTML builder, clearer inline
    fn write(&self, report: &Report, sink: &mut dyn Write) -> io::Result<()> {
        let total = report.grand_total();
        let by_lang = report.by_language();
        let cost = report.cost.as_ref();
        let show_cost = cost.is_some();

        write!(
            sink,
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>bscc report</title><style>{CSS}</style></head><body>"
        )?;
        write!(
            sink,
            "<header><h1>bscc report</h1><p>{} files &middot; {} lines &middot; {} code &middot; {} comments &middot; {} blanks</p></header>",
            fmt_int(total.files),
            fmt_int(total.lines),
            fmt_int(total.code),
            fmt_int(total.comments),
            fmt_int(total.blanks)
        )?;

        // Languages summary
        let cost_header = if show_cost { "<th>Cost</th>" } else { "" };
        write!(
            sink,
            "<section><h2>Languages</h2><table><thead><tr><th>Language</th><th>Files</th><th>Lines</th><th>Code</th><th>Comments</th><th>Blanks</th>{cost_header}</tr></thead><tbody>"
        )?;
        let langs: Vec<_> = by_lang.into_values().collect();
        let blocks = lang_blocks(&langs);
        for block in &blocks {
            let row_class = if block.is_family { "family" } else { "" };
            let head_cost = if show_cost {
                let sum: u64 = if block.is_family {
                    block
                        .members
                        .iter()
                        .filter_map(|m| {
                            cost.and_then(|c| c.per_language.get(&m.name)).map(|e| e.cost_usd)
                        })
                        .sum()
                } else {
                    cost.and_then(|c| c.per_language.get(&block.head.name))
                        .map_or(0, |e| e.cost_usd)
                };
                format!("<td>{}</td>", fmt_usd(sum))
            } else {
                String::new()
            };
            write!(
                sink,
                "<tr class=\"{}\"><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td>{}</tr>",
                row_class,
                escape(&block.head.name),
                fmt_int(block.head.files),
                fmt_int(block.head.lines),
                fmt_int(block.head.code),
                fmt_int(block.head.comments),
                fmt_int(block.head.blanks),
                head_cost,
            )?;
            for m in &block.members {
                let m_cost = if show_cost {
                    let c = cost
                        .and_then(|c| c.per_language.get(&m.name))
                        .map_or(0, |e| e.cost_usd);
                    format!("<td>{}</td>", fmt_usd(c))
                } else {
                    String::new()
                };
                write!(
                    sink,
                    "<tr class=\"member\"><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td>{}</tr>",
                    escape(&m.name),
                    fmt_int(m.files),
                    fmt_int(m.lines),
                    fmt_int(m.code),
                    fmt_int(m.comments),
                    fmt_int(m.blanks),
                    m_cost,
                )?;
            }
        }
        write!(sink, "</tbody></table></section>")?;

        if let Some(c) = cost {
            write_cost_section(sink, c)?;
        }

        // Per-file with metric-bearing columns
        write!(
            sink,
            "<section><h2>Files</h2><table><thead><tr><th>Path</th><th>Lang</th><th>Lines</th><th>Code</th><th>Functions</th><th>CC max</th><th>Changes</th><th>Hotspot</th></tr></thead><tbody>"
        )?;
        let mut files: Vec<_> = report.files.iter().collect();
        files.sort_by(|a, b| {
            let sa = a.git.as_ref().map_or(0.0, |g| g.hotspot_score);
            let sb = b.git.as_ref().map_or(0.0, |g| g.hotspot_score);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
        for f in &files {
            let funcs = f
                .functions
                .map_or_else(|| muted_cell("—"), |n| cell(&fmt_int(n)));
            let cc = f
                .cyclomatic_max
                .map_or_else(|| muted_cell("—"), |n| cell(&fmt_int(n)));
            let changes = f
                .git
                .as_ref()
                .map_or_else(|| muted_cell("—"), |g| cell(&fmt_int(g.changes_in_window)));
            let hotspot = f.git.as_ref().map_or_else(
                || muted_cell("—"),
                |g| {
                    let cls = if g.hotspot_score > 50.0 { "hot" } else { "" };
                    format!("<td class=\"{}\">{:.1}</td>", cls, g.hotspot_score)
                },
            );
            write!(
                sink,
                "<tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td>{}{}{}{}</tr>",
                escape(&f.path.to_string_lossy()),
                escape(&f.language),
                fmt_int(f.lines),
                fmt_int(f.code),
                funcs,
                cc,
                changes,
                hotspot,
            )?;
        }
        write!(sink, "</tbody></table></section>")?;

        write!(
            sink,
            "<footer>Generated by <code>bscc</code> &mdash; schema v1</footer></body></html>"
        )?;
        Ok(())
    }
}

/// Mirrors the table-exporter block builder so HTML and table agree on
/// structure. `head` is the family total (or the standalone language);
/// `members` are the indented per-language rows when `is_family` is true.
struct LangRow {
    name: String,
    files: u32,
    lines: u32,
    code: u32,
    comments: u32,
    blanks: u32,
}

struct LangBlock {
    head: LangRow,
    members: Vec<LangRow>,
    is_family: bool,
}

fn lang_blocks(langs: &[bscc_core::LanguageTotal]) -> Vec<LangBlock> {
    let mut blocks: Vec<LangBlock> = Vec::new();
    let mut consumed: Vec<&str> = Vec::new();

    for fam in FAMILIES {
        let members: Vec<&bscc_core::LanguageTotal> = fam
            .members
            .iter()
            .filter_map(|m| langs.iter().find(|l| l.language == *m))
            .collect();
        if members.len() < 2 {
            continue;
        }
        let head = LangRow {
            name: fam.display.into(),
            files: members.iter().map(|m| m.files).sum(),
            lines: members.iter().map(|m| m.lines).sum(),
            code: members.iter().map(|m| m.code).sum(),
            comments: members.iter().map(|m| m.comments).sum(),
            blanks: members.iter().map(|m| m.blanks).sum(),
        };
        let mut indented: Vec<LangRow> = members
            .iter()
            .map(|m| LangRow {
                name: m.language.clone(),
                files: m.files,
                lines: m.lines,
                code: m.code,
                comments: m.comments,
                blanks: m.blanks,
            })
            .collect();
        indented.sort_by(|a, b| b.lines.cmp(&a.lines).then(a.name.cmp(&b.name)));
        for m in fam.members {
            consumed.push(m);
        }
        blocks.push(LangBlock {
            head,
            members: indented,
            is_family: true,
        });
    }

    for l in langs {
        if !consumed.iter().any(|c| *c == l.language) {
            blocks.push(LangBlock {
                head: LangRow {
                    name: l.language.clone(),
                    files: l.files,
                    lines: l.lines,
                    code: l.code,
                    comments: l.comments,
                    blanks: l.blanks,
                },
                members: Vec::new(),
                is_family: false,
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

fn write_cost_section(sink: &mut dyn Write, c: &CostReport) -> io::Result<()> {
    let p = &c.params;
    write!(
        sink,
        "<section><h2>Cost estimate</h2><p>basic COCOMO (<code>{}</code>, ${}/yr &times; {:.1} overhead)</p><div class=\"cost-cards\">",
        escape(&p.project_type),
        fmt_int(p.avg_wage),
        p.overhead,
    )?;
    write_cost_card(sink, "Baseline", &c.project.baseline, "")?;
    write_cost_card(
        sink,
        &format!("AI-assisted (&times;{:.1})", p.ai_multiplier),
        &c.project.ai_assisted,
        "ai",
    )?;
    write!(
        sink,
        "</div><p class=\"cost-note\">Per-language costs above sum lower than the project total; COCOMO scales as KLOC<sup>1.05</sup> so a unified codebase costs more than its parts. The card values are the headline numbers.</p></section>",
    )?;
    Ok(())
}

fn write_cost_card(
    sink: &mut dyn Write,
    title: &str,
    e: &bscc_core::Estimate,
    class: &str,
) -> io::Result<()> {
    write!(
        sink,
        "<div class=\"cost-card {class}\"><h3>{title}</h3><dl>\
         <dt>Effort</dt><dd>{:.1} person-months</dd>\
         <dt>Schedule</dt><dd>{:.1} months</dd>\
         <dt>People</dt><dd>{:.1}</dd>\
         <dt>Cost</dt><dd class=\"total\">{}</dd>\
         </dl></div>",
        e.effort_months,
        e.schedule_months,
        e.people,
        fmt_usd(e.cost_usd),
    )
}

fn cell(text: &str) -> String {
    format!("<td>{text}</td>")
}

fn muted_cell(text: &str) -> String {
    format!("<td class=\"muted\">{text}</td>")
}

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}
