//! Per-anchor **handle kinds** for the Curve / Free Hand editor — the vector-app continuity types the
//! artist picks from the right-click menu. Pure, transcendental-free geometry (lerp + the brush crate's
//! chordal `auto_handles`, which uses only `sqrt`, HR-5). Split from [`super::curve`] for the LOC cap.
//!
//! - **Free** — the two tangents are independent (a broken corner); edits touch only the dragged side.
//! - **Aligned** — the tangents stay collinear through the anchor (smooth), independent lengths.
//! - **Symmetric** — collinear AND equal length (a dragged tangent mirrors the opposite exactly).
//! - **Vector** — the tangents point straight at the adjacent anchors (a polyline-like segment).
//! - **Auto** — the tangents are recomputed by the no-overshoot chordal smooth on every edit.
//!
//! `Auto` and `Vector` are *derived* (recomputed from the control points by [`rebuild`]); `Aligned`,
//! `Symmetric` and `Free` are *manual* (the artist's handle positions are preserved across anchor moves).

/// How an anchor's two Bézier tangent handles behave under editing. The wire `u8` (`0..=3`) is the value
/// the right-click menu parks in the shell and hands back to [`super::PainterTool::set_curve_handle_kind`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum HandleKind {
    Free,
    Aligned,
    Symmetric,
    Vector,
    Auto,
}

impl HandleKind {
    /// Decode the menu's wire value (`0 = Free`, `1 = Aligned`, `2 = Vector`, `3 = Auto`, `4 = Symmetric`).
    pub(crate) fn from_wire(u: u8) -> Option<Self> {
        match u {
            0 => Some(Self::Free),
            1 => Some(Self::Aligned),
            2 => Some(Self::Vector),
            3 => Some(Self::Auto),
            4 => Some(Self::Symmetric),
            _ => None,
        }
    }

    /// The wire value (inverse of [`Self::from_wire`]) — used by the overlay snapshot.
    pub(crate) fn wire(self) -> u8 {
        match self {
            Self::Free => 0,
            Self::Aligned => 1,
            Self::Vector => 2,
            Self::Auto => 3,
            Self::Symmetric => 4,
        }
    }

    /// `true` for the manual kinds whose handles the artist owns (preserved across anchor moves); `false`
    /// for the derived kinds ([`Self::Auto`] / [`Self::Vector`]) that [`rebuild`] recomputes from points.
    pub(crate) fn is_manual(self) -> bool {
        matches!(self, Self::Free | Self::Aligned | Self::Symmetric)
    }

    /// For a tangent drag, how the opposite handle follows: `Some(false)` = Aligned (re-aim, keep its
    /// length), `Some(true)` = Symmetric (exact reflection), `None` = no mirror (Free / derived kinds).
    pub(crate) fn mirror_mode(self) -> Option<bool> {
        match self {
            Self::Aligned => Some(false),
            Self::Symmetric => Some(true),
            _ => None,
        }
    }
}

/// Recompute the **derived** handles (Auto / Vector) in place from the current control points, leaving the
/// **manual** ones (Aligned / Free) untouched. Called after any structural edit (anchor move / insert /
/// delete / kind change) so the smooth + polyline anchors track their neighbours while hand-placed tangents
/// stay put. No-op on a length mismatch (the degenerate draw-phase state).
pub(super) fn rebuild(
    points: &[[f32; 2]],
    kinds: &[HandleKind],
    handles: &mut Vec<[[f32; 2]; 2]>,
    closed: bool,
) {
    let n = points.len();
    if n == 0 || kinds.len() != n {
        return;
    }
    if handles.len() != n {
        // Size the buffer; the per-kind pass below overwrites Auto/Vector, manual kinds keep this seed.
        *handles = auto_handles(points, closed);
    }
    let auto = kinds
        .contains(&HandleKind::Auto)
        .then(|| auto_handles(points, closed));
    for (i, &kind) in kinds.iter().enumerate() {
        match kind {
            HandleKind::Auto => {
                if let Some(a) = &auto {
                    handles[i] = a[i];
                }
            }
            HandleKind::Vector => handles[i] = vector_handle(points, i, closed),
            HandleKind::Aligned | HandleKind::Free | HandleKind::Symmetric => {}
        }
    }
}

/// Auto (chordal-smooth) tangent handles for every anchor. An OPEN curve defers to the brush crate's
/// no-overshoot chordal fit (single-armed endpoints); a CLOSED loop uses the cardinal-spline tangent with
/// WRAPPING neighbours, so the seam anchors get BOTH arms (the open fit would collapse the first in / last
/// out → the one-armed seam handle Enio hit on a converted Polygon / Ellipse). Transcendental-free.
pub(super) fn auto_handles(points: &[[f32; 2]], closed: bool) -> Vec<[[f32; 2]; 2]> {
    if !closed {
        return ph2d_painter_brush::auto_handles(points);
    }
    let n = points.len();
    (0..n)
        .map(|i| {
            let prev = points[(i + n - 1) % n];
            let next = points[(i + 1) % n];
            let p = points[i];
            let (tx, ty) = ((next[0] - prev[0]) / 6.0, (next[1] - prev[1]) / 6.0);
            [[p[0] - tx, p[1] - ty], [p[0] + tx, p[1] + ty]]
        })
        .collect()
}

/// The **Vector** handles for anchor `i`: each tangent points one-third of the way toward the adjacent
/// anchor (so a run of Vector anchors flattens to straight segments). On an OPEN curve the unused endpoint
/// side collapses onto the anchor; a CLOSED loop wraps, so the seam keeps both arms.
pub(super) fn vector_handle(points: &[[f32; 2]], i: usize, closed: bool) -> [[f32; 2]; 2] {
    let n = points.len();
    let a = points[i];
    let third = |b: [f32; 2]| [a[0] + (b[0] - a[0]) / 3.0, a[1] + (b[1] - a[1]) / 3.0];
    let in_h = if i > 0 {
        third(points[i - 1])
    } else if closed {
        third(points[n - 1])
    } else {
        a
    };
    let out_h = if i + 1 < n {
        third(points[i + 1])
    } else if closed {
        third(points[0])
    } else {
        a
    };
    [in_h, out_h]
}

/// Whether anchor `i`'s handles are collapsed onto it (a sharp point with no draggable tangent) — used to
/// seed a visible tangent when the artist switches a sharp point to a manual kind.
pub(super) fn is_collapsed(handle: Option<&[[f32; 2]; 2]>, anchor: [f32; 2]) -> bool {
    let Some(h) = handle else {
        return true;
    };
    let near = |p: [f32; 2]| (p[0] - anchor[0]).abs() < 1e-3 && (p[1] - anchor[1]).abs() < 1e-3;
    near(h[0]) && near(h[1])
}

#[cfg(test)]
mod tests;
