#![forbid(unsafe_code)]
//! `ph2d-panel-physics` — the WORLD half of physics authoring (ADR-0131 D8).
//!
//! Gravity, solver, air drag, sleep — the settings that describe the world
//! rather than any one body. The per-BODY half (type, density, restitution,
//! friction, collider shape) is the Inspector's "Physics Body" section, and
//! mixing the two is the error D8 names.
//!
//! **A category of its own.** Today's panels are tool-gated (painter, vector,
//! flip) or selection-docked (inspector); this one is world/scene-settings —
//! always reachable, editing the document rather than a tool. That is why it
//! emits [`PhysicsIntent`]s for the shell's bridge to drain instead of
//! forwarding `ToolPanelEvent`: there is no tool to forward to, and inventing
//! one so the existing pipe fits would be a tool that is not a tool.
//!
//! **One table, four consumers.** `paint`, `populate`, `event` and the seam
//! sweep all walk [`rows::SECTIONS`]. A knob is a row; adding one makes it
//! painted, registered, live and swept at once, and there is no second list to
//! drift out of sync.
//!
//! ⚠️ Two numbers here are **displayed, never owned**: the world scale is
//! `ProjectSettings::pixels_per_meter` (a PROJECT setting — D4) and the
//! collider overlay is the shell's `App.show_colliders` (the flag the `B` key
//! toggles). Second doors onto either would diverge, and the divergence would
//! be visible.

pub mod ids;
pub mod interact;
mod paint;
mod populate;
pub mod rows;
pub mod state;

mod event;

pub use state::{
    PhysicsIntent, PhysicsPanelState, PhysicsSnapshot, drain_intents, last_content_h,
    last_visible_h, set_current_physics,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker implementing the typed Physics panel contract.
///
/// ⚠️ The name is load-bearing: `ph2d-panel-sync` parses `pub struct <Name>Panel`
/// out of this file and panics if it is absent.
pub struct PhysicsPanel;

impl Panel for PhysicsPanel {
    type State = PhysicsPanelState;

    const ID: &'static str = "physics";
    const NODE_ID: NodeId = ph2d_editor_core::ids::PHYSICS_PANEL;
    /// Closed until asked for. Physics is global, but most documents have none
    /// — a panel that opens itself for every project would be chrome that has
    /// to be dismissed rather than found.
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut PhysicsPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut PhysicsPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
