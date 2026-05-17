import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";
import { resolveBscc, showMissingBinaryError } from "./bin";
import { createClient } from "./server";
import { showDetails } from "./details-panel";

let client: LanguageClient | undefined;
let bsccPath: string | undefined;
let log: vscode.OutputChannel | undefined;
let status: vscode.StatusBarItem | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  log = vscode.window.createOutputChannel("bscc (extension)");
  context.subscriptions.push(log);
  log.appendLine(`bscc extension activating · platform=${process.platform} · pid=${process.pid}`);

  status = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
  status.command = "bscc.showOutput";
  context.subscriptions.push(status);
  setStatus("starting…");

  // Register commands FIRST, unconditionally, so the palette works even
  // when the binary is missing or the server fails to start. The Restart
  // command is the recovery path users need when fixing PATH problems.
  context.subscriptions.push(
    vscode.commands.registerCommand("bscc.noop", () => onLensClick()),
    vscode.commands.registerCommand("bscc.showDetails", () => onLensClick()),
    vscode.commands.registerCommand("bscc.restart", () => restartClient()),
    vscode.commands.registerCommand("bscc.showOutput", () => log?.show(true)),
  );

  // bscc.toml watcher — server reads thresholds at startup only, so a
  // config edit needs a server restart to take effect.
  const watcher = vscode.workspace.createFileSystemWatcher("**/bscc.toml");
  context.subscriptions.push(
    watcher,
    watcher.onDidChange(() => restartClient()),
    watcher.onDidCreate(() => restartClient()),
    watcher.onDidDelete(() => restartClient()),
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration("bscc.path") || e.affectsConfiguration("bscc.enable")) {
        void restartClient();
      }
    }),
  );

  await startClient();
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
    client = undefined;
  }
}

async function onLensClick(): Promise<void> {
  const uri = vscode.window.activeTextEditor?.document.uri;
  if (!uri) {
    log?.appendLine("lens click ignored: no active editor");
    return;
  }
  if (!bsccPath) {
    await showMissingBinaryError();
    return;
  }
  await showDetails(bsccPath, uri);
}

async function startClient(): Promise<void> {
  const enabled =
    vscode.workspace.getConfiguration("bscc").get<boolean>("enable") !== false;
  if (!enabled) {
    log?.appendLine("bscc.enable=false — server not started");
    setStatus("disabled");
    return;
  }

  bsccPath = await resolveBscc();
  if (!bsccPath) {
    log?.appendLine(`bscc binary not found on PATH or in bscc.path setting`);
    log?.appendLine(`  PATH = ${process.env.PATH ?? "(unset)"}`);
    setStatus("binary not found", true);
    await showMissingBinaryError();
    return;
  }
  log?.appendLine(`bscc binary: ${bsccPath}`);

  try {
    client = createClient(bsccPath);
    await client.start();
    log?.appendLine("language client started");
    setStatus("ready");
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    log?.appendLine(`failed to start language client: ${msg}`);
    if (e instanceof Error && e.stack) log?.appendLine(e.stack);
    setStatus("server failed", true);
    void vscode.window.showErrorMessage(
      `bscc: language server failed to start — ${msg}. See "bscc (extension)" output channel.`,
    );
  }
}

async function restartClient(): Promise<void> {
  log?.appendLine("restart requested");
  if (client) {
    try {
      await client.stop();
    } catch (e) {
      log?.appendLine(`error stopping client: ${e instanceof Error ? e.message : String(e)}`);
    }
    client = undefined;
  }
  await startClient();
}

function setStatus(text: string, warn = false): void {
  if (!status) return;
  status.text = `$(pulse) bscc: ${text}`;
  status.tooltip = "Click to show the bscc extension log";
  status.backgroundColor = warn
    ? new vscode.ThemeColor("statusBarItem.warningBackground")
    : undefined;
  status.show();
}
