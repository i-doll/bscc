;; Functions
(function_declaration) @function
(method_declaration) @function
(func_literal) @function

;; Branches
(if_statement) @branch
(for_statement) @branch
(expression_case) @branch
(default_case) @branch
(type_case) @branch
(communication_case) @branch

;; Short-circuit boolean operators
((binary_expression operator: _ @op) @branch
 (#any-of? @op "&&" "||"))

;; Imports
(import_spec) @import

;; Comments
(comment) @comment
