//! Output formats for `bscc`. M1 ships the table exporter only; JSON, CSV,
//! SARIF, and HTML land in later milestones.

pub mod table;

pub use table::TableExporter;
