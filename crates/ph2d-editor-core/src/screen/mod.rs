//! Editor screen orchestration — `HeroLayout` + (post-Phase-B.2)
//! the `HeroScreen` god-host struct.
//!
//! ADR-0029 Phase B.1 creates this module with `HeroLayout` only.
//! Phase B.2 moves `HeroScreen` here from `ph2d-editor`.

pub mod layout;

pub use layout::{
    EDGE_PAD, HERO_VIEWPORT_H, HERO_VIEWPORT_W, HIER_ROW_H, HIERARCHY_W, HUD_BOTTOM_PAD, HUD_H,
    HeroLayout, INSPECTOR_W, RAIL_W, TOPBAR_GAP, TOPBAR_H,
};
