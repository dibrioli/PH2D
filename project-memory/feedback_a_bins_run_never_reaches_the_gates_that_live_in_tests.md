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
