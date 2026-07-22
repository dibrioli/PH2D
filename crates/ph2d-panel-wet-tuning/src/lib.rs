#![forbid(unsafe_code)]
//! `ph2d-panel-wet-tuning` — the Wet Paint engine's FULL knob table (doc 22),
//! shown BESIDE the painter panel while the Wet Paint section's **Tuning**
//! checkbox is on.
//!
//! **The panel builds itself from the table.** Every row comes from the
//! engine's `KNOB_DEFS` registry (`ph2d-wet-paint`), exactly like the model
//! app: label, range, step and default live in ONE place, and `paint`,
//! `populate`, `event` and the seam sweep all walk [`rows::rows`] — a knob
//! that exists is painted, registered, live and swept by construction (the
//! physics panel's one-table law).
//!
//! **Events ride the normal tool pipe.** Unlike the physics panel (world
//! settings, no tool), every edit here is authored PAINTER state — the panel
//! forwards `EditorAction::ToolPanelEvent` and the painter's
//! `route_brush_wetpaint_event` routes the dynamic id family. The close
//! button forwards the SAME Tuning toggle the basic section owns: visibility
//! is the tool's authored fact, and the bridge mirrors it every frame — a
//! panel-local hide would fight the bridge and lose.

pub mod rows;
pub mod state;

mod event;
mod paint;
mod populate;

pub use state::set_current_brush;

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker implementing the typed Wet Tuning panel contract.
///
/// ⚠️ The name is load-bearing: `ph2d-panel-sync` parses `pub struct <Name>Panel`
/// out of this file and panics if it is absent.
pub struct WetTuningPanel;

impl Panel for WetTuningPanel {
    type State = state::WetTuningPanelState;

    const ID: &'static str = "wet_tuning";
    const NODE_ID: NodeId = ph2d_editor_core::ids::WET_TUNING_PANEL;
    /// Closed until the Wet Paint section's Tuning checkbox opens it (the
    /// bridge mirrors the tool's authored flag every frame).
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut Self::State, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut Self::State,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
