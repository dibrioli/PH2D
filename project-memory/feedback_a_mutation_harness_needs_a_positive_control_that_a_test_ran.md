---
name: feedback_a_mutation_harness_needs_a_positive_control_that_a_test_ran
description: "Prova de mutação: exija `running 1 test`, não só exit code — um filtro que não casa corre ZERO testes e sai 0"
metadata:
  node_type: memory
  type: feedback
---

Um arnês de mutação que julgue só pelo **exit code** do cargo mede o nada quando o filtro erra.
Medido 2026-08-23 (`line/3DModeling`, W43): `cargo test -p X -- --exact <nome_curto>` sobre gates
cujo caminho real era `field3d_smoke::view::tests::<nome>` — com `--exact` o filtro **não casou**,
correram **zero** testes, o cargo saiu **0**, e as três mutações foram declaradas **SOBREVIVENTES**
sem que um teste tivesse existido.

**Why:** é a armadilha do filtro escrito à mão (CLAUDE.md §2: *"797 corridas devolveram literalmente
NADA"*) dentro do sítio onde ela é mais cara — a saída **parece um resultado**, com nome de gate e
veredito. ⚠️ Ali a polaridade salvou (gritou «sobreviveu», não «RED»); a versão simétrica — um arnês
que conclua RED sem ter corrido — é **confiança falsa e silenciosa**.

**How to apply:** todo arnês exige **duas** provas antes de aceitar um veredito: `Compiling <pkg>`
(o binário é novo — ver [[feedback_a_restored_file_keeps_its_old_mtime_and_cargo_reuses_the_mutant]])
**e** `running 1 test` (o teste existiu). Passe o caminho **completo** do teste, e prefira derivá-lo
da corrida verde anterior em vez de o escrever de memória. Irmã de
[[feedback_pipe_masks_script_exit_code]] e de [[feedback_a_mutation_that_survives_may_mean_a_missing_gate]]:
antes de concluir *"o gate está a faltar"*, confirme que houve corrida.
