# Oráculo Godot — fechamento automático e auto-indent (E8)

O `CodeEdit` do Godot 4.7.2 (MIT, `pacman -Qi godot`) é CORRIDO, nunca lido (`CLAUDE.md §0.9`).

- `casos.txt` — entrada NOSSA: texto com o cursor (`|`, `^` = âncora), teclas, estilo de indentação e
  o mapa GDScript→Luau de cada caso. O formato está no topo de `oraculo.gd`.
- `oraculo.gd` — instancia o `CodeEdit` (auto-brace + auto-indent ligados, delimitadores do Luau),
  dá foco no 1.º quadro e no 2.º empurra as teclas por `push_input` (o `insert_text_at_caret` NÃO
  dispara o auto-brace) e escreve o texto e os cursores de cada caso.
- `corre.sh` — corre-o com `timeout` e grava a fixtura com cabeçalho em
  `crates/ph2d-code-syntax/tests/it/fixtures/godot_autoedit.txt`.

Caso novo: acrescente-o a `casos.txt`, corra `bash corre.sh` e o gate
`autoedit::cada_caso_do_godot` passa a compará-lo com a nossa lei (`ph2d_code_syntax::AutoEdit`).
Divergência de propósito vai na lista `DIVERGE` do teste, com o porquê.
