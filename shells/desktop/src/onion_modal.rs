//! Onion settings modal — the shell half (ADR-0142 W3b).
//!
//! The card itself is hero chrome (`ph2d_editor::…::chrome::onion_modal`): it paints from and writes
//! its widget values into the [`WidgetStore`]. This module is the glue the shell owns:
//!
//! * [`read_into`] — each frame the card is open, read its slider/swatch values back into
//!   [`OnionSettings`] (the ghost pass re-reads that struct every frame ⇒ live edits on the canvas).
//!   ⚠️ **The WidgetStore is the shared blackboard** — editor-core cannot see `OnionSettings`. The
//!   count↔slider mapping ([`MAX_GHOSTS`] / [`count_to_frac`] / [`frac_to_count`]) lives in
//!   editor-core, next to the modal's painter that displays the count, and is **re-exported here** so
//!   there is one copy; the read-back and open-seed both use it.
//! * The title-band **drag** ([`App::arm_onion_modal_drag_if_on_handle`] / `…_move` / `…_up`) — a
//!   shell state machine, a byte-for-byte mirror of the Fill modal's (`input_dispatch::fill_drag`).
//!
//! Opening the card is inline in `render_loop` (the `TimelinePanelEvent` handler seeds the store from
//! `self.timeline.onion` — shell-side because the card lives in `hero.store`, out of the panel's
//! reach, mirroring the Motion Path toggle).

use std::cell::Cell;

use ph2d_editor::ids;
use ph2d_editor::interaction::WidgetStore;
use ph2d_timeline::OnionSettings;

use crate::App;

/// The ghost-count↔slider mapping. Defined in editor-core next to the modal's painter (the one that
/// displays the count) so there is ONE copy; re-exported here so `crate::onion_modal::…` and the
/// read-back below keep resolving.
pub use ph2d_editor::screens::hero::chrome::{count_to_frac, frac_to_count};

/// `OnionSettings` RGB (`[f32; 3]`, linear-ish 0..1) → an `[u8; 4]` swatch seed (opaque).
#[must_use]
pub fn rgb_to_u8(rgb: [f32; 3]) -> [u8; 4] {
    let c = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u8;
    [c(rgb[0]), c(rgb[1]), c(rgb[2]), 0xFF]
}

/// An `[u8; 4]` picker read-back → `OnionSettings` RGB (`[f32; 3]`; alpha dropped — a ghost's alpha
/// is the Opacity slider, not the colour).
#[must_use]
pub fn u8_to_rgb(rgba: [u8; 4]) -> [f32; 3] {
    [
        f32::from(rgba[0]) / 255.0,
        f32::from(rgba[1]) / 255.0,
        f32::from(rgba[2]) / 255.0,
    ]
}

/// While the onion modal is open, read its widgets back into `onion` (the count/opacity/colour half;
/// `enabled`/`mode` stay owned by the transport toggles). No-op when the modal is closed — so the
/// transport toggles keep their values. Called each frame before the onion ghost pass.
pub fn read_into(store: &WidgetStore, onion: &mut OnionSettings) {
    if store.onion_modal_pos().is_none() {
        return;
    }
    if let Some((_, v)) = store.slider(ids::TIMELINE_ONION_MODAL_OPACITY) {
        onion.opacity = v;
    }
    if let Some((_, v)) = store.slider(ids::TIMELINE_ONION_MODAL_BEFORE) {
        onion.frames_before = frac_to_count(v);
    }
    if let Some((_, v)) = store.slider(ids::TIMELINE_ONION_MODAL_AFTER) {
        onion.frames_after = frac_to_count(v);
    }
    if let Some(c) = store.widget_color(ids::TIMELINE_ONION_MODAL_COLOR_BEFORE) {
        onion.color_before = u8_to_rgb(c);
    }
    if let Some(c) = store.widget_color(ids::TIMELINE_ONION_MODAL_COLOR_AFTER) {
        onion.color_after = u8_to_rgb(c);
    }
}

thread_local! {
    /// `Some((last_x, last_y))` while the onion modal's title band is being dragged — the last cursor
    /// position, so each move offsets the card by the raw cursor delta (no dead-zone). `None` idle.
    static ONION_MODAL_DRAG: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
}

impl App {
    /// A Primary Down over the onion modal's title band arms a modal-move drag. Returns `true`
    /// (consume the Down) when it hits the handle, so the card moves instead of the Down doing
    /// anything else (and the modal never closes while dragging). Mirror of the Fill modal's.
    pub(crate) fn arm_onion_modal_drag_if_on_handle(&mut self, px: f32, py: f32) -> bool {
        let on_handle = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.hit_index.hit(px, py))
            == Some(ids::TIMELINE_ONION_MODAL_HANDLE);
        if !on_handle {
            return false;
        }
        ONION_MODAL_DRAG.with(|c| c.set(Some((px, py))));
        true
    }

    /// CursorMoved while the onion modal is being dragged: offset the card by the cursor delta.
    /// Returns `true` (consume the move) while dragging, so it doesn't pan / drive a gizmo.
    pub(crate) fn onion_modal_drag_move(&mut self, px: f32, py: f32) -> bool {
        let Some((lx, ly)) = ONION_MODAL_DRAG.with(Cell::get) else {
            return false;
        };
        ONION_MODAL_DRAG.with(|c| c.set(Some((px, py))));
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.store.move_onion_modal(px - lx, py - ly);
        }
        true
    }

    /// Primary Up: end an onion modal title-band drag. No-op when not dragging.
    pub(crate) fn onion_modal_drag_up(&mut self) {
        ONION_MODAL_DRAG.with(|c| c.set(None));
    }
}

#[cfg(test)]
#[path = "onion_modal_tests.rs"]
mod onion_modal_tests;
