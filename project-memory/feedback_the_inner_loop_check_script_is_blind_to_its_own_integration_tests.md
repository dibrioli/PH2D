---
name: feedback-the-inner-loop-check-script-is-blind-to-its-own-integration-tests
description: O `cargo-check-narrow.sh` não passa `--all-targets` — ele lê VERDE com os `tests/it/` da própria crate partidos, e é o script que o CLAUDE.md §2 nomeia como o laço interno.
metadata:
  type: feedback
---

Medido em 2026-09-13 (`line/components`, Tags W3b). Apaguei um campo de uma struct de modelo e corri
`bash scripts/cargo-check-narrow.sh ph2d-panel-inspector`: **verde**. As fixturas de
`crates/ph2d-panel-inspector/tests/it/` continuavam a construir o campo apagado, e um
`cargo check -p ph2d-panel-inspector --all-targets` à mão acusou **quatro** erros de imediato.

**Why:** a linha do script é `cargo check -p "$crate" --message-format=json "$@"` — os alvos extra
viajam em `"$@"`, logo **sem argumento ele compila só a `lib`**. Os `tests/` de integração (e os
`benches`, e os `examples`) ficam de fora. ⚠️ Isto é vizinho da lei já registada
([[feedback_a_bins_run_never_reaches_the_gates_that_live_in_tests]]) e **não é a mesma**: ali o
buraco é do `cargo test --bins` num FECHO; aqui é do `check` do laço interno, que o CLAUDE.md §2
nomeia pelo nome e manda usar a cada passo. *A ferramenta certa, com o default errado.*

⚠️ **E o modo de falha é o caro:** um refactor de assinatura ou de campo lê verde durante toda a
sessão e só cai no portão de fecho, longe da edição que o causou.

**How to apply:** no laço interno de uma crate que TEM `tests/` próprios, passe o alvo:
`bash scripts/cargo-check-narrow.sh <crate> --all-targets`. ⭐ Ou, quando a edição mexe numa
assinatura/num campo público, `cargo check -p <crate> --all-targets` à mão — é o mesmo relógio e
alcança o que o outro deixa de fora.

Irmãs: [[feedback_a_bins_run_never_reaches_the_gates_that_live_in_tests]] ·
[[feedback_a_closing_run_with_a_name_filter_never_reaches_a_tree_scanning_gate]] ·
[[feedback_a_tail_is_a_window_not_a_verdict]]
