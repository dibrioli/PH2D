//! Vertical scrollbar for the track list.
//!
//! Registered as an `InteractiveState::Slider { orientation: Vertical }` so the
//! generic 1-D drag dispatch drives it (the sanctioned pattern — a free 2-D drag
//! would need its own surface). Dispatch's vertical slider maps the pointer to
//! `1.0` at the TOP and `0.0` at the bottom, so the scroll fraction is `1 - v`.
//! `event::apply_event` turns a `ValueChanged` into `state.scroll_y`.
//!
//! Painted (and hit-registered) only while the rows actually overflow.

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_rounded_rect, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{SliderOrientation, SliderState};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Theme};

use crate::ids;
use crate::state::TimelinePanelState;

/// Smallest the thumb may shrink to, so a huge list still leaves a grabbable one.
const MIN_THUMB_H: f32 = 24.0; // LITERAL-PX-OK: minimum scrollbar thumb height

/// The scroll fraction (`0` = top) a slider value maps to, and back.
pub(crate) fn value_to_fraction(v: f32) -> f32 {
    1.0 - v
}

/// The thumb rect for a given scroll position, inside `track`.
fn thumb_rect(track: Rect, scroll_y: f32, scroll_max: f32, content_h: f32) -> Rect {
    let visible_frac = if content_h > 0.0 {
        (track.h / content_h).clamp(0.0, 1.0) // CLAMP-OK: fraction bounds, min<max
    } else {
        1.0
    };
    let h = (track.h * visible_frac).max(MIN_THUMB_H).min(track.h);
    let travel = (track.h - h).max(0.0);
    let t = if scroll_max > 0.0 {
        (scroll_y / scroll_max).clamp(0.0, 1.0) // CLAMP-OK: fraction bounds, min<max
    } else {
        0.0
    };
    Rect::new(track.x, track.y + travel * t, track.w, h)
}

/// Paint the scrollbar + register its hit, mirroring `scroll_y` into the store
/// while the user is not dragging it. A no-op when nothing overflows.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    track: Rect,
    state: &TimelinePanelState,
    content_h: f32,
) {
    if state.scroll_max <= 0.0 || track.h <= 0.0 || track.w <= 0.0 {
        return;
    }
    // Mirror the model into the widget unless the user owns it this frame.
    let dragging = matches!(
        ctx.host.store().slider(ids::TIMELINE_SCROLLBAR),
        Some((SliderState::Dragging, _))
    );
    if !dragging {
        let v = 1.0 - (state.scroll_y / state.scroll_max).clamp(0.0, 1.0); // CLAMP-OK: fraction bounds, min<max
        ctx.host.store_mut().register(
            ids::TIMELINE_SCROLLBAR,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: v,
                orientation: SliderOrientation::Vertical,
            },
        );
    }
    fill_rounded_rect(
        ctx.scene,
        track,
        Radius::Xs.px(),
        resolve(ColorToken::Bg2, theme),
    );
    let thumb = thumb_rect(track, state.scroll_y, state.scroll_max, content_h);
    let tok = if dragging {
        ColorToken::Accent
    } else {
        ColorToken::Border
    };
    fill_rounded_rect(ctx.scene, thumb, Radius::Xs.px(), resolve(tok, theme));
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_SCROLLBAR, track);
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRACK: Rect = Rect::new(0.0, 100.0, 10.0, 200.0);

    #[test]
    fn the_thumb_shrinks_with_the_visible_fraction() {
        // Half the content visible → half-height thumb, parked at the top.
        let t = thumb_rect(TRACK, 0.0, 200.0, 400.0);
        assert_eq!(t.h, 100.0);
        assert_eq!(t.y, TRACK.y);
    }

    #[test]
    fn the_thumb_reaches_the_bottom_at_full_scroll() {
        let t = thumb_rect(TRACK, 200.0, 200.0, 400.0);
        assert_eq!(t.y + t.h, TRACK.y + TRACK.h);
    }

    #[test]
    fn the_thumb_never_vanishes_on_a_huge_list() {
        let t = thumb_rect(TRACK, 0.0, 10_000.0, 100_000.0);
        assert_eq!(t.h, MIN_THUMB_H, "still grabbable");
        // And it can still travel the whole track.
        let end = thumb_rect(TRACK, 10_000.0, 10_000.0, 100_000.0);
        assert_eq!(end.y + end.h, TRACK.y + TRACK.h);
    }

    #[test]
    fn slider_value_inverts_to_the_scroll_fraction() {
        // Dispatch's vertical slider: 1.0 at the top of the track.
        assert_eq!(value_to_fraction(1.0), 0.0, "top = scroll 0");
        assert_eq!(value_to_fraction(0.0), 1.0, "bottom = fully scrolled");
    }
}
