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

use crate::interaction::{HitIndex, WidgetStore};
use crate::project::ProjectSettings;
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
}
