# HANDOFF — Painter / Brush Engine (tracker ÚNICO do módulo)

> Regra (pós-investigação 2026-06-16): **um tracker vivo por módulo**. Handoffs por-task/coord
> são efêmeros e vão pra `docs/archive/` ao fechar. Histórico antigo: `docs/archive/handoffs-2026-06-16/`.
> Toda etapa segue [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md).
> Norte: [ADR-0097](architecture/decisions/0097-brush-engine-procreate-parity-cpu-first-dab-pipeline.md) (CPU-first dab pipeline).

## Estado (2026-06-16)

**Track A — provar a costura (FECHADO):**
- ✅ Golden-image harness end-to-end: [`golden_tests.rs`](../crates/ph2d-tool-painter/src/tool/golden_tests.rs)
  — dirige `begin_stroke→queue_pointer→end_stroke` e afirma PIXEIS (depósito + sem-scallop com
  ripple/depth medido em 0.144, guarda a regressão do bug Dilution-na-taxa).
- ✅ Gate executável dos 8 sites: [`architecture_studio_slider_wiring.rs`](../crates/ph2d-panel-brush-studio/tests/architecture_studio_slider_wiring.rs)
  — A⊆D (vivo→despachado), A⊆P (vivo→registrado), D\A⊆DORMANT. Qualquer fio morto acidental falha.
- ✅ Diagnóstico corrigido: Roundness/AlphaThreshold = **dormência intencional** (engine não lê os
  campos; exigem Stamp ABI 96B), não bug. Codificado na allowlist DORMANT do gate.
- ✅ SelectBrush no-op → **ruidoso** ([`lifecycle.rs`](../crates/ph2d-tool-painter/src/tool/lifecycle.rs) `SelectBrush`).

## Carry-overs (open)

1. **SelectBrush real** — falta um **registry handle→Brush** pra resolver o handle; depois rotear por
   `set_brush(handle, brush)` (runtime.rs). Hoje é no-op ruidoso. Existe `library::{round_hard,round_soft,…}`
   em `ph2d-painter-brush` como ponto de partida do registry.
2. **alpha_threshold** — campo existe (`rendering.rs`) mas **não é lido**. É barato implementar de verdade
   (gate de escrita por-pixel abaixo do threshold em `cpu_render`, NÃO exige Stamp ABI). **Muda render →
   decisão do Enio** antes de expor o slider (senão vira no-op se exposto sem o engine).
3. **shape_roundness** — dormente correto: precisa de campo no **Stamp ABI 96B congelado** (Coord + ADR).
4. **Smudge spacing** — quando Pull/Wet ativo, apertar o spacing no `stamp_scheduler` (0.05–0.10×∅ vs
   ~0.25×∅ default) — `05_auditoria_algoritmos_wet_mix.md` §Gap. Geometria (W1/scheduler), fora do `cpu_render`.

## Próximo (sequência ratificada pelo Enio: A→B→C)

- **Track B — reconciliar base (FECHADO):** links ADR-0096, Tool 10→11, CLAUDE.md §5 (duplo-ativo+data)
  corrigidos. Os 2 `LayerStack`/`LayerId` (u64 runtime vs u32 savefile) **NÃO colapsados** — leitura do
  código revelou split RATIFICADO (Coord 2026-05-31) com ponte u64↔u32 documentada (Chesterton). Resolvido
  por **desambiguação**: savefile → `PersistLayerId`/`PersistLayerStack` (zero quebra de save, provado por
  `persistence_roundtrip`), + gate `architecture_no_layerid_name_collision`. ADRs 0078-0095 mantidos
  (log append-only; §5 já os marca histórico).
- **Track C — canvas GPU: RESOLVIDO ([ADR-0098](architecture/decisions/0098-gpu-resident-canvas-spike-no-go-cpu-first-stands.md)).**
  Spike `spike_cpu_stroke_cost_4k` (4K, release): CPU aguenta brush ≤256px com folga; só >1024px@4K
  estoura. **NO-GO na migração GPU-residente** (não é requisito da paridade). CPU-first (ADR-0097)
  mantido. Gatilho de revisita escrito. Follow-up: relaxar os 9 gates de ABI-freeze do StampPipeline
  (congelam ABI sem consumidor) — não deletar (substrato do gatilho).
