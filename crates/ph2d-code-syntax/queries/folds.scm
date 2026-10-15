; Autoria PH2D (não é cópia). O `folds.scm` do pacote `tree-sitter-luau` 1.2.0 é só
; `; inherits: lua` + `(object_type) @fold`: a herança é do nvim-treesitter e o tree-sitter puro
; não a resolve. Esta lista é a herança escrita por extenso sobre os nós do `node-types.json` da
; gramática Luau, mais o `object_type` do pacote. Só dobram nós de >= 2 linhas (filtro no código).
[
  (function_declaration)
  (function_definition)
  (do_statement)
  (while_statement)
  (repeat_statement)
  (for_statement)
  (if_statement)
  (elseif_statement)
  (else_statement)
  (table_constructor)
  (arguments)
  (parameters)
  (object_type)
  (comment)
] @fold
