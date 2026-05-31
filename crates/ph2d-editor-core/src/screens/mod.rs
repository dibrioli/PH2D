//! Composed full-viewport editor screens.
//!
//! ADR-0029 Phase B.2 moved `HeroScreen` + chrome + fixture from
//! `ph2d-editor::screens` into `ph2d-editor-core::screens`. The
//! `screens` module name (plural) is preserved so existing
//! `ph2d_editor::screens::*` imports continue to resolve through
//! the `ph2d-editor` shim re-export.
//!
//! Each module here paints one of the canonical mockups at
//! `docs/design/screens/`. Screens compose the widget primitives
//! into editor chrome (TopBar / LeftRail / Inspector / Hierarchy /
//! BottomHUD) over a canvas background — they DON'T own engine
//! state. Caller wires real entities/scene state when integrating
//! the screen into a project.

pub mod hero;
pub mod layout;

pub use hero::{
    BottomHudStats, HeroScreen, HeroSelection, InspectorNameInfo, InspectorOrderingInfo,
    InspectorOrderingMixed, InspectorSamplingInfo, InspectorSamplingMixed, InspectorSpriteInfo,
    InspectorSpriteMixed, InspectorSpriteSource, InspectorTransformInfo, InspectorVisibilityInfo,
    InspectorVisibilityMixed, InspectorVisibilitySectionInfo, OrderingFieldEdit,
    RequestedSpriteStrategy, SamplingFieldEdit, SpriteFieldEdit, ViewFocusKind,
    VisibilityFieldEdit, paint_hero_screen,
};
pub use layout::{
    EDGE_PAD, HERO_VIEWPORT_H, HERO_VIEWPORT_W, HIER_ROW_H, HIERARCHY_W, HUD_BOTTOM_PAD, HUD_H,
    HeroLayout, INSPECTOR_W, RAIL_W, TOPBAR_GAP, TOPBAR_H,
};
