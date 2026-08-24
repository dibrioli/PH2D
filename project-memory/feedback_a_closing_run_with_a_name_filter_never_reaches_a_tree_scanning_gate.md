---
name: feedback-a-closing-run-with-a-name-filter-never-reaches-a-tree-scanning-gate
description: O fecho de módulo tem de correr a suíte SEM filtro — um `-p <crate>` nunca alcança um gate que VARRE a árvore, e eu shipei um vermelho por isso duas waves seguidas
metadata:
  type: feedback
---

Um gate que **varre** `crates/*/src/` (LOC cap, tofu, censos) mora na crate onde a REGRA mora — não
onde o arquivo mora. Um fecho que corre `cargo test -p <a crate que toquei>` **nunca o alcança**, e
o vermelho fica no ramo até alguém correr a suíte inteira.

⛔ **Medido duas vezes seguidas na `line/3DModeling`:** a W48–W51 deixou `no_tofu_glyphs` vermelho
por nove `→` em mensagens de `assert!`; e a W56d deixou `architecture_workspace_file_loc_cap`
vermelho por dois arquivos que a própria wave escreveu (889/700 e 795/700) — **com a memória
[[feedback-a-tree-scanning-gate-is-never-reached-by-a-name-filter]] já escrita**.

**Why:** a segunda ocorrência prova que saber a regra não basta: o fecho é o momento em que o filtro
é mais tentador (a corrida completa é lenta) e o gate mais invisível (ele não fala do meu código).

**How to apply:** no fecho da linha, **antes** do handoff, corra os gates de árvore **por nome**:
`cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap --test no_tofu_glyphs`,
mais `cargo check --workspace --all-targets`. E ao cortar por LOC, corte para o **irmão por
responsabilidade** — nunca allowlist ([[feedback-loc-cap-split-not-allowlist-and-fmt-reexpands]]).
