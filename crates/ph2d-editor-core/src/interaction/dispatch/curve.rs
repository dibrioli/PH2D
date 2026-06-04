//! W4 §3 — 2-D curve control-point drag dispatch.
//!
//! Mirror of [`super::blender`]'s `apply_blender_hit`: a Curves/Levels-style
//! editor registers one [`super::super::InteractiveState::CurvePoint`] per
//! draggable control point (carrying the parent editor id, the active channel,
//! the point index, and the plotting `canvas` rect). On a Down/Move drag the
//! dispatch normalizes the pointer within `canvas` to `(x, y)` in `[0, 1]` and
//! stashes it on the [`WidgetStore`] for the panel to drain each frame and
//! forward to its tool (`PainterTool::set_curve_point`) — the curve itself
//! lives in the tool, not the editor-core store, so (like the layer-row drags)
//! the dispatch only computes + stashes, it never mutates curve state.

use super::super::{InteractiveState, WidgetStore};
use ph2d_a11y::NodeId;

/// If `id` is an [`InteractiveState::CurvePoint`], normalize the pointer
/// `(px, py)` within its `canvas` rect to `(x, y)` in `[0, 1]` (y inverted:
/// canvas top = `1.0` = high output) and stash `(parent, channel, index, x, y)`
/// via [`WidgetStore::set_curve_point_drag`]. Returns the editor `parent` (so
/// the caller emits `ValueChanged`), or `None` for a non-`CurvePoint` id.
pub(super) fn apply_curve_point_drag(
    store: &mut WidgetStore,
    id: NodeId,
    px: f32,
    py: f32,
) -> Option<NodeId> {
    let (parent, channel, index, canvas) = match store.get(id)? {
        InteractiveState::CurvePoint {
            parent,
            channel,
            index,
            canvas,
        } => (*parent, *channel, *index, *canvas),
        _ => return None,
    };
    let x = if canvas.w > 0.0 {
        ((px - canvas.x) / canvas.w).clamp(0.0, 1.0)
    } else {
        0.0
    };
    // y inverted: the canvas TOP (small `py`) is the HIGH output (1.0).
    let y = if canvas.h > 0.0 {
        (1.0 - (py - canvas.y) / canvas.h).clamp(0.0, 1.0)
    } else {
        0.0
    };
    store.set_curve_point_drag(parent, channel, index, x, y);
    Some(parent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::InteractiveState;
    use crate::zones::Rect;

    #[test]
    fn normalizes_pointer_in_canvas_with_y_inverted() {
        let mut store = WidgetStore::with_capacity(2);
        let editor = NodeId(100);
        let handle = NodeId(101);
        // Canvas at (10, 20), 200×100. Point index 2 on the R channel.
        store.register(
            handle,
            InteractiveState::CurvePoint {
                parent: editor,
                channel: 1,
                index: 2,
                canvas: Rect::new(10.0, 20.0, 200.0, 100.0),
            },
        );
        // Pointer at canvas mid-x, top quarter → x = 0.5, y = 0.75 (inverted).
        let parent = apply_curve_point_drag(&mut store, handle, 110.0, 45.0).expect("CurvePoint");
        assert_eq!(parent, editor);
        let (p, ch, idx, x, y) = store.take_curve_point_drag().expect("drag stashed");
        assert_eq!((p, ch, idx), (editor, 1, 2));
        assert!((x - 0.5).abs() < 1e-5, "x = {x}");
        assert!((y - 0.75).abs() < 1e-5, "y = {y}");
        // Drained once.
        assert!(store.take_curve_point_drag().is_none());
    }

    #[test]
    fn clamps_outside_canvas_and_ignores_non_curvepoint() {
        let mut store = WidgetStore::with_capacity(2);
        let editor = NodeId(200);
        let handle = NodeId(201);
        store.register(
            handle,
            InteractiveState::CurvePoint {
                parent: editor,
                channel: 0,
                index: 0,
                canvas: Rect::new(0.0, 0.0, 100.0, 100.0),
            },
        );
        // Far below-right of the canvas → clamps to x=1, y=0.
        apply_curve_point_drag(&mut store, handle, 999.0, 999.0).expect("CurvePoint");
        let (_, _, _, x, y) = store.take_curve_point_drag().unwrap();
        assert_eq!((x, y), (1.0, 0.0));
        // A non-CurvePoint id is a no-op.
        store.register(NodeId(202), InteractiveState::Plain);
        assert!(apply_curve_point_drag(&mut store, NodeId(202), 1.0, 1.0).is_none());
    }
}
