//! Composed full-viewport editor screens.
//!
//! Each module here paints one of the canonical mockups at
//! `docs/design/screens/`. Screens compose the widget primitives
//! into editor chrome (TopBar / LeftRail / Inspector / Hierarchy /
//! BottomHUD) over a canvas background — they DON'T own engine
//! state. Caller wires real entities/scene state when integrating
//! the screen into a project.
//!
//! The canonical UI reference for peripheral agents is the
//! **Widget Gallery** floating panel inside the live hero, toggled
//! by the palette pill in the TopBar. It re-uses the section
//! painters preserved in [`hero::inspector`] so there is no separate
//! "frozen reference" build to maintain.

pub mod hero;

pub use hero::{
    BottomHudStats, HeroScreen, HeroSelection, InspectorSpriteInfo, InspectorSpriteSource,
    ViewFocusKind, paint_hero_screen, set_live_component_count,
};
