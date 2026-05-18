//! Thread-locals used by `paint_showcase_body` (Widget Gallery)
//! and shared with the live Inspector panel (section-tops + body
//! origin Y, for right-click "insert note above section" routing).
//!
//! Wave 8 Phase 2.A — hoisted from
//! `ph2d_editor::screens::hero::inspector::state` so the showcase
//! tree (now in editor-core) can publish its measurements without
//! depending back on `ph2d-editor`.

use crate::zones::Rect;

thread_local! {
    /// Pending Dropdown popover capture: when the showcase paints
    /// the open Lists dropdown, it stashes `(selected_index, chip_rect)`
    /// here so the late-paint phase (after every section ran) can
    /// paint the popover on top.
    pub static PENDING_DROPDOWN_CHIP: std::cell::RefCell<Option<(usize, Rect)>> =
        const { std::cell::RefCell::new(None) };

    /// Body-relative top-Y of each painted section's header, indexed
    /// by section position. The right-click dispatch consumes this to
    /// pick which section a new note should be inserted ABOVE. Stored
    /// body-relative so it stays stable across scroll offsets.
    ///
    /// Shared between live Inspector + Widget Gallery — whichever
    /// panel painted last wins, which is intentional: the user's
    /// right-click target is whichever panel was painted underneath.
    pub static LAST_SECTION_TOPS_Y: std::cell::RefCell<Vec<f32>> =
        const { std::cell::RefCell::new(Vec::new()) };

    /// Body-top screen-Y captured each paint frame. Callers convert
    /// screen-y → body-y via `event.y - body_top_screen + scroll_y`.
    pub static LAST_BODY_TOP_SCREEN_Y: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };

    /// Last-known total content height of the Widget Gallery body.
    /// Read by the host after `paint_showcase_body` to clamp the
    /// wheel-scroll bound on `GAL_PANEL` independent of the live
    /// Inspector's `LAST_CONTENT_H`.
    pub static LAST_GALLERY_CONTENT_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };

    /// Visible body height (`content_bottom - content_top`) of the
    /// Widget Gallery's last paint. Paired with `LAST_GALLERY_CONTENT_H`
    /// to derive max scroll.
    pub static LAST_GALLERY_VISIBLE_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
}

pub fn set_pending_dropdown_chip(chip: Option<(usize, Rect)>) {
    PENDING_DROPDOWN_CHIP.with(|c| *c.borrow_mut() = chip);
}

pub fn take_pending_dropdown_chip() -> Option<(usize, Rect)> {
    PENDING_DROPDOWN_CHIP.with(|c| c.borrow_mut().take())
}

/// Last-known total content height of the Widget Gallery body
/// (sum of all section heights). Used by `dispatch_wheel` to clamp
/// the scroll offset.
pub fn last_gallery_content_h() -> f32 {
    LAST_GALLERY_CONTENT_H.with(|c| c.get())
}

pub fn last_gallery_visible_h() -> f32 {
    LAST_GALLERY_VISIBLE_H.with(|c| c.get())
}

pub fn set_last_gallery_content_h(h: f32) {
    LAST_GALLERY_CONTENT_H.with(|c| c.set(h));
}

pub fn set_last_gallery_visible_h(h: f32) {
    LAST_GALLERY_VISIBLE_H.with(|c| c.set(h));
}

/// Find the section index whose body the given body-relative y
/// lies INSIDE. Returns `Some(i)` so callers know a new note
/// should be inserted above `SECTION_IDS[i]`. Returns `None` when
/// y is past the last section's content (note appends to bottom).
pub fn section_index_below_body_y(body_y: f32) -> Option<u8> {
    LAST_SECTION_TOPS_Y.with(|tops| {
        let tops = tops.borrow();
        for i in 0..tops.len() {
            let top = tops[i];
            let next = tops.get(i + 1).copied().unwrap_or(f32::INFINITY);
            if body_y >= top && body_y < next {
                return Some(i as u8);
            }
            if i == 0 && body_y < top {
                return Some(0);
            }
        }
        None
    })
}

pub fn last_body_top_screen_y() -> f32 {
    LAST_BODY_TOP_SCREEN_Y.with(|c| c.get())
}

pub fn push_section_top_y(tops: &mut Vec<f32>, body_y: f32) {
    tops.push(body_y);
}
