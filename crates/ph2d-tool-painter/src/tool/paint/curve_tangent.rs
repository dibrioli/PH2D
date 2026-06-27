//! Bézier **tangent handles** for the Curve / Free Hand on-canvas editor — the per-anchor `[in, out]`
//! control points the artist drags to sculpt the curvature (the vector-editor "handles editáveis"). Pure,
//! transcendental-free geometry (projection + `sqrt` only, HR-5): hit-test, aligned mirror, and the
//! read-only overlay snapshot. Split from [`super::curve`] to keep that file under the workspace LOC cap.

/// The selected anchor's tangent handles, mapped for the shell overlay (drawn as dots on stems off the
/// anchor). A handle is `Some` only when it is BOTH used (interior side of an open curve) AND pulled far
/// enough off the anchor to be grabbable (beyond the anchor's grab tolerance) — so what is drawn is
/// exactly what can be grabbed, and a collapsed (sharp-corner) handle shows nothing.
pub struct TangentHandles {
    /// The anchor the handles belong to (image-space px) — the stem origin.
    pub anchor: [f32; 2],
    /// The incoming tangent control point (image-space px), if pulled out + used (anchor index `> 0`).
    pub in_handle: Option<[f32; 2]>,
    /// The outgoing tangent control point (image-space px), if pulled out + used (not the last anchor).
    pub out_handle: Option<[f32; 2]>,
    /// Which handle is being dragged this gesture (`true` = out, `false` = in), for the accent colour.
    pub grabbed_out: Option<bool>,
}

/// Hit-test the selected anchor's two tangent handles. Returns `(anchor, is_out)` when `pos` is within
/// `tol` of a *grabbable* handle (used side + pulled beyond `tol` of the anchor, so the anchor grab — which
/// the caller tests first — owns the points themselves). Closest of the two wins.
pub(super) fn tangent_hit(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    sel: usize,
    pos: [f32; 2],
    tol: f32,
) -> Option<(usize, bool)> {
    if handles.len() != points.len() || sel >= points.len() {
        return None;
    }
    let anchor = points[sel];
    let tol2 = tol * tol;
    let mut best: Option<(bool, f32)> = None;
    let candidates = [
        (false, sel > 0, handles[sel][0]),
        (true, sel + 1 < points.len(), handles[sel][1]),
    ];
    for (is_out, used, h) in candidates {
        if !used || dist2(h, anchor) <= tol2 {
            continue; // unused, or shadowed by the anchor grab
        }
        let d = dist2(h, pos);
        if d <= tol2 && best.is_none_or(|(_, bd)| d < bd) {
            best = Some((is_out, d));
        }
    }
    best.map(|(is_out, _)| (sel, is_out))
}

/// Mirror the opposite tangent to keep the pair **aligned** (collinear through the anchor), preserving the
/// opposite handle's length — the standard smooth-handle behaviour. A zero-length opposite (a sharp corner)
/// stays collapsed on the anchor, so dragging one side yields a one-sided tangent rather than fabricating one.
pub(super) fn mirror_tangent(h: &mut [[f32; 2]; 2], anchor: [f32; 2], moved_out: bool) {
    let (m, o) = if moved_out { (1, 0) } else { (0, 1) };
    let dx = h[m][0] - anchor[0];
    let dy = h[m][1] - anchor[1];
    let len = (dx * dx + dy * dy).sqrt();
    if len <= f32::EPSILON {
        return; // the moved handle collapsed onto the anchor — leave the opposite untouched
    }
    let ox = h[o][0] - anchor[0];
    let oy = h[o][1] - anchor[1];
    let olen = (ox * ox + oy * oy).sqrt(); // preserve the opposite's length
    h[o] = [anchor[0] - dx / len * olen, anchor[1] - dy / len * olen];
}

/// Build the overlay snapshot of the selected anchor's tangent handles — `None` when no anchor is selected,
/// the handles are absent/mismatched, or neither side is grabbable (both collapsed / endpoints). `tol` is the
/// image-space grab tolerance, so a handle is exposed exactly when [`tangent_hit`] could grab it.
pub(super) fn build_tangents(
    points: &[[f32; 2]],
    handles: &[[[f32; 2]; 2]],
    sel: usize,
    grabbed: Option<(usize, bool)>,
    tol: f32,
) -> Option<TangentHandles> {
    if handles.len() != points.len() || sel >= points.len() {
        return None;
    }
    let anchor = points[sel];
    let tol2 = tol * tol;
    let in_handle = (sel > 0 && dist2(handles[sel][0], anchor) > tol2).then_some(handles[sel][0]);
    let out_handle = (sel + 1 < points.len() && dist2(handles[sel][1], anchor) > tol2)
        .then_some(handles[sel][1]);
    if in_handle.is_none() && out_handle.is_none() {
        return None;
    }
    let grabbed_out = match grabbed {
        Some((i, is_out)) if i == sel => Some(is_out),
        _ => None,
    };
    Some(TangentHandles {
        anchor,
        in_handle,
        out_handle,
        grabbed_out,
    })
}

/// Squared distance between two image-space points.
fn dist2(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests;
