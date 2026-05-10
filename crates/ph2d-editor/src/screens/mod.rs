//! Composed full-viewport editor screens.
//!
//! Each module here paints one of the canonical mockups at
//! `docs/design/screens/`. Screens compose the widget primitives
//! into editor chrome (TopBar / LeftRail / Inspector / Hierarchy /
//! BottomHUD) over a canvas background — they DON'T own engine
//! state. Caller wires real entities/scene state when integrating
//! the screen into a project.

pub mod hero;

pub use hero::{HeroScreen, HeroSelection, paint_hero_screen};
