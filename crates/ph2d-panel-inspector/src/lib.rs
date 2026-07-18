//! `ph2d-panel-inspector` — ADR-0029 Phase C.1 typed panel crate.
//!
//! Owns the Inspector panel's:
//! - retained state ([`InspectorState`]) — held by `ErasedPanel` in
//!   the runtime `panel::PANEL_REGISTRY`.
//! - paint thunk ([`paint`]), event router ([`event`]),
//!   snapshot sync ([`sync`]), widget registration ([`populate`]).
//! - typed contract via [`InspectorPanel`] implementing
//!   [`ph2d_editor_core::panel::Panel`].
//!
//! Per-frame snapshots the host publishes to drive the live sections
//! flow through thread-local setters re-exported below
//! ([`set_current_inspector_sprite`] etc.); these supersede the
//! pre-C.1 `hero.inspector.*` field writes.

#![forbid(unsafe_code)]

mod event;
mod event_joint;
mod event_ordering;
mod event_physics;
mod paint;
mod paint_frame;
mod populate;
mod sections;
pub mod state;
mod sync;

/// The §11 Bake button's label — exported so a gate can hold the claim that
/// the button shows the range it would cover.
pub use sections::bake_label;
pub use state::{
    InspectorState, last_inspector_content_h, last_inspector_visible_h, set_current_display_unit,
    set_current_inspector_blend, set_current_inspector_joint, set_current_inspector_name,
    set_current_inspector_ordering, set_current_inspector_physics, set_current_inspector_sampling,
    set_current_inspector_sprite, set_current_inspector_transform,
    set_current_inspector_visibility, set_current_inspector_visibility_section,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker implementing the typed Inspector panel contract.
pub struct InspectorPanel;

impl Panel for InspectorPanel {
    type State = InspectorState;

    const ID: &'static str = "inspector";
    const NODE_ID: NodeId = ids::INSP_PANEL;
    const DEFAULT_VISIBLE: bool = true;

    fn paint(state: &mut InspectorState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut InspectorState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
