;; Functions
(function_definition) @function

;; Branches
(if_statement) @branch
(while_statement) @branch
(do_statement) @branch
(for_statement) @branch
(case_statement) @branch
(conditional_expression) @branch

;; Short-circuit boolean operators
((binary_expression operator: _ @op) @branch
 (#any-of? @op "&&" "||"))

;; Imports
(preproc_include) @import

;; Comments
(comment) @comment
