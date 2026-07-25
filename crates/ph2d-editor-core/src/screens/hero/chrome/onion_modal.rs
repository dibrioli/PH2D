// ph2d-chrome-sync:z=180 (dispatch priority, ADR-0107; lower = earlier)
//! Onion settings floating modal (ADR-0142 W3b) — the draggable card holding the onion's
//! counts / opacity / colours. The transport bar has room only for the on/off + Keys toggles, so
//! the detail lives here (Enio: *"esse card deve viver num modal já que não cabe na timeline"*).
//! Two halves, colocated, mirroring [`super::fill_modal`]:
//!
//! * [`paint_onion_modal`] — renders the card at `store.onion_modal_pos()` (clamped to the viewport)
//!   with a title drag-handle + close X, an Opacity slider, two ghost-count sliders, and two colour
//!   swatches. Called from the hero paint pass.
//! * [`apply`] — the dispatch half wired into `chrome::dispatch_all`: the close X closes the modal,
//!   the title-band handle click is consumed (the actual move is the shell's drag machine), and the
//!   sliders' `ValueChanged` is consumed so it never leaks to another handler.
//!
//! ⚠️ **The WidgetStore is the shared blackboard.** The slider/swatch values live in the store
//! (updated by the generic pointer dispatch); the SHELL reads them back into `TimelineState::onion`
//! every frame the modal is open (live edits on the canvas). So `apply` forwards NOTHING — there is
//! no tool to forward to (the onion is timeline view state, not a tool), and no `EditorAction`
//! variant naming an onion (editor-core stays timeline-agnostic). The count↔slider mapping (the ghost
//! counts are `u32`, the sliders are `0..1`) lives ONLY in the shell, so there is one copy of it; the
//! labels here are static, and the slider fill communicates the level.

use crate::ids;
use crate::interaction::{HitIndex, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use crate::screens::hero::HeroScreen;
use crate::widget::{Button, ButtonState, Slider, SliderState, paint_button, paint_slider};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Color, VectorScene};

/// Card width — fits a "Ghosts Before" label + slider on one row.
const MODAL_W: f32 = 240.0; // LITERAL-PX-OK: onion settings card width
/// Colour-swatch width in a colour row (the label fills the rest).
const SWATCH_W: f32 = 56.0; // LITERAL-PX-OK: onion colour swatch width
/// Close-X square in the title band.
const CLOSE_W: f32 = 20.0; // LITERAL-PX-OK: onion modal close-X width
/// Row count: title · opacity · before · after · past-colour · future-colour.
const ROWS: f32 = 6.0; // LITERAL-PX-OK: CONTAGEM de linhas (as seis listadas acima), nao medida

