//! [`Scrollbar`] — thin vertical drag-affordance for scrollable
//! panels.
//!
//! Single source of truth for the track/thumb geometry: callers
//! ask [`Scrollbar::thumb_rect`] for the hit zone the dispatch
//! should register, and the painter uses [`paint_scrollbar`] to
//! draw the track + thumb at the same coordinates.
//!
//! No state on the widget itself — `panel_scroll`, `panel_content_h`
//! and `panel_visible_h` all live on the [`crate::interaction::WidgetStore`].
//! The dispatcher reads / writes those side-tables; this module
//! only computes geometry + paints.

use crate::paint::{fill_rounded_rect, resolve};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::{ColorToken, ROW_H_PX, Theme};
use ph2d_vector::VectorScene;

/// Default scrollbar track width. Wide enough to be a comfortable
/// drag target on iPad/tablet (the user's "Fundamental para
/// ipad/tablet" requirement); narrow enough to not encroach on
/// the panel content.
pub const SCROLLBAR_W: f32 = 10.0; // LITERAL-PX-OK: scrollbar track width (chrome-specific, between Spacing::Md and Lg)
/// Minimum thumb height. When the visible viewport is a large
/// fraction of the content, the proportional thumb height could
/// shrink to 0 px; we clamp so the user can always grab it.
pub const SCROLLBAR_THUMB_MIN_H: f32 = ROW_H_PX;

/// Compute the track rect inside the panel body. The caller is
/// responsible for clipping the track to the panel's body region —
/// the rect returned here is just the geometry.
pub fn track_rect(body_rect: Rect) -> Rect {
    Rect::new(
        body_rect.x + body_rect.w - SCROLLBAR_W - 2.0,
        body_rect.y,
        SCROLLBAR_W,
        body_rect.h,
    )
}

/// Compute the thumb rect inside `track`, given the panel's scroll
/// position and the content / visible heights. Returns the full
/// track when `content_h <= visible_h` (no scroll needed — caller
/// should hide).
pub fn thumb_rect(track: Rect, scroll_y: f32, content_h: f32, visible_h: f32) -> Rect {
    if content_h <= visible_h || track.h <= 0.0 {
        return track;
    }
    let raw_h = visible_h / content_h * track.h;
    let thumb_h = raw_h.max(SCROLLBAR_THUMB_MIN_H).min(track.h);
    let max_scroll = (content_h - visible_h).max(1e-3);
    let t = (scroll_y / max_scroll).clamp(0.0, 1.0);
    let thumb_y = track.y + t * (track.h - thumb_h);
    Rect::new(track.x, thumb_y, track.w, thumb_h)
}

/// Returns true iff the scrollbar should be visible at all (some
/// content overflows the viewport). When false, callers can skip
/// paint AND hit registration entirely.
pub fn is_needed(content_h: f32, visible_h: f32) -> bool {
    content_h > visible_h + 0.5
}

/// Paint the scrollbar track + thumb at `body_rect`. No-op when
/// `!is_needed(content_h, visible_h)`. `is_active` = currently
/// being dragged; thumb fills with `Accent` instead of `Border` so
/// the user gets feedback that their drag is "live".
pub fn paint_scrollbar(
    body_rect: Rect,
    scroll_y: f32,
    content_h: f32,
    visible_h: f32,
    is_active: bool,
    scene: &mut VectorScene,
    theme: Theme,
) {
    if !is_needed(content_h, visible_h) {
        return;
    }
    let track = track_rect(body_rect);
    // Subtle track tint so the rail itself reads against the panel
    // surface; uses `Bg3` (the same token the collapsed-section
    // plate uses, for a coherent "interactive surface" palette).
    fill_rounded_rect(
        scene,
        track,
        SCROLLBAR_W * 0.5,
        resolve(ColorToken::Bg3, theme),
    );
    let thumb = thumb_rect(track, scroll_y, content_h, visible_h);
    let thumb_color = if is_active {
        ColorToken::Accent
    } else {
        ColorToken::BorderEmph
    };
    fill_rounded_rect(
        scene,
        thumb,
        (SCROLLBAR_W * 0.5).min(thumb.h * 0.5),
        resolve(thumb_color, theme),
    );
}

/// Convert a cursor y delta to a scroll-y delta given the track
/// dimensions. Used by the drag dispatcher to translate
/// `cursor_y - cursor_y_at_down` into `scroll_y - scroll_y_at_down`.
pub fn delta_for_drag(cursor_delta_y: f32, track_h: f32, content_h: f32, visible_h: f32) -> f32 {
    let raw_h = visible_h / content_h * track_h;
    let thumb_h = raw_h.max(SCROLLBAR_THUMB_MIN_H).min(track_h);
    let drag_range = (track_h - thumb_h).max(1.0);
    let max_scroll = (content_h - visible_h).max(0.0);
    cursor_delta_y * (max_scroll / drag_range)
}

/// Stable hit id for the inspector's scrollbar thumb. (Exported so
/// the dispatch can route Down/Move/Up events without a string
/// lookup. Hierarchy gets a separate id.)
pub const INSPECTOR_SCROLLBAR_ID: NodeId = NodeId(820);
pub const HIERARCHY_SCROLLBAR_ID: NodeId = NodeId(821);
/// Widget Gallery floating panel scrollbar — mirrors the Inspector's
/// behavior with an independent thumb id so dispatch can route
/// drag-scroll events without aliasing the Inspector's scroll state.
pub const GALLERY_SCROLLBAR_ID: NodeId = NodeId(822);
/// Grid Settings floating panel scrollbar — same shape as Inspector /
/// Gallery, independent thumb id keeps dispatch from aliasing.
pub const GRID_SETTINGS_SCROLLBAR_ID: NodeId = NodeId(823);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_scrollbar_when_content_fits() {
        assert!(!is_needed(100.0, 200.0));
    }

    #[test]
    fn thumb_at_top_when_scroll_zero() {
        let body = Rect::new(0.0, 0.0, 100.0, 200.0);
        let track = track_rect(body);
        let thumb = thumb_rect(track, 0.0, 400.0, 200.0);
        assert!((thumb.y - track.y).abs() < 0.001);
    }

    #[test]
    fn thumb_at_bottom_when_scroll_max() {
        let body = Rect::new(0.0, 0.0, 100.0, 200.0);
        let track = track_rect(body);
        let max = 400.0 - 200.0;
        let thumb = thumb_rect(track, max, 400.0, 200.0);
        let expected_bottom = track.y + track.h;
        assert!((thumb.y + thumb.h - expected_bottom).abs() < 0.001);
    }

    #[test]
    fn delta_for_drag_proportional() {
        // track 100 px, content 400, visible 200 → thumb 50 px,
        // drag_range 50 px, max_scroll 200 → delta×4.
        let d = delta_for_drag(10.0, 100.0, 400.0, 200.0);
        // raw_h = 50, drag_range = 50, ratio = 200/50 = 4 → 10*4 = 40.
        assert!((d - 40.0).abs() < 0.001);
    }
}
