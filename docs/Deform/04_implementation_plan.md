# 04 — Plano de implementação

> Ancorado em file:line reais (mapeamento 2026-07-02). **Bloqueado por posse** até o agente
> de Selection landar (ver `00_README.md §Coordenação`). Sem contrato congelado tocado (Wave 1).

## Waves

| Wave | Escopo | Caminho | Contrato | Depende de |
|---|---|---|---|---|
| **W1** | Botão rail + Reshape (Liquify): Push/Twist/Pinch/Wrinkle/Fold/Reconstruct + painel + freeze + undo | (C) rail + (D) painter/painel | não | Selection landar |
| **W2** | Transform gizmo: Uniform/Free/Distort/Warp-mesh (handles interativos) | (C)/(B) Coord | InteractiveState em editor-core (foundational) | W1 |
| **W3** | Puppet/MLS pins (Schaefer 2006) | (D) painter + (C) dispatch | não | W2 (infra de handles) |
| **W4** | Warps paramétricos como nós: polar, spherize, ripple, displace | (A) fan-out `ph2d-node-*` | não | independente |

---

## Wave 1 — Reshape (MVP, já ≥ Procreate Liquify + Freeze)

### 1A. Rail (Coord-only, `ph2d-editor-core`) — pode ir em paralelo à Selection
1. `src/ids/chrome/rail_painter.rs`: `pub const PAINTER_RAIL_DEFORM: NodeId = hash_node_id("painter_rail.deform");` (perto de `:33-39`).
2. `PAINTER_RAIL_TOOL_IDS` (`rail_painter.rs:76-87`): inserir **antes** de `PAINTER_RAIL_MASK_GROUP`; bump `[NodeId; 10] → [NodeId; 11]`.
3. `src/screens/hero/left_rail.rs` `PAINTER_TOOLS` (`:30-55`): inserir tupla `(ids::PAINTER_RAIL_DEFORM, "Deform", IconId::Transform, "DFORM")` **antes** da entrada Mask (`:42`); bump `[…; 9] → […; 10]`. Registro é automático no loop `populate` (`:122-129`).
4. `src/screens/hero/chrome/rail_painter_tools.rs` `push_paint_mode` (`:27-56`): arm `else if tool_id == ids::PAINTER_RAIL_DEFORM { "deform" }`.
   - Dispatch/radio já cobertos pelo branch genérico (`:187-218`).
5. **Ícone:** reusar `IconId::Transform` (`icons.rs:177`) — **sem** SVG/variant novo (não mexe no gate `enum_order_matches_svgs`).
6. Gate: `cargo test -p ph2d-editor-core` (wiring-parity + tool contract surface intactos).

### 1B. Sub-tool no painter (`ph2d-tool-painter`) — **após Selection landar**
1. `src/tool/paint/paint_mode.rs`: variant `Deform` (`:14-30`); bump `PAINT_MODE_COUNT` (`:34`) e `slot()` (`:39-50`).
2. `src/tool/paint/stencil.rs` `set_paint_tool_mode` (`:340-350`): mapear `"deform" → PaintMode::Deform`. Predicate `is_deform_mode` (`matches!`, junto de `:388-403`).
3. **Novo módulo** `src/tool/paint/warp/` (kernel — arquivos ≤600 LOC cada):
   - `mod.rs` — `warp_pointer(ev)` (roteia Down/Move/Up), estado da sessão de deform.
   - `field.rs` — geradores de `D` por modo (Push/Twist/Pinch/Wrinkle/Fold). HR-5: sin/cos gated.
   - `apply.rs` — kernel inverse-gather + bilinear + freeze-lerp (usa `save_region`/`restore_region`/`mark_dirty` de `region.rs`, `selection_coverage_at` de `selection.rs`).
   - `reconstruct.rs` — guarda `pre_deform: Arc<Vec<u8>>` da sessão; reamostra de volta.
4. `src/tool/paint/canvas_pointer.rs`: no ladder (perto do arm Selection `:32`), `if self.paint.paint_mode == PaintMode::Deform { return self.warp_pointer(ev); }`.
5. **Undo:** no `Down` `let before = self.snapshot_model(); self.paint.stroke_undo = Some(before);` (padrão `paint.rs:377-378`); no `Up` `commit_structural_edit(before)` (padrão `paint.rs:502-509`).
6. **Snapshot p/ painel:** `src/tool/paint/brush_settings.rs` (`:63-80`): add `is_deform: bool` + params (`deform_mode`, `size_norm`, `pressure`, `distortion`, `momentum`, `strength`, `freeze_on`, `amount`). Preencher em `snapshot.rs` (`:75-99`). Rides o publish existente `set_current_brush` (`painter_bridge.rs:298`) — **sem bridge novo**.
7. **handle_panel_event** (`stencil.rs::route_brush_dab_event` `:408-427`): arms p/ os ids do painel (SetValue dos sliders, SelectOption do modo, Click dos botões) → `apply_ui_edit` do deform (single-source clamps).