/// Paint the Onion settings modal at `store.onion_modal_pos()`, clamped to `viewport`. No-op when the
/// modal is closed. Mirrors [`super::fill_modal::paint_fill_adjust_modal`]'s card styling.
pub fn paint_onion_modal(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    viewport: Rect,
) {
    let Some((x, y)) = store.onion_modal_pos() else {
        return;
    };
    let row_h = ROW_H_PX;
    let gap = Spacing::Xs.px();
    let pad_y = Spacing::Sm.px();
    let total_h = pad_y * 2.0 + row_h * ROWS + gap * (ROWS - 1.0);

    // Clamp the top-left so the whole card stays inside the viewport (mirror of `fill_modal`).
    let max_x = (viewport.x + viewport.w - MODAL_W).max(viewport.x);
    let max_y = (viewport.y + viewport.h - total_h).max(viewport.y);
    let rect_x = x.clamp(viewport.x, max_x); // CLAMP-OK: bounds ordered (max_x ≥ viewport.x) + non-NaN
    let rect_y = y.clamp(viewport.y, max_y); // CLAMP-OK: bounds ordered (max_y ≥ viewport.y) + non-NaN
    let rect = Rect::new(rect_x, rect_y, MODAL_W, total_h);

    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    let inner_x = rect.x + Spacing::Md.px();
    let inner_w = rect.w - Spacing::Md.px() * 2.0;
    let font = TypeToken::Sm.px();
    let mut cy = rect.y + pad_y;

    // ── Title band = drag handle (left) + close X (right). The handle stops short of the X so the
    //    two hit rects never share a pixel (a Down on the X closes; a Down on the band drags). ──
    let close_x = rect.x + rect.w - CLOSE_W - Spacing::Sm.px();
    let handle_rect = Rect::new(rect.x, rect.y, close_x - rect.x, pad_y + row_h);
    hit_index.register(ids::TIMELINE_ONION_MODAL_HANDLE, handle_rect);
    paint_text(
        text_system,
        scene,
        ph2d_i18n::tr("panel.timeline.onion_settings"),
        inner_x,
        cy + (row_h - font) * 0.5,
        font,
        inner_w - CLOSE_W,
        resolve(ColorToken::Text1, theme),
    );
    let close_rect = Rect::new(close_x, cy, CLOSE_W, row_h);
    hit_index.register(ids::TIMELINE_ONION_MODAL_CLOSE, close_rect);
    let close = Button::new(ids::TIMELINE_ONION_MODAL_CLOSE, "X")
        .state(button_state(store, ids::TIMELINE_ONION_MODAL_CLOSE));
    paint_button(&close, close_rect, scene, text_system, theme);
    cy += row_h + gap;

    // ── Three sliders: Opacity, Ghosts Before, Ghosts After. ──
    for (id, key) in [
        (
            ids::TIMELINE_ONION_MODAL_OPACITY,
            "panel.timeline.onion_opacity",
        ),
        (
            ids::TIMELINE_ONION_MODAL_BEFORE,
            "panel.timeline.onion_before",
        ),
        (
            ids::TIMELINE_ONION_MODAL_AFTER,
            "panel.timeline.onion_after",
        ),
    ] {
        let slider_rect = Rect::new(inner_x, cy, inner_w, row_h);
        hit_index.register(id, slider_rect);
        let (st, v) = store.slider(id).unwrap_or((SliderState::Normal, 0.5));
        let mut slider = Slider::new(id, ph2d_i18n::tr(key)).accent(true).state(st);
        slider.set_value(v);
        paint_slider(&slider, slider_rect, scene, theme);
        cy += row_h + gap;
    }

    // ── Two colour swatches (a Down opens the shared OKLCH picker via `is_picker_swatch`, registered
    //    in `open_onion_modal`). Label left, swatch right. ──
    for (id, key) in [
        (
            ids::TIMELINE_ONION_MODAL_COLOR_BEFORE,
            "panel.timeline.onion_color_before",
        ),
        (
            ids::TIMELINE_ONION_MODAL_COLOR_AFTER,
            "panel.timeline.onion_color_after",
        ),
    ] {
        paint_text(
            text_system,
            scene,
            ph2d_i18n::tr(key),
            inner_x,
            cy + (row_h - font) * 0.5,
            font,
            inner_w - SWATCH_W - gap,
            resolve(ColorToken::Text1, theme),
        );
        let sw_rect = Rect::new(inner_x + inner_w - SWATCH_W, cy, SWATCH_W, row_h);
        hit_index.register(id, sw_rect);
        let rgba = store.widget_color(id).unwrap_or([0x80, 0x80, 0x80, 0xFF]);
        fill_rounded_rect(
            scene,
            sw_rect,
            Radius::Sm.px(),
            Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]), // LITERAL-COLOR-OK: user-color — a cor de fantasma AUTORADA pelo artista, nunca um token de tema
        );
        stroke_rounded_rect(
            scene,
            sw_rect,
            Radius::Sm.px(),
            1.0,
            resolve(ColorToken::Border, theme),
        );
        cy += row_h + gap;
    }
}

/// Hover/press [`ButtonState`] for a modal button id (mirror of `fill_modal::button_state`).
fn button_state(store: &WidgetStore, id: NodeId) -> ButtonState {
    if store.active_id() == Some(id) {
        ButtonState::Pressed
    } else if store.hot_id() == Some(id) {
        ButtonState::Hovered
    } else {
        ButtonState::Normal
    }
}

