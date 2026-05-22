//! Background Removal — UI/action vocabulary (foundation side).
//!
//! ADR-0040 T2: the heavy implementation (`algorithm`, `scratch`, `tool`,
//! `icon`) moved to crate `ph2d-tool-bgremoval`. What stays here is only the
//! **vocabulary** that the foundation legitimately owns and that would
//! otherwise force an `editor-core → tool` cycle:
//!
//! - [`params::BgRemovalUiEdit`] — carried by `EditorAction::BgremovalUiEdit`
//!   (the action bus is editor-core's), so it must be reachable here.
//! - [`params::BgRemovalUiSnapshot`] / [`params::BrushFalloff`] — read by the
//!   `ph2d-panel-bgremoval` crate, which depends on editor-core (not on the
//!   tool crate).
//! - [`params::BgRemovalParams`] + the full-scale mapping consts — the single
//!   source of truth the snapshot's `Default` derives from; the tool crate
//!   imports these (tool → editor-core is the allowed direction).
//!
//! The tool crate re-exports this `params` module at its own root so its moved
//! files' `super::params` paths resolve unchanged.
//!
//! When the generic action channel lands (ADR-0040 T1.2/T1.3), this vocabulary
//! moves into the tool crate too and this module disappears.

pub mod params;

pub use params::{
    BgRemovalParams, BgRemovalUiEdit, BgRemovalUiSnapshot, BrushFalloff, ChromaParams,
    GuidedFilterParams, MAX_EXTRA_BG_COLORS,
};
