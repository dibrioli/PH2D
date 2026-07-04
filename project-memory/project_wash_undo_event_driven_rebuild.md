---
name: project_wash_undo_event_driven_rebuild
description: "Bug \"undo do wash volta ao pintar\" = solver twin-buffer stale, NÃO o controle (ADR-0090)"
metadata: 
  node_type: memory
  type: project
  originSessionId: da867ef3-9b65-4b2c-b452-604f23cca0f9
---

O bug "dei undo no wash, pintei de novo e a mancha desfeita volta" (Enio 2026-06-14) **sobreviveu a 3
ADRs de reescrita do CONTROLE de undo (0088→0089→0090)** porque nunca esteve no controle — era do
**solver** `WashSolver` (`crates/ph2d-painter-wash/src/solver.rs`).

**Causa-raiz (durável):** o solver tem buffers gêmeos ping-pong `pig_a`/`pig_b` (idem `dye`). Um
`encode_step` de **região** escreve `pig_b` só na região pintada e depois copia o `pig_b` **INTEIRO**
de volta pra `pig_a`. Correto enquanto vale a invariante `pig_a == pig_b` (a pintura normal mantém:
todo step copia de volta). **O undo-restore quebrava:** `upload_pigment` escrevia **só `pig_a`** →
`pig_b` ficava com o campo PRÉ-undo → a 1ª pincelada de região seguinte ressuscitava a mancha desfeita
**FORA da região pintada** via o copy-back do gêmeo stale. Fix: overwrite parcial de campo tem que
escrever **os dois gêmeos**. Padrão geral: **qualquer write parcial de um buffer ping-pong com
copy-back full precisa atualizar AMBOS os gêmeos**, senão o próximo step de região ressuscita o stale.

**Meta-lição (a que mais custou):** eu troquei o suspeito "óbvio" (o controle por contagem) sem
**reproduzir o sintoma isolado**. O Enio testou e disse "nada mudou". Foi um `eprintln` de
instrumentação no caminho ativo que provou o controle PERFEITO (`undo=2 redo=1`, pincelada nova =
`[Commit] redo=0` sem evento Redo) e apontou pro solver. **Instrumente o caminho ativo e prove ONDE o
estado diverge ANTES de reescrever** — vide [[feedback_measure_perf_symptom_scale]] e
[[feedback_tool_unit_green_integration_dead]]. Bônus: havia 2 sistemas (wash e fluid, flags
`wash_enabled`/`fluid_enabled` mutuamente exclusivas, ambos bridges no render_loop) — confirme QUAL
roda antes de editar.

**Undo completo = TODO o estado dinâmico:** logo após o fix do gêmeo veio o bug-2 — o restore
recolocava a cor mas não a **água**, então a área desfeita ficava molhada (evap-0 nunca seca, sangra
nas próximas pinceladas). O `FieldSnap` tem que capturar/restaurar os 3 campos dinâmicos do solver
(`pig`+`dye`+`water`), cada um com a escrita dos dois gêmeos (`paper` é estático, fica fora). Regra:
restaure TODO o estado dinâmico, não só o canal visível.

**Gap de teste:** `wash_artifact_repro` testava restore→composite (passava), nunca restore→**pintar**.
O teste novo `restore_then_paint_does_not_resurrect_undone_pigment` (Metal) pega: mancha desfeita =
0.000 após restore+pintar.

O rebuild de controle (eventos `WashUndoEvent` Commit/Undo/Redo na bridge espelhando o `UndoController`
raster, bit `is_wash` na entrada, snapshots esparsos) era defensável (o esquema de contagem era
frágil) mas **não** era o que bloqueava. A cor do 0089 ficou intacta. Ver [[project_watercolor_v2_gpu_first_refactor]].
