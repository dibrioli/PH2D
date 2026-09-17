---
name: feedback-a-bins-run-never-reaches-the-gates-that-live-in-tests
description: `cargo test --bins` leu 4452 verdes e o teto de linhas do shell estava vermelho havia TRÊS fechos — o gate dele vive em `shells/desktop/tests/`, e aquele alvo não lhe toca.
metadata:
  type: feedback
---

Medido em 2026-09-05 (W122, doc 06 §123.6). A corrida de fecho desta linha lia
`cargo test -p ph2d-host-desktop --bins` — `346` e depois `4452` testes verdes. Ao correr
`--tests` pela primeira vez, **duas reprovações**: `field3d_input_tests.rs` a `604` e
`undo_tests.rs` a `602` sobre um teto de `600`. No `main` eles medem `599` e `592` ⇒ foram as waves
**desta própria linha** que os empurraram, e o vermelho sobreviveu a **três fechos**.

**Why:** `--bins` compila e corre os testes **dentro** do binário; os ficheiros de
`shells/desktop/tests/` são alvos de **integração** e não entram. O `CLAUDE.md` §5 já nomeia esta
cegueira, e ela voltou a morder porque a corrida de fecho foi escrita a pensar em *«os meus testes»*
em vez de *«os gates que julgam o meu diff»*.

**How to apply:** o fecho de uma linha que toca o shell corre **`--tests`** (ou `--all-targets`),
com `--no-fail-fast`, e conta as suítes do ficheiro de log
([[feedback_a_tail_is_a_window_not_a_verdict]]). ⚠️ E a cura de um teto é **cortar por assunto**,
nunca a marca de isenção que o próprio gate oferece — aqui saíram `undo_library_tests.rs` (a
biblioteca de imagens no undo) e `field3d_input_undo_seam_tests.rs` (o undo de um arrasto).
Irmãs: [[feedback_a_closing_run_with_a_name_filter_never_reaches_a_tree_scanning_gate]] ·
[[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]] ·
[[feedback_testing_a_crate_alone_hides_every_defect_in_a_feature_the_shell_enables]]

## ⛔⛔ E os dois tectos são um MECANISMO: curar o da FUNÇÃO acende o do FICHEIRO

Medido em 2026-09-13 (`line/components`, W2 das tags). O `project_load_from` chegou a `228 / 200` do
tecto por **função**; a cura mandada pelo gate — partir por responsabilidade — acrescentou uma
assinatura, um doc-comment e um `}` ⇒ **seis linhas ao ficheiro**, que passou de `600` a `606` do
tecto por **FICHEIRO**. O commit fechou com o segundo vermelho dentro, e só a corrida seguinte o viu.

**Why:** partir uma função NUNCA encolhe o ficheiro — ele cresce sempre um pouco. E os dois gates
correm em alvos diferentes (`fn_loc_caps` no `--bins`, `file_loc_caps` em `tests/it/`), logo a
corrida que confirma a cura de um é cega ao outro por construção.

**How to apply:** depois de partir uma função para curar um tecto, corra **os dois** gates de tecto
na mesma linha de comando — `cargo test -p <shell> --test it loc_cap` mais o `--bins` — antes de
comitar. ⚠️ E se o ficheiro estourar, a cura é outra vez **corte por responsabilidade**, no ficheiro
desta vez (aqui saiu o `project_load_read.rs`: *que bytes são estes* separado de *o que a sessão faz
com eles*), ⛔ nunca uma entrada nova no `FILE_OVERAGE_OK`.
