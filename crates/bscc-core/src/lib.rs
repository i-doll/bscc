//! Engine for `bscc`: file walking, language detection via a `Registry`,
//! two-tier dispatch (regex tier vs tree-sitter tier), and aggregation
//! into a `Report` that exporters consume.

pub mod detect;
pub mod exporter;
pub mod metrics;
pub mod registry;
pub mod report;
pub mod walk;

pub use exporter::Exporter;
pub use metrics::{FileMetrics, FunctionDetail, GitMetrics};
pub use registry::{Analyzer, LanguageEntry, Registry, Tier};
pub use report::{LanguageTotal, Report};
pub use walk::{WalkOptions, walk};
