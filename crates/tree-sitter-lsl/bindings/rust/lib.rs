//! LSL (Linden Scripting Language) grammar for [tree-sitter].
//!
//! Typically you'll use [`LANGUAGE`] to attach this grammar to a tree-sitter
//! [`Parser`][tree_sitter::Parser]:
//!
//! ```
//! use tree_sitter::Parser;
//!
//! let code = r#"
//! default {
//!     state_entry() {
//!         llSay(0, "Hello, Avatar!");
//!     }
//! }
//! "#;
//! let mut parser = Parser::new();
//! parser
//!     .set_language(&tree_sitter_lsl::LANGUAGE.into())
//!     .expect("Error loading LSL parser");
//! let tree = parser.parse(code, None).unwrap();
//! assert!(!tree.root_node().has_error());
//! ```
//!
//! [tree-sitter]: https://tree-sitter.github.io/

use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_lsl() -> *const ();
}

/// The tree-sitter [`LanguageFn`] for LSL.
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_lsl) };

/// Contents of the generated `node-types.json` file.
pub const NODE_TYPES: &str = include_str!("../../src/node-types.json");

#[cfg(test)]
mod tests {
    #[test]
    fn can_load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::LANGUAGE.into())
            .expect("Error loading LSL parser");
    }
}
