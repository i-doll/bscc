import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

const LANGUAGE_IDS = [
  "rust",
  "python",
  "typescript",
  "typescriptreact",
  "go",
  "c",
  "cpp",
  "java",
  "hcl",
  "terraform",
  "lsl",
];

/**
 * Build a `LanguageClient` that spawns `<bsccPath> lsp` as a stdio
 * subprocess and forwards LSP traffic for the languages bscc analyzes
 * at the tree-sitter tier.
 *
 * The caller owns the client lifecycle (start / stop / restart) and
 * its disposal — this function just constructs.
 */
export function createClient(bsccPath: string): LanguageClient {
  const serverOptions: ServerOptions = {
    command: bsccPath,
    args: ["lsp"],
    transport: TransportKind.stdio,
    options: {
      // Let `bscc lsp` discover `bscc.toml` upward from the active
      // workspace folder (falls back to the extension host cwd when
      // none is open, which is fine for ad-hoc files).
      cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath,
    },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: LANGUAGE_IDS.map((language) => ({
      scheme: "file",
      language,
    })),
    outputChannelName: "bscc",
    // bscc.toml is the source of truth for thresholds; watch it so the
    // *server* notices changes even before our higher-level restart
    // logic kicks in (defense in depth).
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher("**/bscc.toml"),
    },
  };

  return new LanguageClient(
    "bscc",
    "bscc Language Server",
    serverOptions,
    clientOptions,
  );
}
