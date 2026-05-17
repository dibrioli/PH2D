//! Wave 5 stage C — `PanelManifest` + `PanelRegistry`, the panel-side
//! mirror of `ph2d-tool-registry`'s `ToolManifest` + `Registry`.
//!
//! Each panel (Inspector, Hierarchy, Widget Gallery, Grid Snap) owns
//! a `pub static PANEL_MANIFEST: PanelManifest` const declaring its
//! id, NodeId, default visibility, and three fn pointers:
//!
//! - `paint_fn(ctx: &mut PaintCtx)` — full per-frame paint logic
//!   (visibility check + drag/resize clamp + chrome publish + actual
//!   paint + content_h publish + clear-on-hide).
//! - `apply_event_fn(hero: &mut HeroScreen, ev: WidgetEvent) -> bool`
//!   — routes panel-specific events. Returns `true` when the event
//!   was consumed.
//! - `populate_fn(store: &mut WidgetStore)` — pre-registration of
//!   the panel's widget NodeIds at HeroScreen construction time.
//!
//! `paint_hero_screen` collapses to a single iteration over
//! [`PANEL_REGISTRY`] in z-order, with `paint_fn` doing all per-panel
//! work. Future Wave 6 can lift each panel to `ph2d-panel-<slug>/`
//! crates with zero changes to this registry shape.

use ph2d_a11y::NodeId;

use crate::interaction::{WidgetEvent, WidgetStore};
use crate::screens::hero::{HeroLayout, HeroScreen};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// Per-frame context handed to every panel's `paint_fn`. Holds the
/// mutable refs and immutable layout/viewport the panel needs to
/// run its full frame logic without further state lookups.
///
/// Borrow note: `hero`, `scene`, and `text_system` are independent
/// `&mut`s; Rust's field-level borrow splitting handles disjoint use
/// within a single thunk (`ctx.hero.foo(); ctx.scene.bar();` is fine).
pub struct PaintCtx<'a> {
    /// HeroScreen owns panel visibility flags + WidgetStore + hit
    /// index. The thunk reads `hero.<panel>.visible` (early-return
    /// when hidden) and publishes content/visible heights via
    /// `hero.store.set_panel_content_h(...)`.
    pub hero: &'a mut HeroScreen,
    /// Pre-computed sub-region rects (INSP, HIER clamps already
    /// applied). Floating panels (Widget Gallery, Grid Snap) compute
    /// their own rects locally.
    pub layout: &'a HeroLayout,
    /// Outer viewport — needed by floating-panel clamping math.
    pub viewport: Rect,
    pub scene: &'a mut VectorScene,
    pub text_system: &'a mut TextSystem,
}

/// Paint function pointer — full per-frame logic for one panel.
/// Includes the visibility early-return so the registry iteration
/// in `paint_hero_screen` doesn't need to know which `HeroScreen`
/// field gates each panel.
pub type PaintFn = for<'a> fn(&mut PaintCtx<'a>);

/// Apply-event function pointer — routes a `WidgetEvent` to one
/// panel's apply logic. Returns `true` when the event was consumed
/// (so the dispatcher stops iterating).
///
/// Wave 5 stage D ships paint migration only; `apply_event_fn`
/// thunks are stubs returning `false` for now. The
/// `HeroScreen::apply_event` god-match stays the per-event dispatcher
/// until a later wave folds it into per-panel thunks.
pub type ApplyEventFn = fn(&mut HeroScreen, WidgetEvent) -> bool;

/// Populate function pointer — pre-registers a panel's widget
/// NodeIds against the WidgetStore at construction time. Matches
/// each panel module's existing `pub fn populate(&mut WidgetStore)`.
pub type PopulateFn = fn(&mut WidgetStore);

/// Static manifest declaring one panel's identity + behavior.
/// Mirrors `ph2d-tool-registry::ToolManifest` shape.
pub struct PanelManifest {
    /// Short stable id (used by future MCP exposure, logs, ADRs).
    pub id: &'static str,
    /// NodeId of the panel's outer rect — used by `paint_hero_screen`
    /// to look up the manifest by `z_order`'s `panel_id`. Matches
    /// the const NodeId each panel already publishes via
    /// `set_panel_rect(id, ...)` (e.g., `ids::INSP_PANEL`).
    pub panel_node_id: NodeId,
    /// Default `visible` value used by `HeroScreen::new`. Currently
    /// informational only — `HeroScreen::new` sets visibility per
    /// group struct (`InspectorState { visible: true }`, etc.).
    /// Future: registry iteration on construction could drive this.
    pub default_visible: bool,
    pub paint_fn: PaintFn,
    pub apply_event_fn: ApplyEventFn,
    pub populate_fn: PopulateFn,
}

/// Append-only registry of every panel manifest. Iterated by
/// `paint_hero_screen` in z-order; `find_by_panel_node_id` lets the
/// z_order walk look up the matching manifest.
pub struct PanelRegistry {
    manifests: &'static [&'static PanelManifest],
}

impl PanelRegistry {
    pub const fn new(manifests: &'static [&'static PanelManifest]) -> Self {
        Self { manifests }
    }

    pub fn manifests(&self) -> &'static [&'static PanelManifest] {
        self.manifests
    }

    /// Look up the manifest whose `panel_node_id == id`. Returns
    /// `None` for NodeIds that don't match any registered panel
    /// (e.g., `INSP_BLENDER_PICKER` which is painted out-of-band).
    pub fn find_by_panel_node_id(&self, id: NodeId) -> Option<&'static PanelManifest> {
        self.manifests
            .iter()
            .copied()
            .find(|m| m.panel_node_id == id)
    }
}

/// Append-only list of panel manifests. Adding a new panel = drop a
/// `pub static PANEL_MANIFEST: PanelManifest = ...` into the panel
/// module and add one line here. No edits to `paint_hero_screen` or
/// the chrome match arms — symmetric to `ph2d-tool-registry-init`.
pub static PANEL_REGISTRY: PanelRegistry = PanelRegistry::new(&[
    &crate::screens::hero::widget_gallery::PANEL_MANIFEST,
    &crate::screens::hero::hierarchy::PANEL_MANIFEST,
    &crate::screens::hero::inspector::PANEL_MANIFEST,
    &crate::grid_snap::PANEL_MANIFEST,
]);
