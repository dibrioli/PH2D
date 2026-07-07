//! `ph2d-panel-motion-params` — the Motion Nodes node-params panel (M0.T9).
//!
//! Right-docked in the Inspector slot (takeover, mirror of `ph2d-panel-vector`),
//! visible only while the `motion` tool is active — the shell's `motion_bridge`
//! drives `panel_visible("motion_params")` and hides the real Inspector.
//!
//! **M0 skeleton:** canonical dark-glass panel surface + title. The per-node
//! param rows (slider + chip, generated from the `ParamUiHint` metadata on
//! `ph2d-node-registry`) + the attribute chips land in M1.

#![forbid(unsafe_code)]

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};
use ph2d_editor_core::widget::panel_chrome::{paint_panel_surface, paint_panel_title};

/// Retained per-instance state — none yet (the selected node + its params live
/// shell-side in `MotionState`). Unit struct so the typed registry can
/// default-construct it.
#[derive(Default)]
pub struct MotionParamsPanelState;

/// Zero-size marker implementing the typed node-params panel contract.
pub struct MotionParamsPanel;

impl Panel for MotionParamsPanel {
    type State = MotionParamsPanelState;

    const ID: &'static str = "motion_params";
    const NODE_ID: NodeId = ids::MOTION_PARAMS_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(_state: &mut MotionParamsPanelState, ctx: &mut PaintCtx) {
        if !ctx.host.panel_visible(MotionParamsPanel::ID) {
            ctx.host
                .store_mut()
                .clear_panel_rect(ids::MOTION_PARAMS_PANEL);
            return;
        }
        let rect = ctx.layout.inspector;
        let theme = ctx.host.theme();
        // Publish the rect so wheel/click dispatch can route to this panel.
        ctx.host
            .store_mut()
            .set_panel_rect(ids::MOTION_PARAMS_PANEL, rect);
        // Canonical dark-glass surface + title — identical chrome to the
        // Inspector / Vector panels. Body (param rows) is M1.
        paint_panel_surface(rect, ctx.scene, theme);
        let _ = paint_panel_title(rect, "Motion", 0.0, ctx.scene, ctx.text_system, theme);
    }

    fn apply_event(
        _state: &mut MotionParamsPanelState,
        _host: &mut dyn PanelHostInternal,
        _ev: WidgetEvent,
    ) -> EventOutcome {
        // No interactive widgets in the M0 skeleton.
        EventOutcome::Ignored
    }

    fn populate(_store: &mut WidgetStore) {
        // No focusable widgets in the M0 skeleton.
    }
}
