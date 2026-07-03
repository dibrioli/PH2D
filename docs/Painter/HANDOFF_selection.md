# HANDOFF — Painter Selection system (tracker vivo)

> Tracker único do sistema de seleção (paridade Procreate). Arquitetura + decisões congeladas:
> [ADR-0103](../architecture/decisions/0103-selection-system-procreate-parity.md). Histórico → git log.

## Estado das waves

| Wave | Escopo | Estado |
|---|---|---|
| **0** | Botão compartilhado Mask↔Selection (flyout idêntico ao Shapes) + `PaintMode::Selection` (no-draw stub) | ✅ **FECHADA** (`2638302d`, 15 seam tests) |
| **1** | `selection_mask` state + integração snapshot/undo + gate de pintura | ✅ **FECHADA** (`d38ae362`, DoD test) |
| **2 core** | Motores on-canvas Procreate-default: Rectangle / Ellipse / Freehand / Automatic + Add/Remove/New | ✅ **FECHADA** (`7e321e13`, 4 tests) |
| **3** | Painel mode-exclusive (modos em Toggle + Feather + **Offset** + **botão Edit Selection** + Actions), responsivo | pendente |
| **EDIT** | **Selection Edit Mode** — reuso do sistema de Shapes (ver abaixo) | pendente |
| **4** | Overlays: marching ants + hachura diagonal (needs SMOKE) | pendente |
| **5** | Ações & Save/Load in-memory (Copy&Paste, Color Fill, Clear, Select layer contents) | pendente |

## Wave EDIT — Selection Edit Mode (reuso do sistema de Shapes) — requisitos do Enio

Até apertar o botão **"Edit Selection"** no painel, a seleção é **idêntica ao Procreate** (marquee/lasso/auto,
sem handles). Ao entrar no edit-mode, o **contorno da seleção vira uma Shape editável** reusando o sistema
de Stroke Method do Painter:

1. **Handles / alças / gizmos:** Freehand/Rect/Ellipse reaproveitam os editores de Shape (`curve.rs`/
   `ellipse.rs`/`polygon.rs`/`line.rs`, `ShapeEditState`, `TransformGizmo`, `TangentHandles`). Mapeamento:
   Ellipse→`EllipseState`; Rect→polyline fechada (Line, 4 cantos); Freehand→`CurveState` (como o FreeHand
   stroke). A cada edição, o contorno **rasteriza (fill da região fechada)** de volta no `selection_mask`.
2. **Offset do Stroke:** o sistema de Offset das shapes (`curve_offset.rs`/`line_offset.rs` + slider Offset,
   acumulador `shape_offset_base_px`) vale para seleção = **grow/shrink** do contorno (expand/contract CAD).
3. **Estabilização do Brush no Freehand:** o path do lasso passa pelo filtro de estabilização do brush
   (suavização do traço) antes de virar Curve.
4. **Undo:** edições do contorno entram na timeline única via `shape_snapshot.rs` (`begin/commit_shape_txn`,
   `capture_shape_model`/`restore_shape_overlay`) — intercaladas com o resto.

Design detalhado após mapa do sistema de Shapes; obstáculo conhecido: os editores hoje **stroke** um path,
a seleção precisa **fill** da região fechada (reusar `curve_geom` flatten + scanline even-odd de `raster_lasso`).

## Referência de paridade (Procreate)

- **Modos:** Automatic (flood por threshold, drag ajusta live) · Freehand (lasso + taps poligonais) ·
  Rectangle · Ellipse.
- **Operadores:** Add · Remove · Invert (Automatic tem Add implícito).
- **Refino:** Feather (slider 0..100%).
- **Ações:** Copy & Paste (layer "From selection") · Color Fill (toggle) · Save & Load (slots +
  thumbnail) · Clear · Select layer contents (do alpha da layer).
- **Feedback:** marching ants (editando) → hachura diagonal semi-transparente sobre a área NÃO
  selecionada (commitado).

