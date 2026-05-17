# bscc — "Better scc"

A from-scratch code-metrics tool inspired by [scc]. Two-tier engine:
tree-sitter where grammars are available (Rust, Python, TypeScript / TSX,
Go, C, C++, Java in v1), declarative regex tokenizer for everything else
— 360+ languages, ported from scc's `languages.json` plus bscc-specific
additions (LSL, JSX, Git/Editor Config).

Beyond LOC counting, `bscc` produces:

- **Structural complexity** — functions, cyclomatic (total + max),
  longest function, todo comments, imports — via tree-sitter
- **Git hotspots** — per-file churn, authors, last-modified, and a
  `complexity × ln(1 + changes)` hotspot score
- **Exporters** — colored table, JSON (versioned schema), CSV, SARIF
  (for CI), self-contained HTML
- **LSP server** (`bscc lsp` / `bscc-lsp`) — diagnostics and code lenses
  for functions exceeding configured thresholds

## Build

This repo uses [mise] to pin the Rust toolchain and provide task aliases.

```sh
mise install
mise run test
cargo build --release
./target/release/bscc count .
```

Without mise: `rust-toolchain.toml` pins the same Rust version for `rustup`.

## Usage

```
bscc count    [paths…]   [--format table|json|csv|sarif|html] [--no-gitignore] [--hidden]
bscc hotspots [paths…]   [--top N] [--window-days N]
bscc explain  <file>      per-function breakdown (tree-sitter tier only)
bscc languages            list registered languages and which tier they use
bscc lsp                  run the LSP server over stdio (same as bscc-lsp binary)
```

### Examples

```sh
# scc-style summary table
bscc count .

# Per-file rows, JSON for downstream tools
bscc count --format json . > report.json

# Self-contained HTML report
bscc count --format html . > report.html

# Hotspots in the current git repo, top 20
bscc hotspots --top 20 .

# Per-function complexity breakdown for one file
bscc explain crates/bscc-core/src/walk.rs

# SARIF for CI (uses thresholds from bscc.toml or defaults)
bscc count --format sarif . > bscc.sarif

# Run the LSP server (configure your editor to point at this command)
bscc lsp
```

## Configuration

`bscc` searches from the working directory upward for a `bscc.toml` and
applies it on top of defaults. Example:

```toml
[thresholds]
cyclomatic_max = 10
longest_function_lines = 100

[git]
window_days = 90
```

Thresholds drive the SARIF exporter and the LSP server's diagnostics.
The git window applies to `bscc hotspots`.

## Languages

`bscc languages` lists everything registered at startup. The default
binary ships 8 tree-sitter languages (Rust, Python, TypeScript, TSX, Go,
C, C++, Java) and 360+ regex-tier languages ported from scc plus
bscc-specific entries (LSL, JSX, Git/Editor Config) — see
[`crates/bscc-regex-tier/data/languages.toml`]. The regex-tier file is
auto-converted from [scc's `languages.json`][scc-langs] so file-type
coverage tracks scc's broad inventory.

### LSL

LSL ([Linden Scripting Language] — Second Life) is handled by the regex
tier in v1. A dedicated `tree-sitter-lsl` grammar + `bscc-lang-lsl`
plugin is the natural v1.1 — it would move LSL into the tree-sitter tier
so complexity/hotspot data becomes available for it.

## LSP setup

`bscc lsp` (or the standalone `bscc-lsp` binary) speaks LSP over stdio.
It publishes diagnostics for functions whose cyclomatic complexity or
length exceed the configured thresholds, and emits code lenses with
per-function `CC=N · M lines` annotations.

For VS Code or any LSP client, wire the command up the same way you
would any other server. The repo doesn't ship an extension.

## Architecture

```
crates/
├── bscc-core            # engine + Registry + FileMetrics/Report + traits
├── bscc-regex-tier      # scc-style declarative tokenizer + ~60 lang configs
├── bscc-ast-tier        # generic tree-sitter analyzer driven by metrics.scm
├── bscc-lang-rust       \
├── bscc-lang-python      \
├── bscc-lang-typescript   |  per-language plugins: grammar + metrics.scm
├── bscc-lang-go          /   + register()
├── bscc-lang-c          /
├── bscc-lang-cpp       /
├── bscc-lang-java     /
├── bscc-git             # git log → per-file churn/authors/hotspot_score
├── bscc-export          # table, json, csv, sarif, html exporters
├── bscc-cli             # binary: subcommands wire it all together
└── bscc-lsp             # binary: LSP server for editor integration
```

Tree-sitter language plugins each declare a `queries/metrics.scm` file
that names `@function`, `@branch`, `@import`, and `@comment` captures.
`bscc-ast-tier` runs the query generically and computes metrics — no
per-language analysis code lives in the engine.

`bscc-regex-tier` provides a byte-level state-machine tokenizer driven
by per-language comment + string delimiters in `data/languages.toml`.
Languages without a tree-sitter plugin use this fallback, so coverage
breadth doesn't suffer when grammars aren't available.

## Status

v1 implemented across milestones M1–M5 (see
[`docs/superpowers/specs/2026-05-17-bscc-design.md`]).

Known limitations:

- Regex tier does not support nested block comments (Haskell, OCaml may
  slightly miscount in deeply nested cases).
- `bscc-git` shells out to `git`; pure-Rust `gix` migration is a
  follow-up.
- LSP server is FULL-sync only; large files re-parse on every change.
- No cognitive complexity (Sonar-style) yet — cyclomatic is the
  primary complexity metric.

[scc]: https://github.com/boyter/scc
[scc-langs]: https://github.com/boyter/scc/blob/master/languages.json
[mise]: https://mise.jdx.dev/
[`crates/bscc-regex-tier/data/languages.toml`]: crates/bscc-regex-tier/data/languages.toml
[`docs/superpowers/specs/2026-05-17-bscc-design.md`]: docs/superpowers/specs/2026-05-17-bscc-design.md
[Linden Scripting Language]: https://wiki.secondlife.com/wiki/LSL_Portal
