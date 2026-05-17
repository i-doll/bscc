;; Syntax highlighting for LSL — capture names follow the tree-sitter
;; "standard captures" convention used by helix, neovim, zed, etc.
;;
;; https://tree-sitter.github.io/tree-sitter/3-syntax-highlighting.html#highlights

;; ───────── Comments ─────────

(line_comment) @comment
(block_comment) @comment

;; ───────── Preprocessor ─────────

(preproc_directive) @keyword.directive

;; ───────── Keywords ─────────

[
  "if"
  "else"
  "while"
  "do"
  "for"
  "return"
  "jump"
] @keyword

[
  "default"
  "state"
] @keyword.control

;; LSL built-in types
(type) @type.builtin

;; ───────── Literals ─────────

(integer_literal) @number
(float_literal) @number.float
(string_literal) @string

;; ───────── Punctuation / operators ─────────

[
  "+" "-" "*" "/" "%"
  "==" "!=" "<" "<=" ">" ">="
  "&&" "||" "!" "~"
  "&" "|" "^" "<<" ">>"
  "=" "+=" "-=" "*=" "/=" "%="
  "++" "--"
] @operator

[ "(" ")" "{" "}" "[" "]" ] @punctuation.bracket
[ ";" "," "." ] @punctuation.delimiter
"@" @punctuation.special

;; ───────── Definitions and uses ─────────

(function_declaration
  name: (identifier) @function)

(event_handler
  name: (identifier) @function.method)

(parameter
  name: (identifier) @variable.parameter)

(global_variable
  name: (identifier) @variable)

(variable_declaration
  name: (identifier) @variable)

(call_expression
  function: (identifier) @function.call)

(label_statement
  name: (identifier) @label)

(jump_statement
  label: (identifier) @label)

(state_declaration
  name: (identifier) @type)

(state_change_statement
  name: (identifier) @type)

(member_expression
  property: (identifier) @property)

;; Fallback identifier highlight
(identifier) @variable
