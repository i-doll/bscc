# tree-sitter-lsl

[Tree-sitter] grammar for [LSL] — Linden Scripting Language, the language
embedded in Second Life and OpenSimulator.

Built originally to back the LSL plugin in [bscc], but published as an
independent grammar so other editors (neovim, helix, emacs, zed, …) can
consume it.

## Use from Rust

```rust
use tree_sitter::Parser;

let mut parser = Parser::new();
parser.set_language(&tree_sitter_lsl::LANGUAGE.into()).unwrap();
let tree = parser.parse("default { state_entry() { llSay(0, \"hi\"); } }", None).unwrap();
assert!(!tree.root_node().has_error());
```

## Development

Generate the parser from `grammar.js` and run the corpus tests:

```sh
tree-sitter generate
tree-sitter test
```

(Both available via `mise run grammar:generate` / `mise run grammar:test` if
you're inside the bscc workspace.)

## Coverage

| Feature | Status |
|---|---|
| States (`default` + named) | ✓ |
| Event handlers | ✓ |
| Global functions and variables | ✓ |
| Control flow (if/else/while/do/for/return/jump/@label/state) | ✓ |
| All LSL operators with correct precedence | ✓ |
| `<x,y,z>` vector and `<x,y,z,s>` rotation literals | ✓ |
| List literals `[a, b, c]` | ✓ |
| Member access for vectors/rotations (`v.x`, `r.s`) | ✓ |
| Comments (line + block) | ✓ |
| Firestorm/OSSL preprocessor (`#include`, `#define`, …) | planned |
| Editor query files (`highlights.scm`, `locals.scm`) | planned |

## License

Dual-licensed under either of MIT or Apache-2.0 at your option.

[Tree-sitter]: https://tree-sitter.github.io/
[LSL]: https://wiki.secondlife.com/wiki/LSL_Portal
[bscc]: https://github.com/i-doll/bscc
