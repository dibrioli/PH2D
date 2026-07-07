//! `ph2d-panel-motion-graph` — the Motion Nodes graph-editor panel (M0.T9).
//!
//! Docked in the `motion_graph` region of [`HeroLayout`] (the graph half of the
//! center split), visible only while the `motion` tool is active — the shell's
//! `motion_bridge` drives `panel_visible("motion_graph")`.
//!
//! **M0 skeleton:** paints the opaque graph-canvas background (`graph-bg`, Fase A
//! of plan §2.1 — the fill covers the sprite render beneath the graph half) and
//! publishes its rect so pointer dispatch can route to it. Node cards, wiring,
//! pan/zoom and gestures land in M1 (the `GraphSurface` dispatch + snapshot come
//! from M0.T2/T3 + the bridge).

#![forbid(unsafe_code)]

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::paint::{fill_rounded_rect, resolve};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};
use ph2d_tokens::ColorToken;

/// Retained per-instance state — none yet (the document lives shell-side in
/// `MotionState`; this panel only renders it). Unit struct so the typed registry
/// can default-construct it.
#[derive(Default)]
pub struct MotionGraphPanelState;

/// Zero-size marker implementing the typed graph-editor panel contract.
pub struct MotionGraphPanel;

impl Panel for MotionGraphPanel {
    type State = MotionGraphPanelState;

    const ID: &'static str = "motion_graph";
    const NODE_ID: NodeId = ids::MOTION_GRAPH_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(_state: &mut MotionGraphPanelState, ctx: &mut PaintCtx) {
        if !ctx.host.panel_visible(MotionGraphPanel::ID) {
            // Stale-rect cleanup so `panel_at` stops returning this panel once the
            // tool deactivates.
            ctx.host
                .store_mut()
                .clear_panel_rect(ids::MOTION_GRAPH_PANEL);
            return;
        }
        let rect = ctx.layout.motion_graph;
        if rect.w <= 0.0 || rect.h <= 0.0 {
            // No split (or a degenerate one) → nothing to draw.
            ctx.host
                .store_mut()
                .clear_panel_rect(ids::MOTION_GRAPH_PANEL);
            return;
        }
        let theme = ctx.host.theme();
        // Publish the rect so wheel/click dispatch can route to this panel.
        ctx.host
            .store_mut()
            .set_panel_rect(ids::MOTION_GRAPH_PANEL, rect);
        // Fase A: opaque graph canvas fill (dark in every theme) — covers the
        // sprite render underneath the graph half of the split.
        fill_rounded_rect(ctx.scene, rect, 0.0, resolve(ColorToken::GraphBg, theme));
    }

    fn apply_event(
        _state: &mut MotionGraphPanelState,
        _host: &mut dyn PanelHostInternal,
        _ev: WidgetEvent,
    ) -> EventOutcome {
        // No interactive widgets yet — graph gestures arrive via the
        // `GraphSurface` dispatch channel (M0.T2/T3), not per-widget events.
        EventOutcome::Ignored
    }

    fn populate(_store: &mut WidgetStore) {
        // No focusable widgets in the M0 skeleton.
    }
}
