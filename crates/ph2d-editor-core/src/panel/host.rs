//! `PanelHost` + `PanelHostInternal` — interface that panels consume
//! from the host orchestrator (typically `HeroScreen` in
//! `ph2d-host-desktop`'s editor stack).
//!
//! ADR-0029 §4.1 + §4.2. Two tiers:
//!
//! - **`PanelHost`** (public, stable post-6-months) — minimal surface
//!   for 3rd-party panel authors. Currently empty; will carve out
//!   from `PanelHostInternal` once usage stabilizes. Subset of
//!   methods that 3rd parties can rely on across 0.x.y releases.
//!
//! - **`PanelHostInternal: PanelHost`** (`#[doc(hidden)]` unstable)
//!   — full surface used by the 4 in-tree panels (Inspector,
//!   Hierarchy, Widget Gallery, Grid Snap). 3rd parties can opt in
//!   at their own risk; semver doesn't apply.
//!
//! The split exists to give 3rd-party panel ergonomics without
//! freezing the internal API while the editor evolves. ADR-0030
//! (future, ~6 months post-merge) does the carve-out based on
//! observed usage.

use crate::action_bus::ActionBus;
use crate::interaction::{HitIndex, WidgetStore};
use crate::project::ProjectSettings;
use crate::screens::HeroSelection;
use ph2d_tokens::Theme;

/// Public-stable subset of host operations panels can consume across
/// 0.x.y PH2D releases.
///
/// ADR-0029 §4.1: surface is intentionally minimal pre-stabilization.
/// `panel_state_*<S>` is the typed downcast accessor for per-panel
/// state stored under `ErasedPanel`; consumers prefer this over
/// reaching into the internal tier directly.
///
/// Architecture gate: `tests/architecture_panel_host_surface.rs`
/// asserts ≤ 12 methods on this trait. Adding requires explicit
/// review.
pub trait PanelHost {
    fn theme(&self) -> Theme;
    fn project(&self) -> &ProjectSettings;
}

/// Full unstable surface used by in-tree panels. `#[doc(hidden)]`
/// at re-export sites.
///
/// ADR-0029 §4.2: tier expected to land with ~25-30 methods. Each
/// method documented with (a) which panel uses it, (b) reason.
/// Architecture gate: surface_count test asserts ≤ 35.
///
/// **Stability:** anything here may change shape between any two
/// 0.x.y releases. Internal-only — 3rd parties opt in at their
/// own risk.
#[doc(hidden)]
pub trait PanelHostInternal: PanelHost {
    fn store(&self) -> &WidgetStore;
    fn store_mut(&mut self) -> &mut WidgetStore;
    fn hit_index_mut(&mut self) -> &mut HitIndex;

    /// Joint borrow: returns `(&WidgetStore, &mut HitIndex)` in one
    /// call so panel paint code can pass them to inner painters
    /// without tripping the borrow checker on aliased dyn-trait
    /// method calls. Concrete impls split the fields directly.
    fn store_and_hit_index_mut(&mut self) -> (&WidgetStore, &mut HitIndex);

    /// Outbound editor-action queue. Panels push commits here for the
    /// shell to drain each frame.
    fn bus(&self) -> &ActionBus;
    fn bus_mut(&mut self) -> &mut ActionBus;

    /// Currently selected hero entity (canvas-side). `None` when
    /// nothing is selected. Inspector reads to drive its placeholder
    /// text; hierarchy reads/writes via [`Self::selection_mut`].
    fn selection(&self) -> Option<&HeroSelection>;
    fn selection_mut(&mut self) -> &mut Option<HeroSelection>;

    /// Whether the panel identified by `id` (matching
    /// [`crate::panel::Panel::ID`]) is currently visible. Host
    /// maintains the visibility map; left-rail toggles + panel-close
    /// affordances mutate it via [`Self::set_panel_visible`].
    fn panel_visible(&self, id: &str) -> bool;
    fn set_panel_visible(&mut self, id: &str, value: bool);
}
