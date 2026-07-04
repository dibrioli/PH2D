//! The **shared editing core** of the on-canvas Bézier curve — anchors + parallel `[in, out]` Bézier
//! handles + per-anchor [`HandleKind`] + the selected anchor + open/closed. Owned by BOTH the stroke
//! Shape editor ([`super::curve::CurveEditor`], slot `self.paint.curve`) AND the selection **Convert to
//! Curve** editor ([`super::SelectionShape::Freehand`]), so the two behave *identically by construction*
//! (Enio's repeated mandate — one system, not a parallel reimplementation; `docs/Painter/`).
//!
//! This is the "one curve core, two owners" from the handoff: sharing the pure editing state + ops is the
//! sanctioned form of reuse (ADR-0103 Am.2 only forbids sharing the live stroke *slot* so a selection edit
//! can't corrupt a half-open brush shape). All geometry defers to the sibling free fns
//! ([`super::curve_geom`] / [`super::curve_handle`] / [`super::curve_tangent`] + the brush crate's
//! `fit_curve`); transcendental-free (HR-5) throughout — the model itself only routes.

use super::curve_geom;
use super::curve_handle::{self, HandleKind};
use super::curve_tangent;

/// The editable curve: control `points` (image-space px), their parallel `[in, out]` Bézier `handles`,
/// the per-anchor `kinds`, the `selected` (highlighted / Delete-target) anchor, and whether the path is a
/// closed loop. A **raw lasso** carries empty `handles` + `kinds` (a polyline drawn straight through the
/// points); a **converted** curve carries `handles.len() == points.len()` (see [`Self::is_curve`]). The
/// transient grab lives in the owner ([`CurveGrab`]), so the model stays a pure value (Clone for undo).
// `pub(crate)` (not `pub(super)`): a converted selection's `CurveModel` rides inside `SelectionShape` into
// the crate-wide undo `ModelSnapshot`, so the type must be nameable crate-wide. Fields stay `pub(super)`.
#[derive(Clone, Debug, Default)]
pub(crate) struct CurveModel {
    /// Control points (image-space px).
    pub(super) points: Vec<[f32; 2]>,
    /// Per-anchor Bézier handles `[in, out]` (px), parallel to `points`. EMPTY for a raw polyline.
    pub(super) handles: Vec<[[f32; 2]; 2]>,
    /// Per-anchor handle kind. EMPTY for a raw polyline; Auto/Vector are re-derived on every edit.
    pub(super) kinds: Vec<HandleKind>,
    /// The selected (highlighted) point — the Delete target + the tangent-handle owner. `None` = none.
    pub(super) selected: Option<usize>,
    /// `true` for a closed loop (selection region / converted Ellipse-Polygon): spine + fill wrap last→first.
    pub(super) closed: bool,
}

/// What a press landed on — the owner remembers it as the active grab and feeds each Move to
/// [`CurveModel::drag`]. A whole-curve transform gizmo is NOT here: each owner layers its own on top
/// (the stroke's `curve_gizmo` bbox; the selection's box gizmo for the raw lasso).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CurveGrab {
    /// The anchor at this index is grabbed (drag moves the point, carrying its manual handles).
    Anchor(usize),
    /// A tangent handle off anchor `.0` — `.1 = true` is the OUT handle, `false` the IN handle.
    Tangent(usize, bool),
}

impl CurveModel {
    /// A **raw lasso** polyline (no Bézier handles → the transform-box mode, not the point editor).
    pub(super) fn raw_lasso(points: Vec<[f32; 2]>, closed: bool) -> Self {
        Self {
            points,
            handles: Vec::new(),
            kinds: Vec::new(),
            selected: None,
            closed,
        }
    }

    /// A **converted** curve from explicit anchors + handles (e.g. the Ellipse's exact 4-arc), with every
    /// anchor seeded to `seed_kind` (a manual kind, so the handles survive edits) and nothing selected.
    pub(super) fn from_curve(
        points: Vec<[f32; 2]>,
        handles: Vec<[[f32; 2]; 2]>,
        closed: bool,
        seed_kind: HandleKind,
    ) -> Self {
        let kinds = vec![seed_kind; points.len()];
        Self {
            points,
            handles,
            kinds,
            selected: None,
            closed,
        }
    }

