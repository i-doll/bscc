use serde::Serialize;
use std::path::PathBuf;

/// Per-file metrics. Regex-tier analyzers fill the line counts and leave the
/// tree-sitter-only fields as `None`. Tree-sitter-tier analyzers fill both.
#[derive(Debug, Clone, Serialize)]
pub struct FileMetrics {
    pub path: PathBuf,
    pub language: String,
    pub bytes: u64,
    pub lines: u32,
    pub code: u32,
    pub comments: u32,
    pub blanks: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cyclomatic_total: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cyclomatic_max: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cognitive: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_nesting_depth: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longest_function_lines: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub todo_comments: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imports: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<GitMetrics>,
}

/// Per-file git metrics. Set by `bscc-git` when enriching a `Report`; left
/// `None` for non-repo paths or when `--no-git` is passed.
#[derive(Debug, Clone, Serialize)]
pub struct GitMetrics {
    pub changes_in_window: u32,
    pub authors_count: u32,
    /// Unix timestamp of the most recent commit touching this file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<i64>,
    /// `complexity × ln(1 + changes_in_window)`, where complexity is
    /// `cyclomatic_total` if available else `code` LOC.
    pub hotspot_score: f64,
}

/// Per-function breakdown produced by `Analyzer::explain` (tree-sitter tier
/// only). Returned by the `bscc explain` subcommand.
#[derive(Debug, Clone, Serialize)]
pub struct FunctionDetail {
    /// 1-based line number where the function header starts.
    pub start_line: u32,
    /// 1-based line number where the function ends.
    pub end_line: u32,
    pub lines: u32,
    pub cyclomatic: u32,
}

impl FileMetrics {
    pub fn new(path: PathBuf, language: String, bytes: u64) -> Self {
        Self {
            path,
            language,
            bytes,
            lines: 0,
            code: 0,
            comments: 0,
            blanks: 0,
            functions: None,
            cyclomatic_total: None,
            cyclomatic_max: None,
            cognitive: None,
            max_nesting_depth: None,
            longest_function_lines: None,
            todo_comments: None,
            imports: None,
            git: None,
        }
    }
}
