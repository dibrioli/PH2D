//! `ph2d-panel-widget-gallery` — ADR-0029 Phase C.3 typed panel crate.
//!
//! Owns the Widget Gallery panel's:
//! - retained state ([`WidgetGalleryState`]) — held by `ErasedPanel`
//!   in the runtime `panel::PANEL_REGISTRY`. Just the persisted
//!   `Option<Rect>` post-Phase-C; visibility lives on the host's
//!   `panel_visibility` map.
//! - paint thunk ([`paint`]), event router ([`event`]),
//!   widget registration ([`populate`]).
//! - typed contract via [`WidgetGalleryPanel`] implementing
//!   [`ph2d_editor_core::panel::Panel`].
//!
//! Design intent: peripheral agents run `./play.command`, click the
//! palette pill in the TopBar (`ids::TOPBAR_WIDGET_GALLERY`), and see
//! every canonical widget rendered the way the editor expects so they
//! can replicate the decoration in their own modules. Single in-app
//! source of UI truth. Reuses the 10-section showcase painters owned
//! by `ph2d-editor-core::widget::showcase`.

#![forbid(unsafe_code)]

mod event;
mod paint;
mod populate;
pub mod state;

pub use paint::default_rect;
pub use state::WidgetGalleryState;

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Zero-size marker implementing the typed Widget Gallery panel
/// contract.
pub struct WidgetGalleryPanel;

impl Panel for WidgetGalleryPanel {
    type State = WidgetGalleryState;

    const ID: &'static str = "widget_gallery";
    const NODE_ID: NodeId = ids::GAL_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut WidgetGalleryState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut WidgetGalleryState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
