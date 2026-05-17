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
        }
    }
}
