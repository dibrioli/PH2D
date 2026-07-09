//! Timeline panel geometry — the single place that turns the panel rect into the
//! sub-rects everything else paints and hit-tests against, plus the resize
//! grippers. `paint` resolves it twice per frame (once before `interact::process`
//! so gestures see the current geometry, once after, so a resize lands the same
//! frame it is dragged).

use ph2d_editor_core::widget::panel_chrome::{PANEL_HEAD_PAD, PANEL_TITLE_BASELINE};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};

use crate::ids;
use crate::ruler;
use crate::tracks;

/// Thickness of the resize grip strips along the panel border.
pub(crate) const GRIP: f32 = 6.0; // LITERAL-PX-OK: resize gripper thickness
/// Width of the vertical scrollbar reserved at the right of the lanes.
pub(crate) const SCROLLBAR_W: f32 = 10.0; // LITERAL-PX-OK: track-list scrollbar width
/// Smallest the panel may be resized to.
pub(crate) const MIN_W: f32 = 320.0; // LITERAL-PX-OK: min panel width
/// Smallest the panel may be resized to.
pub(crate) const MIN_H: f32 = 120.0; // LITERAL-PX-OK: min panel height

// Resize edge bitmask (opaque to editor-core, which only routes it back).
pub(crate) const EDGE_L: u8 = 1;
pub(crate) const EDGE_R: u8 = 2;
pub(crate) const EDGE_T: u8 = 4;
pub(crate) const EDGE_B: u8 = 8;

/// The resolved sub-rects of one frame's dope sheet.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Geom {
    /// Dope-sheet region below the transport bar (label column + time area).
    pub region: Rect,
    /// Width of the left label column.
    pub label_w: f32,
    /// Time column (ruler + lanes), right of the labels, left of the scrollbar.
    pub time_area: Rect,
    /// Lane rows below the ruler strip (the scrollable band).
    pub rows: Rect,
    /// Vertical scrollbar strip at the right of the rows.
    pub scrollbar: Rect,
}

/// The panel body below the title (the transport bar + dope sheet live here).
pub(crate) fn body(rect: Rect, title_size: f32) -> Rect {
    let top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Sm.px();
    Rect::new(
        rect.x + PANEL_HEAD_PAD,
        top,
        (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0),
        (rect.y + rect.h - top - PANEL_HEAD_PAD).max(0.0),
    )
}

/// The left edge of the time area — where `view_start_s` maps to. Depends only
/// on the panel rect, so `interact::process` can have it before the transport
/// bar paints.
pub(crate) fn time_x(rect: Rect) -> f32 {
    let body_x = rect.x + PANEL_HEAD_PAD;
    let body_w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    body_x + tracks::LABEL_COL_W.min(body_w)
}

/// Resolve the dope-sheet sub-rects from the panel `rect` and the transport
/// bar's bottom edge. (`region.h` does not depend on the title size: the body's
/// bottom is `rect.y + rect.h - PANEL_HEAD_PAD` either way.)
pub(crate) fn resolve(rect: Rect, after_transport: f32) -> Geom {
    let x = rect.x + PANEL_HEAD_PAD;
    let w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    let bottom = rect.y + rect.h - PANEL_HEAD_PAD;
    let region = Rect::new(x, after_transport, w, (bottom - after_transport).max(0.0));
    let label_w = tracks::LABEL_COL_W.min(region.w);
    let bar_x = region.x + region.w - SCROLLBAR_W;
    let time_area = Rect::new(
        region.x + label_w,
        region.y,
        (bar_x - (region.x + label_w)).max(0.0),
        region.h,
    );
    let rows = Rect::new(
        region.x,
        region.y + ruler::RULER_H,
        (region.w - SCROLLBAR_W).max(0.0),
        (region.h - ruler::RULER_H).max(0.0),
    );
    let scrollbar = Rect::new(bar_x, rows.y, SCROLLBAR_W, rows.h);
    Geom {
        region,
        label_w,
        time_area,
        rows,
        scrollbar,
    }
}

/// Total height the track rows want, for the scroll range.
pub(crate) fn content_h(track_count: usize) -> f32 {
    track_count as f32 * ROW_H_PX
}

/// How far the rows can scroll before the last one is flush with the bottom.
pub(crate) fn scroll_max(track_count: usize, rows_h: f32) -> f32 {
    (content_h(track_count) - rows_h).max(0.0)
}

