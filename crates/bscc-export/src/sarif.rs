use bscc_core::{Exporter, Report};
use serde::Serialize;
use std::io::{self, Write};

/// Thresholds that cause a file to produce a SARIF result. Hardcoded defaults
/// here; M5 wires these up to `bscc.toml`.
#[derive(Debug, Clone, Copy)]
pub struct SarifThresholds {
    pub cyclomatic_max: u32,
    pub longest_function_lines: u32,
}

impl Default for SarifThresholds {
    fn default() -> Self {
        Self {
            cyclomatic_max: 10,
            longest_function_lines: 100,
        }
    }
}

#[derive(Default)]
pub struct SarifExporter {
    pub thresholds: SarifThresholds,
}

impl Exporter for SarifExporter {
    fn write(&self, report: &Report, sink: &mut dyn Write) -> io::Result<()> {
        let mut results = Vec::new();
        for f in &report.files {
            if let Some(cc) = f.cyclomatic_max
                && cc > self.thresholds.cyclomatic_max
            {
                results.push(SarifResult {
                    rule_id: "complexity/cyclomatic",
                    level: "warning",
                    message: SarifMessage {
                        text: format!(
                            "Cyclomatic complexity {cc} exceeds threshold {}",
                            self.thresholds.cyclomatic_max
                        ),
                    },
                    locations: vec![file_location(&f.path)],
                });
            }
            if let Some(len) = f.longest_function_lines
                && len > self.thresholds.longest_function_lines
            {
                results.push(SarifResult {
                    rule_id: "size/longest-function",
                    level: "warning",
                    message: SarifMessage {
                        text: format!(
                            "Longest function {len} lines exceeds threshold {}",
                            self.thresholds.longest_function_lines
                        ),
                    },
                    locations: vec![file_location(&f.path)],
                });
            }
        }

        let log = SarifLog {
            version: "2.1.0",
            schema: "https://json.schemastore.org/sarif-2.1.0.json",
            runs: vec![SarifRun {
                tool: SarifTool {
                    driver: SarifDriver {
                        name: "bscc",
                        information_uri: "https://github.com/i-doll/bscc",
                        rules: vec![
                            SarifRule {
                                id: "complexity/cyclomatic",
                                name: "CyclomaticComplexity",
                                short_description: SarifMessage {
                                    text: "Function cyclomatic complexity exceeds threshold".into(),
                                },
                            },
                            SarifRule {
                                id: "size/longest-function",
                                name: "LongestFunctionLines",
                                short_description: SarifMessage {
                                    text: "Function length exceeds threshold".into(),
                                },
                            },
                        ],
                    },
                },
                results,
            }],
        };

        let bytes = serde_json::to_vec_pretty(&log).map_err(io::Error::other)?;
        sink.write_all(&bytes)?;
        sink.write_all(b"\n")
    }
}

fn file_location(path: &std::path::Path) -> SarifLocation {
    SarifLocation {
        physical_location: SarifPhysicalLocation {
            artifact_location: SarifArtifactLocation {
                uri: path.to_string_lossy().into_owned(),
            },
        },
    }
}

#[derive(Serialize)]
struct SarifLog {
    version: &'static str,
    #[serde(rename = "$schema")]
    schema: &'static str,
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
}

#[derive(Serialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Serialize)]
struct SarifDriver {
    name: &'static str,
    #[serde(rename = "informationUri")]
    information_uri: &'static str,
    rules: Vec<SarifRule>,
}

#[derive(Serialize)]
struct SarifRule {
    id: &'static str,
    name: &'static str,
    #[serde(rename = "shortDescription")]
    short_description: SarifMessage,
}

#[derive(Serialize)]
struct SarifResult {
    #[serde(rename = "ruleId")]
    rule_id: &'static str,
    level: &'static str,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
}

#[derive(Serialize)]
struct SarifMessage {
    text: String,
}

#[derive(Serialize)]
struct SarifLocation {
    #[serde(rename = "physicalLocation")]
    physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
struct SarifPhysicalLocation {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation,
}

#[derive(Serialize)]
struct SarifArtifactLocation {
    uri: String,
}