    /// Fit a dense outline (`points` + parallel `handles`; pass degenerate `[p, p]` handles for a polygon,
    /// or empty for a Catmull-Rom smooth) to a **sparse** anchor+handle curve (Schneider, via
    /// [`curve_geom::simplify_curve`] — flatten the closed outline, re-fit cubics, merge the seam). The same
    /// fit a stroke Free-Hand / Simplify uses, so a converted selection keeps few anchors instead of the raw
    /// contour's hundreds. Seeds every anchor to `Aligned` (manual smooth). `None` (caller keeps the outline)
    /// when the fit is degenerate or exceeds `max_points`.
    pub(super) fn from_fit(
        points: &[[f32; 2]],
        handles: &[[[f32; 2]; 2]],
        closed: bool,
        max_error: f32,
        max_points: usize,
    ) -> Option<Self> {
        let (points, handles) =
            curve_geom::simplify_curve(points, handles, closed, max_error, max_points)?;
        Some(Self::from_curve(
            points,
            handles,
            closed,
            HandleKind::Aligned,
        ))
    }

    /// Whether this carries an editable Bézier curve (converted → the point editor) vs a raw polyline
    /// (lasso → the transform box). The two-mode switch Enio ratified ("editable only after Convert").
    pub(super) fn is_curve(&self) -> bool {
        !self.handles.is_empty() && self.handles.len() == self.points.len()
    }

    /// Hit-test a **point** for a press: the anchor under `pos` (selects it), else a tangent handle off the
    /// already-selected anchor. No insert / nearest — the owner composes those (and its own gizmo) around
    /// this, matching the stroke `curve_down` order (anchor → tangent → [gizmo] → insert → nearest).
    pub(super) fn hit_point(&mut self, pos: [f32; 2], tol: f32) -> Option<CurveGrab> {
        if let Some(i) = curve_geom::curve_hit(&self.points, pos, tol) {
            self.selected = Some(i);
            return Some(CurveGrab::Anchor(i));
        }
        self.selected
            .and_then(|s| {
                curve_tangent::tangent_hit(&self.points, &self.handles, s, pos, tol, self.closed)
            })
            .map(|(i, is_out)| CurveGrab::Tangent(i, is_out))
    }

    /// Select the nearest control point (no tolerance) — the fallback when the point cap blocks an insert.
    pub(super) fn select_nearest(&mut self, pos: [f32; 2]) {
        self.selected = curve_geom::curve_nearest(&self.points, pos);
    }

    /// Select the control point under `pos` (within `tol`) without grabbing — the shell's secondary-click
    /// before opening the handle-kind menu, so the pick applies to the right point. `true` iff one was hit.
    pub(super) fn select_point_at(&mut self, pos: [f32; 2], tol: f32) -> bool {
        let Some(i) = curve_geom::curve_hit(&self.points, pos, tol) else {
            return false;
        };
        self.selected = Some(i);
        true
    }

    /// **Insert** a new anchor on the curve at the nearest point to `pos` (de Casteljau split — the shape is
    /// unchanged), splice its neighbour handles, seed it `Aligned` (a smooth split), and re-derive the rest.
    /// Returns the new anchor's index; the caller sets `selected`/grab. Mirrors the stroke insert exactly.
    pub(super) fn insert(&mut self, pos: [f32; 2]) -> usize {
        let ins = curve_geom::curve_insert(&self.points, &self.handles, self.closed, pos);
        let parallel = self.handles.len() == self.points.len();
        if parallel {
            self.handles[ins.prev_idx][1] = ins.prev_out; // pre-insert indices still valid
            self.handles[ins.next_idx][0] = ins.next_in;
        }
        self.points.insert(ins.index, ins.anchor);
        self.kinds.insert(ins.index, HandleKind::Aligned);
        if parallel {
            self.handles.insert(ins.index, ins.handles);
        }
        curve_handle::rebuild(&self.points, &self.kinds, &mut self.handles, self.closed);
        ins.index
    }

