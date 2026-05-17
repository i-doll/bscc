;; Functions
(function_definition) @function
(lambda) @function

;; Branches
(if_statement) @branch
(elif_clause) @branch
(while_statement) @branch
(for_statement) @branch
(except_clause) @branch
(conditional_expression) @branch

;; Short-circuit boolean operators
((boolean_operator operator: _ @op) @branch
 (#any-of? @op "and" "or"))

;; Imports
(import_statement) @import
(import_from_statement) @import

;; Comments
(comment) @comment
