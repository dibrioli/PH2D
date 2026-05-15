//! Thread-local snapshots + last-paint measurements for the Inspector.
//!
//! Extracted from [`super`] (Track C1). The painter has no direct
//! access to the host's ECS — instead, `paint_hero_screen` sets
//! these thread-locals just before calling `paint_inspector` and
//! clears them afterwards so a stale snapshot can't leak into the
//! next frame.
//!
//! The painter also publishes paint-pass measurements back via this
//! module (last content height / visible height for scroll-clamping;
//! per-section body-relative top-Y for the right-click "insert note
//! above section" routing).

use super::super::{
    InspectorNameInfo, InspectorSpriteInfo, InspectorTransformInfo, InspectorVisibilityInfo,
};
use crate::zones::Rect;

thread_local! {
    pub(super) static PENDING_DROPDOWN_CHIP: std::cell::RefCell<Option<(usize, Rect)>> =
        const { std::cell::RefCell::new(None) };
    /// Content height measured during the previous paint pass. The
    /// wheel dispatch reads this via [`last_inspector_content_h`] to
    /// clamp `scroll_y` to `[0, content_h - visible_h]`. One frame of
    /// staleness is invisible since paint runs every frame.
    pub(super) static LAST_CONTENT_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
    /// Exact visible body height (`content_bottom - content_top`) of
    /// the inspector's last paint. Used together with content_h to
    /// derive max_scroll. Bypasses the rough `panel.h - 60` heuristic
    /// which over-estimated visible_h and clamped the scroll too
    /// early — last few px of new notes weren't reachable.
    pub(super) static LAST_VISIBLE_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
    /// Body-relative top-Y of each painted section's header, indexed
    /// by section position in `SECTION_IDS`. The right-click dispatch
    /// uses this to pick which section a new note should be inserted
    /// ABOVE (the user's "nota deve ser inserida acima do objeto
    /// selecionado"). Body-relative so it stays stable across
    /// scroll offsets — the lookup converts the click's screen y
    /// into body-y via `event.y - body_top_screen + scroll_y`.
    pub(super) static LAST_SECTION_TOPS_Y: std::cell::RefCell<Vec<f32>> =
        const { std::cell::RefCell::new(Vec::new()) };
    /// Body-relative top-Y in screen coords reference, captured each
    /// frame so callers (the hero) can convert screen-y → body-y.
    pub(super) static LAST_BODY_TOP_SCREEN_Y: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
    /// M14.5 inspector phase (6.4/§9): live snapshot the host
    /// publishes each frame so `paint_inspector` can render the
    /// Render Source section + Reimport button without crossing the
    /// ADR-0021 / HR-8 boundary into SimWorld. `None` when nothing is
    /// selected or selection isn't a sprite.
    pub(super) static CURRENT_INSPECTOR_SPRITE: std::cell::RefCell<Option<InspectorSpriteInfo>> =
        const { std::cell::RefCell::new(None) };
    /// M14.A: same shape as `CURRENT_INSPECTOR_SPRITE` for the live
    /// `Transform` editor section. `paint_hero_screen` publishes this
    /// before `paint_inspector` and clears it after so a stale
    /// snapshot can't leak into the next frame.
    pub(super) static CURRENT_INSPECTOR_TRANSFORM: std::cell::RefCell<Option<InspectorTransformInfo>> =
        const { std::cell::RefCell::new(None) };
    /// M14.D: same pattern for the Visibility checkbox row. Held as
    /// a `Cell` (struct is `Copy`) so the painter's read is
    /// allocation-free.
    pub(super) static CURRENT_INSPECTOR_VISIBILITY:
        std::cell::Cell<Option<InspectorVisibilityInfo>> =
        const { std::cell::Cell::new(None) };
    /// M14.E: editable entity-name snapshot. `RefCell` because the
    /// inner `InspectorNameInfo` carries an owned `String` (entity
    /// names can be longer than `Copy` is convenient for).
    pub(super) static CURRENT_INSPECTOR_NAME: std::cell::RefCell<Option<InspectorNameInfo>> =
        const { std::cell::RefCell::new(None) };
    /// Mirror of `LAST_CONTENT_H` / `LAST_VISIBLE_H` for the floating
    /// Widget Gallery panel painted by [`super::paint_showcase_body`].
    /// Tracked independently so the gallery and Inspector scroll
    /// without aliasing each other's clamp bound.
    pub(super) static LAST_GALLERY_CONTENT_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
    pub(super) static LAST_GALLERY_VISIBLE_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
}

/// Set the inspector sprite snapshot for the current paint. Hero
/// publishes this before `paint_inspector` runs and clears it after,
/// matching the [[hierarchy_live_entries]] thread-local pattern.
pub(in crate::screens::hero) fn set_current_inspector_sprite(info: Option<InspectorSpriteInfo>) {
    CURRENT_INSPECTOR_SPRITE.with(|c| *c.borrow_mut() = info);
}

pub(super) fn current_inspector_sprite() -> Option<InspectorSpriteInfo> {
    CURRENT_INSPECTOR_SPRITE.with(|c| c.borrow().clone())
}

