//! `ph2d-panel-timeline` — typed `Panel<State>` for the app-general timeline
//! (ADR-0029; plan `docs/Timeline/`).
//!
//! Bottom-docked in the `timeline` layout slot; shown while
//! `panel_visible("timeline")` (the shell drives that from the timeline toggle).
//! The authoritative document lives in the shell (`AppGfx.timeline`,
//! `ph2d_timeline::TimelineState`); the panel is a pure view — it paints a
//! [`TimelineViewSnapshot`] the shell publishes each frame (via
//! [`set_current_timeline`]) and emits `TimelineIntent`s the shell drains
//! (mirror of the vector/motion docked panels; document ≠ tool, ADR-0040).
//!
//! W2.E0 is the scaffold: docked chrome + title. Transport bar, ruler/scrub and
//! dope-sheet lanes land in W2.E2+.
//!
//! [`TimelineViewSnapshot`]: ph2d_timeline::TimelineViewSnapshot

#![forbid(unsafe_code)]

mod anchor_drag;
mod box_select;
mod clip_rename;
mod event;
mod geom;
mod graph;
mod graph_paint;
pub mod ids;
mod interact;
mod key_drag;
mod loop_drag;
mod marker_drag;
mod marker_rename;
mod paint;
pub mod populate;
mod resize;
mod ruler;
mod scrollbar;
mod stack_ease_grip;
mod stack_lane_paint;
pub mod state;
mod strip_drag;
mod summary;
mod summary_paint;
mod tracks;
mod transport;
mod view;

pub use state::{
    TimelinePanelState, drain_intents, last_content_h, last_visible_h, request_fit,
    request_reveal_playhead, set_current_timeline,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker implementing the typed timeline panel contract.
pub struct TimelinePanel;

impl Panel for TimelinePanel {
    type State = TimelinePanelState;

    const ID: &'static str = "timeline";
    const NODE_ID: NodeId = ph2d_editor_core::ids::TIMELINE_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut TimelinePanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut TimelinePanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
