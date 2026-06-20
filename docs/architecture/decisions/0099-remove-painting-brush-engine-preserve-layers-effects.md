# ADR-0099 — Remoção da ferramenta de pintura / brush engine; preservar layers + efeitos

- **Status:** ACEITO (Enio, 2026-06-20).
- **Data:** 2026-06-20.
- **Supersede:** toda a linha de **pintura / brush engine** —
  [ADR-0043](0043-painter-contract.md)..[ADR-0053](0053-painter-tier-policy.md) (cascata Painter, **nas partes
  de pintura**: brush engine, stamp/dab, stroke history + WAL/durabilidade, input de stroke, MCP de stroke,
  contratos congelados de pincel),
  [ADR-0097](0097-brush-engine-procreate-parity-cpu-first-dab-pipeline.md) (brush engine CPU-first, paridade
  Procreate) e [ADR-0098](0098-painter-canvas-gpu-spike-no-go.md) (spike canvas GPU). **MANTÉM como histórico**
  (não apaga os ADRs; ficam como registro da pesquisa/decisões anteriores). Conjuga com
  [ADR-0096](0096-remove-watercolor-fluid-pivot-mixer-brush.md) (que já removera a simulação de aquarela): este
  ADR remove o restante — todo o ato de **pintar**. **Revoga também [ADR-0067](0067-brush-traits-decoupling.md)
  §2.6** (crate `ph2d-brush-traits` + `BrushEngine`/`StampSpec`): decoupling Painter↔Vector da era do brush
  engine, órfão (o nó vetor `pattern-along-path` nunca o consumiu — o bridge raster era Painter-gated, nunca
  wirado); removido junto, e seu gate em `ph2d-vector-doc` foi retirado.
- **PRESERVA (intacto e funcional):** o **sistema de layers + efeitos** — `LayerStack` (layers, máscaras,
  grupos, blend modes, visibilidade/opacidade), **adjustment layers** (HSB, Curves, Levels, Channel Mixer,
  Selective Color, Gradient Map, Gaussian/Motion blur, Sharpen, Chromatic, Bloom, Shadows/Highlights, Color
  Lookup, Halftone, Noise…), o **compositor CPU de referência** e o **compositor GPU 22-modos** do
  `ph2d-render`, o **undo estrutural** de layer, e o **Apply** (bake do composite no sprite). O painel de
  Layers (`ph2d-panel-painter-layers`) continua a UI viva da feature.
- **NÃO afeta:** módulo vetor, BgRemoval (incl. seu protect-brush/eyedropper, que NÃO são pintura), demais
  tools de imagem, render de sprite, `ph2d-vector::Brush` (primitivo de fill/stroke do Vello — não é o brush
  engine).

## 1. Contexto

Após [ADR-0096](0096-remove-watercolor-fluid-pivot-mixer-brush.md) (remoção da aquarela) e o spike NO-GO de
canvas GPU ([ADR-0098](0098-painter-canvas-gpu-spike-no-go.md)), o Enio decidiu **remover por completo a
ferramenta de pintura** — o pincel e qualquer implementação relacionada a pintar — mantendo o sistema de
**layers + efeitos** como feature usável. O Painter deixa de ser um editor de pintura e passa a ser um
**host de layers + adjustments** (composição não-destrutiva sobre o sprite + Apply).

O desafio não era "deletar crates": as fronteiras passavam **por dentro** das crates. Os efeitos
(`adjustments/` + `blend.rs`) moravam **dentro** da crate de pincel `ph2d-painter-brush`; o sistema de layers
morava **dentro** de `ph2d-tool-painter` (que importava o brush engine); a persistência de layer morava em
`ph2d-painter-stroke`. Foi preciso **extração cirúrgica** antes de deletar, senão os layers/efeitos cairiam
junto.

## 2. Decisão

1. **Extrair os efeitos** para uma crate nova **`ph2d-painter-effects`** (deps: só `ph2d-color` + `serde`):
   recebe `adjustments/` (todos os adjustment kinds + kernels CPU) e `blend.rs` (os 22 blend modes W3C +
   `apply_blend`). Cycle-free por construção, consumida por `ph2d-tool-painter` e pelos testes de paridade do
   `ph2d-render`.
2. **Esvaziar `ph2d-tool-painter`** para um host de layers/efeitos: remover `PainterTool` campos e métodos de
   pintura (scheduler, brush, stroke, wash/wet, journal/WAL, color picker), o `apply_ui_edit`/`lifecycle`, e
   o input de stroke. Manter `layers/`, `compositor/`, edição de adjustment, undo **estrutural**, `RasterEditTool`
   (set_source/preview/run_full/Apply). O undo passa a ser **só estrutural** (sem dep de stroke).
