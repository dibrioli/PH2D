//! Timeline panel geometry — the single place that turns the panel rect into the
//! sub-rects everything else paints and hit-tests against, plus the resize
//! grippers. `paint` resolves it twice per frame (once before `interact::process`
//! so gestures see the current geometry, once after, so a resize lands the same
//! frame it is dragged).

use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE,
};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::ROW_H_PX;

use crate::ids;
use crate::ruler;

/// Thickness of the resize grip strips along the panel border.
pub(crate) const GRIP: f32 = 6.0; // LITERAL-PX-OK: resize gripper thickness
/// Width of the vertical scrollbar reserved at the right of the lanes.
pub(crate) const SCROLLBAR_W: f32 = 10.0; // LITERAL-PX-OK: track-list scrollbar width
/// Smallest the panel may be resized to.
pub(crate) const MIN_W: f32 = 320.0; // LITERAL-PX-OK: min panel width
/// Smallest the panel may be resized to.
pub(crate) const MIN_H: f32 = 120.0; // LITERAL-PX-OK: min panel height

// Resize edge bitmask. Named in editor-core so the shell can pick the matching
// double-arrow cursor; the resize itself stays entirely here.
pub(crate) use ph2d_editor_core::interaction::{
    TIMELINE_EDGE_B as EDGE_B, TIMELINE_EDGE_L as EDGE_L, TIMELINE_EDGE_R as EDGE_R,
    TIMELINE_EDGE_T as EDGE_T,
};

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
    let top = rect.y + head_h(title_size);
    Rect::new(
        rect.x + PANEL_HEAD_PAD,
        top,
        (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0),
        (rect.y + rect.h - top - PANEL_HEAD_PAD).max(0.0),
    )
}

/// The panel's header band: exactly ONE control row, inset.
///
/// The transport flows beside the title now (Enio, 2026-07-12) and the title sits
/// ON that row — so the band is the row, and there is no second line to leave
/// space for. Everything the row cannot hold wraps into the body below it.
fn head_h(_title_size: f32) -> f32 {
    HEAD_PAD_Y * 2.0 + ROW_H_PX
}

/// Vertical inset above and below the header's single control row.
const HEAD_PAD_Y: f32 = 6.0; // LITERAL-PX-OK: header row inset

/// Baseline for the panel title so it sits on the SAME row as the controls
/// ([`PANEL_TITLE_BASELINE`] is a text box's top-to-baseline offset).
pub(crate) fn title_baseline(rect: Rect) -> f32 {
    rect.y + HEAD_PAD_Y + (ROW_H_PX - PANEL_TITLE_BASELINE) * 0.5
}

/// Width reserved for the panel's own title before the controls may start.
///
/// A RESERVE, not a measurement — the same trick the chrome already plays on the
/// other side with [`PANEL_HEADER_CLOSE_RESERVE`]. There is no text-measuring API
/// to ask (and `chars().count()` is banned as a width proxy, correctly: Inter is
/// proportional, so a character count is not a pixel advance). Wide enough for
/// "Timeline" at the title size, and the title paints clipped to its own `max_w`
/// anyway, so a longer one shortens rather than collides.
const TITLE_RESERVE: f32 = 92.0; // LITERAL-PX-OK: header column reserved for the panel title

/// The strip in the header where the transport controls flow: right of the title,
/// left of the close (X) button, vertically centred in the band.
pub(crate) fn header_controls(rect: Rect, _title_size: f32) -> Rect {
    let x = rect.x + PANEL_HEAD_PAD + TITLE_RESERVE;
    let right = rect.x + rect.w - PANEL_HEAD_PAD - PANEL_HEADER_CLOSE_RESERVE;
    Rect::new(x, rect.y + HEAD_PAD_Y, (right - x).max(0.0), ROW_H_PX)
}

/// Narrowest the track-name column may be dragged.
pub(crate) const MIN_LABEL_W: f32 = 56.0; // LITERAL-PX-OK: min track-name column width
/// Narrowest the time area may be squeezed to by widening the names.
const MIN_TIME_W: f32 = 120.0; // LITERAL-PX-OK: min time-area width
/// Half-width of the splitter's grab strip.
pub(crate) const SPLIT_GRIP: f32 = 4.0; // LITERAL-PX-OK: label splitter grab half-width
/// Bare gutter between the label/time seam and where `view_start_s` maps.
///
/// The splitter grip owns `[seam - SPLIT_GRIP, seam + SPLIT_GRIP]` and, being
/// registered last, wins every hit inside it. Without the gutter a keyframe at
/// the left edge of the view would have its grab rect swallowed there, and the
/// key would be undraggable. Wide enough to clear both (asserted in the tests).
pub(crate) const TIME_GUTTER: f32 = 12.0; // LITERAL-PX-OK: splitter/first-key separation

