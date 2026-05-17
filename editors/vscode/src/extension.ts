import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";
import { resolveBscc, showMissingBinaryError } from "./bin";
import { createClient } from "./server";
import { showDetails } from "./details-panel";

let client: LanguageClient | undefined;
let bsccPath: string | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const cfg = vscode.workspace.getConfiguration("bscc");
  if (cfg.get<boolean>("enable") === false) {
    return;
  }

  bsccPath = await resolveBscc();
  if (!bsccPath) {
    await showMissingBinaryError();
    return;
  }

  await startClient();

  // Lens command — the server emits `bscc.noop` for every function;
  // we hijack that to open the per-file details panel.
  context.subscriptions.push(
    vscode.commands.registerCommand("bscc.noop", async () => {
      const uri = vscode.window.activeTextEditor?.document.uri;
      if (!uri || !bsccPath) return;
      await showDetails(bsccPath, uri);
    }),
    vscode.commands.registerCommand("bscc.showDetails", async () => {
      const uri = vscode.window.activeTextEditor?.document.uri;
      if (!uri || !bsccPath) return;
      await showDetails(bsccPath, uri);
    }),
    vscode.commands.registerCommand("bscc.restart", async () => {
      await restartClient();
    }),
  );

  // The server reads `bscc.toml` once at startup; restart on changes so
  // threshold tweaks take effect without a manual reload.
  const watcher = vscode.workspace.createFileSystemWatcher("**/bscc.toml");
  context.subscriptions.push(
    watcher,
    watcher.onDidChange(() => restartClient()),
    watcher.onDidCreate(() => restartClient()),
    watcher.onDidDelete(() => restartClient()),
  );

  // Live-update the binary path if the user changes `bscc.path`.
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration(async (e) => {
      if (e.affectsConfiguration("bscc.path") || e.affectsConfiguration("bscc.enable")) {
        await restartClient();
      }
    }),
  );
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
    client = undefined;
  }
}

async function startClient(): Promise<void> {
  if (!bsccPath) return;
  client = createClient(bsccPath);
  await client.start();
}

async function restartClient(): Promise<void> {
  if (client) {
    await client.stop();
    client = undefined;
  }
  bsccPath = await resolveBscc();
  const enabled =
    vscode.workspace.getConfiguration("bscc").get<boolean>("enable") !== false;
  if (!enabled) return;
  if (!bsccPath) {
    await showMissingBinaryError();
    return;
  }
  await startClient();
}
