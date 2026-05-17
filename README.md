# bscc — "Better scc"

A from-scratch line-counter + complexity-metrics tool inspired by [scc].
Two-tier engine: tree-sitter where grammars are available (Rust, Python,
TypeScript, Go, C, C++, Java in v1), declarative regex tokenizer everywhere
else (~50 languages including LSL).

Beyond LOC counting, `bscc` produces:

- Structural complexity metrics (functions, cyclomatic, cognitive, nesting,
  longest function, todo comments, imports) via tree-sitter
- Git churn integration and hotspot scoring
- Multiple exporters (table, JSON, CSV, SARIF, self-contained HTML)
- An LSP server mode (`bscc lsp`) that surfaces metrics as diagnostics and
  code lenses inside editors

## Build

This repo uses [mise] to pin the Rust toolchain and provide task aliases:

```sh
mise install
mise run test
cargo build --release
./target/release/bscc count .
```

Without mise: `rust-toolchain.toml` pins the same Rust version for `rustup`.

## Status

Pre-v1. Built incrementally across milestones M1–M5; see
[`docs/superpowers/specs/2026-05-17-bscc-design.md`](docs/superpowers/specs/2026-05-17-bscc-design.md)
for the design.

[scc]: https://github.com/boyter/scc
[mise]: https://mise.jdx.dev/
