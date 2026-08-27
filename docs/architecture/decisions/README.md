# ADRs — índice

> ⚠️ **Gerado por `bash scripts/adr-index.sh` — não edite à mão.** Uma lista de 160 itens
> mantida à mão envelhece na primeira semana; esta é derivada do cabeçalho de cada ADR.
>
> O **estado por-módulo** vive no [`CLAUDE.md §5`](../../../CLAUDE.md); os **contratos
> congelados**, no §6. Um ADR descreve a decisão **no dia em que foi tomada** — use-o para
> responder *"por que isto ficou assim?"*, nunca para decidir a próxima ação.

| # | Status | Decisão |
|---|---|---|
| [0003](0003-ecs-choice.md) | Accepted (rev2) | Escolha de ECS — bevy_ecs 0.18 |
| [0019](0019-spike-scripting-output.md) | Accepted | Output do spike de scripting (2026-05) |
| [0020](0020-amendment-1.md) | Accepted (2026-05-29) | ADR-0020 Amendment 1 — Metal Direct Overlay + `PlatformHost::register_metal_overlay()` extension |
| [0020](0020-surface-lifecycle.md) | Accepted | Surface lifecycle e device-lost recovery |
| [0021](0021-simulation-presentation-boundary.md) | Accepted | Fronteira simulation ↔ presentation (SubWorld pattern) |
| [0022](0022-no-hashmap-in-simulation.md) | Accepted | Banimento de `HashMap`/`HashSet` iteration em simulation crates |
| [0023](0023-ui-ux-baseline.md) | Accepted | UI/UX baseline — designer-first, Procreate-inspired, WCAG 2.2 AA |
| [0024](0024-editor-input-and-widget-state.md) | Accepted (2026-05-10 — Enio aprovou Modelo B com plano de conformidade HR-3 detalhado) | Editor input pipeline + retained widget state |
| [0025](0025-amendment-1.md) | Accepted (2026-05-28) — ratificado pelo Enio junto com ADRs 0069..0074 pós 5 lentes … | ADR-0025 amendment-1 — Transform 2D skew (skew_x, skew_y) — bump `Transform::VERSION` 1→2 |
| [0025](0025-gameobject-model.md) | Accepted | Modelo de GameObject — ECS-composition (Unity-style) sobre bevy_ecs |
| [0026](0026-sprite-source-strategies.md) | Accepted | Pluggable sprite source strategies (M14.5) |
| [0027](0027-convention-by-discovery.md) | Accepted | Convention-by-discovery + Shell decomposition + HR-18 |
| [0028](0028-wave-2-codegen-design-canonical.md) | Accepted | Wave 2: build-time codegen + design canonical sources + lint guards |
| [0029](0029-trait-driven-panel-host.md) | Closed | Trait-driven panel host (PanelHost + Panel<State>) — endgame post-Wave-7 |
| [0030](0030-multi-domain-node-engine.md) | Accepted (ratificado pelo Enio 2026-05-21; … | Multi-domain node engine: substrato unificado + avaliadores plurais + membrana como tipo |
| [0031](0031-node-and-tool-as-feature-unit.md) | Accepted (ratificado pelo Enio 2026-05-21; … | Nó e ferramenta como unidade de feature (isolamento FBP = unidade multi-agente) |
| [0032](0032-nodegraph-substrate.md) | Accepted (ratificado pelo Enio 2026-05-21; … | `ph2d-nodegraph`: substrato unificado (atributos, portas algébricas, efeitos, formato textual) |
| [0033](0033-shared-compute-expr.md) | Accepted (ratificado pelo Enio 2026-05-21; … | `ph2d-expr`: substrato de compute compartilhado (Fields + escape textual → WGSL\|Luau) |
| [0034](0034-plural-evaluators.md) | Accepted (ratificado pelo Enio 2026-05-21; … | Avaliadores plurais por modelo-de-avaliação + lowering plural por nó |
| [0035](0035-cook-vs-live-and-attribute-stream.md) | Accepted (ratificado pelo Enio 2026-05-21; … | Cook vs live por-subgrafo; stream de atributos ≠ ECS; cloner |
| [0036](0036-gameplay-authoring-blocks-and-nodes.md) | Accepted (ratificado pelo Enio 2026-05-21; … | Autoria de gameplay: blocos e node-programming → Luau; colisão 2D lite |
| [0037](0037-stable-entity-wire-id-scenedoc.md) | Accepted (ratificado pelo Enio 2026-05-21; … | Id de entidade estável no SceneDoc (postcard), desacoplado do `to_bits` do bevy |
| [0038](0038-artist-first-node-ux.md) | Accepted (ratificado pelo Enio 2026-05-21; … | UX baseline de nós artista-primeiro (esconder contexto, viewport-first, presets, ferramentas terminais) |
| [0039](0039-nodegraph-contract-freeze-w2t4.md) | Accepted (2026-05-22) | FREEZE do contrato de nós (`ph2d-nodegraph` + `ph2d-expr`) — W2.T4 |
| [0040](0040-amendment-1.md) | Accepted (2026-05-29) | ADR-0040 Amendment 1 — `EditorAction` cap-bump 4 → 5 (Vector Module `VectorOp` variant) |
| [0040](0040-tool-as-isolated-feature-crate.md) | Accepted (implementado e FREEZE ratificado 2026-05-22 — vide §7 Histórico de execução) | Ferramenta como crate de feature isolado (contrato `Tool`/`ImageEditTool` + canal de ação genérico + registro por codegen) |
| [0041](0041-rasteredit-rename-and-deactivate.md) | Accepted (2026-05-23) | Amendment to ADR-0040: `ImageEditTool` → `RasterEditTool` + `deactivate` method |
| [0042](0042-wave-10-closure.md) | Accepted (2026-05-24) | Wave 10 closure: gates ortogonais, typed color, fan-out drop-crate consolidated |
| [0043](0043-painter-contract.md) | ⛔ superseded por 0099 · Accepted (2026-05-26) | Painter contract (`ph2d-tool-painter` surface + caps) |
| [0044](0044-amendment-1.md) | ⛔ superseded por 0077, 0099 · Accepted | ADR-0044 Amendment 1 — `Stamp._pad` → `roundness` (W2.9 shape squash) |
| [0044](0044-brush-engine-gpu.md) | ⛔ superseded por 0077, 0099 · Accepted (2026-05-26) | Brush Engine GPU contract (Brush + Stamp + Mixbox + Procedural Grain) |
| [0045](0045-adjustment-layers.md) | ⛔ superseded por 0099 · Accepted (2026-05-26) | Adjustment Layers contract (12 non-destructive + 5 destructive-only) |
| [0045](0045-amendment-1.md) | ⛔ superseded por 0099 · Accepted | ADR-0045 Amendment 1 — AdjustmentLayer crate-placement reconciliation |
| [0046](0046-amendment-1.md) | ⛔ superseded por 0099 · Accepted (2026-05-31) | ADR-0046 Amendment 1 — `.ph2d-painter` v2: layer-stack persistível (`LayerStackEntry::Node`) |
| [0046](0046-stroke-vector-history.md) | ⛔ superseded por 0099 · Accepted (2026-05-26) | Stroke Vector History format (`.ph2d-painter` v1) |
| [0047](0047-painter-mcp-stroke-engine.md) | ⛔ superseded por 0099 · Accepted (2026-05-26) | Painter MCP Stroke Engine (4 tools + governance HR-11) |
| [0048](0048-stroke-inspector.md) | ⛔ superseded por 0099 · Accepted (2026-05-26) | Stroke Inspector retroativo (W14) |
| [0049](0049-fluid-brushes.md) | ⛔ superseded por 0078, 0085, 0099 · Accepted (2026-05-26) | Fluid Brushes Extension (`ph2d-painter-fluid`, opt-in W15) |
| [0050](0050-device-heterogeneity-layer.md) | ⛔ superseded por 0099 · Accepted (2026-05-26) | Device heterogeneity layer (pointer source + per-device curves + palm rejection + driver quirks) |
| [0051](0051-color-profile-pipeline.md) | ⛔ superseded por 0099 · Accepted (2026-05-26) | Color profile pipeline (sRGB / Linear scRGB / Display P3 / ProPhoto / OKLab invariants) |
| [0052](0052-tear-resistant-stroke.md) | ⛔ superseded por 0099 · Accepted (2026-05-26) | Tear-resistant stroke commit + crash recovery (journal + auto-save + suspend handling) |
| [0053](0053-cross-platform-tier.md) | ⛔ superseded por 0085, 0099 · Accepted (2026-05-26) | Cross-platform tier policy (universal feature parity + graceful degrade) |
| [0054](0054-imageio-pipeline.md) | Accepted (W0 fechada 2026-05-26 — T1-T6 ✅ shipped + auditoria 5-lente remediada + s… | Image I/O pipeline (contrato `ImageImporter`/`ImageExporter` + canal genérico + registro por codegen) |
| [0055](0055-cooked-texture-compression-pipeline.md) | Accepted (v4 enxuta, 2026-05-27) | Cooked Texture Compression Pipeline |
| [0056](0056-amendment-2.md) | ⛔ superseded por 0108 · Accepted (2026-05-28) | amendment-2 — Bounded-decode caps extension (7 new `AssetBounds` defaults) |
| [0056](0056-amendment-3.md) | ⛔ superseded por 0108 · Accepted (2026-06-05) | amendment-3 — Region→procedural-fill reference (W6 shader graph / W7 diffusion) |
| [0056](0056-vector-network-data-model.md) | ⛔ superseded por 0108 · Accepted (2026-05-29) | Vector Network data model (topologia + dual-representation + edit log) |
| [0057](0057-vector-edit-dispatch-crdt.md) | ⛔ superseded por 0108 · Accepted (2026-05-29) | Vector edit dispatch + CRDT data model |
| [0058](0058-amendment-1.md) | ⛔ superseded por 0108 · Accepted (2026-06-03) | amendment-1 — Vector domain cook payload: type-erased opaque geometry value |
| [0058](0058-vector-geometry-graph.md) | ⛔ superseded por 0108 · Accepted (2026-05-29) | Vector geometry graph (domain `vector` em `ph2d-nodegraph`) |
| [0059](0059-vector-renderer-pipeline.md) | ⛔ superseded por 0108 · Accepted (2026-05-29) | Vector renderer pipeline (Vello + GPU stroke + draft+reconcile boolean) |
| [0060](0060-vector-procedural-fill.md) | ⛔ superseded por 0108 · Accepted (2026-05-29) | Procedural fill shader graph (topology vs UBO + diffusion curve Poisson) |
| [0061](0061-vector-llm-authoring.md) | ⛔ superseded por 0108 · Accepted (2026-05-29) | Vector LLM authoring (MCP tools + LLM4SVG semantic tokens + sanitizer) |
| [0062](0062-painter-vector-bridge.md) | ⛔ superseded por 0108 · Accepted (2026-05-29) | Painter ↔ Vector bridge bidirecional + `ph2d-brush-traits` decoupling |
| [0063](0063-vector-runtime-physics-dormant-fractures.md) | ⛔ superseded por 0108 · Accepted (2026-05-29) | Vector runtime + Dynamic Physics Colliders + Dormant Fractures + LOD |
| [0064](0064-vector-multi-platform-input.md) | ⛔ superseded por 0108 · Accepted (2026-05-29) | Vector multi-platform input (Pencil/Wacom/S Pen + predict+reconcile + Metal overlay) |
| [0065](0065-vector-sdf-hybrid-gpu.md) | ⛔ superseded por 0108 · Accepted (2026-05-29) | Vector-SDF Hybrid GPU Pipeline (boolean 120 FPS via min/max compute) |
| [0066](0066-variable-font-glyph-as-vector-network.md) | ⛔ superseded por 0108 · Accepted (2026-05-29) | Variable Font Glyph as Vector Network (typography generativa) |
| [0067](0067-brush-traits-decoupling.md) | ⛔ superseded por 0099, 0108 · Accepted (2026-05-29) | `ph2d-brush-traits` decoupling crate (resolve circular dep Painter ↔ Vector) |
| [0068](0068-mobile-core-tier.md) | ⛔ superseded por 0108 · Accepted (2026-05-29) | DeviceTier Mobile Core (Vector Module <12 MB rival Rive) |
| [0069](0069-sprite-inspector-v2.md) | Accepted (2026-05-28) — ratificado pelo Enio pós 5 lentes adversariais (147 findings… | Sprite Inspector v2 (decisão-mãe) |
| [0070](0070-amendment-2.md) | Accepted (W0 carry-over, 2026-05-28) | amendment-2 — Wrapper enum is the SOLE back-compat path (empirical T0.13 finding) |
| [0070](0070-amendment-3.md) | Accepted (W1.T1.10/T1.11 enabler, 2026-05-28) | amendment-3 — `RenderInstance.flip_uv` is a general flags bitfield (packs `flip_x`, `flip_y`, `tint_fill`) |
| [0070](0070-amendment-4.md) | Accepted (W2.T2.x skew render step, 2026-05-30) | amendment-4 — `RenderInstance.rotation: f32` → `basis: [f32; 4]` (render skew as a true parallelogram) |
| [0070](0070-amendment-5.md) | Accepted (W3.T3.11, 2026-05-30) — render shipado + smoke do Enio (§9 Sampling demons… | amendment-5 — `RenderInstance.sampling: u32` (per-node TextureFilter/Repeat, CPU-only) |
| [0070](0070-amendment-6.md) | Accepted (W3 §9, 2026-05-30) — render shipado + smoke do Enio (TextureRepeat demonst… | amendment-6 — `UvTransform` component + `RenderInstance.uv_xform: [f32;4]` GPU @location(15) (UV tiling/scroll) |
| [0070](0070-amendment-7.md) | Accepted (W3 Phase 6, 2026-05-30) — ratificado pelo Enio (smoke ClipChildren + Mask I… | amendment-7 — `RenderInstance` CPU-tail `clip_group` + `clip_meta` (ClipChildren + Mask2D stencil grouping) |
| [0070](0070-amendment-8.md) | Accepted (ADR-0164 F1 passo 6, 2026-08-25) — implementado e verde; … | amendment-8 — `Sprite` v4 → **v5**: sete campos saem para três componentes (20 → 13) |
| [0070](0070-amendment-9.md) | Accepted (doc 89 folha 17, 2026-08-25) — pendente de smoke do Enio (cena `=98`). | amendment-9 — `RenderInstance` CPU-tail `sub_order` (a sub-ordem DENTRO de uma fatia de `z_order`) |
| [0070](0070-sprite-schema-v4.md) | Accepted (2026-05-28) — ratificado pelo Enio pós 5 lentes adversariais. | Sprite schema v4 (`Sprite::VERSION` 3→4) + RenderInstance ABI bump |
| [0071](0071-amendment-1.md) | Accepted (ADR-0164 F1 passo 6, 2026-08-25). | amendment-1 — o 4.º canal de tinta muda de CASA: `per_corner_tint` → `SpriteCornerTint` |
| [0071](0071-tint-channels-multiplicative.md) | Accepted (2026-05-28) — ratificado pelo Enio pós 5 lentes adversariais. | Tint channels — matemática multiplicativa canônica (4 canais) |
| [0072](0072-amendment-1.md) | Accepted (2026-08-22, `line/Sprite`) — implementado, com gate de paridade entre as du… | amendment-1 — Montar numa âncora é um QUADRO na hierarquia (o consumidor do §2.6) |
| [0072](0072-named-anchor-unification.md) | Accepted (2026-05-28) — ratificado pelo Enio pós 5 lentes adversariais. | Named Anchor unification (socket + slice + image_point num único tipo) |
| [0073](0073-amendment-1.md) | Accepted (W3.T3.8/T3.20, 2026-05-30) — pipeline canônico shipado + golden-hash deter… | amendment-1 — Z bucketiza ANTES do YSort (reconcilia spec §5.1 lista vs §5.2 passo-4) |
| [0073](0073-amendment-2.md) | Accepted (sorting audit 2026-05-31) — fix implementado, golden intacto, repro test verde. | amendment-2 — Y-Sort self-inclusive (o próprio sprite participa, não só via ancestral) |
| [0073](0073-sorting-canonical-order.md) | Accepted (2026-05-28) — ratificado pelo Enio pós 5 lentes adversariais. | Sorting canonical order (Z + ZAsRelative + YSort + SortingGroup + ShowBehindParent + DFS) |
| [0074](0074-amendment-1.md) | Accepted (W3 Phase 6, 2026-05-30) — ratificado pelo Enio (smoke ClipChildren + Mask I… | amendment-1 — Clip & Mask = stencil (não back-buffer); `Mask2D` como Component opcional mínimo |
| [0074](0074-sprite-component-boundary.md) | Accepted (2026-05-28) — ratificado pelo Enio pós 5 lentes adversariais. | Sprite struct vs Component ECS — princípio operacional |
| [0075](0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md) | Accepted | Arquitetura de paralelismo multi-agente — ECS-decoupling + build-speed, NÃO plugins em runtime |
| [0076](0076-vector-as-scene-object.md) | Accepted (2026-06-02) | Vector como objeto de cena (Rank 10): entidade ECS + offset no render, sem desgelar o schema |
| [0077](0077-brush-engine-physics-overhaul.md) | Accepted — pressure (D1), wet/burnt edges + intensity slider (D2/D3), pixel-relative … | Brush engine physics overhaul (pressure, wet/burnt edges, dab AA, taper) |
| [0078](0078-watercolor-gold-standard-resident-tiled-shallow-water.md) | ⛔ superseded por 0086 · Proposed (2026-06-08) — North Star ratificado pelo Enio ("chegue ao padrão ouro, vá… | Watercolor Gold Standard: GPU-Resident, Tiled-Sparse, Three-Layer Shallow-Water |
| [0079](0079-watercolor-params-per-brush-exposure.md) | ⛔ superseded por 0086 · Accepted (2026-06-08) — ratificado pelo Enio ("Quero todos os controles expostos ao u… | Watercolor Params: per-brush exposure (full artist control) |
| [0080](0080-watercolor-km-multipigment-field.md) | ⛔ superseded por 0085, 0086 · Accepted (2026-06-09) — pedido pelo Enio (a "mágica da aquarela que falta": azul + a… | Watercolor: Kubelka–Munk multi-pigment wet-on-wet field |
| [0081](0081-watercolor-real-pigment-palette.md) | ⛔ superseded por 0085, 0086 · Accepted (2026-06-09) — pedido pelo Enio após o smoke OK do K–M ([ADR-0080](0080-w… | Watercolor: real artist-pigment palette (granulation · staining/lift · transparency) |
| [0082](0082-watercolor-branched-capillary-fringe.md) | ⛔ superseded por 0085, 0086 · Accepted (2026-06-09) — pedido pelo Enio (#2 da fila): "MoXi/LBM na franja capilar | Watercolor: branched (fiber-channeled) capillary fringe — opt-in |
| [0083](0083-4k-fullres-watercolor-field-residency.md) | ⛔ superseded por 0086 · Accepted (2026-06-09) — pedido pelo Enio (#3 da fila): "4K full-res GPU-residency do … | 4K full-res watercolor field residency (lift the storage-buffer cap) |
| [0084](0084-watercolor-backdrop-lift.md) | ⛔ superseded por 0086 · Accepted (2026-06-09) — o Enio testou o `Lift` (ADR-0081) e não viu efeito. … | Watercolor: backdrop lift (wet brush re-mobilizes dry paint) — opt-in |
| [0085](0085-watercolor-v2-gpu-first-realtime.md) | ⛔ superseded por 0086, 0096 · Accepted (2026-06-10) — ratificado pelo Enio (R0 smoke: "ratifico! GO"). … | Watercolor v2: arquitetura GPU-first real-time (supersede a paridade bit-a-bit CPU↔GPU) |
| [0086](0086-watercolor-minimal-core-wash.md) | ⛔ superseded por 0094, 0096 · PROPOSTO (aguarda ratificação do Enio antes de codar) | Núcleo mínimo de aquarela (`ph2d-painter-wash`): difusão-gateada + edge-darkening, GPU, RGB linear |
| [0087](0087-wash-integration-parallel-watercolor-mode.md) | ⛔ superseded por 0096 · IMPLEMENTADO v1 (Fases 1-3, 2026-06-13 — ratificado pelo Enio) | Integração do `ph2d-painter-wash` como modo de aquarela PARALELO (lado-a-lado com o v2) |
| [0088](0088-wash-persistent-pigment-canvas-and-undo.md) | ⛔ superseded por 0089, 0090 · ACEITO (Enio 2026-06-13), em implementação. | Wash: canvas de pigmento persistente + undo por snapshot de campo |
| [0089](0089-wash-dual-field-faithful-color-and-synchronous-undo.md) | ⛔ superseded por 0090, 0091 · ACEITO (Enio 2026-06-13), em implementação. | Wash: campo DUPLO (concentração + dye RGB), cor fiel, e undo síncrono |
| [0090](0090-wash-event-driven-undo-rebuild.md) | ⛔ superseded por 0091, 0096 · ACEITO (Enio 2026-06-14), implementado. … | Wash: undo/redo reconstruído (pilha-dupla por EVENTOS, snapshots esparsos) |
| [0091](0091-wash-mixbox-residual-faithful-pigment-color.md) | ⛔ superseded por 0094, 0095, 0096 · ACEITO (Enio 2026-06-14), implementado. | Wash: residual Mixbox no K–M (cor escolhida fiel, mistura espectral) |
| [0092](0092-wash-capillary-fringe-realistic-deposition-edge.md) | ⛔ superseded por 0094, 0095, 0096 · ACEITO (Enio, 2026-06-14); … | Wash: borda de aquarela (edge-darkening rim + franja capilar) |
| [0093](0093-gpu-resident-painter-canvas.md) | ⛔ superseded por 0094, 0095, 0096 · ACEITO (Enio, 2026-06-14); … | Canvas residente na GPU (Painter) |
| [0094](0094-wash-gpu-resident-simplified-core.md) | ⛔ superseded por 0095, 0096 · ACEITO (Enio, 2026-06-14). | Wash GPU-residente (núcleo simplificado, tempo-real-only) |
| [0095](0095-wash-curtis-gd-deposition-topology.md) | ⛔ superseded por 0096 · ACEITO (Enio, 2026-06-15). | Wash: topologia Curtis `g`/`d` (suspenso/depositado) com TransferPigment |
| [0096](0096-remove-watercolor-fluid-pivot-mixer-brush.md) | ⛔ superseded por 0099 · ACEITO (Enio, 2026-06-15). | Remoção da simulação de aquarela/fluido; pivô para mixer-brush + Mixbox |
| [0097](0097-brush-engine-procreate-parity-cpu-first-dab-pipeline.md) | ⛔ superseded por 0099 · ACEITO (Enio, 2026-06-16) — CPU-first ratificado (§6). … | Brush Engine: paridade Procreate, dab pipeline CPU-first |
| [0098](0098-gpu-resident-canvas-spike-no-go-cpu-first-stands.md) | ⛔ superseded por 0099 · ACEITO (Enio, 2026-06-16 — escolheu "spike de viabilidade primeiro"). | Spike canvas GPU-residente: NO-GO agora; CPU-first (ADR-0097) mantido |
| [0099](0099-remove-painting-brush-engine-preserve-layers-effects.md) | ACEITO (Enio, 2026-06-20). | Remoção da ferramenta de pintura / brush engine; preservar layers + efeitos |
| [0100](0100-dual-texture-slots-shape-grain.md) | Accepted (implementado 2026-06-25; … | Dois slots de textura no brush: Shape (silhueta) + Grain (textura) |
| [0101](0101-rake-heading-on-dab-length-weighted-ema.md) | Accepted (implementado 2026-06-26; … | Rake: heading como propriedade do Dab (EMA length-weighted no motor) |
| [0102](0102-inpaint-multiscale-patchmatch-cpu-gpu.md) | Accepted (pesquisa 2026-07-02; … | Inpaint: PatchMatch multi-escala (referência CPU + compute GPU reconciliado) |
| [0103](0103-selection-system-procreate-parity.md) | Accepted (Enio, 2026-07-02) | Selection system (Procreate parity), snapshot-integrated undo |
| [0104](0104-hardware-tiered-speed-strategy.md) | Accepted (Enio, 2026-07-04) | Speed strategy is hardware-tiered, not fixed to the 8 GiB Mac |
| [0105](0105-file-loc-cap-600-to-700.md) | Accepted (Enio, 2026-07-04) | Workspace file LOC cap raised 600 → 700 |
| [0106](0106-parallel-dev-lines-worktrees-workstation.md) | Accepted (Enio, 2026-07-05) | Linhas de desenvolvimento paralelas via `git worktree` no tier `workstation` (Modo L) |
| [0107](0107-concurrent-foundational-lines-tested-gate-syntactic-merge.md) | Accepted (Enio, 2026-07-05) | Linhas concorrentes em foundational: merge sintático (Mergiraf) + gate de integração testado (Modo L) |
| [0108](0108-vector-reposition-rive-referenced-native-editor-first.md) | ⛔ superseded por 0110, 0121, 0128 · ACEITO (Enio, 2026-07-05). | Reposicionamento do Vector: reimplementação nativa (ECS/Vello) do modelo Rive, editor-first; boolean edit-time; sem boolean em runtime |
| [0109](0109-rayon-exception-watercolor-composite.md) | ⛔ superseded por 0153 · ACEITO (Enio, 2026-07-07). | Exceção sancionada ao "sem rayon": composite óptico do Watercolor paralelizado (byte-idêntico, replay-safe) |
| [0110](0110-vector-nodes-are-ecs-entities-one-hierarchy.md) | ⛔ superseded por 0128 · aceito (Enio, 2026-07-09) | Nós vetoriais são entidades ECS: uma Hierarquia só |
| [0111](0111-vector-shapes-have-transforms-and-use-the-sprite-gizmo.md) | ⛔ superseded por 0128, 0153 · aceito (Enio, 2026-07-09) | Formas vetoriais têm `Transform`, e quem as manipula é o gizmo de sprite |
| [0112](0112-vector-select-node-pen-are-three-tools.md) | aceito (Enio, 2026-07-09) | Select, Node e Pen são três ferramentas, e o pivô nasce no centro da forma |
| [0113](0113-audio-export-ogg-vorbis-via-vorbis-rs-opus-deferred.md) | ⛔ superseded por 0116 · aceito (Enio, 2026-07-11 — "siga Export OGG/Opus") | Export comprimido de áudio: Ogg Vorbis via `vorbis_rs` (Opus adiado) |
| [0114](0114-grease-pencil-as-native-2d-medium-flip-no-3d-viewport.md) | aceito (Enio, 2026-07-11) | Grease Pencil vira o meio nativo 2D **"Flip"**; sem viewport 3D |
| [0115](0115-clip-composition-sequencer-overlap-crossfade-sparse-lanes.md) | ACEITO (ratificado pelo Enio, 2026-07-12) — em implementação | Composição de clips: faixas de instâncias, crossfade por sobreposição, canais esparsos |
| [0116](0116-audio-export-opus-isolated-unsafe-crate.md) | **ACEITO** — Enio autorizou em 2026-07-12 ("Opus, depois memória"). | Export Opus: `unsafe-libopus` + `ogg`, numa crate irmã isolada |
| [0117](0117-audio-editor-memory-is-measured-not-declared.md) | ACCEPTED | A memória do Audio Editor é MEDIDA, não declarada |
| [0118](0118-audio-streaming-voices-residency.md) | ⛔ superseded por 0119 · ACCEPTED | Vozes por STREAMING: o codec passa a valer para a RAM |
| [0119](0119-audio-loop-regions-in-the-mixer.md) | ACCEPTED (Enio, 2026-07-12) | Loop regions live in the mixer (and intro→loop falls out) |
| [0120](0120-audio-preview-is-a-buffer-you-own-not-a-buffer-you-rebuild.md) | Proposto (aguarda ratificação do Enio) — implementado e gateado | O preview é um buffer que você POSSUI, não um que você reconstrói |
| [0121](0121-vector-live-corners-authored-source-cooked-geometry.md) | ⛔ superseded por 0153 · aceito (implementado, pendente smoke do Enio) | Live Corners: o documento guarda a quina AFIADA; o mundo consome a COZIDA |
| [0122](0122-audio-spectral-fft-via-realfft.md) | **ACEITO** — Enio autorizou a dep em 2026-07-12. … | FFT para o módulo espectral de áudio (W5): `realfft`, em crate isolada |
| [0123](0123-audio-w7-ml-boundary-tract-native-denoise-reject-ort.md) | **ACEITO (direção)** — o Enio delegou a escolha ("padrão-ouro, custo à parte") e o | A fronteira ML do áudio (W7): denoise nativo via `tract` (opt-in), **`ort`/ONNX rejeitado** |
| [0124](0124-audio-a-range-edit-must-be-told-its-range.md) | accepted (implemented, `line/audio`, 2026-07-16) | A range edit must be **told** its range, not made to rediscover it |
| [0125](0125-audio-pricing-a-shipping-target-is-export-work-not-edit-work.md) | aceito (2026-07-16) | Precificar um shipping target é trabalho de EXPORT, não de edição |
| [0126](0126-gpu-node-kernels-are-side-metadata-contract-stays-frozen.md) | **ACEITO** (Enio, 2026-07-15: *"temos a liberdade para trabalhar na fundação **dessa … | O kernel GPU de um nó é metadata LATERAL (o contrato congelado NÃO é tocado) |
| [0127](0127-gpu-simulation-pre-is-arc-pingpong-plan-becomes-a-dag.md) | PROPOSTA (fatia 3 do briefing da Fase 2 — *"é uma extensão do MOTOR, não um port de | A simulação na GPU: o `pre` é ping-pong de `Arc`, o plano vira **DAG**, e o scrub fica no device |
| [0128](0128-vector-blend-object-live-virtual-steps-editable-spine.md) | aceito (desenho; … | Blend Object VIVO: passos virtuais, fontes editáveis, spine editável |
| [0129](0129-vector-envelope-warp-one-spine-cage-as-container-entity.md) | proposto | Envelope / Warp: UMA espinha (`sample + fit`), a gaiola é uma ENTIDADE-container, e os gestos são dois |
| [0130](0130-gpu-emitter-the-id-gather-is-arithmetic-because-the-window-is-dense.md) | PROPOSTA (a fatia que o [ADR-0127 D3](0127-gpu-simulation-pre-is-arc-pingpong-plan-beco… | O emitter na GPU: o gather por `id` é ARITMÉTICO porque a janela é densa (e uma propriedade PROVÁVEL de plano é o que separa isso de um bug mudo) |
| [0131](0131-physics-global-runtime-truth-rapier-ecs-bridge.md) | Accepted (direção — desenho de abertura W0; … | Física global: a simulação É o estado, o rígido primeiro, e a ponte tem UMA porta de escala |
| [0132](0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md) | proposto | Live Path Effects são uma PILHA por-path dentro do `cooked()`, não um grafo de nós — e a quina é o estágio ZERO |
| [0133](0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md) | proposto | Nesting: uma instância de container é um STRIP, e o relógio é do PAI |
| [0134](0134-wet-paint-fluid-sim-returns-cpu-first-parity-tested.md) | Accepted (ordem explícita do Enio, 2026-07-20; … | Wet Paint: a simulação de fluido VOLTA, CPU-first, com paridade testada — e o modo desligado é byte-idêntico |
| [0135](0135-gpu-sim-zone-is-a-conditional-passthrough-and-a-partial-claim-retreats.md) | PROPOSTA. … | A família `sim.zone` na GPU: o contêiner de laço de estado é um passthrough CONDICIONAL, e um claim parcial RECUA |
| [0136](0136-gpu-count-changing-nodes-order-preserving-compaction-hosted-counts.md) | aceito (implementado nesta linha, `line/gpu-nodes`) | GPU: nós que MUDAM CONTAGEM — compaction ordem-preservante, contagem no HOST, boundary estático é híbrido legal |
| [0137](0137-motion-scrub-rings-backfill-min-gap-thinning-byte-budget.md) | aceito (implementado nesta linha, `line/gpu-nodes`) | os rings de scrub aprendem: backfill + thinning por gap-mínimo com janela recente protegida + orçamento em BYTES |
| [0138](0138-motion-stream-columns-are-arc-shared-clone-is-a-refcount.md) | aceito (implementado nesta linha, `line/gpu-nodes`) | as colunas do `Stream` vivem atrás de `Arc`: clonar é refcount, escrever substitui |
| [0139](0139-gpu-voronoi-lloyd-via-jump-flooding-integer-centroids.md) | aceito (implementado nesta linha, `line/gpu-nodes`) | `motion.voronoi` na GPU: Lloyd via Jump Flooding, centroides em INTEIROS, e o cap de 600 cai |
| [0140](0140-gpu-multi-pass-kernels-neighborhood-sims-build-a-spatial-grid-on-device.md) | PROPOSTA. … | Kernels GPU multi-passe: a simulação de VIZINHANÇA (boids/collide/SPH) constrói uma grade espacial no dispositivo |
| [0141](0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md) | proposto — aguardando aceite do Enio (nada é construído antes) | Motion path: a posição é UM canal com trajetória, e eixos separados são um MODO |
| [0142](0142-timeline-onion-ghost-poses-non-destructive-pose-at.md) | aceito (provisório na `line/anim`; … | O onion da TIMELINE: poses-fantasma por `pose_at` não-destrutivo |
| [0143](0143-timeline-signals-a-marker-emits-a-decoupled-event-not-a-call.md) | aceito (provisório na `line/anim`; … | Sinais da timeline: um marker EMITE um evento desacoplado, nunca uma chamada |
| [0144](0144-timeline-expressions-frozen-ir-separate-post-composition-pass.md) | aceito (provisório na `line/anim`; … | Expressões na timeline: IR congelado, num passe pós-composição SEPARADO |
| [0145](0145-wet-paint-solver-row-parallel-passes-rayon-exception.md) | ACEITO (Enio, 2026-07-29 — ordem literal: *"rayon"*). | 2ª exceção sancionada ao "sem rayon": os três passes ROW-DISJUNTOS do solver do Wet Paint |
| [0146](0146-wet-paint-gpu-solver-is-a-second-model-not-a-faster-one.md) | — | A sim do Wet Paint na GPU é um SEGUNDO MODELO, não o mesmo mais rápido |
| [0147](0147-wet-paint-order-invariant-solver.md) | ACEITO (Enio, 2026-07-30 — ordem literal: *"GPU do Wet Paint"*). | O solver do Wet Paint é INDEPENDENTE DE ORDEM (e é por isso que ele paraleliza) |
| [0148](0148-vector-live-width-profile-is-an-ecs-component-and-one-baker-serves-preview-and-apply.md) | ⛔ superseded por 0153 · aceito (2026-07-29) | A largura variável é um COMPONENTE ECS, e UM motor serve o preview e o Apply |
| [0149](0149-physics-ik-is-a-transient-posing-tree-not-a-second-joint-representation.md) | Aceito (2026-07-27) | A IK é uma ÁRVORE DE POSE transitória, não uma segunda representação do joint |
| [0150](0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md) | proposto — aguarda aceite do Enio. | A escultura 3D é uma MALHA que doa sombreamento, referenciada no SculptGL (MIT) |
| [0151](0151-timeline-expressions-are-per-clip-so-a-strip-windows-them.md) | aceito (Enio 2026-07-27) | A expressão é POR-CLIP, então um strip a janela |
| [0152](0152-timeline-expressions-are-a-first-class-lane-source-that-fades.md) | — | Timeline expressions are a first-class LANE SOURCE that fades (they compose inside the blend, not after it) |
| [0153](0153-vector-auto-layout-is-taffy-behind-one-leaf-crate-and-the-pose-is-derived.md) | — | O auto layout é o `taffy` atrás de uma crate-folha, e a pose que ele produz é DERIVADA |
| [0154](0154-motion-shapes-are-live-gpu-vector-not-baked-tiles.md) | Aceito (proposto na linha `line/motion-value`). … | As formas do Motion são VETOR VIVO na GPU, não tiles assadas |
| [0155](0155-motion-graph-setup-is-diagnosed-and-healed-not-refused.md) | Aceito (proposto na linha `line/motion-value`). … | O setup do grafo de Motion é DIAGNOSTICADO e CURADO no gesto, não recusado |
| [0156](0156-sculpt3d-ao-trace-is-a-per-vertex-gather-rayon-exception.md) | **ACEITO** pelo Enio em 2026-08-06 (*"pode usar rayon. … | O traço de AO é um GATHER por-vértice, e por isso o `rayon` entra na `ph2d-sdf` |
| [0157](0157-liquify-is-an-authored-dab-list-cooked-on-the-device-never-a-stored-dense-field.md) | Aceito (linha `line/Painter`, integrada 2026-08-08). … | Uma deformação de Liquify é uma LISTA DE DABS autorada, cozida no device; o campo denso é cache, nunca estado |
| [0158](0158-solid-fill-running-sum-is-row-disjoint-rayon-exception.md) | **ACEITO** pelo Enio em 2026-08-15 (*"siga e corrija os abertos"*), sobre o item 1 da | A soma corrida do preenchimento é POR LINHA, e por isso o `rayon` entra na `ph2d-painter-brush` |
| [0159](0159-sculpt3d-the-dab-vertex-loop-is-a-row-disjoint-map-rayon-exception.md) | aceito (ordem do Enio, 2026-08-13: *"ambos"*, sobre a avaliação que | O laço de vértices de um dab é um MAP disjunto: exceção `rayon` na `ph2d-sculpt3d` |
| [0160](0160-quad-remesh-is-a-native-cross-field-port-quadriflow-referenced.md) | aceito (ordem do Enio, 2026-08-19: *"investigue o melhor algoritmo | O quad remesh é um porte NATIVO de campo cruzado, QuadriFlow referenciado |
| [0161](0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md) | proposto — o **caráter** já foi aprovado pelo Enio no smoke de 2026-08-19 | A modelagem 3D é uma ÁRVORE DE CAMPO IMPLÍCITO, e o que o artista vê é o campo TRAÇADO |
| [0162](0162-quad-remesh-pivots-to-the-global-family-clean-room-from-papers-gpl-oracle-outside.md) | — | O quad remesh PIVOTA para a família GLOBAL: clean-room a partir dos papers, oráculo GPL fora da árvore |
| [0163](0163-a-node-may-cook-its-own-input-at-n-instants-a-time-fan.md) | Accepted | Um nó pode cozinhar a PRÓPRIA entrada em N instantes (o *leque de tempo*) |
| [0164](0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md) | Accepted (aprovado pelo Enio em 2026-08-24 ao ordenar a implementação) | Instância = objetos REAIS ligados por id ao mestre, sync vivo no mesmo mundo, e o undo vira INCREMENTAL |
| [0165](0165-assets-are-born-inside-the-app-three-level-identity-index-before-browser.md) | Accepted (aprovado pelo Enio em 2026-08-24, junto com o ADR-0164) | O asset nasce DENTRO do app: identidade em 3 níveis, o ÍNDICE antes do navegador, catálogos por UUID — e o mestre É um asset |
| [0166](0166-the-inspector-shows-what-the-object-has-and-components-attach-through-one-palette-filtered-by-object-type.md) | Accepted (Enio, 2026-08-24 — instruções complementares à ordem de implementação … | O Inspector mostra o que o objeto TEM; componente anexa-se por UMA porta, com categorias e filtro por TIPO DE OBJETO |
| [0167](0167-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md) | Accepted | A EXTRAÇÃO de malha quad é clean-room dos *papers*; a biblioteca MPL-2.0 é ORÁCULO, não fonte a portar |

---

**172 ADRs** · **59** marcados ⛔ · **4** sem linha `Status:` no próprio texto.

⚠️ **⛔ diz «o ADR NNNN alega supersedê-lo»**, e a alegação pode ser PARCIAL: o ADR-0085
supersede uma *regra* dentro do ADR-0049, não o ADR inteiro. O índice reporta a alegação
e a sua origem — julgar o alcance é leitura humana.

⚠️ Um ADR sem `Status:` é um **achado**, não um defeito deste índice: é um ADR cujo
próprio texto não diz se ainda vale. O índice mostra o que existe, não o que devia existir.
