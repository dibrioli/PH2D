# ARQUIVO — CLAUDE.md §6 e §7 (texto de 2026-10-02, verbatim)

> ⚠️ **Isto NÃO é o estado atual.** É o texto das §6 e §7 do `CLAUDE.md` como estava no `main` @ `faad97fa5`,
> copiado sem edição (a única alteração: os alvos de link `docs/…` e `crates/…` reancorados para esta
> pasta, como o `doc-split.py` faz). O vivo foi encurtado (as emendas do `Tool` 10→11→12 e o detalhe das revogações
> ficam aqui). Os contratos continuam os mesmos: o vivo é o [`CLAUDE.md §6`](../../../CLAUDE.md).

---

## §6 — Contratos congelados (mexer = Coord-only + ADR; DIRETRIZ §4)

- **Nodes** ([ADR-0039](../../../docs/architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md)): `NodeOp=2`/`OpResolver=1`/`NodeManifest=8` — gate `architecture_contract_surface`.
- **Tools** ([ADR-0040](../../../docs/architecture/decisions/0040-tool-as-isolated-feature-crate.md)+[0041](../../../docs/architecture/decisions/0041-rasteredit-rename-and-deactivate.md)): `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4` — gate `architecture_tool_contract_surface`. (`Tool` 10→11 em [ADR-0040-amendment-2](../../../docs/architecture/decisions/0040-tool-as-isolated-feature-crate.md): `on_tick` heartbeat p/ aquarela live, ADR-0049/0077-D11. `Tool` 11→12 + sub-trait `CanvasPaintTool` em ADR-0040-amendment-3: `as_canvas_paint_mut`/`on_canvas_pointer` p/ entrega de ponteiro de canvas ao novo Painter, `docs/Painter/`.)
- ~~**Painter (pintura)** (ADR-0043..0053)~~ — **REVOGADO** por [ADR-0099](../../../docs/architecture/decisions/0099-remove-painting-brush-engine-preserve-layers-effects.md): os ABIs de pintura (`PainterUiEdit`/`Brush`/`Stamp=96B`/`RenderingMode=6`/`PointerSource`/`DeviceTier`…) e o gate `architecture_painter_contract_surface` (crate `ph2d-painter-contracts`) foram **removidos** junto com a pintura. A superfície de **efeitos** que sobrevive (`AdjustmentKind≤32`/`AdjustmentParams`/`BlendMode`+`MAX_BLEND_MODES`/`apply_blend`) vive agora em **`ph2d-painter-effects`** (não-gateada; re-capear é follow-up). `ColorProfile` vive em **`ph2d-imageio`** ([`color.rs`](../../../crates/ph2d-imageio/src/color.rs), com gate `architecture_imageio_contract_surface`) — ⚠️ **não** em `ph2d-color`, que nunca o teve.
- ~~**Watercolor (física)** (ADR-0049/0078-0084)~~ — **REVOGADO** por [ADR-0096](../../../docs/architecture/decisions/0096-remove-watercolor-fluid-pivot-mixer-brush.md): a sim de aquarela e seus gates (`gpu_parity`/`composite_parity`) foram removidos junto com a crate `ph2d-painter-wash`. O modelo K–M espectral é histórico (backup); o pivot para mixer-brush usa Kubelka–Munk/Mixbox no blend do pigmento, não shallow-water. Nada congelado aqui.
- **Vector (data-model foundational)** ([ADR-0056..0068](../../../docs/architecture/decisions/)): `VectorOp≤16`/`Vertex`SmallVec32/`Segment`64/`Region.segments`16/`AnimValue` enum/`sample(t:f64)`/`MAX_SPIRAL_TURNS=64`/`MAX_POLYGON_SIDES=128`/`MAX_VERTICES_PER_LLM_GEN=1000` — gate `architecture_vector_contract_surface` (escaneia só `ph2d-vector-doc`+`-traits`). **PERMANECE congelado** e o gate FICA — mesmo após [ADR-0108](../../../docs/architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md) ter **retirado** as tools/nodes/panels de edição vetorial (o cutover mexe só em crates satélite, não na superfície do doc). O **motor novo** (`ph2d-vec-*`, §5) tem contrato **próprio, ainda NÃO congelado** (re-congelar é follow-up). Gate `vello_kurbo_only_in_ph2d_vector` nunca existiu (era W2-deferred).

## §7 — Design system

[`docs/design/PROMPT_CLAUDE_DESIGN.md`](../../../docs/design/PROMPT_CLAUDE_DESIGN.md) (brief: tokens.json + mockups + icons + specs) alimenta os widgets em Vello sobre [`ph2d-editor-core`](../../../crates/ph2d-editor-core/) (ADR-0023). Mockup de referência: [`docs/design/component-library.html`](../../../docs/design/component-library.html).
