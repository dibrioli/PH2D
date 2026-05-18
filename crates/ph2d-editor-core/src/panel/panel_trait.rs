//! `Panel` trait — typed contract each panel crate implements.
//!
//! ADR-0029 §4.3 — chosen over `dyn Any`-based registry. Rationale:
//! compile-time check of `Panel ↔ State` relationship. Refactor of
//! state struct (rename, split, mover fields) caught by rustc, not
//! by runtime smoke. Cost: ~50 LOC of glue in [`crate::panel::erased`]
//! that does the downcast once at install. Worth the cost for
//! foundation that lasts years.

use super::EventOutcome;
use super::PaintCtx;
use super::PanelHostInternal;
use crate::interaction::{WidgetEvent, WidgetStore};
use ph2d_a11y::NodeId;

/// A panel implementation. Each `ph2d-panel-*` crate defines exactly
/// one type implementing this trait + a `pub static MANIFEST: PanelManifest`
/// constructed via [`super::PanelManifest::for_panel::<Self>()`].
///
/// `State` is the panel's per-instance retained state — what was
/// previously a `pub(super) struct InspectorState`/`HierarchyState`/etc.
/// living flat on `HeroScreen`. Now owned by `ErasedPanel<Self>` and
/// passed typed to each fn.
pub trait Panel: Sized + 'static {
    /// Per-instance retained state. Allocated once when the panel
    /// is registered, owned by the registry, passed `&mut` to each
    /// fn. `Default` enforced so `register_all_panels()` can build
    /// the registry without painel-specific constructors.
    type State: Default + Send + 'static;

    /// Short stable identifier for logs/MCP/ADR refs. Snake-case
    /// (`"inspector"`, `"widget_gallery"`).
    const ID: &'static str;

    /// `NodeId` of the panel's outer rect — used by `paint_hero_screen`
    /// to look up the manifest by `z_order`'s panel_id.
    const NODE_ID: NodeId;

    /// Default `visible` value when the host doesn't supply one.
    const DEFAULT_VISIBLE: bool;

    /// Paint the panel for one frame. State + host are typed; the
    /// orchestrator dispatches through `ErasedPanel` which downcasts
    /// once at install.
    fn paint(state: &mut Self::State, ctx: &mut PaintCtx);

    /// Handle one `WidgetEvent`. Returns [`EventOutcome`] declaring
    /// whether the event was consumed exclusively, observed (side
    /// effect, no exclusivity), or ignored.
    fn apply_event(
        state: &mut Self::State,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome;

    /// Pre-register the panel's widget `NodeId`s against the
    /// `WidgetStore` at construction time. Matches the existing
    /// `pub fn populate(&mut WidgetStore)` shape.
    fn populate(store: &mut WidgetStore);
}
