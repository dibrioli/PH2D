//! Composed full-viewport editor screens.
//!
//! Each module here paints one of the canonical mockups at
//! `docs/design/screens/`. Screens compose the widget primitives
//! into editor chrome (TopBar / LeftRail / Inspector / Hierarchy /
//! BottomHUD) over a canvas background — they DON'T own engine
//! state. Caller wires real entities/scene state when integrating
//! the screen into a project.
//!
//! ## Reference snapshot
//!
//! Building with the `reference-snapshot` feature swaps the `hero`
//! module for `hero_ref` — a frozen verbatim copy used as a visual
//! baseline while iterating on the working hero. Default builds use
//! `hero`. The launcher script `reference.command` enables the
//! feature; `play.command` does not.

#[cfg(not(feature = "reference-snapshot"))]
pub mod hero;

#[cfg(feature = "reference-snapshot")]
pub mod hero_ref;
#[cfg(feature = "reference-snapshot")]
pub use hero_ref as hero;

pub use hero::{
    BottomHudStats, HeroScreen, HeroSelection, ViewFocusKind, paint_hero_screen,
    set_live_component_count,
};
