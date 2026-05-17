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

/// Tree-sitter syntax highlighting query, for editor integration.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../../queries/highlights.scm");

#[cfg(test)]
mod tests {
    use tree_sitter::{Language, Parser, Query};

    fn lang() -> Language {
        super::LANGUAGE.into()
    }

    #[test]
    fn can_load_grammar() {
        let mut parser = Parser::new();
        parser
            .set_language(&lang())
            .expect("Error loading LSL parser");
    }

    #[test]
    fn highlights_query_compiles() {
        Query::new(&lang(), super::HIGHLIGHTS_QUERY).expect("highlights.scm must compile");
    }

    #[test]
    fn parses_realistic_vendor_script_without_errors() {
        let source = r#"
            integer PRICE = 100;
            key buyer = NULL_KEY;

            integer collect(integer amount) {
                if (amount < PRICE) {
                    return FALSE;
                }
                llSay(0, "thank you");
                return TRUE;
            }

            default {
                state_entry() {
                    llSetText("Click to buy", <1.0, 1.0, 1.0>, 1.0);
                    llListen(0, "", NULL_KEY, "");
                }

                money(key giver, integer amount) {
                    if (collect(amount)) {
                        llGiveInventory(giver, "ItemName");
                        state idle;
                    } else {
                        llGiveMoney(giver, amount);
                    }
                }
            }

            state idle {
                state_entry() { llSetTimerEvent(60.0); }
                timer() { state default; }
            }
        "#;
        let mut parser = Parser::new();
        parser.set_language(&lang()).unwrap();
        let tree = parser.parse(source, None).unwrap();
        assert!(
            !tree.root_node().has_error(),
            "parse error in: {}",
            tree.root_node().to_sexp()
        );
    }

    #[test]
    fn parses_preprocessor_directives() {
        let source = r#"
            #include "common.lsl"
            #define MAX 100

            default {
                state_entry() {
                #ifdef DEBUG
                    llSay(0, "debug");
                #endif
                    llSay(0, "ready");
                }
            }
        "#;
        let mut parser = Parser::new();
        parser.set_language(&lang()).unwrap();
        let tree = parser.parse(source, None).unwrap();
        assert!(!tree.root_node().has_error());
    }
}
