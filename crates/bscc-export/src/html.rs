use crate::fmt::{FAMILIES, fmt_int};
use bscc_core::{Exporter, Report};
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
tr.family td { background: #f8fafc; font-style: italic; color: #555; border-top: 1px solid #ddd; }
code { font: 12.5px ui-monospace, SF Mono, Consolas, monospace; color: #4338ca; }
footer { margin-top: 3em; padding-top: 1em; border-top: 1px solid #eee; font-size: .85em; color: #999; }
";

impl Exporter for HtmlExporter {
    #[allow(clippy::too_many_lines)] // self-contained HTML builder, clearer inline
    fn write(&self, report: &Report, sink: &mut dyn Write) -> io::Result<()> {
        let total = report.grand_total();
        let by_lang = report.by_language();

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
        write!(
            sink,
            "<section><h2>Languages</h2><table><thead><tr><th>Language</th><th>Files</th><th>Lines</th><th>Code</th><th>Comments</th><th>Blanks</th></tr></thead><tbody>"
        )?;
        let mut langs: Vec<_> = by_lang.into_values().collect();
        langs.sort_by(|a, b| b.lines.cmp(&a.lines));
        for t in &langs {
            write!(
                sink,
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                escape(&t.language),
                fmt_int(t.files),
                fmt_int(t.lines),
                fmt_int(t.code),
                fmt_int(t.comments),
                fmt_int(t.blanks)
            )?;
        }
        // Family sub-totals (TS+TSX, JS+JSX, C+C++).
        for fam in FAMILIES {
            let members: Vec<_> = fam
                .members
                .iter()
                .filter_map(|m| langs.iter().find(|l| l.language == *m))
                .collect();
            if members.len() < 2 {
                continue;
            }
            let files: u32 = members.iter().map(|m| m.files).sum();
            let lines: u32 = members.iter().map(|m| m.lines).sum();
            let code: u32 = members.iter().map(|m| m.code).sum();
            let comments: u32 = members.iter().map(|m| m.comments).sum();
            let blanks: u32 = members.iter().map(|m| m.blanks).sum();
            write!(
                sink,
                "<tr class=\"family\"><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                escape(fam.display),
                fmt_int(files),
                fmt_int(lines),
                fmt_int(code),
                fmt_int(comments),
                fmt_int(blanks)
            )?;
        }
        write!(sink, "</tbody></table></section>")?;

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
