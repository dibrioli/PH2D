// ph2d-chrome-sync:z=180 (dispatch priority, ADR-0107; lower = earlier)
//! Fill (Bucket) "Fill adjust" floating modal — the compact draggable card that appears at the
//! ColorDrop release point so the artist can tune the flood-fill threshold LIVE and confirm/cancel
//! (Procreate ColorDrop-Threshold). Two halves, colocated:
//!
//! * [`paint_fill_adjust_modal`] — renders the card at `store.fill_modal_pos()` (clamped to the
//!   viewport) with a title drag-handle, a threshold slider, and Cancel / Done buttons. Called from the
//!   hero paint pass (mirrors `context_menu_dialogs::paint_new_image_dialog`).
//! * [`apply`] — the dispatch half wired into `chrome::dispatch_all`: forwards the slider's live value +
//!   the Done/Cancel clicks to the active `PainterTool` over the frozen `PanelEvent` channel, then
//!   closes the modal on Done/Cancel. The title-band DRAG (move without closing) is a shell state
//!   machine (`input_dispatch::fill_drag`), not a widget event.

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::{HitIndex, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use crate::screens::hero::HeroScreen;
use crate::tool::PanelEvent;
use crate::widget::{Button, ButtonState, Slider, SliderState, paint_button, paint_slider};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Card width — compact, fits "Threshold" slider + a Cancel/Done button row.
const MODAL_W: f32 = 200.0; // LITERAL-PX-OK: compact Fill-adjust card width

/// Paint the Fill (Bucket) "Fill adjust" modal at `store.fill_modal_pos()`, clamped to `viewport`.
/// No-op when the modal is closed. Rows: a title band ("Fill") that doubles as the drag handle, a
/// threshold slider, and a Cancel / Done button row (Done accent). Mirrors the New-image dialog's
/// `fill_rounded_rect(BgElev)` + `stroke_rounded_rect(Border)` card + `Radius::Md` styling.
pub fn paint_fill_adjust_modal(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    viewport: Rect,
) {
    let Some((x, y)) = store.fill_modal_pos() else {
        return;
    };
    let row_h = ROW_H_PX;
    let gap = Spacing::Xs.px();
    let pad_y = Spacing::Sm.px();
    // Three stacked rows: title (drag handle) · threshold slider · Cancel/Done.
    let total_h = pad_y * 2.0 + row_h * 3.0 + gap * 2.0; // LITERAL-PX-OK: 2 inter-row gaps

    // Clamp the top-left so the whole card stays inside the viewport. max_x/max_y are `.max(viewport.x/y)`,
    // so min ≤ max always; viewport coords are non-NaN — hence the CLAMP-OK markers below.
    let max_x = (viewport.x + viewport.w - MODAL_W).max(viewport.x);
    let max_y = (viewport.y + viewport.h - total_h).max(viewport.y);
    let rect_x = x.clamp(viewport.x, max_x); // CLAMP-OK: bounds ordered (max_x ≥ viewport.x) + non-NaN
    let rect_y = y.clamp(viewport.y, max_y); // CLAMP-OK: bounds ordered (max_y ≥ viewport.y) + non-NaN
    let rect = Rect::new(rect_x, rect_y, MODAL_W, total_h);

    // Floating panel surface.
    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    let inner_x = rect.x + Spacing::Md.px();
    let inner_w = rect.w - Spacing::Md.px() * 2.0;
    let font = TypeToken::Sm.px();
    let mut cy = rect.y + pad_y;

    // ── Title band = the drag handle (Primary Down here starts a modal-move, shell-driven). ──
    let handle_rect = Rect::new(rect.x, rect.y, rect.w, pad_y + row_h);
    hit_index.register(ids::PAINTER_FILL_MODAL_HANDLE, handle_rect);
    paint_text(
        text_system,
        scene,
        "Fill",
        inner_x,
        cy + (row_h - font) * 0.5,
        font,
        inner_w,
        resolve(ColorToken::Text1, theme),
    );
    cy += row_h + gap;

    // ── Threshold slider (`0..1`; the tool maps it to a per-channel tolerance). ──
    let slider_rect = Rect::new(inner_x, cy, inner_w, row_h);
    hit_index.register(ids::PAINTER_FILL_MODAL_SLIDER, slider_rect);
    let (sl_state, sl_val) = store
        .slider(ids::PAINTER_FILL_MODAL_SLIDER)
        .unwrap_or((SliderState::Normal, 0.5));
    let mut slider = Slider::new(ids::PAINTER_FILL_MODAL_SLIDER, "Threshold")
        .accent(true)
        .state(sl_state);
    slider.set_value(sl_val);
    paint_slider(&slider, slider_rect, scene, theme);
    cy += row_h + gap;

    // ── Cancel (left) + Done (right, accent) button row. ──
    let bw = ((inner_w - gap) * 0.5).max(1.0);
    let cancel_rect = Rect::new(inner_x, cy, bw, row_h);
    hit_index.register(ids::PAINTER_FILL_MODAL_CANCEL, cancel_rect);
    let cancel = Button::new(ids::PAINTER_FILL_MODAL_CANCEL, "Cancel")
        .state(button_state(store, ids::PAINTER_FILL_MODAL_CANCEL));
    paint_button(&cancel, cancel_rect, scene, text_system, theme);

    let done_rect = Rect::new(inner_x + bw + gap, cy, bw, row_h);
    hit_index.register(ids::PAINTER_FILL_MODAL_DONE, done_rect);
    let done = Button::new(ids::PAINTER_FILL_MODAL_DONE, "Done")
        .accent()
        .state(button_state(store, ids::PAINTER_FILL_MODAL_DONE));
    paint_button(&done, done_rect, scene, text_system, theme);
}

/// Hover/press [`ButtonState`] for a modal button id, from the store's hot/active tracking (mirror of
/// the New-image dialog's local helper).
fn button_state(store: &WidgetStore, id: NodeId) -> ButtonState {
    if store.active_id() == Some(id) {
        ButtonState::Pressed
    } else if store.hot_id() == Some(id) {
        ButtonState::Hovered
    } else {
        ButtonState::Normal
    }
}

/// Dispatch the Fill modal's widget events (wired into `chrome::dispatch_all`). Only acts while the
/// modal is open. The threshold slider forwards its live value to the painter as `SetValue`; Done /
/// Cancel forward a `Click` then close the modal. A click on the drag handle is consumed as a no-op
/// (the actual move is the shell's title-band drag machine).
pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    if hero.store.fill_modal_pos().is_none() {
        return false;
    }
    match event {
        WidgetEvent::Click(id) if id == ids::PAINTER_FILL_MODAL_DONE => {
            hero.bus
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(
                    ids::PAINTER_FILL_MODAL_DONE,
                )));
            hero.store.close_fill_modal();
            true
        }
        WidgetEvent::Click(id) if id == ids::PAINTER_FILL_MODAL_CANCEL => {
            hero.bus
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(
                    ids::PAINTER_FILL_MODAL_CANCEL,
                )));
            hero.store.close_fill_modal();
            true
        }
        // A bare click on the title band (no drag) — consume so it never leaks to other chrome.
        WidgetEvent::Click(id) if id == ids::PAINTER_FILL_MODAL_HANDLE => true,
        // Slider drag: forward the dispatched `0..1` track — the painter re-fills live from the snapshot.
        WidgetEvent::ValueChanged(id) if id == ids::PAINTER_FILL_MODAL_SLIDER => {
            let v = hero.store.slider(id).map(|(_, v)| v).unwrap_or(0.0);
            hero.bus
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id, v as f64,
                )));
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a headless text system for paint tests (no system fonts → fast).
    fn text_system() -> TextSystem {
        TextSystem::without_system_fonts()
    }

    #[test]
    fn paints_and_registers_hit_rects_only_when_open() {
        let mut hero = HeroScreen::new(NodeId(1));
        let viewport = Rect::new(0.0, 0.0, 1280.0, 800.0);
        let mut ts = text_system();

        // Closed: no hit rects for the modal ids.
        let mut scene = VectorScene::new();
        hero.hit_index.clear_for_frame();
        paint_fill_adjust_modal(
            &mut scene,
            &mut ts,
            hero.theme,
            &mut hero.hit_index,
            &hero.store,
            viewport,
        );
        assert!(
            hero.hit_index
                .rect_for(ids::PAINTER_FILL_MODAL_SLIDER)
                .is_none()
        );
        assert!(
            hero.hit_index
                .rect_for(ids::PAINTER_FILL_MODAL_DONE)
                .is_none()
        );
        assert!(
            hero.hit_index
                .rect_for(ids::PAINTER_FILL_MODAL_CANCEL)
                .is_none()
        );

        // Open at (400, 300), seed threshold 0.25.
        hero.store.open_fill_modal(400.0, 300.0, 0.25);
        let mut scene = VectorScene::new();
        hero.hit_index.clear_for_frame();
        paint_fill_adjust_modal(
            &mut scene,
            &mut ts,
            hero.theme,
            &mut hero.hit_index,
            &hero.store,
            viewport,
        );
        assert!(
            hero.hit_index
                .rect_for(ids::PAINTER_FILL_MODAL_HANDLE)
                .is_some()
        );
        assert!(
            hero.hit_index
                .rect_for(ids::PAINTER_FILL_MODAL_SLIDER)
                .is_some()
        );
        assert!(
            hero.hit_index
                .rect_for(ids::PAINTER_FILL_MODAL_DONE)
                .is_some()
        );
        assert!(
            hero.hit_index
                .rect_for(ids::PAINTER_FILL_MODAL_CANCEL)
                .is_some()
        );
    }

    #[test]
    fn done_forwards_a_click_and_closes() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.store.open_fill_modal(100.0, 100.0, 0.5);
        assert!(apply(
            &mut hero,
            WidgetEvent::Click(ids::PAINTER_FILL_MODAL_DONE)
        ));
        assert!(
            hero.store.fill_modal_pos().is_none(),
            "Done closes the modal"
        );
        let forwarded = hero.bus.drain().any(|a| {
            matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(id))
                    if id == ids::PAINTER_FILL_MODAL_DONE
            )
        });
        assert!(forwarded, "Done forwarded a Click to the painter");
    }

    #[test]
    fn cancel_forwards_a_click_and_closes() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.store.open_fill_modal(100.0, 100.0, 0.5);
        assert!(apply(
            &mut hero,
            WidgetEvent::Click(ids::PAINTER_FILL_MODAL_CANCEL)
        ));
        assert!(
            hero.store.fill_modal_pos().is_none(),
            "Cancel closes the modal"
        );
        let forwarded = hero.bus.drain().any(|a| {
            matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(id))
                    if id == ids::PAINTER_FILL_MODAL_CANCEL
            )
        });
        assert!(forwarded, "Cancel forwarded a Click to the painter");
    }

    #[test]
    fn slider_change_forwards_the_value_and_keeps_the_modal_open() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.store.open_fill_modal(100.0, 100.0, 0.5);
        // Simulate the dispatch having written a new slider value (0.25 = exact in both f32 and f64).
        hero.store.open_fill_modal(100.0, 100.0, 0.25);
        assert!(apply(
            &mut hero,
            WidgetEvent::ValueChanged(ids::PAINTER_FILL_MODAL_SLIDER)
        ));
        assert!(
            hero.store.fill_modal_pos().is_some(),
            "the slider drag does not close the modal"
        );
        let value = hero.bus.drain().find_map(|a| match a {
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, v))
                if id == ids::PAINTER_FILL_MODAL_SLIDER =>
            {
                Some(v)
            }
            _ => None,
        });
        assert_eq!(value, Some(0.25), "forwarded the slider's live threshold");
    }

    #[test]
    fn ignores_events_when_closed() {
        let mut hero = HeroScreen::new(NodeId(1));
        assert!(!apply(
            &mut hero,
            WidgetEvent::Click(ids::PAINTER_FILL_MODAL_DONE)
        ));
    }
}
