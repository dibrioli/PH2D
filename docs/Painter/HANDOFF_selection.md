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
