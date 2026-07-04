---
name: project-painting-removed-layers-effects-kept
description: "Toda a pintura/brush engine foi DELETADA (ADR-0099, 2026-06-20); sobra o host de Layers+Efeitos"
metadata: 
  node_type: memory
  type: project
  originSessionId: 9f655ff9-bc9e-4a06-aebb-5e6a25363bab
---

**2026-06-20 (ADR-0099):** o Enio mandou **deletar por completo a ferramenta de pintura, sem vestígio**, preservando **layers + efeitos**. Sucede [[project-wash-curtis-gd-migration-2026-06-15]] (ADR-0096 já removera a aquarela) — agora saiu o resto.

**Deletadas (8 crates):** `ph2d-painter-brush`, `ph2d-painter-stroke`, `ph2d-tool-brush`, `ph2d-panel-painter-sidebar`, `ph2d-panel-brush-studio`, `ph2d-brush-along-path`, `ph2d-brush-traits`, `ph2d-painter-contracts` (gate `architecture_painter_contract_surface` foi junto).

**Nova:** `ph2d-painter-effects` (deps só `ph2d-color`+serde) ← recebeu `adjustments/` + `blend.rs` (22 blend modes + apply_blend) que moravam DENTRO do brush. É a fonte canônica de efeitos; consumida por `ph2d-tool-painter` e pelos testes de paridade do `ph2d-render`.

**Sobrevive e funciona:** `ph2d-tool-painter` (host slim: LayerStack, máscaras/grupos, compositor CPU, edição de adjustment, undo **estrutural**, RasterEditTool/Apply), `ph2d-panel-painter-layers` (UI viva), compositor GPU 22-modos do `ph2d-render`. **Default tool: brush → `move`** (`MoveTool::is_default`).

**Why:** [[feedback-perfection-no-deferrals]] — gaps in-scope fecham; padrão-ouro. O desafio era que as fronteiras passavam DENTRO das crates (efeitos no brush; layers no tool; persist no stroke) → exigiu **extração cirúrgica antes de deletar**.

**How to apply:**
- Painter agora é **host de layers/efeitos, não pinta**. Conteúdo de raster layer vem de import/Apply, não de pincel.
- Memórias antigas que tratam o brush engine como frontier ativo (ex.: [[project-brush-audit-2026-06-18]], [[feedback-painter-inefficiency-4-causes]]) são **históricas** — o código que citam não existe mais.
- **Sem migração de save / sem bump de schema:** `PaintProject`/WAL/autosave/recovery não tinham consumidor externo (layers são in-memory + baked no sprite via Apply → persist normal de asset).
- **Nomes `painter` mantidos** no que sobrevive (crate/painel/chrome ids `PAINTER_*`/IconId/magic) p/ minimizar churn em superfícies sensíveis. Chrome ids `PAINTER_STUDIO_*`/`PAINTER_SIDEBAR_*` ficaram órfãos (inertes, espelhados no teste de colisão).
- **Gate de perf do compositor GPU** (`tests/layer_compositor_gpu.rs::measure_composite`) passou a reportar o **MÍNIMO** (floor do hardware), não a mediana — robusto à contenção da GPU headless/compartilhada do sandbox (mediana/cauda inflam 3-100×; min ~4.6ms estável). Vide [[feedback-measure-perf-symptom-scale]].
- **GOTCHA durável (auditoria 2026-06-20): deletar crate quebra gates que a referenciam POR STRING — `cargo check -p`/`clippy -p` NÃO pegam.** A auditoria achou 2 quebras de CI que meu grep de "sem vestígio" + check-loop perderam: (1) **`.config/nextest.toml`** com `filter = 'package(<crate-deletada>)'` → `cargo nextest` ABORTA inteiro (quebra ship.sh+CI); (2) **arch-gate em OUTRA crate** (`ph2d-vector-doc` gateava `StampSpec` de `ph2d-brush-traits` via ADR-0067 §2.6) → teste FALHA strict. Ao deletar crate: grepar `.config/`, e arch-gate tests cross-crate que citam o nome por string. Rodar `nextest run --workspace` + `clippy --workspace --all-targets` (não só `-p`). Reforça [[feedback-full-gate-periodically]].
