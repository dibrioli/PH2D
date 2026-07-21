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
use crate::tab::Tab;

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
/// The label column's minimum **while the clip stack has lanes**.
///
/// A lane's row carries a weight field, a mute, a "+ strip" and — in what is left
/// of it — the surface that opens the lane menu (mode, and the ONLY way to delete
/// a lane). Squeezed to [`MIN_LABEL_W`] that surface is 0 px wide and the menu
/// becomes unreachable, while the weight field is laid out 38 px off the panel's
/// left edge. A column has a minimum because of what lives in it; when lanes live
/// there, this is it.
pub(crate) const MIN_LANE_LABEL_W: f32 = 148.0; // LITERAL-PX-OK: lane controls (94) + a name

/// What the label column may not go below, given what it currently holds.
///
/// **What it holds is what the TAB shows.** The lane controls that force the wider
/// floor only exist in the Arrange tab; asking "does the document have lanes"
/// instead of "are they on screen" would hold the Keys tab's column hostage to a
/// stack it is not showing — the rule is *a column has a minimum because of what
/// lives in it*, and a tab decides what lives in it.
///
/// The Arrange floor does NOT ask whether lanes exist: the tab's header carries the
/// "+ Lane"/"+ Container" pair even over an empty stack (they are how the first lane
/// is made), and the lanes-only floor let exactly that state crush "+ Container"
/// down to a bare "+" (Enio's screenshot, 2026-07-20).
pub(crate) fn min_label_w(_snap: &TimelineViewSnapshot, tab: Tab) -> f32 {
    if tab.shows_lanes() {
        MIN_LANE_LABEL_W
    } else {
        MIN_LABEL_W
    }
}
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
pub(crate) fn clamp_label_w(label_w: f32, region_w: f32, min: f32) -> f32 {
    let widest = (region_w - MIN_TIME_W).max(min);
    label_w.min(widest).max(min).min(region_w)
}

/// Where `view_start_s` maps — the seam plus [`TIME_GUTTER`]. Depends only on the
/// panel rect + the label width, so `interact::process` can have it before the
/// transport bar paints.
pub(crate) fn time_x(rect: Rect, label_w: f32, min_label: f32) -> f32 {
    let body_x = rect.x + PANEL_HEAD_PAD;
    let body_w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    body_x + clamp_label_w(label_w, body_w, min_label) + TIME_GUTTER
}

/// Resolve the dope-sheet sub-rects from the panel `rect`, the transport bar's
/// bottom edge and the (user-resizable) label-column width. (`region.h` does not
/// depend on the title size: the body's bottom is `rect.y + rect.h -
/// PANEL_HEAD_PAD` either way.)
pub(crate) fn resolve(rect: Rect, after_transport: f32, label_w: f32, min_label: f32) -> Geom {
    let x = rect.x + PANEL_HEAD_PAD;
    let w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    let bottom = rect.y + rect.h - PANEL_HEAD_PAD;
    let region = Rect::new(x, after_transport, w, (bottom - after_transport).max(0.0));
    let label_w = clamp_label_w(label_w, region.w, min_label);
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

/// Height the clip-stack lanes occupy. Zero in the Keys tab — and zero when the
/// document has no stack, which is the norm, and is why a timeline that never
/// touches the feature looks exactly as it always did.
///
/// **The tab is asked HERE, once.** Every other measure below (`content_h`,
/// `row_bands`, `stack_bands`, `summary_band`) is built out of this and
/// [`summary_h`], so a tab showing the wrong half is not expressible: the rows
/// cannot disagree with the heights they are stacked from.
pub(crate) fn stack_h(snap: &TimelineViewSnapshot, tab: Tab) -> f32 {
    if tab.shows_lanes() {
        ROW_H_PX * snap.lanes.len() as f32
    } else {
        0.0
    }
}

/// Every lane's `(index, top_y, height)` in the scrolled band — and **nothing at
/// all outside the Arrange tab**, where the lanes have no height to sit in.
pub(crate) fn stack_bands(
    snap: &TimelineViewSnapshot,
    tab: Tab,
    rows_top: f32,
    scroll_y: f32,
) -> impl Iterator<Item = (usize, f32, f32)> + '_ {
    let top = rows_top - scroll_y;
    let n = if tab.shows_lanes() {
        snap.lanes.len()
    } else {
        0
    };
    (0..n).map(move |i| (i, top + ROW_H_PX * i as f32, ROW_H_PX))
}

/// Height the Summary channel occupies above the tracks. Zero in the Arrange tab
/// (it summarises KEYS), and zero when nothing is bound: an empty timeline shows
/// no master row to grab.
pub(crate) fn summary_h(snap: &TimelineViewSnapshot, tab: Tab) -> f32 {
    if tab.shows_keys() && !snap.tracks.is_empty() {
        ROW_H_PX
    } else {
        0.0
    }
}

/// Total height the rows want, for the scroll range — whichever half this tab
/// shows. Built from the same heights the bands are, so the scrollbar can never
/// promise a row the layout does not place.
pub(crate) fn content_h(
    snap: &TimelineViewSnapshot,
    tab: Tab,
    expanded: &[u64],
    graph_h: f32,
) -> f32 {
    stack_h(snap, tab)
        + summary_h(snap, tab)
        + row_bands(snap, tab, expanded, graph_h, 0.0, 0.0)
            .map(|(_, _, h)| h)
            .sum::<f32>()
}

/// The Summary channel's row rect, or `None` when there is no track to summarise.
/// It scrolls with the rows it aggregates, so its diamonds always sit directly
/// above the columns they stand for.
pub(crate) fn summary_band(
    snap: &TimelineViewSnapshot,
    tab: Tab,
    rows_top: f32,
    scroll_y: f32,
) -> Option<(f32, f32)> {
    let h = summary_h(snap, tab);
    (h > 0.0).then_some((rows_top - scroll_y + stack_h(snap, tab), h))
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
    tab: Tab,
    expanded: &'a [u64],
    graph_h: f32,
    rows_top: f32,
    scroll_y: f32,
) -> impl Iterator<Item = (usize, f32, f32)> + 'a {
    // The clip lanes sit above everything, then the Summary channel, then track 0
    // — and all three scroll together. THIS is the one place that ordering lives:
    // the painters read the bands, they never re-derive a y.
    let mut y = rows_top - scroll_y + stack_h(snap, tab) + summary_h(snap, tab);
    // The Arrange tab has no track rows at all — `take(0)` rather than an early
    // return, so the iterator's type stays one type.
    let n = if tab.shows_keys() {
        snap.tracks.len()
    } else {
        0
    };
    snap.tracks.iter().take(n).enumerate().map(move |(i, t)| {
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
#[path = "geom_tests.rs"]
mod tests;