## Touchpoints (verificados)

**Rail (editor-core) — Wave 0 (feito):** `ids/chrome/rail_painter.rs` (`PAINTER_RAIL_SELECTION`,
`PAINTER_RAIL_MASK_GROUP`, `PAINTER_RAIL_MASK_SUB_IDS`), `screens/hero/left_rail.rs` (grupo + flyout +
`active_mask_sub`), `screens/hero/chrome/rail_painter_tools.rs` (dispatch + tests),
`interaction/state/{mod,store_core,chrome_ops}.rs` (`painter_mask_flyout_open`).

**Undo (snapshot-based, fila única):**
- `tool/undo.rs` — `ModelSnapshot` (add `selection_mask`, `selection_active`), `UndoController`.
- `tool/layers/undo.rs` — `snapshot_model` (capturar) + `restore_model` (restaurar); espelho de
  `mask_scratch_for_snapshot`/`restore_mask_scratch`.
- `tool/paint/mask.rs` — precedente: scratch mask `Arc<Vec<u8>>` já undo-integrado + `restore_protected_region` (padrão do gate).
- `documents.rs` — stash per-doc (checar `reset_transient_edit_state` no rebind).

**Engine (novo módulo `tool/paint/selection.rs`, irmão de `mask.rs`):** state + `combine`/`clear`/
`invert`/`feather`; `canvas_pointer.rs` (hoje no-op em Selection) roteia p/ os motores.

**Painel (`ph2d-panel-painter-layers`):** `is_selection` em `BrushSettings` + `is_selection_mode()`
(`stencil.rs`) + `snapshot.rs` + `brush_fallback.rs`; early-return exclusivo em `paint_brush.rs::paint_brush_body`
(padrão `is_inpaint`); novo `paint_selection.rs`; ids em `ids/chrome/painter_selection.rs`; register em
`populate.rs`; forward em `event.rs` → `handle_panel_event`. Widgets: `SegmentedAdaptive`
(`widget/segmented_adaptive.rs`, reflui estreito), `paint_collapsible_section`, `paint_slider_chip_row`.

**Overlays (shell):** `shells/desktop/src/render_loop/painter_bridge_overlays.rs`; animação via
`on_tick` (heartbeat do Tool, ADR-0040-am2).

## Gates a satisfazer
`architecture_panel_wiring_parity` · `architecture_interactive_crate_has_behavioral_test`
(seam `ph2d-ui-testkit`, modelar em `paint_inpaint.rs:120`) · undo round-trip headless.

## Kill-criterion
Ants+flood @4K > 16 ms/frame após 2ª tentativa → PARA, prova o modelo; fallback hachura-só + ants low-res.

---

## Wave EDIT v2 — LANDED (2026-07-03, ADR-0103 Amendment 2)

**Modelo de lista (fonte de verdade):** `selection_shapes: Vec<SelectionEntry>` (Ellipse / Rect /
Freehand / Raster + boolean op). A máscara é cache derivado (`recompose_selection_mask` =
rasteriza + compõe a lista). Cada gesto de criação empurra uma entrada (New limpa, Add/Remove
empilham). `tool/paint/selection_shapes.rs`.

**Gizmos nativos por-forma (Edit mode):** `enter_selection_edit` instala o editor NATIVO da última
forma editável — Ellipse → o gizmo de elipse; Rect → curva fechada de 4 cantos (Vector/sharp);
Freehand → curva Bézier FECHADA ajustada (mesmo `fit_curve` do stroke Free Hand + estabilização);
sem forma paramétrica → traça o contorno (fallback). Editar UM gizmo recompõe a lista inteira (as
outras formas sobrevivem). Bake de volta na saída/`Apply`. `tool/paint/selection_edit.rs`.

**Convert to Curve:** achata a lista numa única curva Bézier editável (elipse única → 4-arcos;
várias → traça a máscara composta). Botão `PAINTER_SEL_CONVERT`.

