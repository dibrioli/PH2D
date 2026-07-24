//! `ph2d-panel-vector` — typed `Panel<State>` for the Vector tool's Style
//! (ADR-0029 / ADR-0108 cutover).
//!
//! Right-docked in the Inspector slot; visible only while the `vector` tool is
//! active (the shell drives `panel_visible("vector")` from the active-tool id).
//! Holds a **Width** slider+chip (1..20 px) + **Stroke** and **Fill** colour
//! swatches (each opens the shared OKLCH picker) + a Fill **None** button.
//!
//! Tool `FloatingPanel`s are unpainted in this app (input-dispatch only), so the
//! authoritative `VectorTool` lives in the shell's `ToolRegistry`, unreachable
//! from `HeroScreen`. Therefore:
//! - the host publishes a [`VectorStyleSnapshot`] each frame via
//!   [`set_current_vector_style`] → [`paint`] reads it;
//! - Width / Fill-None edits flow out over `EditorAction::ToolPanelEvent` → the
//!   shell calls `VectorTool::handle_panel_event`;
//! - the Stroke / Fill swatches open the OKLCH picker (generic dispatch) and the
//!   shell's `vector_bridge` reads the pick back into
//!   `VectorTool::set_stroke_rgba` / `set_fill_rgba`.
//!
//! Mirrors `ph2d-panel-padding` (+ the retired vector inspector's picker
//! swatch).
//!
//! [`VectorStyleSnapshot`]: ph2d_tool_vector::VectorStyleSnapshot

#![forbid(unsafe_code)]

mod event;
mod font_dropdown;
pub mod ids;
mod paint;

/// A faixa do Spacing do Pattern on Path (plano 23): o avanço por cópia em múltiplos da largura do
/// motivo. `0.25` sobrepõe forte, `4.0` deixa vãos largos, `1.0` encaixa borda-a-borda. O track do
/// slider é `0..1` e mapeia nesta faixa — a MESMA porta é lida pelo paint (track→display), pelo
/// populate (scale/offset do chip) e pelo event (track→valor), senão os três divergiriam.
pub(crate) const SPACING_MIN: f64 = 0.25; // LITERAL-PX-OK: faixa no domínio do documento
/// Ver [`SPACING_MIN`].
pub(crate) const SPACING_MAX: f64 = 4.0; // LITERAL-PX-OK: faixa no domínio do documento
/// A meia-faixa do Offset perpendicular do Pattern on Path, em unidades de MUNDO: o slider é
/// bipolar `−OFFSET_MAX..OFFSET_MAX` (track `0..1`, `0.5` = sobre a curva). Empurra as cópias para
/// fora/dentro da curva; lido pelo paint, populate e event pela MESMA porta.
pub(crate) const OFFSET_MAX: f64 = 2.0;
/// A meia-faixa da ORIENTAÇÃO do motivo no Pattern on Path, em GRAUS: o slider é bipolar
/// `−ROTATION_MAX..ROTATION_MAX` (track `0..1`, `0.5` = deitado ao longo da curva). Meia volta para
/// cada lado cobre o círculo inteiro sem repetir ângulo, e o neutro cai no centro do curso — o
/// mesmo desenho do Offset, e lido pelo paint, populate e event pela MESMA porta.
pub(crate) const ROTATION_MAX: f64 = 180.0; // LITERAL-PX-OK: faixa no domínio do documento
mod paint_arrange;
/// O catálogo de formas (categoria em dropdown + grade de thumbnails cozidos).
mod paint_catalog;
/// A seção do CONECTOR (Route / Jetty / Spread) — snapshot + seed + paint + evento.
mod paint_connector;
/// As pontas do traço (arrowheads) — dois chips na seção Stroke + o popover diferido.
mod paint_markers;
mod paint_modes;
mod paint_sections;
mod paint_transform;
pub mod populate;
/// De QUEM são os campos de parâmetro deste frame — e quando eles não são de ninguém (a
/// seção some). Irmão de `state` (teto de LOC).
mod shape_focus;
pub mod state;

pub use paint_connector::{ConnectorSnapshot, set_current_connector};
pub use state::{
    FillKind, FontPreview, FxParamView, FxRowView, PathFillRule, TextAxisSlot, VectorPanelState,
    expand_join, expand_side, last_content_h, last_visible_h, set_current_convertible,
    set_current_effects, set_current_envelope_mode, set_current_envelope_presets, set_current_fill,
    set_current_fill_rule, set_current_grad_influence, set_current_grad_jitter,
    set_current_has_envelope, set_current_path_closed, set_current_patternpath,
    set_current_patternpath_can_link, set_current_patternpath_can_pick, set_current_pivot_edit,
    set_current_selection_count, set_current_shape_focus, set_current_snap, set_current_text,
    set_current_text_align, set_current_text_axes, set_current_text_font,
    set_current_text_font_previews, set_current_text_seed, set_current_text_visible,
    set_current_textpath, set_current_textpath_can_link, set_current_transform,
    set_current_vector_style, set_expand_join, set_expand_side, set_selected_vertex_type,
    take_want_font_previews,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker implementing the typed Vector Style panel contract.
pub struct VectorPanel;

impl Panel for VectorPanel {
    type State = VectorPanelState;

    const ID: &'static str = "vector";
    const NODE_ID: NodeId = ph2d_editor_core::ids::VECTOR_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut VectorPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut VectorPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
