//! Output formats for `bscc`. Table and JSON ship in M2; CSV/SARIF in M3;
//! HTML in M5.

pub mod json;
pub mod table;

pub use json::JsonExporter;
pub use table::TableExporter;