/// Dispatch the Onion modal's widget events (wired into `chrome::dispatch_all`). Only acts while the
/// modal is open. The close X closes it; the handle click is consumed (the move is the shell's drag
/// machine); the sliders' `ValueChanged` is consumed so it never leaks to another handler (their
/// values are read back into `TimelineState::onion` by the shell — nothing to forward). The swatches
/// are handled earlier by the generic picker-swatch Down (`pointer_down`), so they never reach here.
pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    if hero.store.onion_modal_pos().is_none() {
        return false;
    }
    match event {
        WidgetEvent::Click(id) if id == ids::TIMELINE_ONION_MODAL_CLOSE => {
            hero.store.close_onion_modal();
            true
        }
        // A bare click on the title band (no drag) — consume so it never leaks to other chrome.
        WidgetEvent::Click(id) if id == ids::TIMELINE_ONION_MODAL_HANDLE => true,
        // Slider drags: the store value is already updated by the generic dispatch; consume so the
        // event doesn't fall through to `transport::apply` (mirrors `fill_modal`'s consume).
        WidgetEvent::ValueChanged(id)
            if id == ids::TIMELINE_ONION_MODAL_OPACITY
                || id == ids::TIMELINE_ONION_MODAL_BEFORE
                || id == ids::TIMELINE_ONION_MODAL_AFTER =>
        {
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    //! Gates for the Onion settings modal (ADR-0142 W3b), mirroring `fill_modal`'s suite. Inline (not
    //! a `_tests.rs` sibling) because the chrome-sync scanner treats every `chrome/*.rs` as a handler.

    use super::*;
    use crate::zones::Rect;
    use ph2d_text::TextSystem;

    fn text_system() -> TextSystem {
        TextSystem::without_system_fonts()
    }

    /// Closed → no hit rects; open → the handle, close, three sliders and two swatches are all
    /// hit-registered. (Mutation: gate the paint on `.is_some()` — dropping it paints rects while
    /// closed, which this catches by asserting the closed state registers nothing.)
    #[test]
    fn paints_and_registers_hit_rects_only_when_open() {
        let mut hero = HeroScreen::new(NodeId(1));
        let viewport = Rect::new(0.0, 0.0, 1280.0, 800.0);
        let mut ts = text_system();
        let ids_all = [
            ids::TIMELINE_ONION_MODAL_HANDLE,
            ids::TIMELINE_ONION_MODAL_CLOSE,
            ids::TIMELINE_ONION_MODAL_OPACITY,
            ids::TIMELINE_ONION_MODAL_BEFORE,
            ids::TIMELINE_ONION_MODAL_AFTER,
            ids::TIMELINE_ONION_MODAL_COLOR_BEFORE,
            ids::TIMELINE_ONION_MODAL_COLOR_AFTER,
        ];

        // Closed: nothing registered.
        let mut scene = VectorScene::new();
        hero.hit_index.clear_for_frame();
        paint_onion_modal(
            &mut scene,
            &mut ts,
            hero.theme,
            &mut hero.hit_index,
            &hero.store,
            viewport,
        );
        for id in ids_all {
            assert!(
                hero.hit_index.rect_for(id).is_none(),
                "closed modal must register nothing"
            );
        }

        // Open at (400, 300): every widget gets a hit rect.
        hero.store.open_onion_modal(
            400.0,
            300.0,
            0.5,
            0.25,
            0.25,
            [10, 100, 10, 255],
            [30, 20, 130, 255],
        );
        let mut scene = VectorScene::new();
        hero.hit_index.clear_for_frame();
        paint_onion_modal(
            &mut scene,
            &mut ts,
            hero.theme,
            &mut hero.hit_index,
            &hero.store,
            viewport,
        );
        for id in ids_all {
            assert!(
                hero.hit_index.rect_for(id).is_some(),
                "open modal must register every widget"
            );
        }
        // The handle and the close X never share a pixel (a Down decides drag vs close unambiguously).
        let h = hero
            .hit_index
            .rect_for(ids::TIMELINE_ONION_MODAL_HANDLE)
            .unwrap();
        let x = hero
            .hit_index
            .rect_for(ids::TIMELINE_ONION_MODAL_CLOSE)
            .unwrap();
        assert!(
            h.x + h.w <= x.x + 0.5,
            "handle ends before the close X begins"
        );
    }

    /// The close X closes the modal.
    #[test]
    fn close_x_closes_the_modal() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.store
            .open_onion_modal(100.0, 100.0, 0.5, 0.5, 0.5, [0, 0, 0, 255], [0, 0, 0, 255]);
        assert!(apply(
            &mut hero,
            WidgetEvent::Click(ids::TIMELINE_ONION_MODAL_CLOSE)
        ));
        assert!(
            hero.store.onion_modal_pos().is_none(),
            "the X closes the modal"
        );
    }

    /// A handle click is consumed but does NOT close (the drag machine moves it; a bare click is a no-op).
    #[test]
    fn handle_click_is_consumed_but_does_not_close() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.store
            .open_onion_modal(100.0, 100.0, 0.5, 0.5, 0.5, [0, 0, 0, 255], [0, 0, 0, 255]);
        assert!(apply(
            &mut hero,
            WidgetEvent::Click(ids::TIMELINE_ONION_MODAL_HANDLE)
        ));
        assert!(
            hero.store.onion_modal_pos().is_some(),
            "the handle click does not close the modal"
        );
    }

    /// A slider ValueChanged is consumed (so it never leaks to `transport::apply`) and keeps the modal
    /// open. The value read-back is the shell's job — here we only prove the event is owned.
    #[test]
    fn slider_change_is_consumed_and_keeps_the_modal_open() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.store
            .open_onion_modal(100.0, 100.0, 0.5, 0.5, 0.5, [0, 0, 0, 255], [0, 0, 0, 255]);
        for id in [
            ids::TIMELINE_ONION_MODAL_OPACITY,
            ids::TIMELINE_ONION_MODAL_BEFORE,
            ids::TIMELINE_ONION_MODAL_AFTER,
        ] {
            assert!(apply(&mut hero, WidgetEvent::ValueChanged(id)));
        }
        assert!(hero.store.onion_modal_pos().is_some());
    }

    /// Opening seeds the slider values + marks the two swatches so a Down opens the picker, and `move`
    /// offsets the position by the raw delta.
    #[test]
    fn open_seeds_widgets_and_move_offsets() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.store.open_onion_modal(
            200.0,
            150.0,
            0.6,
            0.75,
            0.25,
            [11, 22, 33, 255],
            [44, 55, 66, 255],
        );
        assert_eq!(
            hero.store
                .slider(ids::TIMELINE_ONION_MODAL_OPACITY)
                .map(|(_, v)| v),
            Some(0.6)
        );
        assert_eq!(
            hero.store
                .slider(ids::TIMELINE_ONION_MODAL_BEFORE)
                .map(|(_, v)| v),
            Some(0.75)
        );
        assert!(
            hero.store
                .is_picker_swatch(ids::TIMELINE_ONION_MODAL_COLOR_BEFORE)
        );
        assert!(
            hero.store
                .is_picker_swatch(ids::TIMELINE_ONION_MODAL_COLOR_AFTER)
        );
        assert_eq!(
            hero.store
                .widget_color(ids::TIMELINE_ONION_MODAL_COLOR_AFTER),
            Some([44, 55, 66, 255])
        );
        hero.store.move_onion_modal(15.0, -25.0);
        assert_eq!(hero.store.onion_modal_pos(), Some((215.0, 125.0)));
    }

    /// Events are ignored while the modal is closed.
    #[test]
    fn ignores_events_when_closed() {
        let mut hero = HeroScreen::new(NodeId(1));
        assert!(!apply(
            &mut hero,
            WidgetEvent::Click(ids::TIMELINE_ONION_MODAL_CLOSE)
        ));
    }
}
