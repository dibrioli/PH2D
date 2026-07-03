# HANDOFF — Painter Selection system (tracker vivo)

> Tracker único do sistema de seleção (paridade Procreate). Arquitetura + decisões congeladas:
> [ADR-0103](../architecture/decisions/0103-selection-system-procreate-parity.md). Histórico → git log.

## Estado das waves

| Wave | Escopo | Estado |
|---|---|---|
| **0** | Botão compartilhado Mask↔Selection (flyout idêntico ao Shapes) + `PaintMode::Selection` (no-draw stub) | ✅ **FECHADA** (commit `2638302d`, 15 seam tests verdes) |
| **1** | `selection_mask` state + integração snapshot/undo + gate de pintura | ⏳ em andamento |
| **2** | Motores on-canvas: Rectangle / Ellipse / Freehand / Automatic + Add/Remove/Invert | pendente |
| **3** | Painel de Seleção mode-exclusive (SegmentedAdaptive + Feather + Actions) | pendente |
| **4** | Overlays: marching ants + hachura diagonal (needs SMOKE) | pendente |
| **5** | Ações & Save/Load in-memory (Copy&Paste, Color Fill, Clear, Select layer contents) | pendente |

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
