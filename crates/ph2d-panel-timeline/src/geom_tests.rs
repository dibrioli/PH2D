//! Tests for [`super`] — the panel's geometry.
//!
//! A sibling module (`#[path]`) rather than an inline `mod tests`, to keep the
//! parent under the 600-line cap (HR-18) — the same split `graph.rs` and
//! `stack_lane_paint.rs` already use.

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

const K: Tab = Tab::Keys;
const A: Tab = Tab::Arrange;

/// One empty lane.
fn lane() -> ph2d_timeline::LaneView {
    ph2d_timeline::LaneView {
        name: "L".into(),
        muted: false,
        weight: 1.0,
        mode: ph2d_timeline::LaneMode::Override,
        strips: Vec::new(),
    }
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
                buffer_ghost: None,
                pre: ph2d_timeline::Extrap::Hold,
                post: ph2d_timeline::Extrap::Hold,
                expr: None,
                keys: Vec::new(),
            })
            .collect(),
        ..TimelineViewSnapshot::default()
    }
}

#[test]
fn scroll_max_is_zero_when_everything_fits() {
    const GH: f32 = 132.0;
    assert_eq!(scroll_max(content_h(&snap_with(2), K, &[], GH), 500.0), 0.0);
    let tall = content_h(&snap_with(20), K, &[], GH);
    assert_eq!(scroll_max(tall, 100.0), tall - 100.0);
}

#[test]
fn an_expanded_row_is_taller_and_pushes_the_rows_below_it_down() {
    const GH: f32 = 132.0;
    // The Summary channel is row zero, so every track sits one row lower and
    // the content is one row taller.
    const S: f32 = ROW_H_PX;
    let snap = snap_with(3);
    assert_eq!(content_h(&snap, K, &[], GH), S + ROW_H_PX * 3.0);
    assert_eq!(content_h(&snap, K, &[1], GH), S + ROW_H_PX * 3.0 + GH);

    // Row 0 collapsed, row 1 expanded: row 2 starts below BOTH.
    let bands: Vec<_> = row_bands(&snap, K, &[1], GH, 0.0, 0.0).collect();
    assert_eq!(bands[0], (0, S, ROW_H_PX));
    assert_eq!(bands[1], (1, S + ROW_H_PX, ROW_H_PX + GH));
    assert_eq!(bands[2].1, S + ROW_H_PX * 2.0 + GH);

    // A taller graph pushes them further: the height is not a constant.
    let taller: Vec<_> = row_bands(&snap, K, &[1], GH * 2.0, 0.0, 0.0).collect();
    assert_eq!(taller[2].1, S + ROW_H_PX * 2.0 + GH * 2.0);
}

#[test]
fn scrolling_shifts_every_row_band_up_by_the_same_amount() {
    const GH: f32 = 132.0;
    let snap = snap_with(3);
    let unscrolled: Vec<_> = row_bands(&snap, K, &[], GH, 10.0, 0.0).collect();
    let scrolled: Vec<_> = row_bands(&snap, K, &[], GH, 10.0, 30.0).collect();
    for (a, b) in unscrolled.iter().zip(&scrolled) {
        assert_eq!(a.1 - 30.0, b.1);
        assert_eq!(a.2, b.2, "scrolling never changes a row's height");
    }
}

#[test]
fn the_label_column_stays_between_its_bounds() {
    // Wide panel: the drag is honoured verbatim.
    assert_eq!(clamp_label_w(200.0, 800.0, MIN_LABEL_W), 200.0);
    // Dragged to nothing: floors at MIN_LABEL_W.
    assert_eq!(clamp_label_w(0.0, 800.0, MIN_LABEL_W), MIN_LABEL_W);
    // Dragged past the right edge: the time area keeps MIN_TIME_W.
    assert_eq!(
        clamp_label_w(10_000.0, 800.0, MIN_LABEL_W),
        800.0 - MIN_TIME_W
    );
}

#[test]
fn a_panel_too_narrow_for_both_still_yields_a_finite_column() {
    // MIN_LABEL_W + MIN_TIME_W does not fit: the label wins, capped by the
    // region, and never inverts into a negative time area.
    let w = clamp_label_w(500.0, 80.0, MIN_LABEL_W);
    assert!(w > 0.0 && w <= 80.0, "{w}");
    assert!(clamp_label_w(0.0, 20.0, MIN_LABEL_W) <= 20.0);
}

