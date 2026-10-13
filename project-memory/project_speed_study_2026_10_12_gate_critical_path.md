---
name: project-speed-study-2026-10-12-gate-critical-path
description: Medição de 12/10 de onde vai o tempo de fecho/integração — caminho crítico do gate é o grupo luz2d-placa; vermelho aceito no main paga-se 3×; ph2d-ecs→ph2d-i18n infla o impactado
metadata:
  node_type: memory
  type: project
  originSessionId: d00af5bf-c31f-4689-9602-ebc38343104d
  modified: 2026-10-12T21:35:30.525Z
---

Estudo de velocidade (12/10/2026), pedido pelo dono após «Batched closing gate F13» 39 min e «Fix the 13 Luz2D red tests» 41 min.

- Gate F13: 37,6 de 38,9 min = espera de comando. `nextest-impacted` 19 950 testes, 1 190 s, compilar só 1m44s. **O grupo `luz2d-placa` (max-threads=2, 125 testes) ≈ 1 157 s = o caminho crítico**; a máquina usou ~5 de 32 núcleos. 31 testes ≥ 60 s = 51 % dos test-segundos. Testes da Luz 2D a FHD com varreduras e gabarito CPU sem cache.
- **A UI arrasta a Luz 2D pela aresta `ph2d-ecs → ph2d-i18n`** (só 2 chamadas `tr_em(Ingles, label_key())` em `audio_2d.rs:112`, `signal_actions.rs:233`). Fan-in: tokens 104, i18n 117, editor-core 91 de 392.
- **Vermelho aceito no main pagou-se 3×**: Luz2D integrou com vermelhos conhecidos (b1dd77e477), MiroClone e UIUX re-provaram «herdado», depois 41 min a curar.
- 14 dias de transcrições: polling 38 h; reruns de teste sem edição 47,7 h (filtros diferentes, em série); 76 de 146 cortes de 10 min pediram timeout > 10 min (ignorado); test:check 2,9; reruns de nextest-impacted/clippy/censos na mesma sessão ≈ 21 h. Rebase (4,2 h) e ship.sh (0,5 h) NÃO são gargalo. Tempo por passo do modelo ~3 s — o custo é o número de passos.
- Cada linha roda a suíte impactada 3–4× (fecho, integração, ship, CI).

**Why:** o dono quer integração/implementação mais rápidas sem perder qualidade.
**How to apply:** antes de mexer em velocidade, comece pelo grupo `luz2d-placa` (encolher com prova de mutação, cachear gabarito), recusar vermelho no main, e cortar `ph2d-ecs → ph2d-i18n`. Ver [[feedback_plan_every_measurement_upfront_and_batch_it]].