/// The eight resize grippers as `(id, edges, rect)`, outermost-last so the
/// corners win the hit-test over the edges they overlap.
pub(crate) fn resize_grips(rect: Rect) -> [(ph2d_a11y::NodeId, u8, Rect); 8] {
    let (x, y, w, h) = (rect.x, rect.y, rect.w, rect.h);
    let r = x + w;
    let b = y + h;
    [
        // Edges first (lower priority).
        (ids::TIMELINE_RESIZE_L, EDGE_L, Rect::new(x, y, GRIP, h)),
        (
            ids::TIMELINE_RESIZE_R,
            EDGE_R,
            Rect::new(r - GRIP, y, GRIP, h),
        ),
        (ids::TIMELINE_RESIZE_T, EDGE_T, Rect::new(x, y, w, GRIP)),
        (
            ids::TIMELINE_RESIZE_B,
            EDGE_B,
            Rect::new(x, b - GRIP, w, GRIP),
        ),
        // Corners last (registered on top).
        (
            ids::TIMELINE_RESIZE_TL,
            EDGE_T | EDGE_L,
            Rect::new(x, y, GRIP, GRIP),
        ),
        (
            ids::TIMELINE_RESIZE_TR,
            EDGE_T | EDGE_R,
            Rect::new(r - GRIP, y, GRIP, GRIP),
        ),
        (
            ids::TIMELINE_RESIZE_BL,
            EDGE_B | EDGE_L,
            Rect::new(x, b - GRIP, GRIP, GRIP),
        ),
        (
            ids::TIMELINE_RESIZE_BR,
            EDGE_B | EDGE_R,
            Rect::new(r - GRIP, b - GRIP, GRIP, GRIP),
        ),
    ]
}

/// Apply a resize drag to `start`: each set edge moves by the pointer delta.
/// Never smaller than [`MIN_W`]×[`MIN_H`], never outside `viewport`.
pub(crate) fn resized(start: Rect, edges: u8, dx: f32, dy: f32, viewport: Rect) -> Rect {
    let (mut x, mut y, mut w, mut h) = (start.x, start.y, start.w, start.h);
    if edges & EDGE_L != 0 {
        // Growing left must not shrink past MIN_W, so clamp the delta first.
        let dx = dx.min(w - MIN_W);
        x += dx;
        w -= dx;
    }
    if edges & EDGE_R != 0 {
        w = (w + dx).max(MIN_W);
    }
    if edges & EDGE_T != 0 {
        let dy = dy.min(h - MIN_H);
        y += dy;
        h -= dy;
    }
    if edges & EDGE_B != 0 {
        h = (h + dy).max(MIN_H);
    }
    clamp_to(Rect::new(x, y, w.max(MIN_W), h.max(MIN_H)), viewport)
}

/// Keep `rect` inside `viewport` (size first, then position).
pub(crate) fn clamp_to(rect: Rect, viewport: Rect) -> Rect {
    let w = rect.w.min(viewport.w).max(MIN_W);
    let h = rect.h.min(viewport.h).max(MIN_H);
    let x = rect
        .x
        .clamp(viewport.x, (viewport.x + viewport.w - w).max(viewport.x)); // CLAMP-OK: viewport bounds, min<=max
    let y = rect
        .y
        .clamp(viewport.y, (viewport.y + viewport.h - h).max(viewport.y)); // CLAMP-OK: viewport bounds, min<=max
    Rect::new(x, y, w, h)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VP: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);

    #[test]
    fn dragging_the_top_edge_up_grows_the_panel_upward() {
        let start = Rect::new(100.0, 600.0, 800.0, 240.0);
        let out = resized(start, EDGE_T, 0.0, -100.0, VP);
        assert_eq!((out.y, out.h), (500.0, 340.0), "top moved up, bottom fixed");
        assert_eq!((out.x, out.w), (100.0, 800.0), "x untouched");
    }

    #[test]
    fn dragging_a_corner_moves_both_axes() {
        let start = Rect::new(100.0, 600.0, 800.0, 240.0);
        let out = resized(start, EDGE_T | EDGE_L, 50.0, -40.0, VP);
        assert_eq!((out.x, out.w), (150.0, 750.0));
        assert_eq!((out.y, out.h), (560.0, 280.0));
    }

    #[test]
    fn a_resize_never_goes_below_the_minimum() {
        let start = Rect::new(100.0, 600.0, 400.0, 200.0);
        // Drag the left edge far right: width floors at MIN_W and x stops with it.
        let out = resized(start, EDGE_L, 10_000.0, 0.0, VP);
        assert_eq!(out.w, MIN_W);
        assert_eq!(
            out.x,
            100.0 + (400.0 - MIN_W),
            "x stopped where MIN_W begins"
        );
        // Drag the top edge far down: height floors at MIN_H.
        let out = resized(start, EDGE_T, 0.0, 10_000.0, VP);
        assert_eq!(out.h, MIN_H);
    }

    #[test]
    fn a_resize_stays_inside_the_viewport() {
        let start = Rect::new(1500.0, 800.0, 400.0, 200.0);
        let out = resized(start, EDGE_R | EDGE_B, 500.0, 500.0, VP);
        assert!(out.x + out.w <= VP.w + f32::EPSILON);
        assert!(out.y + out.h <= VP.h + f32::EPSILON);
    }

    #[test]
    fn scroll_max_is_zero_when_everything_fits() {
        assert_eq!(scroll_max(2, 500.0), 0.0);
        assert_eq!(scroll_max(20, 100.0), content_h(20) - 100.0);
    }

    #[test]
    fn corners_are_registered_after_the_edges_they_overlap() {
        let grips = resize_grips(Rect::new(0.0, 0.0, 100.0, 100.0));
        let corner_start = grips.iter().position(|(_, e, _)| e.count_ones() == 2);
        let last_edge = grips.iter().rposition(|(_, e, _)| e.count_ones() == 1);
        assert!(
            corner_start.unwrap() > last_edge.unwrap(),
            "corners last = on top"
        );
    }
}