**Offset (grow/shrink):** slider `PAINTER_SEL_OFFSET_SLIDER` (só em Edit mode) via o acumulador de
Offset do Stroke; recompõe ao vivo pelo gizmo ativo.

**Wave 5 actions (`tool/paint/selection_actions.rs`):** Select layer contents (alpha>0), Color Fill
(cor do brush × cobertura), Copy/Paste (clipboard in-memory `selection_clipboard`). Ids
`PAINTER_SEL_WAVE5_IDS` + `PAINTER_SEL_FILL`/`_COPY`/`_PASTE`/`_LAYER_CONTENTS`.

**Split LOC:** `selection.rs` (845→221) quebrado em `selection_input`/`_raster`/`_overlay`/`_edit`/
`_shapes`/`_actions`. Overflows latentes desta sessão também resolvidos por split:
`stamp_preview` (ex-`paint.rs`), `brush_texture_settings` (ex-`brush_settings.rs`),
`trait_impls_raster` (ex-`trait_impls.rs`), `painter_gradient` ids (ex-`painter.rs`).

**Testes:** 20 testes de seleção (headless) — install de gizmo nativo por tipo, preservação
multi-forma, convert, offset cresce, color-fill dentro-só, copy/paste round-trip, layer-contents.

### DEFERIDO (precisa de smoke visual) — único gap aberto
Renderização SIMULTÂNEA de vários gizmos na tela ao mesmo tempo. Hoje: a SELEÇÃO multi-forma é
correta (a lista + a máscara compõem todas as formas) e o gizmo da forma ATIVA (última editável) é
editável com aparência idêntica ao stroke. Desenhar TODOS os gizmos de uma vez + dispatch de ponteiro
entre eles é um loop de overlay no shell (`painter_bridge_*_overlay`) que é puramente visual e exige
smoke para acertar — próximo passo pós-smoke.

---

## Gizmo polish round (2026-07-03) — LANDED

- **Auto-hide gizmos on tool switch:** leaving Select unchecks "Show Selection Gizmos" (`set_paint_tool_mode` → `exit_selection_edit` when `new_mode != Selection`).
- **Freehand = whole-shape TRANSFORM gizmo** (move/scale/rotate about the bbox centre; rotate is transcendental-free via the grab vectors' dot/cross). The anchor **points are NOT editable** in gizmos-phase — only after Convert to Curve. `SelectionGrab` now carries the pristine geometry so transforms are drift-free.
- **Distinct fluorescent colours** (Mask palette: yellow/pink/green/orange) per gizmo — selection gizmos cycle by index; stroke shape types get a fixed distinct accent (ellipse=yellow, polygon=pink, curve=green, line=orange). `painter_bridge_gizmo::{GIZMO_ACCENTS, palette_accent}`; `draw_transform_gizmo` now takes a `&GizmoPalette`.
- **Sprite-gizmo bbox look:** freehand draws a `frame_box` (corner squares + connecting lines + centre-move square). Polygon **sides** handle is a diamond (distinct from the round rotate).
- **Stroke "E" → "Convert to Curve"** full-width button (own row above Apply; the cramped square is gone). Same `PAINTER_BRUSH_STROKE_EDIT` id → `convert_open_shape_to_curve`.

### DEFERRED (next dedicated round) — Stroke multi-shape
"Assim como em Mask/Seleção, o Stroke deve criar várias shapes simultâneas editáveis." This is a LARGE architectural change to the stroke shape editors (currently single-slot `self.paint.{curve,ellipse,polygon,line}`) — comparable in size to the whole selection multi-gizmo redesign. It needs its own build + smoke; NOT landed in this polish round.

---

## Gizmo standardization (2026-07-03) — LANDED

All 3 selection gizmos (Ellipse / Polygon / Freehand) unified to the **Sprite transform gizmo**
(`selection_gizmo.rs` rewritten around an oriented `Frame` = center/u/hx/hy):
- **8 scale squares** (4 corners + 4 edge mids) — corners scale both axes, edges scale one.
- **Rotate** by grabbing the ring just OUTSIDE a square (the square reads as a **circle** on that
  hover — `scale_tol`/`rotate_tol` drive the cue in `painter_bridge_selection_gizmos`).
- **Centre-move square** + (Polygon only) the **sides diamond**.
- Ellipse/Polygon boxes ride the shape orientation `u`; Freehand uses the anchors' AABB (transform
  applied to every point). Drift-free (grab carries the pristine shape). Rotate is transcendental-free.
