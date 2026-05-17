# bscc — "Better scc"

## Context

`scc` is a fast, single-binary line-of-code counter with ~250 languages defined declaratively in one JSON file. It works well as a counter but doesn't go beyond LOC, doesn't integrate with git, has no LSP surface, and its regex-based tokenizer can't express languages that need real parsing.

`bscc` is a from-scratch Rust tool that keeps scc's "single-binary, fast walker over a source tree" feel but adds four things scc lacks:

1. A tree-sitter-driven tier for **structural complexity metrics** (functions, cyclomatic, cognitive, nesting, longest function, todo comments, imports).
2. **Git churn integration** for hotspot detection (`complexity × log(changes)`).
3. **Multiple exporters** (table, JSON, CSV, SARIF, self-contained HTML).
4. An **LSP server mode** that surfaces metrics as diagnostics and code lenses inside editors.

Languages without a tree-sitter grammar (including LSL at v0.1) fall back to a regex tier with scc-style LOC/comment/blank counting, so coverage breadth doesn't suffer. The repo is local-only (no CI, no release pipeline).

## Architecture

### Crate layout (Cargo workspace at repo root)

| Crate | Purpose |
|---|---|
| `bscc-core` | Engine. File walker, language detection, two-tier dispatch, `Registry`, `FileMetrics` & `Report` types, exporter trait. No tree-sitter dependency itself. |
| `bscc-regex-tier` | scc-style declarative tokenizer. Single crate of data + tokenizer; ships ~50 popular langs + LSL via a `languages.toml`. |
| `bscc-lang-rust`, `bscc-lang-python`, `bscc-lang-typescript`, `bscc-lang-go`, `bscc-lang-c`, `bscc-lang-cpp`, `bscc-lang-java` | One crate per tree-sitter-enabled language. Vendors the grammar, ships `queries/metrics.scm`, exposes `pub fn register(&mut Registry)`. Future: `bscc-lang-lsl`. |
| `bscc-git` | `gix`-based git integration. Per-file churn, authors, last-modified, hotspot score. |
| `bscc-export` | Exporters: table, json, csv, sarif, html (askama templates). |
| `bscc-cli` | Binary. Subcommands, flag parsing, config-file loading. Default-features the seven popular language crates + regex tier + git + all exporters. |
| `bscc-lsp` | Binary. `tower-lsp` server. Reuses `Registry` and metric extractors. |

### Two-tier engine flow (in `bscc-core`)

```
walk files → detect language (extension → shebang → content sniff)
          → Registry::lookup(lang)
                ├── TreeSitter plugin? → parse → run .scm queries → FileMetrics (full)
                └── Regex tier entry?  → tokenize lines        → FileMetrics (LOC only)
          → aggregate into Report → hand to exporters
```

`FileMetrics` is a single struct with `Option<T>` for tier-specific fields. The regex tier leaves complexity/AST fields as `None`. Exporters degrade gracefully.

### Language registry pattern

Plugin crates expose `pub fn register(reg: &mut Registry)`. The CLI calls each enabled plugin's `register()` at startup. No dynamic loading, no `inventory` macro magic — explicit registration keeps the build understandable and feature-flag-friendly.

### Metrics

**Regex tier (every registered language):** `lines`, `code`, `comments`, `blanks`, `bytes`, `complexity_stub` (scc-style branching-keyword count).

**Tree-sitter tier (adds):** `functions`, `cyclomatic` (per-function max + sum), `cognitive` (Sonar-style), `max_nesting_depth`, `longest_function_lines`, `todo_comments`, `imports`.

Each tree-sitter language crate ships `queries/metrics.scm` naming the relevant nodes (`function`, `branch`, `comment`, `import`). `bscc-core` runs queries generically — no per-language metric logic in the engine.

### Git integration (`bscc-git`)

Uses `gix` (pure-Rust, no libgit2 cgo). Provides `changes_last_90d`, `authors_count`, `last_modified`, `hotspot_score = complexity × log(changes)`. Disabled cleanly when target isn't a git repo or `--no-git` is passed.

### CLI shape (`bscc-cli`)

```
bscc count    [paths…]   [--format table|json|csv|sarif|html] [--by lang|file]
bscc metrics  [paths…]   # full metrics (incl. complexity if tree-sitter avail)
bscc hotspots [paths…]   # metrics × git churn, sorted by hotspot score
bscc languages           # list registered languages & which tier
bscc explain  <file>     # per-function breakdown for one file
bscc lsp                 # exec the bscc-lsp binary
```

Common flags: `--include`, `--exclude` (globs), `--threads`, `--no-git`, `--config <toml>`.
`bscc.toml` lets repos pin thresholds, ignore paths, and select exporters.

### LSP server (`bscc-lsp`)

Minimal v1.0 surface (built on `tower-lsp`):
- `textDocument/publishDiagnostics` — diagnostics for functions exceeding configured thresholds (cyclomatic > N, length > M, nesting > K).
- `textDocument/codeLens` — complexity + churn next to function headers.
- No completion / hover / rename. This is a metrics surface, not a language server.

## Repo layout

