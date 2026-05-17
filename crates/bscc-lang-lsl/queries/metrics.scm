;; Functions and event handlers both count toward the function tally and
;; serve as scopes for per-function cyclomatic attribution.
(function_declaration) @function
(event_handler) @function

;; Cyclomatic branches.
(if_statement) @branch
(while_statement) @branch
(do_statement) @branch
(for_statement) @branch

;; Short-circuit boolean operators add to cyclomatic complexity too.
((binary_expression operator: _ @op) @branch
 (#any-of? @op "&&" "||"))

;; Comments (line + block). Drives line classification and todo scanning.
(line_comment) @comment
(block_comment) @comment

;; No @import captures: LSL has no native import mechanism. Firestorm-style
;; `#include` directives are exposed as `preproc_directive` nodes and could
;; be captured here once we want them counted.