    /// Apply a drag of the active `grab` to `pos`. A **tangent** drag turns a derived (Auto/Vector) anchor
    /// Aligned, sets the handle, and mirrors the opposite per the kind (Aligned re-aim / Symmetric reflect);
    /// an **anchor** drag moves the point, carries its MANUAL handles rigidly, and re-derives the rest.
    /// Byte-identical to the stroke `curve_move` edit branches. Out-of-range indices no-op.
    pub(super) fn drag(&mut self, grab: CurveGrab, pos: [f32; 2]) {
        match grab {
            CurveGrab::Tangent(i, is_out) => {
                if i < self.handles.len() && i < self.points.len() && i < self.kinds.len() {
                    if matches!(self.kinds[i], HandleKind::Auto | HandleKind::Vector) {
                        self.kinds[i] = HandleKind::Aligned;
                    }
                    let anchor = self.points[i];
                    self.handles[i][usize::from(is_out)] = pos;
                    if let Some(eq) = self.kinds[i].mirror_mode() {
                        curve_tangent::mirror_tangent(&mut self.handles[i], anchor, is_out, eq);
                    }
                }
            }
            CurveGrab::Anchor(g) => {
                if g >= self.points.len() {
                    return;
                }
                let old = self.points[g];
                self.points[g] = pos;
                if self.kinds.get(g).is_some_and(|k| k.is_manual())
                    && let Some(h) = self.handles.get_mut(g)
                {
                    let d = [pos[0] - old[0], pos[1] - old[1]];
                    *h = [
                        [h[0][0] + d[0], h[0][1] + d[1]],
                        [h[1][0] + d[0], h[1][1] + d[1]],
                    ];
                }
                curve_handle::rebuild(&self.points, &self.kinds, &mut self.handles, self.closed);
            }
        }
    }

    /// Delete the selected anchor (keeps `≥ 2` points), re-derive neighbours, and re-select a survivor.
    /// `true` when one was removed. Mirrors the stroke `curve_delete_selected` (minus the owner's undo txn).
    pub(super) fn delete_selected(&mut self) -> bool {
        let Some(sel) = self
            .selected
            .filter(|&s| self.points.len() > 2 && s < self.points.len())
        else {
            return false;
        };
        self.points.remove(sel);
        if sel < self.handles.len() {
            self.handles.remove(sel); // keep the parallel handles + kinds invariants
        }
        if sel < self.kinds.len() {
            self.kinds.remove(sel);
        }
        curve_handle::rebuild(&self.points, &self.kinds, &mut self.handles, self.closed);
        self.selected = Some(sel.min(self.points.len() - 1));
        true
    }

    /// Set the selected anchor's **handle kind** from the menu's wire `u8`, seeding a visible tangent when a
    /// sharp point turns manual, enforcing Symmetric, and re-deriving. `true` on success. Mirrors the stroke
    /// `set_curve_handle_kind` (minus the owner's undo txn).
    pub(super) fn set_kind(&mut self, wire: u8) -> bool {
        let Some(kind) = HandleKind::from_wire(wire) else {
            return false;
        };
        let Some(sel) = self.selected.filter(|&s| s < self.points.len()) else {
            return false;
        };
        if self.kinds.len() != self.points.len() {
            self.kinds = vec![HandleKind::Auto; self.points.len()];
        }
        self.kinds[sel] = kind;
        if kind.is_manual() && curve_handle::is_collapsed(self.handles.get(sel), self.points[sel]) {
            let auto = curve_handle::auto_handles(&self.points, self.closed);
            if let (Some(h), Some(a)) = (self.handles.get_mut(sel), auto.get(sel)) {
                *h = *a;
            }
        }
        if let Some(true) = kind.mirror_mode() {
            curve_tangent::mirror_tangent(&mut self.handles[sel], self.points[sel], true, true);
        }
        curve_handle::rebuild(&self.points, &self.kinds, &mut self.handles, self.closed);
        true
    }

    /// The selected anchor's handle-kind wire `u8` (for the shell to mark the active menu item), if any.
    pub(super) fn selected_kind_wire(&self) -> Option<u8> {
        self.selected
            .and_then(|s| self.kinds.get(s))
            .map(|k| k.wire())
    }
}

#[cfg(test)]
mod tests;
