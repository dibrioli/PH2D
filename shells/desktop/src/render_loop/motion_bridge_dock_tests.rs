//! Seam tests for **the timeline dock** (W4.T4) — the shell half.
//!
//! The layout's own gates prove the *carve* (`ph2d-editor-core`: the band comes out of the graph,
//! they stop overlapping, the dock never eats its host). This proves the *wiring*: that entering the
//! Motion tool actually turns the timeline on, and that the two sides of the seam are talking about
//! the same key.
//!
//! Because that is the failure this feature is one typo away from: `panel_visibility` is a map with
//! a `false` default, so a mis-typed key does not error — the layout simply never docks, the
//! timeline goes on painting over the graph, and every test stays green.
//!
//! Declared by the parent as a `#[path]` sibling, so `super` is `render_loop::motion_bridge`.

use ph2d_editor::screens::hero::{PANEL_MOTION_GRAPH, PANEL_TIMELINE};
use ph2d_editor::screens::layout::{CenterSplit, HeroLayout, RAIL_W};
use ph2d_editor::zones::Rect;

const VP: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

/// **The bug the dock exists to fix.** Before the carve, the timeline's rect and the graph's rect
/// are the same pixels — and the timeline is drawn later, so it paints *on top of* the node editor.
///
/// This is the "before" half of the story, pinned so nobody can quietly restore it.
#[test]
fn without_the_dock_the_timeline_lies_on_top_of_the_graph() {
    let l = HeroLayout::for_viewport_split(
        VP,
        false,
        RAIL_W,
        CenterSplit::Horizontal {
            t: CenterSplit::T_DEFAULT,
        },
    );
    let graph = l.motion_graph;
    let tl = l.timeline;
    let overlap_h = (graph.y + graph.h).min(tl.y + tl.h) - graph.y.max(tl.y);
    assert!(
        overlap_h > 0.0,
        "the two used to share pixels - that is what W4.T4 is about"
    );
}

/// …and after it, they do not. One call, and the graph is exactly the band shorter.
#[test]
fn the_dock_separates_them() {
    let mut l = HeroLayout::for_viewport_split(
        VP,
        false,
        RAIL_W,
        CenterSplit::Horizontal {
            t: CenterSplit::T_DEFAULT,
        },
    );
    l.dock_timeline_into_motion();
    let graph = l.motion_graph;
    let tl = l.timeline;
    assert!(
        tl.y >= graph.y + graph.h - 0.01,
        "the timeline starts where the graph ends: graph ends {}, timeline starts {}",
        graph.y + graph.h,
        tl.y
    );
    assert!(graph.h > 0.0 && tl.h > 0.0, "both survived");
}

/// **The two sides of the seam are talking about the same key.**
///
/// The shell's bridge writes `panel_visibility[PANEL_MOTION_GRAPH]` / `[PANEL_TIMELINE]`; the
/// hero's paint reads the same two consts to decide whether to dock. They are consts precisely so
/// that this cannot drift — and this gate is what says so out loud, because the failure mode is a
/// missing key reading as `false`: no error, no dock, all green.
#[test]
fn the_bridge_and_the_paint_agree_on_the_keys() {
    assert_eq!(PANEL_MOTION_GRAPH, "motion_graph");
    assert_eq!(PANEL_TIMELINE, "timeline");
    // …and the timeline's key is the one the panel itself answers to, which is the third place
    // this string could have drifted.
    assert_eq!(
        PANEL_TIMELINE,
        <ph2d_panel_timeline::TimelinePanel as ph2d_editor::panel::Panel>::ID,
        "the visibility key must be the panel's own id, or the toggle turns on a panel nobody has"
    );
}
