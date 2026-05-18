//! Thread-local snapshots + last-paint measurements for the
//! live Inspector panel.
//!
//! Wave 8 Phase 3 (audit A1): items here are `pub` so the host
//! shell can publish snapshots before paint, but they are NOT
//! stable user API. The module is `#[doc(hidden)]` at re-export
//! sites; this is panel-author infrastructure.
//!
//! Wave 8 Phase 2.A — gallery-related thread-locals
//! (LAST_BODY_TOP_SCREEN_Y, LAST_SECTION_TOPS_Y, PENDING_DROPDOWN_CHIP,
//! LAST_GALLERY_*, push_section_top_y, section_index_below_body_y,
//! take_pending_dropdown_chip, etc.) moved to
//! `crate::widget::showcase::state` so the showcase tree
//! (also in editor-core) can write to them without depending on
//! `ph2d-editor`. Re-exported from this module for backwards
//! compatibility with existing call sites in `ph2d-editor`.
//!
//! The remaining items here are LIVE Inspector-specific: the
//! `CURRENT_INSPECTOR_*` host-supplied snapshots + LAST_CONTENT_H /
//! LAST_VISIBLE_H for inspector-panel scroll clamping.

use super::super::{
    InspectorNameInfo, InspectorSpriteInfo, InspectorTransformInfo, InspectorVisibilityInfo,
};

// Wave 8 Phase 2.A re-exports — gallery / shared section state lives
// in editor-core. Existing inspector consumers keep `use state::*` style
// imports working.
pub use crate::widget::showcase::{
    LAST_BODY_TOP_SCREEN_Y, LAST_SECTION_TOPS_Y, last_body_top_screen_y, last_gallery_content_h,
    last_gallery_visible_h, push_section_top_y, section_index_below_body_y,
    take_pending_dropdown_chip,
};

thread_local! {
    /// Content height measured during the previous paint pass of the
    /// LIVE Inspector. The wheel dispatch reads this via
    /// [`last_inspector_content_h`] to clamp `scroll_y`. One frame of
    /// staleness is invisible since paint runs every frame.
    pub(super) static LAST_CONTENT_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };

    /// Exact visible body height of the inspector's last paint.
    pub(super) static LAST_VISIBLE_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };

    /// M14.5: live sprite snapshot published by the host before
    /// each paint_inspector call and cleared after.
    pub(super) static CURRENT_INSPECTOR_SPRITE: std::cell::RefCell<Option<InspectorSpriteInfo>> =
        const { std::cell::RefCell::new(None) };

    /// M14.A: live Transform snapshot for the Transform editor section.
    pub(super) static CURRENT_INSPECTOR_TRANSFORM: std::cell::RefCell<Option<InspectorTransformInfo>> =
        const { std::cell::RefCell::new(None) };

    /// M14.D: live Visibility checkbox row.
    pub(super) static CURRENT_INSPECTOR_VISIBILITY:
        std::cell::Cell<Option<InspectorVisibilityInfo>> =
        const { std::cell::Cell::new(None) };

    /// M14.E: editable entity-name snapshot.
    pub(super) static CURRENT_INSPECTOR_NAME: std::cell::RefCell<Option<InspectorNameInfo>> =
        const { std::cell::RefCell::new(None) };

    /// Per-paint display unit + pixels_per_meter for Transform section
    /// labels + value formatting.
    pub(super) static CURRENT_DISPLAY_UNIT: std::cell::Cell<crate::project::DisplayUnit> =
        const { std::cell::Cell::new(crate::project::DisplayUnit::Meters) };
    pub(super) static CURRENT_PIXELS_PER_METER: std::cell::Cell<f32> =
        const { std::cell::Cell::new(crate::project::DEFAULT_PIXELS_PER_METER) };
}

/// Set the inspector sprite snapshot for the current paint.
pub fn set_current_inspector_sprite(info: Option<InspectorSpriteInfo>) {
    CURRENT_INSPECTOR_SPRITE.with(|c| *c.borrow_mut() = info);
}

pub(super) fn current_inspector_sprite() -> Option<InspectorSpriteInfo> {
    CURRENT_INSPECTOR_SPRITE.with(|c| c.borrow().clone())
}

pub fn set_current_inspector_transform(info: Option<InspectorTransformInfo>) {
    CURRENT_INSPECTOR_TRANSFORM.with(|c| *c.borrow_mut() = info);
}

pub(super) fn current_inspector_transform() -> Option<InspectorTransformInfo> {
    CURRENT_INSPECTOR_TRANSFORM.with(|c| *c.borrow())
}

pub fn set_current_inspector_visibility(info: Option<InspectorVisibilityInfo>) {
    CURRENT_INSPECTOR_VISIBILITY.with(|c| c.set(info));
}

pub(super) fn current_inspector_visibility() -> Option<InspectorVisibilityInfo> {
    CURRENT_INSPECTOR_VISIBILITY.with(|c| c.get())
}

pub fn set_current_inspector_name(info: Option<InspectorNameInfo>) {
    CURRENT_INSPECTOR_NAME.with(|c| *c.borrow_mut() = info);
}

pub(super) fn current_inspector_name_is_some() -> bool {
    CURRENT_INSPECTOR_NAME.with(|c| c.borrow().is_some())
}

pub fn set_current_display_unit(unit: crate::project::DisplayUnit, pixels_per_meter: f32) {
    CURRENT_DISPLAY_UNIT.with(|c| c.set(unit));
    CURRENT_PIXELS_PER_METER.with(|c| c.set(pixels_per_meter));
}

pub(super) fn current_display_unit() -> crate::project::DisplayUnit {
    CURRENT_DISPLAY_UNIT.with(|c| c.get())
}

pub(super) fn current_pixels_per_meter() -> f32 {
    CURRENT_PIXELS_PER_METER.with(|c| c.get())
}

/// Last-known total content height of the inspector body. Used by
/// `dispatch_wheel` to clamp the scroll offset.
pub fn last_inspector_content_h() -> f32 {
    LAST_CONTENT_H.with(|c| c.get())
}

pub(super) fn set_last_inspector_content_h(h: f32) {
    LAST_CONTENT_H.with(|c| c.set(h));
}

pub fn last_inspector_visible_h() -> f32 {
    LAST_VISIBLE_H.with(|c| c.get())
}

pub(super) fn set_last_inspector_visible_h(h: f32) {
    LAST_VISIBLE_H.with(|c| c.set(h));
}