#[test]
fn the_time_area_starts_a_gutter_past_the_label_column() {
    let rect = Rect::new(0.0, 0.0, 900.0, 400.0);
    let g = resolve(rect, 40.0, 200.0, MIN_LABEL_W);
    assert_eq!(g.label_w, 200.0);
    assert_eq!(g.time_area.x, g.region.x + 200.0 + TIME_GUTTER);
    assert_eq!(
        time_x(rect, 200.0, MIN_LABEL_W),
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

/// **A column has a minimum because of what lives in it.** With lanes on
/// screen the label column carries a weight field, a mute, a "+ strip" and the
/// surface that opens the lane menu — which is the only way to DELETE a lane.
/// Squeezed to the track-row minimum that surface is 0 px wide (the menu
/// becomes unreachable) and the weight field lands 38 px off the panel's left
/// edge.
///
/// And with NO lanes the Arrange column still holds the "+ Lane"/"+ Container"
/// header — the two buttons that create the first lane. The old lanes-only floor
/// let an empty-stack Arrange squeeze to the track-row minimum and crushed
/// "+ Container" down to a bare "+" (Enio's screenshot, 2026-07-20): the floor
/// follows the TAB, because the header lives on the tab, not on the lanes.
#[test]
fn the_label_column_cannot_be_squeezed_below_what_a_lane_row_needs() {
    let mut snap = TimelineViewSnapshot::default();
    assert_eq!(
        min_label_w(&snap, A),
        MIN_LANE_LABEL_W,
        "no lanes yet, but Arrange still shows the ADD header"
    );

    snap.lanes.push(lane());
    let min = min_label_w(&snap, A);
    assert_eq!(min, MIN_LANE_LABEL_W);
    assert!(
        min > MIN_LABEL_W,
        "and it is strictly wider than the track-row minimum"
    );
    // A splitter dragged to nothing floors THERE, not at 56.
    assert_eq!(clamp_label_w(0.0, 800.0, min), MIN_LANE_LABEL_W);
}

/// **What the column holds is what the TAB shows.** The lane controls that
/// force the wider floor do not exist in the Keys tab, so a document with a
/// stack must not hold that tab's column hostage to lanes it is not showing.
#[test]
fn the_keys_tab_column_is_not_held_hostage_by_lanes_it_does_not_show() {
    let mut snap = snap_with(2);
    snap.lanes.push(lane());
    assert_eq!(
        min_label_w(&snap, A),
        MIN_LANE_LABEL_W,
        "Arrange shows them"
    );
    assert_eq!(
        min_label_w(&snap, K),
        MIN_LABEL_W,
        "Keys does not, so the floor is the track row's"
    );
}

/// Each tab lays out ONLY its half — the bug this whole split exists to kill
/// is the two halves sharing one ruler, so a tab that measured both would be
/// the bug wearing a tab strip.
#[test]
fn a_tab_gives_height_to_its_own_half_and_none_to_the_other() {
    const GH: f32 = 132.0;
    let mut snap = snap_with(3);
    snap.lanes.push(lane());
    snap.lanes.push(lane());

    // Keys: the Summary + three track rows, and not one pixel of lane.
    assert_eq!(stack_h(&snap, K), 0.0);
    assert_eq!(summary_h(&snap, K), ROW_H_PX);
    assert_eq!(row_bands(&snap, K, &[], GH, 0.0, 0.0).count(), 3);
    assert_eq!(stack_bands(&snap, K, 0.0, 0.0).count(), 0);
    assert_eq!(content_h(&snap, K, &[], GH), ROW_H_PX * 4.0);

    // Arrange: two lanes, and no Summary and no track rows.
    assert_eq!(stack_h(&snap, A), ROW_H_PX * 2.0);
    assert_eq!(summary_h(&snap, A), 0.0);
    assert_eq!(row_bands(&snap, A, &[], GH, 0.0, 0.0).count(), 0);
    assert_eq!(stack_bands(&snap, A, 0.0, 0.0).count(), 2);
    assert_eq!(content_h(&snap, A, &[], GH), ROW_H_PX * 2.0);
    assert_eq!(summary_band(&snap, A, 0.0, 0.0), None);
}

/// The Keys tab's rows start at the TOP of the band: with the lanes gone,
/// nothing sits above the Summary. (Track 0 used to be pushed down by every
/// lane in the document — including ones on another tab.)
#[test]
fn the_keys_tab_starts_at_the_top_however_many_lanes_the_document_has() {
    const GH: f32 = 132.0;
    let bare = snap_with(2);
    let mut stacked = bare.clone();
    for _ in 0..5 {
        stacked.lanes.push(lane());
    }
    let bands = |s: &TimelineViewSnapshot| -> Vec<_> {
        row_bands(s, K, &[], GH, 40.0, 0.0)
            .map(|(_, y, _)| y)
            .collect()
    };
    assert_eq!(
        bands(&bare),
        bands(&stacked),
        "the lanes are not on this tab"
    );
    assert_eq!(
        bands(&bare)[0],
        40.0 + ROW_H_PX,
        "the Summary, and nothing else"
    );
}