### 1C. Painel (`ph2d-panel-painter-layers`) — **após Selection landar** (compartilha arquivos)
1. `src/ids…` (core): declarar ids da seção (header, dot, reset) + cada control (5 sliders + chips, toggle Freeze, botões Reset/Apply/Apply&Keep, segmented de modo + option-ids, slider Amount).
2. `src/populate.rs`: sliders no loop (`:63-101`); chips + `link_slider_number` + `set_number_range` (padrão `:130`/`:176-212`); toggles/botões (`:230-302`); header colável em `register_collapsible_sections` (`:353-401`); segmented como grupo.
3. **Novo** `src/paint_deform.rs` + `mod paint_deform;` (`lib.rs:27-60`): `paint_deform_section(...)` usando `paint_collapsible_section` + helpers de row (`paint_slider_chip_row`, `paint_checkbox_row`) + `SegmentedAdaptive` p/ o Card A (spec `03`).
4. `src/paint_brush.rs` `paint_brush_body` (`:41-231`): early-return `if brush.is_deform { return paint_deform::paint_deform_section(...) }` (topo, como `is_selection` `:52-54`).
5. `src/event.rs`: sliders em `event_brush_forward::is_forwardable_brush_slider` (`:11-31`); toggles/botões no allowlist Click (`:411-461`); segmented no `option_route` (`event/option_route.rs:28-72`) + decoder (`event/decode.rs`).
6. `paint_brush_top::header_title` (`:25-33`): arm `"Deform"`.
7. **`tests/seam.rs`:** uma asserção por control-shape (dirige o evento real via `ph2d-ui-testkit` → efeito observável).

### 1D. DoD Wave 1 (DIRETIVA §5 — compile-verde **não** conta)
- [ ] Seam test **verde**: cada control dirige `PanelEvent` real → muta o deform state observável.
- [ ] Kernel: teste de paridade numérica do inverse-warp (campo identidade `D=0` ⇒ imagem **byte-idêntica**; Push conhecido ⇒ deslocamento esperado em pixels amostrados).
- [ ] Freeze: com seleção ativa, texels cobertos permanecem **inalterados** (assert por-pixel).
- [ ] Undo: 1 stroke de deform = **1** entrada na timeline; undo restaura byte-idêntico.
- [ ] Smoke do Enio: arrastar Push/Twist/Pinch no canvas, ver deformar em tempo real, Apply baka, Freeze protege.
- [ ] Sem no-op silencioso: Distortion/Momentum somem em Reconstruct; Freeze desabilitado sem seleção mostra hint.
- [ ] **Responsivo (spec `03` §5):** todo row usa variante `*_adaptive`; painel reflui sem cortar/estourar em ≥2 larguras (dock ~300px e tablet estreito ~200px); alturas de row derivadas do helper, hit-rects acompanham o layout pintado. Smoke em largura estreita obrigatório.

### 1E. Kill-criteria / perf (fixar ANTES do build — DIRETIVA §5)
- **Alvo interativo:** um dab de deform (raio típico ~256px) deve reescrever sua bbox e subir o dirty-rect em **≤ 8 ms** num layer 4K no M-series. Se **> 16 ms após a 2ª tentativa de otimização CPU**, a Reshape para de ser CPU-residente nesta forma → migra o kernel p/ GPU-residente (segue `project_painter_composite_perf`, cs-warp) antes de qualquer 3ª tentativa (regra two-strikes).
- **Escala primeiro (memória `feedback_measure_perf_symptom_scale`):** medir ms real antes de otimizar; frame(≤16ms) vs ⅓s muda a classe de causa.

---

## Wave 2 — Transform gizmo (Coord, foundational)
Handles interativos de bounding-box/mesh em `editor-core` (InteractiveState/BlenderHit, padrão
`reference_panel_2d_drag_needs_dispatch`). Modos Uniform/Free/Distort(homografia)/Warp(mesh
Coons). Reusa o kernel inverse-warp (`D` afim/mesh). **Coord-only** (foundational). Possível
ADR se exigir superfície nova de dispatch (avaliar; provavelmente sem bump de cap).

## Wave 3 — Puppet / MLS pins
Portar Schaefer 2006 (rígido) em `warp/mls.rs`; pinos como handles (infra da W2). Handle add/drag
via dispatch. Aceitação: reconstrução byte-estável + kill-criterion perf por nº de pinos.

## Wave 4 — Warps paramétricos como nós (fan-out (A))
Drop-crates `ph2d-node-warp-{polar,spherize,ripple,displace}/` + `cargo run -p ph2d-node-sync`.
Cada um emite o campo `D` como transform de imagem no grafo (não-destrutivo/animável). Zero edit
central; paraleliza. Segue o briefing `DIRETRIZ §3.A`.

---

## Isolamento & posse (crítico)
- **W1B/W1C editam os MESMOS arquivos que a Selection em vôo** (`canvas_pointer.rs`, `paint_mode.rs`,
  `brush_settings.rs`/`snapshot.rs`, `paint.rs`, `ph2d-panel-painter-layers/*`). **Não iniciar** até
  a Selection landar e a posse liberar no `SESSION_ACTIVE.md`.
- **W1A (rail)** toca `ph2d-editor-core` (arquivos disjuntos da Selection) → pode adiantar como Coord-only.
- Git: `git add -- <só meus paths>`; nunca `-A`; commit escopado. **Sem commit nesta sessão.**
