import * as fs from "node:fs/promises";
import * as path from "node:path";
import * as vscode from "vscode";

/**
 * Resolve the path to the `bscc` binary.
 *
 * Precedence:
 *   1. The `bscc.path` workspace/user setting (when non-empty).
 *   2. The first `bscc` (or `bscc.exe` on Windows) found on `PATH`.
 *
 * Returns `undefined` if no candidate exists; callers should surface a
 * user-actionable error in that case.
 */
export async function resolveBscc(): Promise<string | undefined> {
  const cfg = vscode.workspace.getConfiguration("bscc");
  const explicit = (cfg.get<string>("path") ?? "").trim();
  if (explicit) {
    return (await fileExists(explicit)) ? explicit : undefined;
  }

  const exe = process.platform === "win32" ? "bscc.exe" : "bscc";
  for (const dir of (process.env.PATH ?? "").split(path.delimiter)) {
    if (!dir) continue;
    const candidate = path.join(dir, exe);
    if (await fileExists(candidate)) return candidate;
  }
  return undefined;
}

/**
 * Show an error notification with actionable next steps when the binary
 * cannot be resolved. The notification offers a settings-jump and a
 * link to the project's install instructions.
 */
export async function showMissingBinaryError(): Promise<void> {
  const settings = "Open settings";
  const install = "Install instructions";
  const choice = await vscode.window.showErrorMessage(
    "bscc binary not found. Install bscc and either add it to your PATH or set `bscc.path` in settings.",
    settings,
    install,
  );
  if (choice === settings) {
    await vscode.commands.executeCommand(
      "workbench.action.openSettings",
      "bscc.path",
    );
  } else if (choice === install) {
    await vscode.env.openExternal(
      vscode.Uri.parse("https://github.com/i-doll/bscc#install"),
    );
  }
}

async function fileExists(p: string): Promise<boolean> {
  try {
    const st = await fs.stat(p);
    return st.isFile();
  } catch {
    return false;
  }
}
