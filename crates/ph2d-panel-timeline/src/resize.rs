//! Chrome resizes of the timeline panel: its own edges, the track-name column,
//! and the height of the expanded graph bands. Split from `view` (the time-axis
//! camera) under the HR-18 panel LOC cap.
//!
//! All three share one shape: capture the value and the pointer at Begin, apply
//! the delta to THAT on every Update, and let `paint` clamp the result. Applying
//! deltas to the live value instead would accumulate rounding across a slow drag.
//! None of them raise intents — none is undoable.

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture};
use ph2d_editor_core::zones::Rect;

use crate::geom;
use crate::state::{ResizeDrag, TimelinePanelState};

/// Splitter drag: widen or narrow the track-name column. The width applies to
/// the one captured at Begin (no drift), and `paint` clamps it into the panel.
pub(crate) fn apply_label_drag(state: &mut TimelinePanelState, g: TimelineGesture) {
    match g.phase {
        GesturePhase::Begin => state.label_drag = Some((state.label_w, g.x)),
        GesturePhase::Update => {
            if let Some((w0, x0)) = state.label_drag {
                state.label_w = w0 + (g.x - x0);
            }
        }
        _ => state.label_drag = None,
    }
}

/// Graph-band grip drag: taller or shorter curves. Applies to the height captured
/// at Begin (no drift); `paint` clamps it.
pub(crate) fn apply_graph_resize(state: &mut TimelinePanelState, g: TimelineGesture) {
    match g.phase {
        GesturePhase::Begin => state.graph_resize = Some((state.graph_h, g.y)),
        GesturePhase::Update => {
            if let Some((h0, y0)) = state.graph_resize {
                state.graph_h = h0 + (g.y - y0);
            }
        }
        _ => state.graph_resize = None,
    }
}

/// Edge/corner drag: move the set edges by the pointer delta from Begin. Deltas
/// apply to the rect captured at Begin, so a slow drag never accumulates drift.
pub(crate) fn apply_resize(
    state: &mut TimelinePanelState,
    rect: Rect,
    viewport: Rect,
    edges: u8,
    g: TimelineGesture,
) {
    match g.phase {
        GesturePhase::Begin => {
            state.resize = Some(ResizeDrag {
                edges,
                start_rect: rect,
                start_pointer: (g.x, g.y),
            });
        }
        GesturePhase::Update => {
            if let Some(d) = state.resize {
                let (dx, dy) = (g.x - d.start_pointer.0, g.y - d.start_pointer.1);
                state.rect = Some(geom::resized(d.start_rect, d.edges, dx, dy, viewport));
            }
        }
        _ => state.resize = None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::interaction::{GestureMods, TimelineHitKind};
    use ph2d_host::PointerButton;

    const VP: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);

    fn drag(button: PointerButton, phase: GesturePhase, x: f32, y: f32) -> TimelineGesture {
        TimelineGesture {
            surface: ph2d_a11y::NodeId(0),
            kind: TimelineHitKind::Lane,
            phase,
            x,
            y,
            button,
            mods: GestureMods::default(),
        }
    }

    #[test]
    fn dragging_the_splitter_widens_the_label_column() {
        let mut st = TimelinePanelState {
            label_w: 132.0,
            ..TimelinePanelState::default()
        };
        apply_label_drag(
            &mut st,
            drag(PointerButton::Primary, GesturePhase::Begin, 200.0, 0.0),
        );
        apply_label_drag(
            &mut st,
            drag(PointerButton::Primary, GesturePhase::Update, 260.0, 0.0),
        );
        assert_eq!(st.label_w, 192.0);
        // A second Update still measures from Begin, never from the live width.
        apply_label_drag(
            &mut st,
            drag(PointerButton::Primary, GesturePhase::Update, 150.0, 0.0),
        );
        assert_eq!(st.label_w, 82.0, "no drift accumulation");
        apply_label_drag(
            &mut st,
            drag(PointerButton::Primary, GesturePhase::End, 150.0, 0.0),
        );
        assert!(st.label_drag.is_none());
    }

    #[test]
    fn an_update_without_a_begin_moves_nothing() {
        let mut st = TimelinePanelState::default();
        let before = st.label_w;
        apply_label_drag(
            &mut st,
            drag(PointerButton::Primary, GesturePhase::Update, 900.0, 0.0),
        );
        assert_eq!(st.label_w, before);
    }

    #[test]
    fn dragging_the_graph_grip_resizes_every_expanded_band() {
        let mut st = TimelinePanelState {
            graph_h: 132.0,
            ..TimelinePanelState::default()
        };
        apply_graph_resize(
            &mut st,
            drag(PointerButton::Primary, GesturePhase::Begin, 0.0, 400.0),
        );
        apply_graph_resize(
            &mut st,
            drag(PointerButton::Primary, GesturePhase::Update, 0.0, 460.0),
        );
        assert_eq!(st.graph_h, 192.0);
        // Measured from Begin, never from the live height.
        apply_graph_resize(
            &mut st,
            drag(PointerButton::Primary, GesturePhase::Update, 0.0, 380.0),
        );
        assert_eq!(st.graph_h, 112.0, "no drift accumulation");
        apply_graph_resize(
            &mut st,
            drag(PointerButton::Primary, GesturePhase::End, 0.0, 380.0),
        );
        assert!(st.graph_resize.is_none());
    }

    #[test]
    fn the_graph_height_stays_between_its_bounds() {
        use crate::graph::clamp_graph_h;
        assert_eq!(clamp_graph_h(200.0), 200.0);
        assert!(clamp_graph_h(-500.0) > 0.0, "a band is never inverted");
        assert!(clamp_graph_h(10_000.0) < 10_000.0, "and never unbounded");
    }

    #[test]
    fn dragging_the_top_edge_resizes_from_the_rect_captured_at_begin() {
        let mut st = TimelinePanelState::default();
        let start = Rect::new(100.0, 600.0, 800.0, 240.0);
        let g = |phase, y| drag(PointerButton::Primary, phase, 400.0, y);

        apply_resize(
            &mut st,
            start,
            VP,
            geom::EDGE_T,
            g(GesturePhase::Begin, 600.0),
        );
        apply_resize(
            &mut st,
            start,
            VP,
            geom::EDGE_T,
            g(GesturePhase::Update, 550.0),
        );
        let r = st.rect.expect("resized");
        assert_eq!((r.y, r.h), (550.0, 290.0), "grew upward");

        // A second Update still measures from Begin, not from the live rect.
        apply_resize(
            &mut st,
            start,
            VP,
            geom::EDGE_T,
            g(GesturePhase::Update, 500.0),
        );
        let r = st.rect.expect("resized");
        assert_eq!((r.y, r.h), (500.0, 340.0), "no drift accumulation");

        apply_resize(
            &mut st,
            start,
            VP,
            geom::EDGE_T,
            g(GesturePhase::End, 500.0),
        );
        assert!(st.resize.is_none());
    }
}