/// M14.A: paired with [`set_current_inspector_sprite`] for the
/// Transform live-binding section. `paint_hero_screen` is the only
/// publisher.
pub(in crate::screens::hero) fn set_current_inspector_transform(
    info: Option<InspectorTransformInfo>,
) {
    CURRENT_INSPECTOR_TRANSFORM.with(|c| *c.borrow_mut() = info);
}

pub(super) fn current_inspector_transform() -> Option<InspectorTransformInfo> {
    CURRENT_INSPECTOR_TRANSFORM.with(|c| *c.borrow())
}

/// M14.D: same as `set_current_inspector_transform` for the
/// Visibility checkbox row.
pub(in crate::screens::hero) fn set_current_inspector_visibility(
    info: Option<InspectorVisibilityInfo>,
) {
    CURRENT_INSPECTOR_VISIBILITY.with(|c| c.set(info));
}

pub(super) fn current_inspector_visibility() -> Option<InspectorVisibilityInfo> {
    CURRENT_INSPECTOR_VISIBILITY.with(|c| c.get())
}

/// M14.E: same shape for the editable entity-name field.
pub(in crate::screens::hero) fn set_current_inspector_name(info: Option<InspectorNameInfo>) {
    CURRENT_INSPECTOR_NAME.with(|c| *c.borrow_mut() = info);
}

/// Audit #2 fix (LOW, clone elision): presence check without cloning
/// the inner `String`. The painter reads the live `name` from the
/// store buffer (the host writes it during the selection-change reset
/// in `paint_hero_screen`); the snapshot is only consulted to decide
/// whether to paint the row at all.
pub(super) fn current_inspector_name_is_some() -> bool {
    CURRENT_INSPECTOR_NAME.with(|c| c.borrow().is_some())
}

pub(super) fn set_pending_dropdown_chip(chip: Option<(usize, Rect)>) {
    PENDING_DROPDOWN_CHIP.with(|c| *c.borrow_mut() = chip);
}

pub(super) fn take_pending_dropdown_chip() -> Option<(usize, Rect)> {
    PENDING_DROPDOWN_CHIP.with(|c| c.borrow_mut().take())
}

/// Last-known total content height of the inspector body (sum of all
/// section heights + gaps). Used by `dispatch_wheel` to clamp the
/// scroll offset so the user can't scroll past the last element.
pub(in crate::screens::hero) fn last_inspector_content_h() -> f32 {
    LAST_CONTENT_H.with(|c| c.get())
}

pub(super) fn set_last_inspector_content_h(h: f32) {
    LAST_CONTENT_H.with(|c| c.set(h));
}

pub(in crate::screens::hero) fn last_inspector_visible_h() -> f32 {
    LAST_VISIBLE_H.with(|c| c.get())
}

pub(super) fn set_last_inspector_visible_h(h: f32) {
    LAST_VISIBLE_H.with(|c| c.set(h));
}

/// Gallery counterparts of `last_inspector_content_h` / `last_inspector_visible_h`.
/// Read by the host after [`super::paint_showcase_body`] to clamp the
/// wheel-scroll bound on `GAL_PANEL`.
pub(in crate::screens::hero) fn last_gallery_content_h() -> f32 {
    LAST_GALLERY_CONTENT_H.with(|c| c.get())
}

pub(in crate::screens::hero) fn last_gallery_visible_h() -> f32 {
    LAST_GALLERY_VISIBLE_H.with(|c| c.get())
}

pub(super) fn set_last_gallery_content_h(h: f32) {
    LAST_GALLERY_CONTENT_H.with(|c| c.set(h));
}

pub(super) fn set_last_gallery_visible_h(h: f32) {
    LAST_GALLERY_VISIBLE_H.with(|c| c.set(h));
}

/// Find the section index whose body the given body-relative y
/// lies INSIDE. Returns `Some(i)` so callers know a new note
/// should be inserted above `SECTION_IDS[i]` (i.e. above the
/// section the user right-clicked into). Returns `None` when y is
/// past the last section's content (note appends to the bottom).
///
/// Previous version returned "the first section whose top > y" —
/// which for clicks INSIDE section A returned the index of
/// section B, so the new note went BELOW A's separator instead of
/// above A's header (user reported "note created below the separator
/// of the section the right-click landed in").
pub(in crate::screens::hero) fn section_index_below_body_y(body_y: f32) -> Option<u8> {
    LAST_SECTION_TOPS_Y.with(|tops| {
        let tops = tops.borrow();
        // Walk pairs (top[i], top[i+1]); the click is "inside"
        // section i when top[i] <= y < top[i+1]. The last section
        // has no successor — clicks past its top fall through to
        // `None` (trailing note).
        for i in 0..tops.len() {
            let top = tops[i];
            let next = tops.get(i + 1).copied().unwrap_or(f32::INFINITY);
            if body_y >= top && body_y < next {
                return Some(i as u8);
            }
            // Click ABOVE the very first section's top → insert
            // before that section.
            if i == 0 && body_y < top {
                return Some(0);
            }
        }
        None
    })
}

pub(in crate::screens::hero) fn last_body_top_screen_y() -> f32 {
    LAST_BODY_TOP_SCREEN_Y.with(|c| c.get())
}

pub(super) fn push_section_top_y(tops: &mut Vec<f32>, body_y: f32) {
    tops.push(body_y);
}
