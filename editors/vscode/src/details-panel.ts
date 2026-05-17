import { spawn } from "node:child_process";
import * as path from "node:path";
import * as vscode from "vscode";

/**
 * One row parsed out of `bscc explain` text output.
 *
 * v1.1 will switch to `bscc explain --format json`; until then we
 * regex the table.
 */
interface FunctionDetail {
  startLine: number;
  endLine: number;
  cyclomatic: number;
  lines: number;
}

interface Summary {
  functions: number;
  cyclomaticTotal: number;
  cyclomaticMax: number;
}

interface ExplainResult {
  language: string;
  details: FunctionDetail[];
  summary?: Summary;
}

/** Per-instance cache so re-clicks reuse the same panel. */
const panels = new Map<string, vscode.WebviewPanel>();

export async function showDetails(
  bsccPath: string,
  fileUri: vscode.Uri,
): Promise<void> {
  const filePath = fileUri.fsPath;
  const existing = panels.get(filePath);
  if (existing) {
    existing.reveal();
    refresh(existing, bsccPath, fileUri).catch((e: unknown) => {
      console.error("bscc: refresh failed", e);
    });
    return;
  }

  const panel = vscode.window.createWebviewPanel(
    "bsccDetails",
    `bscc: ${path.basename(filePath)}`,
    vscode.ViewColumn.Beside,
    { enableScripts: false, retainContextWhenHidden: true },
  );
  panels.set(filePath, panel);
  panel.onDidDispose(() => panels.delete(filePath));

  await refresh(panel, bsccPath, fileUri);
}

async function refresh(
  panel: vscode.WebviewPanel,
  bsccPath: string,
  fileUri: vscode.Uri,
): Promise<void> {
  panel.webview.html = renderLoading(fileUri.fsPath);
  try {
    const result = await runExplain(bsccPath, fileUri.fsPath);
    panel.webview.html = renderResult(fileUri.fsPath, result);
  } catch (e) {
    panel.webview.html = renderError(fileUri.fsPath, e);
  }
}

function runExplain(
  bsccPath: string,
  filePath: string,
): Promise<ExplainResult> {
  return new Promise((resolve, reject) => {
    const child = spawn(bsccPath, ["explain", filePath]);
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk: Buffer) => {
      stdout += chunk.toString();
    });
    child.stderr.on("data", (chunk: Buffer) => {
      stderr += chunk.toString();
    });
    child.on("error", reject);
    child.on("close", (code) => {
      if (code !== 0) {
        reject(
          new Error(
            `bscc explain exited with code ${code}\n${stderr || stdout}`,
          ),
        );
        return;
      }
      try {
        resolve(parseExplainText(stdout));
      } catch (parseErr) {
        reject(parseErr);
      }
    });
  });
}

/**
 * Parse `bscc explain` text output. Shape:
 *
 *     path/to/foo.rs  (Rust)
 *     --------------------------------------------------
 *      start     end    cc  lines
 *         43      97    10  55
 *         67      67     1   1
 *     --------------------------------------------------
 *     functions=2  cyclomatic_total=11  cyclomatic_max=10
 */