- Distinct fluorescent accent per gizmo stays.

## QUEUE (next, in order) — Enio 2026-07-03
1. ~~**Stabilization slider for Free selection**~~ — **LANDED** (commit c4ff7d26): its own
   `selection_stabilizer` knob (independent of the brush), shown only in Freehand mode.
2. ~~**Selection offset system**~~ — **LANDED** (commit 0f8c675b): signed-distance grow/shrink + concentric
   alternating protected/paint rings via Apply / Apply & Keep (`selection_offset.rs`, ADR-0103 Am.3).
3. ~~**C&F vs shape-tool click-through fix**~~ — **LANDED** (Enio 2026-07-03). The C&F (`PAINTER_RAIL_FILL`)
   Down now CONSUMES the event (`arm_fill_drag_if_on_button` returns armed → the dispatch `return`s), so it no
   longer falls through to `painter_canvas_down` and the shape tool no longer drops a stray point behind the
   button. Click = colour picker only (Fill never activates on a plain click — already the case); click+drag =
   momentary Fill via `PainterTool::begin_colordrop_fill(prev_mode)`, which records the mode active at press
   and RESTORES it when the fill finalizes (`fill_commit` / `fill_cancel` / the picker path for a missed drag)
   — so a ColorDrop returns to the shape/brush the user was using. `active_paint_mode_id()` is the capture.
4. **Mask-panel buttons — no hover/press decoration** (Enio 2026-07-03) — **BUG.** The Mask section buttons
   (Brushes / Modifiers / Overlay Color / Apply Mask, `paint_mask.rs`) show no mouse-over / mouse-down visual
   feedback. Investigation (2026-07-03): the paint path IS correct — `paint_button_cell` reads
   `store.button_state(id)` → `flat_button_surface` (Bg2/BgElev/AccentSoft, distinct), the ids ARE registered
   in `populate` as `InteractiveState::Button`, and Move events reach `forward_to_hero → handle_pointer →
   update_hover` on the SAME hero store + hit_index that clicks use (so hover *should* fire generically). No
   static root cause found — needs a RUNTIME repro (pixel/layout): likely a hit-index z-order overlap from the
   pinned-at-top Mask section (a later-registered rect shadowing the buttons — `HitIndex::hit` is last-wins) OR
   the token deltas being imperceptible. Repro: hover a Modifiers button with the Mask tool active; instrument
   `hit_index.hit(cursor)` to see which id resolves under the button.
4. **Convert / Simplify Curve — missing point handles** (Enio 2026-07-03) — **BUG.** After **Convert to
   Curve** or **Simplify Curve**, the resulting selection curve should show editable **anchor points + Bézier
   handles** with the SAME look/capability as the stroke Shape-system curves, but they no longer appear. The
   curve rasterizes/edits, yet the on-canvas handles/points are not drawn (regression from the isolated
   selection-gizmo rewrite — the Freehand selection shows the transform box gizmo, not per-anchor handles).
   Wire the selection Freehand curve to the same anchor/handle overlay + hit-testing the stroke Curve editor
   uses, so Convert/Simplify yields a point-editable curve.
5. **Stroke multi-shape** — multiple simultaneous editable stroke shapes (large architectural round,
   ≈ the selection multi-gizmo redesign; its own build + smoke).
