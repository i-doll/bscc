# bscc — VS Code extension

Editor integration for [bscc](https://github.com/i-doll/bscc), the
tree-sitter-powered code-metrics tool. Provides:

- **Diagnostics**: warnings on functions exceeding configured
  cyclomatic-complexity or length thresholds (defaults: CC > 10, lines
  > 100; configure via `bscc.toml` at the workspace root).
- **Code lenses**: every function gets a `bscc: CC=N · M lines` lens.
  Click it to open a details panel with the full per-function breakdown.
- **Cost estimate** (from `bscc count`): coming in a follow-up.

## Requirements

You need the `bscc` binary on your `PATH`, or set
`bscc.path` in your VS Code settings to its absolute location.

Until the official release ships:

```sh
git clone https://github.com/i-doll/bscc
cd bscc
cargo build --release
# either symlink the binary onto your PATH:
ln -s "$(pwd)/target/release/bscc" ~/.local/bin/bscc
# or set bscc.path in VS Code settings to:
echo "$(pwd)/target/release/bscc"
```

## Supported languages

Tree-sitter tier (where diagnostics + lenses light up): Rust, Python,
TypeScript, TSX, Go, C, C++, Java, LSL, HCL, Terraform, OpenTofu,
Packer.

Other languages bscc detects (~360 of them, via the regex tier) get
line counts in the CLI but no editor diagnostics.

## Settings

| Setting | Default | Description |
|---|---|---|
| `bscc.path` | `""` | Override the bscc binary location. Empty = look on PATH. |
| `bscc.enable` | `true` | Master toggle for the language server. |
| `bscc.trace.server` | `"off"` | Trace LSP traffic to the bscc output channel. |

Threshold knobs (`cyclomatic_max`, `longest_function_lines`) live in
`bscc.toml` at your workspace root, not in VS Code settings. The
extension watches that file and restarts the server when it changes.

## Building locally

```sh
npm install
npm run compile        # esbuild bundle + tsc --noEmit
npm run package        # produces bscc-0.0.1.vsix
```

Hit `F5` in VS Code to launch an Extension Development Host with the
extension loaded.

## License

MIT OR Apache-2.0 (same as the parent project).