```
bscc/
├── .mise.toml
├── Cargo.toml             # workspace manifest, shared lints/profile
├── rust-toolchain.toml    # fallback for non-mise users
├── crates/
│   ├── bscc-core/
│   ├── bscc-regex-tier/   (with languages.toml data file)
│   ├── bscc-lang-rust/  bscc-lang-python/  bscc-lang-typescript/
│   ├── bscc-lang-go/    bscc-lang-c/       bscc-lang-cpp/  bscc-lang-java/
│   ├── bscc-git/
│   ├── bscc-export/
│   ├── bscc-cli/
│   └── bscc-lsp/
├── tests/                 # workspace-level integration tests on fixture trees
├── fixtures/              # sample source trees per language for tests + benches
└── benches/               # criterion benches vs. scc for parity tracking
```

`Cargo.toml` uses `[workspace.lints]` for repo-wide `clippy::pedantic` and `[workspace.dependencies]` so versions pin in one place.

## Tooling

`.mise.toml` pins:
- `rust = "<latest stable at build start>"`
- `[tools]`: `cargo-nextest`, `cargo-insta`
- `[tasks]`: `test` (`cargo nextest run --workspace`), `lint` (`cargo clippy --workspace --all-targets -- -D warnings`), `fmt` (`cargo fmt --all`), `cover` (`cargo llvm-cov --workspace`), `bench` (`cargo bench --workspace`)

No CI workflow. No release pipeline. No `cargo-deny`. Build/install for users = `cargo build --release` from a clone; binary at `target/release/bscc`. Document in README.

## Testing strategy

Three layers:
1. **Unit tests** in each crate (tokenizer edge cases, query extractors).
2. **Golden tests** in `tests/` — engine runs over `fixtures/`, snapshot JSON output via `insta`.
3. **Parity benches** in `benches/` — `bscc count` vs `scc` on the same fixture trees, asserting LOC numbers within ±1% for shared languages. Drift-detection, not strict equality.

## Build milestones

| M | Deliverable |
|---|---|
| M1 | Workspace skeleton + `bscc-core` walker + `bscc-regex-tier` with 10 langs incl. LSL + `table` exporter + `bscc count`. End-to-end loop. |
| M2 | First `bscc-lang-rust` plugin + generic query runner + `functions`/`cyclomatic` metrics + JSON exporter. Proves the tree-sitter tier and tiered-fill model. |
| M3 | Remaining six tree-sitter language crates + rest of regex tier (~40 more langs) + CSV/SARIF exporters. |
| M4 | `bscc-git` + `bscc hotspots` + `bscc explain` subcommands. |
| M5 | `bscc-lsp` (diagnostics + code lens) + HTML exporter + `bscc.toml` config + README install docs. Ship. |

Each milestone is a usable checkpoint. Stopping at M3 already gives a defensible scc replacement; M4–M5 deliver the differentiator.

Post-v1.0: write `tree-sitter-lsl` (sibling repo, vendored) and add `bscc-lang-lsl`, moving LSL from regex tier to tree-sitter tier.

## Critical files to be created

All new. Repo currently contains only an empty `README.md` and `.git/`. No existing code to modify or reuse.

- `Cargo.toml` (workspace root)
- `.mise.toml`
- `rust-toolchain.toml`
- `crates/bscc-core/{Cargo.toml,src/lib.rs,src/walk.rs,src/detect.rs,src/registry.rs,src/metrics.rs,src/report.rs,src/exporter.rs}`
- `crates/bscc-regex-tier/{Cargo.toml,src/lib.rs,src/tokenizer.rs,data/languages.toml}`
- `crates/bscc-lang-<lang>/{Cargo.toml,src/lib.rs,queries/metrics.scm}` × 7
- `crates/bscc-git/{Cargo.toml,src/lib.rs}`
- `crates/bscc-export/{Cargo.toml,src/{table,json,csv,sarif,html}.rs}`
- `crates/bscc-cli/{Cargo.toml,src/main.rs,src/cmd/{count,metrics,hotspots,languages,explain,lsp}.rs,src/config.rs}`
- `crates/bscc-lsp/{Cargo.toml,src/main.rs}`
- `tests/golden.rs`
- `fixtures/<lang>/...`
- `benches/scc_parity.rs`
- `README.md` (overwrite the empty one)

## External dependencies / utilities to reuse

- `ignore` — file walker with .gitignore support (used by ripgrep)
- `tree-sitter` + per-language `tree-sitter-<lang>` crates from crates.io
- `gix` — pure-Rust git
- `serde` + `serde_json` — schema + JSON exporter
- `clap` (derive) — CLI parsing
- `tower-lsp` — LSP server
- `askama` — HTML templating
- `insta` — snapshot tests
- `criterion` — benches
- `cargo-nextest` — test runner via mise

Nothing in the existing repo to reuse — it's empty.

## Verification

After M1: `mise run test` green; `cargo run -p bscc-cli -- count fixtures/` prints a table; LSL files in `fixtures/` are detected and counted.

After M2: `cargo run -p bscc-cli -- metrics --format json fixtures/rust/` returns JSON with non-null `cyclomatic` and `functions` fields for `.rs` files; snapshot tests in `tests/golden.rs` pass.

After M3: `bscc count` over a mixed-language fixture tree (~40 langs) matches `scc` within ±1% (parity bench passes).

After M4: `bscc hotspots .` run inside a git repo returns files ranked by `complexity × log(changes)`; `--no-git` flag works in non-git directories.

After M5: `bscc lsp` started against a VS Code instance publishes diagnostics for functions exceeding configured thresholds; `bscc count --format html` produces a self-contained file that opens correctly in a browser.

End-to-end smoke: from a clean clone, `mise install && mise run test && cargo build --release && ./target/release/bscc count .` works on this repo itself.
