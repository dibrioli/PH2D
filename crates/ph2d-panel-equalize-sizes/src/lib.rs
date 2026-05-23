//! `ph2d-panel-equalize-sizes` — typed `Panel<State>` stub for the
//! stateful Equalize Sizes tool (ADR-0029). Multi-sprite obrigatório
//! (Fase 0 commits 28e1761 / 9a0cb80 land the SelectionSet API the
//! tool iterates via `hero.gizmo.iter_selected()`).
//!
//! Right-docked in the Inspector geometry slot; visible only while the
//! `equalize_sizes` tool is active (shell drives
//! `panel_visible("equalize_sizes")`).
//!
//! Implementador preenche `state.rs` (UI snapshot mirror — target dim
//! pair, grid unit toggle, upscale-if-smaller toggle, rasterize
//! toggle, optional algorithm dropdown when upscale is enabled),
//! `paint.rs`, `event.rs`, `populate.rs`. Vocab
//! `EqualizeSizesUiEdit` / `EqualizeSizesUiSnapshot` /
//! `EqualizeSizesParams` lives in the tool crate's `params.rs`
//! (TG-B/TG-C single-source-of-truth).

#![forbid(unsafe_code)]

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};
use ph2d_tool_registry::hash_node_id;

pub struct EqualizeSizesPanel;

#[derive(Default)]
pub struct EqualizeSizesPanelState;

impl Panel for EqualizeSizesPanel {
    type State = EqualizeSizesPanelState;
    const ID: &'static str = "equalize_sizes";
    const NODE_ID: NodeId = hash_node_id("panel.equalize_sizes");
    const DEFAULT_VISIBLE: bool = false;

    fn paint(_state: &mut EqualizeSizesPanelState, _ctx: &mut PaintCtx) {
        // Implementador preenche.
    }

    fn apply_event(
        _state: &mut EqualizeSizesPanelState,
        _host: &mut dyn PanelHostInternal,
        _ev: WidgetEvent,
    ) -> EventOutcome {
        EventOutcome::Ignored
    }

    fn populate(_store: &mut WidgetStore) {
        // Implementador registra ids (W/H number inputs, toggles, Apply, Cancel).
    }
}
