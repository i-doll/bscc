;; Functions
(function_declaration) @function
(function_expression) @function
(arrow_function) @function
(method_definition) @function

;; Branches
(if_statement) @branch
(while_statement) @branch
(do_statement) @branch
(for_statement) @branch
(for_in_statement) @branch
(switch_case) @branch
(catch_clause) @branch
(ternary_expression) @branch

;; Short-circuit boolean operators
((binary_expression operator: _ @op) @branch
 (#any-of? @op "&&" "||" "??"))

;; Imports
(import_statement) @import

;; Comments
(comment) @comment
