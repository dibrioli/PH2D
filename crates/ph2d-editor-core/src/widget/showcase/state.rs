//! Thread-locals used by `paint_showcase_body` (Widget Gallery).
//!
//! ⛔ As alturas das secções e a origem do corpo (`LAST_SECTION_TOPS_Y`,
//! `LAST_BODY_TOP_SCREEN_Y`) SAÍRAM em 2026-09-30: a secção onde uma nota nasce ou cai lê-se
//! dos cabeçalhos que o quadro anterior registou no hit-index
//! (`interaction::dispatch::note_drag::seccao_sob`) — uma lista paralela a menos para discordar.
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
