//! **Converted-selection-curve point editor** (ADR-0103 Am.2 §3, Enio 2026-07-03). After **Convert to
//! Curve** / **Simplify Curve** the selection is a single `Freehand` carrying explicit Bézier `handles`, and
//! it must edit like the stroke Shape system's Curve — per-anchor **points** with in/out **Bézier handles** —
//! NOT the whole-shape transform box a RAW lasso Freehand (empty handles) shows. This module owns that
//! point-level view + hit-test + drag, staying inside the isolated selection gizmo system (it never touches
//! the stroke editors). Transcendental-free (drift-free from the pristine grab).

use super::selection_shapes::SelectionShape;

/// Handle-part discriminants of a curve grab: the anchor, or its IN / OUT Bézier handle.
pub(super) const PART_ANCHOR: u8 = 0;
pub(super) const PART_IN: u8 = 1;
pub(super) const PART_OUT: u8 = 2;

/// A grabbed converted-curve target: which `anchor` of the Freehand, and which `part` (anchor / in / out).
#[derive(Clone, Copy, Debug)]
pub(super) struct CurveGrab {
    pub anchor: usize,
    pub part: u8,
}

/// The drawable point-editor of a converted selection curve (image px): anchors + their in/out Bézier
/// handles (parallel to `anchors`). The shell draws anchors as squares, handles as circles joined to their
/// anchor by a thin line — the stroke Curve editor's look.
pub struct SelectionCurveEdit {
    pub anchors: Vec<[f32; 2]>,
    pub in_h: Vec<[f32; 2]>,
    pub out_h: Vec<[f32; 2]>,
}

/// `true` when `shape` is a CONVERTED curve (a `Freehand` carrying explicit Bézier handles, one `[in, out]`
/// per anchor) — the Convert / Simplify output. A raw lasso `Freehand` has EMPTY handles ⇒ `false` (it shows
/// the transform box instead). This is the switch between the two Freehand editing modes.
#[must_use]
pub(super) fn is_converted_curve(shape: &SelectionShape) -> bool {
    matches!(
        shape,
        SelectionShape::Freehand { points, handles, .. }
            if !points.is_empty() && handles.len() == points.len()
    )
}

/// Build the point-editor view of a converted curve (`None` for any other shape).
#[must_use]
pub(super) fn curve_edit_view(shape: &SelectionShape) -> Option<SelectionCurveEdit> {
    let SelectionShape::Freehand {
        points, handles, ..
    } = shape
    else {
        return None;
    };
    if points.is_empty() || handles.len() != points.len() {
        return None;
    }
    Some(SelectionCurveEdit {
        anchors: points.clone(),
        in_h: handles.iter().map(|h| h[0]).collect(),
        out_h: handles.iter().map(|h| h[1]).collect(),
    })
}

/// Hit-test the converted curve's points + handles at `pos` within `tol` — handles win over anchors (they
/// sit on top), nearest wins within each. `None` when nothing is grabbed / the shape isn't a converted curve.
#[must_use]
pub(super) fn hit_curve(shape: &SelectionShape, pos: [f32; 2], tol: f32) -> Option<CurveGrab> {
    let SelectionShape::Freehand {
        points, handles, ..
    } = shape
    else {
        return None;
    };
    if points.is_empty() || handles.len() != points.len() {
        return None;
    }
    let tol2 = tol * tol;
    // 1) Bézier handles (in / out) first — they overlay the anchors.
    let mut best: Option<(CurveGrab, f32)> = None;
    for (i, h) in handles.iter().enumerate() {
        for (part, hp) in [(PART_IN, h[0]), (PART_OUT, h[1])] {
            let d = dist2(hp, pos);
            if d <= tol2 && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((CurveGrab { anchor: i, part }, d));
            }
        }
    }
    // 2) Anchors — only if no handle was grabbed (a handle atop its anchor stays grabbable).
    if best.is_none() {
        for (i, a) in points.iter().enumerate() {
            let d = dist2(*a, pos);
            if d <= tol2 && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((
                    CurveGrab {
                        anchor: i,
                        part: PART_ANCHOR,
                    },
                    d,
                ));
            }
        }
    }
    best.map(|(g, _)| g)
}

