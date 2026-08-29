---
name: feedback-a-closing-run-with-a-name-filter-never-reaches-a-tree-scanning-gate
description: A suíte SEM filtro corre antes de eu dizer VERDE — um `-p <crate>` nunca alcança um gate que VARRE a árvore, e já aconteceu em CINCO waves, em TRÊS linhas diferentes
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

⛔⛔ **E de novo na `line/motion-value`, 2026-08-25 — DUAS waves, com esta memória já escrita e a
apontar o comando exacto.** O bloco das bases de ruído deixou `noise.rs` a **753** e o bloco do
`value` deixou `value-wrap/lib.rs` a **703**; os dois foram **commitados e reportados ao Enio como
verdes**, e só apareceram quando uma terceira wave correu a workspace inteira por outro motivo.

⛔⛔⛔ **E a QUINTA na `line/quadextract`, 2026-08-29 — TRÊS waves seguidas, cada uma com o seu
portão de fecho, e nenhuma o alcançou.** As waves da almofada, da mordida e da agulha deixaram
`ph2d-remesh-iso/src/lib.rs` a **875** e `ph2d-quadextract/src/cells.rs` a **758**; os três portões
correram `-p ph2d-quadextract`, `-p ph2d-remesh-iso` e `--bins retopo`, e os três disseram verde.
⚠️ **Só apareceu quando o pedido foi «deixe pronto para USAR»** — que é a primeira vez que alguém
pergunta pela ÁRVORE em vez de pela crate. ⚠️ E o mesmo `cargo fmt` da árvore expôs **dois ficheiros
por formatar** commitados numa wave anterior: `cargo fmt -p <crate> -- --check` também nunca entrou
em portão nenhum.

**Why:** cinco ocorrências em três linhas provam que saber a regra não basta, e mostram *quando* ela
falha: o filtro é mais tentador exactamente no fecho (a corrida completa é lenta) e o gate é mais
invisível (ele não fala do meu código, e mora numa crate que eu não toquei). ⚠️ **E o modo de falha
é CUMULATIVO** — três waves da mesma linha herdam o mesmo ponto cego, então o vermelho não fica um
bloco no ramo: fica a jornada inteira.

⚠️ **E o gatilho não é «no fecho da linha» — é ANTES DE DIZER VERDE.** As cinco vezes o vermelho
entrou num commit e num relatório ao Enio, muito antes de qualquer fecho: um bloco que se reporta
como pronto já afirmou que a suíte passa.

**How to apply:** **antes de reportar um bloco como verde** (não só no fecho), corra os gates de
árvore **por nome**:
`cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap --test no_tofu_glyphs`,
mais `cargo fmt --all -- --check` e `cargo check --workspace --all-targets`. E ao cortar por LOC,
corte para o **irmão por responsabilidade** — nunca allowlist
([[feedback-loc-cap-split-not-allowlist-and-fmt-reexpands]]). ⭐ O corte bom nomeia **uma pergunta
por módulo** (*«que células fecham?»* contra *«que vértice não devia existir?»*), e o sinal de que
ele é o certo é as duas metades **cruzarem-se num sítio só**.

## ⚠️ A TERCEIRA variante do filtro, achada em 2026-08-29 — `--bins` não é `--tests`

As duas primeiras variantes eram `-p <crate>` e `cargo fmt -p <crate>`. A terceira é **dentro da
crate certa**: `cargo test -p ph2d-host-desktop --bins` corre **3 834** testes do shell e **não toca
em `shells/desktop/tests/`** — onde vive `shell_files_respect_hr18_loc_cap`, o gate de LOC do próprio
shell. *Um portão pode correr quase quatro mil testes da crate certa e ainda assim não alcançar o
gate dela.* ⇒ o comando é `cargo test -p <crate>` **sem** `--bins` (ou `--tests` explícito ao lado).
