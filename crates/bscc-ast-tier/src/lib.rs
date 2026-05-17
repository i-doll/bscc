// File sizes (bytes, row/column positions, capture counts) all fit in usize.
// The `as u32` casts here clamp metric counts that can't realistically exceed
// u32::MAX, so saturation is acceptable.
#![allow(clippy::cast_possible_truncation)]

//! Generic tree-sitter analyzer. Per-language crates own the grammar and the
//! `metrics.scm` query; this crate runs the query and turns captures into
//! `FileMetrics`. Capture names recognized:
//!
//! - `@function` — function-like declaration. One per function. Each contributes
//!   `+1` to `functions`, defines a byte range for cyclomatic attribution,
//!   and updates `longest_function_lines`.
//! - `@branch` — control-flow node that adds `+1` to cyclomatic complexity.
//! - `@import` — import/use/include declaration.
//! - `@comment` — comment node. Used for line classification *and* for
//!   `todo_comments` (substring scan inside the captured text).
//!
//! Captures with other names are ignored, so per-language queries can include
//! helper captures freely.

use bscc_core::{Analyzer, FileMetrics};
use std::path::Path;
use thiserror::Error;
use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator};

#[derive(Debug, Error)]
pub enum AstError {
    #[error("invalid tree-sitter query: {0}")]
    Query(#[from] tree_sitter::QueryError),
    #[error("incompatible tree-sitter language")]
    Language,
}

/// One-time setup: parse + compile the query for a given language. Cheap to
/// clone via `Arc<TreeSitterAnalyzer>` in the registry.
pub struct TreeSitterAnalyzer {
    name: String,
    language: Language,
    query: Query,
    cap_function: Option<u32>,
    cap_branch: Option<u32>,
    cap_import: Option<u32>,
    cap_comment: Option<u32>,
}

impl TreeSitterAnalyzer {
    pub fn new(name: String, language: Language, query_src: &str) -> Result<Self, AstError> {
        let query = Query::new(&language, query_src)?;
        let lookup = |n: &str| query.capture_index_for_name(n);
        Ok(Self {
            name,
            language,
            cap_function: lookup("function"),
            cap_branch: lookup("branch"),
            cap_import: lookup("import"),
            cap_comment: lookup("comment"),
            query,
        })
    }
}

impl Analyzer for TreeSitterAnalyzer {
    fn analyze(&self, path: &Path, source: &[u8]) -> FileMetrics {
        let mut m = FileMetrics::new(path.to_path_buf(), self.name.clone(), source.len() as u64);

        let mut parser = Parser::new();
        if parser.set_language(&self.language).is_err() {
            // Should not happen — Query::new would have failed first.
            return m;
        }
        let Some(tree) = parser.parse(source, None) else {
            return m;
        };

        let mut function_ranges: Vec<(usize, usize, u32, u32)> = Vec::new();
        let mut branches: Vec<usize> = Vec::new();
        let mut comment_ranges: Vec<(usize, usize)> = Vec::new();
        let mut imports: u32 = 0;
        let mut todo_comments: u32 = 0;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.query, tree.root_node(), source);
        while let Some(mat) = matches.next() {
            for cap in mat.captures {
                let idx = cap.index;
                let node = cap.node;
                if Some(idx) == self.cap_function {
                    let sp = node.start_position();
                    let ep = node.end_position();
                    function_ranges.push((
                        node.start_byte(),
                        node.end_byte(),
                        sp.row as u32,
                        ep.row as u32,
                    ));
                } else if Some(idx) == self.cap_branch {
                    branches.push(node.start_byte());
                } else if Some(idx) == self.cap_import {
                    imports += 1;
                } else if Some(idx) == self.cap_comment {
                    comment_ranges.push((node.start_byte(), node.end_byte()));
                    if contains_todo(&source[node.start_byte()..node.end_byte()]) {
                        todo_comments += 1;
                    }
                }
            }
        }

        let lc = classify_lines(source, &mut comment_ranges);
        m.lines = lc.lines;
        m.code = lc.code;
        m.comments = lc.comments;
        m.blanks = lc.blanks;

        let mut cyclomatic_total = 0u32;
        let mut cyclomatic_max = 0u32;
        let mut longest_function_lines = 0u32;
        // Branches must be sorted to attribute via binary search; functions are
        // attributed in input order.
        branches.sort_unstable();
        for &(start, end, srow, erow) in &function_ranges {
            let lo = branches.partition_point(|&b| b < start);
            let hi = branches.partition_point(|&b| b < end);
            let cc = 1 + (hi - lo) as u32;
            cyclomatic_total += cc;
            if cc > cyclomatic_max {
                cyclomatic_max = cc;
            }
            let lines = erow.saturating_sub(srow) + 1;
            if lines > longest_function_lines {
                longest_function_lines = lines;
            }
        }

        m.functions = Some(function_ranges.len() as u32);
        m.cyclomatic_total = Some(cyclomatic_total);
        m.cyclomatic_max = Some(cyclomatic_max);
        m.longest_function_lines = Some(longest_function_lines);
        m.imports = Some(imports);
        m.todo_comments = Some(todo_comments);
        m
    }
}

struct LineCounts {
    lines: u32,
    code: u32,
    comments: u32,
    blanks: u32,
}

/// Walk the source once, attributing each line to code, comment, or blank.
/// A line is `code` if any non-whitespace byte lies outside every comment
/// range; otherwise `comment` if any non-whitespace byte lies inside a
/// comment; otherwise `blank`.
fn classify_lines(source: &[u8], comment_ranges: &mut [(usize, usize)]) -> LineCounts {
    comment_ranges.sort_unstable_by_key(|&(s, _)| s);

    let mut lines = 0u32;
    let mut code = 0u32;
    let mut comments = 0u32;
    let mut blanks = 0u32;

    let mut line_has_code = false;
    let mut line_has_comment = false;
    let mut comment_idx = 0usize;

    for (i, &b) in source.iter().enumerate() {
        if b == b'\n' {
            lines += 1;
            if line_has_code {
                code += 1;
            } else if line_has_comment {
                comments += 1;
            } else {
                blanks += 1;
            }
            line_has_code = false;
            line_has_comment = false;
            continue;
        }
        if b.is_ascii_whitespace() {
            continue;
        }
        // Skip comments that ended before i.
        while comment_idx < comment_ranges.len() && comment_ranges[comment_idx].1 <= i {
            comment_idx += 1;
        }
        let in_comment = comment_ranges
            .get(comment_idx)
            .is_some_and(|&(s, e)| i >= s && i < e);
        if in_comment {
            line_has_comment = true;
        } else {
            line_has_code = true;
        }
    }

    if !source.is_empty() && source[source.len() - 1] != b'\n' {
        lines += 1;
        if line_has_code {
            code += 1;
        } else if line_has_comment {
            comments += 1;
        } else {
            blanks += 1;
        }
    }

    LineCounts {
        lines,
        code,
        comments,
        blanks,
    }
}

fn contains_todo(text: &[u8]) -> bool {
    const NEEDLES: [&[u8]; 4] = [b"TODO", b"FIXME", b"XXX", b"HACK"];
    NEEDLES
        .iter()
        .any(|n| memchr::memmem::find(text, n).is_some())
}