3. **Deletar 8 crates** de pintura: `ph2d-painter-brush`, `ph2d-painter-stroke`, `ph2d-tool-brush`,
   `ph2d-panel-painter-sidebar`, `ph2d-panel-brush-studio`, `ph2d-brush-along-path`, `ph2d-brush-traits`,
   `ph2d-painter-contracts`.
4. **Shell desktop:** remover o input de stroke (`input_dispatch/painter_input.rs` + dispatch) e o color
   picker; **manter** o pipeline de preview/flatten/Apply de layer (`painter_gpu_flatten`/`painter_gpu_preview`/
   `painter_bridge`) repontado para `ph2d-painter-effects`.
5. **Default tool:** o boot/fallback era o `BrushTool` (deletado) → passa a ser o **`MoveTool`**
   (`is_default = true`).
6. **Registries:** `ph2d-tool-sync` + `ph2d-panel-sync` regeneram `tool-registry-init`/`panel-registry-init`
   sem as crates removidas (o `painter` tool e o painel de layers permanecem).

## 3. Persistência / contratos

- **Sem migração, sem bump de schema.** `PaintProject` / WAL / autosave / recovery (a durabilidade de stroke,
  ADR-0046/0052) **não tinham consumidor externo** (nem desktop, nem `ph2d-save`, nem `ph2d-host`): os layers
  vivem em memória e são "baked" no sprite via **Apply** → persistem pelo pipeline normal de asset. Removidos
  junto com a crate.
- **Gate `architecture_painter_contract_surface`** (crate `ph2d-painter-contracts`) **removido**: os ABIs
  congelados de pintura (`PainterUiEdit`/`Brush`/`Stamp=96B`/`RenderingMode=6`/`PointerSource`/`DeviceTier`…)
  deixam de existir. A superfície de efeitos que sobrevive (`AdjustmentKind`/`AdjustmentParams`/`BlendMode`)
  vive agora em `ph2d-painter-effects`; capear esses limites de novo, se desejado, é follow-up.
- **Nomes `painter` mantidos** no que sobrevive (crate `ph2d-tool-painter`, painel `painter-layers`, chrome ids
  `PAINTER_*`, IconId `Painter`, magic de save) — decisão deliberada de **minimizar churn** em superfícies
  sensíveis (ordem alfabética de IconId, gate de colisão de NodeId, magic de save). Os chrome ids
  `PAINTER_STUDIO_*`/`PAINTER_SIDEBAR_*` agora órfãos ficam como constantes inertes (espelhadas no teste de
  colisão hand-maintained); não são "implementação de pintura".

## 4. Consequências

- **Layers + efeitos 100% funcionais:** ao ativar o tool "painter" abre o painel de Layers; criar/duplicar/
  reordenar/agrupar layer, máscaras, blend/opacidade/visibilidade, adjustment layers (curves/HSB/blur/bloom/…),
  undo/redo estrutural, preview composto (CPU + GPU), e **Apply** (bake do composite no sprite) — tudo preservado.
- **Pintar não faz nada:** o input de ponteiro sobre o canvas não deposita stamps (não há brush). Conteúdo de
  raster layer vem de **import de imagem** / Apply, não mais de pincel.
- **`ph2d-render` desacoplado do pincel:** a dep era dev-only (testes de paridade); repontada para
  `ph2d-painter-effects`. O compositor GPU de produção é auto-contido (fala códigos `u8` crus).
- **Gate de perf do compositor GPU endurecido:** `tests/layer_compositor_gpu.rs::measure_composite` passou a
  reportar o **mínimo** (floor do hardware) em vez da mediana — robusto à contenção de GPU compartilhada/headless
  (mediana/cauda inflam por scheduling do SO; a regressão real — dirty-rect → full-recompose — também sobe o
  floor, então o gate segue válido).

## 5. Verificação

- `cargo check --workspace` verde; `cargo clippy` (crates afetadas) sem warnings; `cargo machete` limpo.
- `cargo test -p ph2d-tool-painter` (layers/máscaras/grupos/adjustment/undo) verde; `ph2d-editor-core` arch
  gates verdes; `ph2d-tool-registry-init` (cluster/alfabético) + `ph2d-panel-registry-init` (EXPECTED_TYPED)
  verdes.
- `ph2d-render` paridade GPU↔CPU dos efeitos (Metal headless, `--ignored --release`): **26/26** —
  confirma que a extração de `ph2d-painter-effects` é bit-correta contra o compositor GPU.
