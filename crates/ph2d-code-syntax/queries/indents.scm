; Autoria PH2D (não é cópia). O `indents.scm` do pacote `tree-sitter-luau` 1.2.0 é só
; `; inherits: lua` + `(object_type) @indent.begin`; a herança e as capturas `@indent.*` são do
; nvim-treesitter e o tree-sitter puro não as interpreta — quem as lê é `src/indent.rs`:
;   @indent.begin  — o corpo deste nó fica +1 nível (um por LINHA de abertura, nunca por nó)
;   @indent.end    — a linha que COMEÇA por este token recua para o nível do nó que ele fecha
;   @indent.branch — idem, para os ramos (`else`/`elseif`) que reabrem o corpo a seguir
;   @indent.open   — token que abre um nível quando está solto num ERROR (ver o fim)
[
  (function_declaration)
  (function_definition)
  (do_statement)
  (while_statement)
  (repeat_statement)
  (for_statement)
  (if_statement)
  (table_constructor)
  (arguments)
  (parameters)
  (parenthesized_expression)
  (object_type)
] @indent.begin

[
  "end"
  "until"
  "}"
  ")"
] @indent.end

[
  "else"
  "elseif"
] @indent.branch

; Só lida DENTRO de um nó ERROR: texto meio escrito (`function f()` sem `end`) não vira nó de
; bloco, vira um ERROR com os tokens soltos — estes abrem um nível, `@indent.end` fecham-no.
[
  "function"
  "then"
  "do"
  "repeat"
  "else"
  "{"
  "("
] @indent.open
