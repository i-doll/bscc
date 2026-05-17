;; Functions
(method_declaration) @function
(constructor_declaration) @function
(lambda_expression) @function

;; Branches
(if_statement) @branch
(while_statement) @branch
(do_statement) @branch
(for_statement) @branch
(enhanced_for_statement) @branch
(switch_block_statement_group) @branch
(catch_clause) @branch
(ternary_expression) @branch

;; Short-circuit boolean operators
((binary_expression operator: _ @op) @branch
 (#any-of? @op "&&" "||"))

;; Imports
(import_declaration) @import

;; Comments
(line_comment) @comment
(block_comment) @comment
