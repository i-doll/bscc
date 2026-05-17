;; Top-level blocks only: resource, module, data, variable, output,
;; locals, provider, terraform — the unit of organization in HCL2.
;; Nested blocks (lifecycle, connection, content, provisioner) are
;; configuration and do not get their own complexity scope.
(config_file
  (body
    (block) @function))

;; Cyclomatic branches.
(conditional) @branch        ;; ternary  a ? b : c
(for_expr)   @branch         ;; for x in y : ...

;; Template-side control flow (inside ${...} / %{...}).
(template_if)  @branch
(template_for) @branch

;; dynamic blocks: dynamic "label" { for_each = ... ; content { ... } }.
;; Detected by block-type identifier; counts as a branch (a loop).
((block (identifier) @_blk_type) @branch
 (#eq? @_blk_type "dynamic"))

;; count / for_each meta-arguments on resources & modules. Captured by
;; attribute identifier; rare elsewhere so false positives are negligible.
((attribute (identifier) @_attr_name) @branch
 (#any-of? @_attr_name "count" "for_each"))

;; Short-circuit boolean operators. Matching the anonymous operator
;; token (not the whole expression text) keeps nested expressions from
;; double-counting.
(binary_operation "&&") @branch
(binary_operation "||") @branch

;; Comments. `#`, `//`, `/* */` all map to the single `comment` node type.
(comment) @comment

;; No @import: HCL has no native import statement. `source = "..."` in
;; modules is a plain attribute; could be added later as a separate
;; capture if module-dependency counting becomes a goal.
