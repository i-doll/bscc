;; Functions
(function_item) @function
(closure_expression) @function

;; Cyclomatic branches (each adds 1)
(if_expression) @branch
(while_expression) @branch
(for_expression) @branch
(loop_expression) @branch
(match_arm) @branch

;; Short-circuit boolean operators count as branches.
((binary_expression operator: _ @op) @branch
 (#any-of? @op "&&" "||"))

;; Imports
(use_declaration) @import

;; Comments
(line_comment) @comment
(block_comment) @comment