/// Apply a converted-curve drag to the PRISTINE `initial` shape (drift-free): moving an ANCHOR carries its
/// two handles with it (the tangent rides along); moving an IN / OUT handle moves just that handle (a Free
/// corner — independent tangents, like the stroke editor's Free kind). Returns the edited `Freehand`.
#[must_use]
pub(super) fn apply_curve_drag(
    initial: &SelectionShape,
    grab: CurveGrab,
    start: [f32; 2],
    pos: [f32; 2],
) -> SelectionShape {
    let SelectionShape::Freehand { points, handles, u } = initial else {
        return initial.clone();
    };
    if grab.anchor >= points.len() || handles.len() != points.len() {
        return initial.clone();
    }
    let d = [pos[0] - start[0], pos[1] - start[1]];
    let mv = |p: [f32; 2]| [p[0] + d[0], p[1] + d[1]];
    let mut points = points.clone();
    let mut handles = handles.clone();
    let i = grab.anchor;
    match grab.part {
        PART_ANCHOR => {
            points[i] = mv(points[i]);
            handles[i] = [mv(handles[i][0]), mv(handles[i][1])];
        }
        PART_IN => handles[i][0] = mv(handles[i][0]),
        _ => handles[i][1] = mv(handles[i][1]),
    }
    SelectionShape::Freehand {
        points,
        handles,
        u: *u,
    }
}

fn dist2(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests {
    use super::*;

    fn converted() -> SelectionShape {
        SelectionShape::Freehand {
            points: vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]],
            handles: vec![
                [[-2.0, 0.0], [2.0, 0.0]],
                [[8.0, 0.0], [12.0, 0.0]],
                [[10.0, 8.0], [10.0, 12.0]],
            ],
            u: [1.0, 0.0],
        }
    }

    #[test]
    fn only_a_curve_with_handles_is_point_editable() {
        assert!(is_converted_curve(&converted()), "handles present ⇒ curve");
        let raw = SelectionShape::Freehand {
            points: vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]],
            handles: Vec::new(), // a raw lasso ⇒ transform box, not point editing
            u: [1.0, 0.0],
        };
        assert!(
            !is_converted_curve(&raw),
            "empty handles ⇒ box, not a curve"
        );
    }

    #[test]
    fn hits_the_out_handle_then_drags_only_it() {
        let shape = converted();
        // The out-handle of anchor 0 is at (2,0); grabbing there and dragging moves ONLY that handle.
        let g = hit_curve(&shape, [2.0, 0.0], 2.0).expect("grabbed a handle");
        assert_eq!((g.anchor, g.part), (0, PART_OUT));
        let out = apply_curve_drag(&shape, g, [2.0, 0.0], [5.0, 3.0]);
        let SelectionShape::Freehand {
            points, handles, ..
        } = out
        else {
            panic!("still a freehand")
        };
        assert_eq!(points[0], [0.0, 0.0], "the anchor did not move");
        assert_eq!(
            handles[0][1],
            [5.0, 3.0],
            "the out handle moved to the pointer"
        );
        assert_eq!(handles[0][0], [-2.0, 0.0], "the in handle is untouched");
    }

    #[test]
    fn dragging_an_anchor_carries_its_handles() {
        let shape = converted();
        let g = hit_curve(&shape, [0.0, 0.0], 1.5).expect("grabbed the anchor");
        assert_eq!(g.part, PART_ANCHOR);
        let out = apply_curve_drag(&shape, g, [0.0, 0.0], [4.0, 0.0]);
        let SelectionShape::Freehand {
            points, handles, ..
        } = out
        else {
            panic!("freehand")
        };
        assert_eq!(points[0], [4.0, 0.0], "anchor moved by +4x");
        assert_eq!(
            handles[0],
            [[2.0, 0.0], [6.0, 0.0]],
            "both handles rode along"
        );
    }
}