export function parseExplainText(text: string): ExplainResult {
  const lines = text.split(/\r?\n/);
  const header = lines[0] ?? "";
  const langMatch = /\(([^()]+)\)\s*$/.exec(header);
  const language = langMatch ? langMatch[1] : "unknown";

  const details: FunctionDetail[] = [];
  // Data rows: four whitespace-separated integers. The column header
  // ("start end cc lines") has the word "lines" which fails the integer
  // test, so a single regex correctly filters it out.
  const rowRe = /^\s*(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s*$/;
  for (const line of lines) {
    const m = rowRe.exec(line);
    if (!m) continue;
    details.push({
      startLine: Number(m[1]),
      endLine: Number(m[2]),
      cyclomatic: Number(m[3]),
      lines: Number(m[4]),
    });
  }

  let summary: Summary | undefined;
  const summaryRe = /functions=(\d+)\s+cyclomatic_total=(\d+)\s+cyclomatic_max=(\d+)/;
  for (const line of lines) {
    const m = summaryRe.exec(line);
    if (m) {
      summary = {
        functions: Number(m[1]),
        cyclomaticTotal: Number(m[2]),
        cyclomaticMax: Number(m[3]),
      };
      break;
    }
  }

  return { language, details, summary };
}

function renderLoading(filePath: string): string {
  return wrap(`<p>Running <code>bscc explain ${escapeHtml(filePath)}</code>…</p>`);
}

function renderError(filePath: string, err: unknown): string {
  const msg = err instanceof Error ? err.message : String(err);
  return wrap(
    `<h2>bscc explain failed</h2>` +
      `<p><code>${escapeHtml(filePath)}</code></p>` +
      `<pre>${escapeHtml(msg)}</pre>`,
  );
}

function renderResult(filePath: string, r: ExplainResult): string {
  // Threshold thresholds for highlighting — keep in sync with the
  // bscc-lsp defaults; v1.1 will read these from bscc.toml.
  const ccMax = 10;
  const linesMax = 100;

  if (r.details.length === 0) {
    return wrap(
      `<h2>${escapeHtml(path.basename(filePath))}</h2>` +
        `<p class="muted">(${escapeHtml(r.language)}) — no functions detected.</p>`,
    );
  }

  const rows = r.details
    .map((d) => {
      const ccClass = d.cyclomatic > ccMax ? "hot" : "";
      const lenClass = d.lines > linesMax ? "hot" : "";
      return (
        `<tr>` +
        `<td><a href="#L${d.startLine}">L${d.startLine}</a></td>` +
        `<td>L${d.endLine}</td>` +
        `<td class="${ccClass}">${d.cyclomatic}</td>` +
        `<td class="${lenClass}">${d.lines}</td>` +
        `</tr>`
      );
    })
    .join("");

  const summary = r.summary
    ? `<p class="summary">${r.summary.functions} function(s) · ` +
      `total CC ${r.summary.cyclomaticTotal} · max CC ${r.summary.cyclomaticMax}</p>`
    : "";

  return wrap(
    `<h2>${escapeHtml(path.basename(filePath))}</h2>` +
      `<p class="muted">(${escapeHtml(r.language)})</p>` +
      summary +
      `<table>` +
      `<thead><tr><th>Start</th><th>End</th><th>CC</th><th>Lines</th></tr></thead>` +
      `<tbody>${rows}</tbody>` +
      `</table>` +
      `<p class="footnote">Rows above the configured thresholds (CC > ${ccMax}, lines > ${linesMax}) are highlighted. Adjust in <code>bscc.toml</code>.</p>`,
  );
}

function wrap(body: string): string {
  return `<!doctype html><html lang="en"><head><meta charset="utf-8"><style>
body { font: 13px/1.5 -apple-system, system-ui, sans-serif; color: var(--vscode-foreground); padding: 1em 1.25em; }
h2 { margin: 0 0 .25em; font-size: 1.1em; }
.muted { color: var(--vscode-descriptionForeground); margin: 0 0 1em; }
.summary { margin: 0 0 1em; font-weight: 600; }
table { border-collapse: collapse; width: 100%; font-variant-numeric: tabular-nums; }
th, td { padding: 4px 10px; text-align: right; border-bottom: 1px solid var(--vscode-panel-border); }
th:first-child, td:first-child { text-align: left; }
th { font-weight: 600; background: var(--vscode-editorWidget-background); }
td.hot { color: var(--vscode-errorForeground); font-weight: 600; }
a { color: var(--vscode-textLink-foreground); text-decoration: none; }
a:hover { text-decoration: underline; }
pre { background: var(--vscode-textCodeBlock-background); padding: .75em 1em; overflow-x: auto; }
code { font: 12px ui-monospace, Consolas, monospace; }
.footnote { margin-top: 1em; color: var(--vscode-descriptionForeground); font-size: .9em; }
</style></head><body>${body}</body></html>`;
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}