/// The user's requested label-column width, held inside the panel's bounds. Never
/// wider than what leaves [`MIN_TIME_W`] of time area, never below
/// [`MIN_LABEL_W`] — and never wider than the region itself on a tiny panel.
pub(crate) fn clamp_label_w(label_w: f32, region_w: f32) -> f32 {
    let widest = (region_w - MIN_TIME_W).max(MIN_LABEL_W);
    label_w.min(widest).max(MIN_LABEL_W).min(region_w)
}

/// Where `view_start_s` maps — the seam plus [`TIME_GUTTER`]. Depends only on the
/// panel rect + the label width, so `interact::process` can have it before the
/// transport bar paints.
pub(crate) fn time_x(rect: Rect, label_w: f32) -> f32 {
    let body_x = rect.x + PANEL_HEAD_PAD;
    let body_w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    body_x + clamp_label_w(label_w, body_w) + TIME_GUTTER
}

/// Resolve the dope-sheet sub-rects from the panel `rect`, the transport bar's
/// bottom edge and the (user-resizable) label-column width. (`region.h` does not
/// depend on the title size: the body's bottom is `rect.y + rect.h -
/// PANEL_HEAD_PAD` either way.)
pub(crate) fn resolve(rect: Rect, after_transport: f32, label_w: f32) -> Geom {
    let x = rect.x + PANEL_HEAD_PAD;
    let w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    let bottom = rect.y + rect.h - PANEL_HEAD_PAD;
    let region = Rect::new(x, after_transport, w, (bottom - after_transport).max(0.0));
    let label_w = clamp_label_w(label_w, region.w);
    let bar_x = region.x + region.w - SCROLLBAR_W;
    // The time column starts a gutter past the seam, so the splitter grip and the
    // first keyframe never fight over the same pixels.
    let time_left = region.x + label_w + TIME_GUTTER;
    let time_area = Rect::new(time_left, region.y, (bar_x - time_left).max(0.0), region.h);
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

/// Height of one track row: the dope-sheet strip, plus the graph band when the
/// row is expanded (W3.E1).
pub(crate) fn row_h(expanded: bool, graph_h: f32) -> f32 {
    ROW_H_PX + if expanded { graph_h } else { 0.0 }
}

/// Height the Summary channel occupies above the tracks. Zero when nothing is
/// bound: an empty timeline shows no master row to grab.
pub(crate) fn summary_h(snap: &TimelineViewSnapshot) -> f32 {
    if snap.tracks.is_empty() {
        0.0
    } else {
        ROW_H_PX
    }
}

/// Total height the rows want, for the scroll range — the Summary channel plus
/// every track (an expanded one carries its graph band).
pub(crate) fn content_h(snap: &TimelineViewSnapshot, expanded: &[u64], graph_h: f32) -> f32 {
    summary_h(snap)
        + snap
            .tracks
            .iter()
            .map(|t| row_h(expanded.contains(&t.target.get()), graph_h))
            .sum::<f32>()
}

/// The Summary channel's row rect, or `None` when there is no track to summarise.
/// It scrolls with the rows it aggregates, so its diamonds always sit directly
/// above the columns they stand for.
pub(crate) fn summary_band(
    snap: &TimelineViewSnapshot,
    rows_top: f32,
    scroll_y: f32,
) -> Option<(f32, f32)> {
    let h = summary_h(snap);
    (h > 0.0).then_some((rows_top - scroll_y, h))
}

/// How far the rows can scroll before the last one is flush with the bottom.
pub(crate) fn scroll_max(content_h: f32, rows_h: f32) -> f32 {
    (content_h - rows_h).max(0.0)
}

/// Every track's `(index, top_y, height)` in the scrolled rows band. The single
/// source of the row layout — paint, key hit-testing and box-select all walk it,
/// so a row's diamonds can never disagree with the row its curve is drawn in.
pub(crate) fn row_bands<'a>(
    snap: &'a TimelineViewSnapshot,
    expanded: &'a [u64],
    graph_h: f32,
    rows_top: f32,
    scroll_y: f32,
) -> impl Iterator<Item = (usize, f32, f32)> + 'a {
    // The Summary channel sits above track 0 and scrolls with it.
    let mut y = rows_top - scroll_y + summary_h(snap);
    snap.tracks.iter().enumerate().map(move |(i, t)| {
        let h = row_h(expanded.contains(&t.target.get()), graph_h);
        let top = y;
        y += h;
        (i, top, h)
    })
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

    /// `n` collapsed tracks, with those in `expanded` opened.
    fn snap_with(n: usize) -> TimelineViewSnapshot {
        TimelineViewSnapshot {
            tracks: (0..n)
                .map(|i| ph2d_timeline::TrackView {
                    target: ph2d_timeline::AnimTarget::new(i as u64),
                    prop: ph2d_timeline::PropKind::TranslationX,
                    entity: 1,
                    missing: false,
                    keys: Vec::new(),
                })
                .collect(),
            ..TimelineViewSnapshot::default()
        }
    }

    #[test]
    fn scroll_max_is_zero_when_everything_fits() {
        const GH: f32 = 132.0;
        assert_eq!(scroll_max(content_h(&snap_with(2), &[], GH), 500.0), 0.0);
        let tall = content_h(&snap_with(20), &[], GH);
        assert_eq!(scroll_max(tall, 100.0), tall - 100.0);
    }

    #[test]
    fn an_expanded_row_is_taller_and_pushes_the_rows_below_it_down() {
        const GH: f32 = 132.0;
        // The Summary channel is row zero, so every track sits one row lower and
        // the content is one row taller.
        const S: f32 = ROW_H_PX;
        let snap = snap_with(3);
        assert_eq!(content_h(&snap, &[], GH), S + ROW_H_PX * 3.0);
        assert_eq!(content_h(&snap, &[1], GH), S + ROW_H_PX * 3.0 + GH);

        // Row 0 collapsed, row 1 expanded: row 2 starts below BOTH.
        let bands: Vec<_> = row_bands(&snap, &[1], GH, 0.0, 0.0).collect();
        assert_eq!(bands[0], (0, S, ROW_H_PX));
        assert_eq!(bands[1], (1, S + ROW_H_PX, ROW_H_PX + GH));
        assert_eq!(bands[2].1, S + ROW_H_PX * 2.0 + GH);

        // A taller graph pushes them further: the height is not a constant.
        let taller: Vec<_> = row_bands(&snap, &[1], GH * 2.0, 0.0, 0.0).collect();
        assert_eq!(taller[2].1, S + ROW_H_PX * 2.0 + GH * 2.0);
    }

    #[test]
    fn scrolling_shifts_every_row_band_up_by_the_same_amount() {
        const GH: f32 = 132.0;
        let snap = snap_with(3);
        let unscrolled: Vec<_> = row_bands(&snap, &[], GH, 10.0, 0.0).collect();
        let scrolled: Vec<_> = row_bands(&snap, &[], GH, 10.0, 30.0).collect();
        for (a, b) in unscrolled.iter().zip(&scrolled) {
            assert_eq!(a.1 - 30.0, b.1);
            assert_eq!(a.2, b.2, "scrolling never changes a row's height");
        }
    }

    #[test]
    fn the_label_column_stays_between_its_bounds() {
        // Wide panel: the drag is honoured verbatim.
        assert_eq!(clamp_label_w(200.0, 800.0), 200.0);
        // Dragged to nothing: floors at MIN_LABEL_W.
        assert_eq!(clamp_label_w(0.0, 800.0), MIN_LABEL_W);
        // Dragged past the right edge: the time area keeps MIN_TIME_W.
        assert_eq!(clamp_label_w(10_000.0, 800.0), 800.0 - MIN_TIME_W);
    }

    #[test]
    fn a_panel_too_narrow_for_both_still_yields_a_finite_column() {
        // MIN_LABEL_W + MIN_TIME_W does not fit: the label wins, capped by the
        // region, and never inverts into a negative time area.
        let w = clamp_label_w(500.0, 80.0);
        assert!(w > 0.0 && w <= 80.0, "{w}");
        assert!(clamp_label_w(0.0, 20.0) <= 20.0);
    }

    #[test]
    fn the_time_area_starts_a_gutter_past_the_label_column() {
        let rect = Rect::new(0.0, 0.0, 900.0, 400.0);
        let g = resolve(rect, 40.0, 200.0);
        assert_eq!(g.label_w, 200.0);
        assert_eq!(g.time_area.x, g.region.x + 200.0 + TIME_GUTTER);
        assert_eq!(
            time_x(rect, 200.0),
            g.time_area.x,
            "the two ways to find the time origin must agree"
        );
    }

    #[test]
    fn the_gutter_keeps_the_splitter_off_the_first_keyframe() {
        // The splitter grip is registered LAST, so it wins every hit it covers.
        // A key at the left edge of the view is drawn at `time_x`; its grab rect
        // starts `KEY_HIT_HW` to the left of that. The gutter is what stops the
        // two overlapping — drag the key, not the column.
        let seam = 0.0;
        let splitter_right = seam + SPLIT_GRIP;
        let first_key_left = seam + TIME_GUTTER - crate::tracks::KEY_HIT_HW;
        assert!(
            first_key_left >= splitter_right,
            "the splitter grip eats the first keyframe: key hit starts at \
             {first_key_left}, splitter ends at {splitter_right}"
        );
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
