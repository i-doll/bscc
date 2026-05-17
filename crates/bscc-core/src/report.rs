use crate::FileMetrics;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Serialize)]
pub struct Report {
    pub files: Vec<FileMetrics>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct LanguageTotal {
    pub language: String,
    pub files: u32,
    pub lines: u32,
    pub code: u32,
    pub comments: u32,
    pub blanks: u32,
    pub bytes: u64,
}

impl Report {
    pub fn push(&mut self, m: FileMetrics) {
        self.files.push(m);
    }

    pub fn by_language(&self) -> BTreeMap<String, LanguageTotal> {
        let mut totals: BTreeMap<String, LanguageTotal> = BTreeMap::new();
        for f in &self.files {
            let t = totals.entry(f.language.clone()).or_default();
            if t.language.is_empty() {
                t.language.clone_from(&f.language);
            }
            t.files += 1;
            t.lines += f.lines;
            t.code += f.code;
            t.comments += f.comments;
            t.blanks += f.blanks;
            t.bytes += f.bytes;
        }
        totals
    }

    pub fn grand_total(&self) -> LanguageTotal {
        let mut t = LanguageTotal {
            language: "Total".into(),
            ..LanguageTotal::default()
        };
        for f in &self.files {
            t.files += 1;
            t.lines += f.lines;
            t.code += f.code;
            t.comments += f.comments;
            t.blanks += f.blanks;
            t.bytes += f.bytes;
        }
        t
    }
}
