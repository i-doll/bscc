//! LSP server for `bscc`. Surfaces per-function metrics as diagnostics
//! (when thresholds exceeded) and as code lenses (always, informational).
//!
//! Only tree-sitter-tier languages produce diagnostics/lenses; regex-tier
//! files are ignored (no per-function data available).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CodeLens, CodeLensOptions, CodeLensParams, Command, Diagnostic, DiagnosticSeverity,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, InitializeResult, InitializedParams, MessageType, Position, Range,
    ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug, Clone, Copy)]
pub struct Thresholds {
    pub cyclomatic_max: u32,
    pub longest_function_lines: u32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            cyclomatic_max: 10,
            longest_function_lines: 100,
        }
    }
}

pub struct Backend {
    client: Client,
    registry: Arc<bscc_core::Registry>,
    sources: Mutex<HashMap<Url, String>>,
    thresholds: Thresholds,
}

impl Backend {
    fn path_from_uri(uri: &Url) -> Option<PathBuf> {
        uri.to_file_path().ok()
    }

    async fn analyze(&self, uri: &Url, source: &str) {
        let Some(path) = Self::path_from_uri(uri) else {
            return;
        };
        let Some(entry) = self.registry.lookup_by_path(&path) else {
            // Clear any prior diagnostics for unknown languages.
            self.client
                .publish_diagnostics(uri.clone(), vec![], None)
                .await;
            return;
        };
        let Some(details) = entry.analyzer.explain(&path, source.as_bytes()) else {
            self.client
                .publish_diagnostics(uri.clone(), vec![], None)
                .await;
            return;
        };
        let diagnostics: Vec<Diagnostic> = details
            .iter()
            .filter_map(|d| {
                let cc_over = d.cyclomatic > self.thresholds.cyclomatic_max;
                let len_over = d.lines > self.thresholds.longest_function_lines;
                if !cc_over && !len_over {
                    return None;
                }
                let reason = match (cc_over, len_over) {
                    (true, true) => format!(
                        "CC {} > {} and length {} > {}",
                        d.cyclomatic,
                        self.thresholds.cyclomatic_max,
                        d.lines,
                        self.thresholds.longest_function_lines
                    ),
                    (true, false) => format!(
                        "CC {} > threshold {}",
                        d.cyclomatic, self.thresholds.cyclomatic_max
                    ),
                    (false, true) => format!(
                        "{} lines > threshold {}",
                        d.lines, self.thresholds.longest_function_lines
                    ),
                    (false, false) => unreachable!(),
                };
                Some(Diagnostic {
                    range: Range {
                        start: Position {
                            line: d.start_line.saturating_sub(1),
                            character: 0,
                        },
                        end: Position {
                            line: d.end_line.saturating_sub(1),
                            character: 0,
                        },
                    },
                    severity: Some(DiagnosticSeverity::WARNING),
                    source: Some("bscc".into()),
                    message: reason,
                    ..Diagnostic::default()
                })
            })
            .collect();
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(false),
                }),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "bscc-lsp".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "bscc-lsp ready")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text.clone();
        self.sources.lock().await.insert(uri.clone(), text.clone());
        self.analyze(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // FULL sync — last change has the entire document.
        if let Some(change) = params.content_changes.into_iter().last() {
            let uri = params.text_document.uri.clone();
            self.sources
                .lock()
                .await
                .insert(uri.clone(), change.text.clone());
            self.analyze(&uri, &change.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.sources.lock().await.remove(&params.text_document.uri);
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        let uri = params.text_document.uri;
        let source = self.sources.lock().await.get(&uri).cloned();
        let Some(source) = source else {
            return Ok(None);
        };
        let Some(path) = Self::path_from_uri(&uri) else {
            return Ok(Some(vec![]));
        };
        let Some(entry) = self.registry.lookup_by_path(&path) else {
            return Ok(Some(vec![]));
        };
        let Some(details) = entry.analyzer.explain(&path, source.as_bytes()) else {
            return Ok(Some(vec![]));
        };

        let lenses = details
            .into_iter()
            .map(|d| CodeLens {
                range: Range {
                    start: Position {
                        line: d.start_line.saturating_sub(1),
                        character: 0,
                    },
                    end: Position {
                        line: d.start_line.saturating_sub(1),
                        character: 0,
                    },
                },
                command: Some(Command {
                    title: format!("bscc: CC={} · {} lines", d.cyclomatic, d.lines),
                    command: "bscc.noop".into(),
                    arguments: None,
                }),
                data: None,
            })
            .collect();
        Ok(Some(lenses))
    }
}

/// Run the LSP server on stdin/stdout. Blocks until the client disconnects.
pub fn run(registry: bscc_core::Registry, thresholds: Thresholds) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let (service, socket) = LspService::new(|client| Backend {
            client,
            registry: Arc::new(registry),
            sources: Mutex::new(HashMap::new()),
            thresholds,
        });
        Server::new(stdin, stdout, socket).serve(service).await;
        Ok::<_, anyhow::Error>(())
    })
}
