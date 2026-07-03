//! **Selection shape list** (ADR-0103 Amendment 2) — the parametric source of truth behind the selection
//! mask. Every creation gesture appends a [`SelectionEntry`] (a parametric shape + a boolean op); the
//! `selection_mask` is a DERIVED cache produced by [`PainterTool::recompose_selection_mask`] (rasterize
//! each entry, combine by its op, feather). Editing a native gizmo in Edit mode rewrites ONE entry and
//! recomposites, so multiple shapes stay independently editable while the mask always reflects the whole
//! list. `Raster` carries a non-parametric coverage buffer (Automatic flood) that has no gizmo.

use super::PainterTool;
use std::sync::Arc;

/// A parametric selection primitive. `Ellipse`/`Rect`/`Freehand` carry a native on-canvas gizmo in Edit
/// mode; `Raster` (Automatic flood, or any op with no parametric form) is a frozen coverage buffer.
#[derive(Clone, Debug)]
pub(crate) enum SelectionShape {
    /// Axis-or-rotated ellipse: `center`, unit axis `u`, semi-axes `rx`/`ry` (image px). Native gizmo =
    /// the Ellipse editor.
    Ellipse {
        center: [f32; 2],
        u: [f32; 2],
        rx: f32,
        ry: f32,
    },
    /// Axis-aligned rectangle between opposite corners `a`, `b` (image px) — the freshly-drawn marquee.
    /// Editing it promotes the entry to `Freehand` (4 draggable corners), so this variant is only the
    /// pristine rectangle.
    Rect { a: [f32; 2], b: [f32; 2] },
    /// Closed editable curve: anchors `points` with `[in, out]` Bézier `handles` and per-anchor `kinds`
    /// (wire `u8`, matching [`crate::undo::CurveState`]). When `handles` is empty the anchors are filled as
    /// a plain polygon. Native gizmo = the Curve editor (closed loop).
    Freehand {
        points: Vec<[f32; 2]>,
        handles: Vec<[[f32; 2]; 2]>,
        kinds: Vec<u8>,
    },
    /// Non-parametric coverage (Automatic flood) — no gizmo; rasterizes to its stored buffer verbatim.
    Raster { crisp: Arc<Vec<u8>> },
}

/// One entry in the selection list: a parametric shape plus the boolean operator that folds it into the
/// running mask (`0` New/replace · `1` Add/union · `2` Remove/subtract).
#[derive(Clone, Debug)]
pub(crate) struct SelectionEntry {
    pub shape: SelectionShape,
    pub op: u8,
}

impl SelectionShape {
    /// `true` for a parametric shape that carries a native editable gizmo (everything but `Raster`).
    #[must_use]
    pub fn is_editable(&self) -> bool {
        !matches!(self, SelectionShape::Raster { .. })
    }
}

impl PainterTool {
    /// Append a parametric entry to the selection list, honouring the boolean op: **New** clears the list
    /// first (it replaces the whole selection), **Add**/**Remove** push onto the existing shapes. Does NOT
    /// recomposite — the caller drives the mask (creation already wrote a live preview).
    pub(super) fn push_selection_entry(&mut self, shape: SelectionShape, op: u8) {
        if op == 0 {
            self.paint.selection_shapes.clear();
        }
        self.paint.selection_shapes.push(SelectionEntry { shape, op });
    }

    /// Rasterize one selection shape into a fresh canvas-sized coverage buffer (`0`/`255`). Ellipse uses the
    /// exact rotated inside-test (transcendental-free); Rect/Freehand reuse the scanline polygon fill; Raster
    /// returns its stored buffer.
    pub(super) fn rasterize_selection_shape(&self, shape: &SelectionShape) -> Vec<u8> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let mut cov = vec![0u8; w * h];
        if w == 0 || h == 0 {
            return cov;
        }
        match shape {
            SelectionShape::Ellipse { center, u, rx, ry } => {
                rasterize_ellipse(*center, *u, *rx, *ry, w, h, &mut cov);
                cov
            }
            SelectionShape::Rect { a, b } => self.raster_lasso(&rect_corners(*a, *b)),
            SelectionShape::Freehand {
                points,
                handles,
                kinds: _,
            } => {
                let spine = self.freehand_spine(points, handles);
                if spine.len() < 3 {
                    return cov;
                }
                self.raster_lasso(&spine)
            }
            SelectionShape::Raster { crisp } => {
                if crisp.len() == w * h {
                    (**crisp).clone()
                } else {
                    cov
                }
            }
        }
    }

    /// The flattened closed outline of a Freehand entry: the Bézier spine when explicit handles are present,
    /// else the raw anchor polygon. Reuses the Curve flatten so edits (dragged handles) rasterize faithfully.
    pub(super) fn freehand_spine(
        &self,
        points: &[[f32; 2]],
        handles: &[[[f32; 2]; 2]],
    ) -> Vec<[f32; 2]> {
        if handles.len() == points.len() && points.len() >= 2 {
            let mut spine = Vec::new();
            super::curve_geom::flatten_spine(points, handles, true, &mut spine);
            spine
        } else {
            points.to_vec()
        }
    }

    /// Re-derive `selection_crisp` (and thence the Feathered `selection_mask`) by rasterizing + compositing
    /// the whole shape list in order. The single funnel after any list mutation (create / edit gizmo /
    /// convert / offset) — keeps the mask a faithful cache of the parametric truth.
    pub(super) fn recompose_selection_mask(&mut self) {
        let n = (self.source_size.0 as usize) * (self.source_size.1 as usize);
        if n == 0 {
            return;
        }
        let entries = self.paint.selection_shapes.clone();
        let edit_idx = self.paint.selection_edit_idx;
        let mut crisp = vec![0u8; n];
        for (i, e) in entries.iter().enumerate() {
            // The entry under an open native gizmo rasterizes from the LIVE editor (its in-progress edits +
            // Offset), so the mask tracks the gizmo before it's baked back into the list; the rest use their
            // stored parametric shape.
            let region = if Some(i) == edit_idx {
                self.rasterize_active_editor()
                    .unwrap_or_else(|| self.rasterize_selection_shape(&e.shape))
            } else {
                self.rasterize_selection_shape(&e.shape)
            };
            if region.len() != n {
                continue;
            }
            combine_into(&mut crisp, &region, e.op);
        }
        self.set_selection_from_crisp(crisp);
    }
}

/// The four corners of the axis-aligned rectangle `a..b`, CCW, as a closed polygon for the scanline fill.
#[must_use]
pub(super) fn rect_corners(a: [f32; 2], b: [f32; 2]) -> Vec<[f32; 2]> {
    let (x0, x1) = (a[0].min(b[0]), a[0].max(b[0]));
    let (y0, y1) = (a[1].min(b[1]), a[1].max(b[1]));
    vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
}

/// Fill `cov` (0/255) inside the rotated ellipse (`center`, unit axis `u`, semi-axes `rx`/`ry`) via the
/// exact `(du/rx)² + (dv/ry)² ≤ 1` test — no transcendentals (HR-5-clean, though selection is view-side).
fn rasterize_ellipse(center: [f32; 2], u: [f32; 2], rx: f32, ry: f32, w: usize, h: usize, cov: &mut [u8]) {
    let rx = rx.max(0.5);
    let ry = ry.max(0.5);
    let v = [-u[1], u[0]]; // perpendicular axis
    for yy in 0..h {
        for xx in 0..w {
            let px = xx as f32 + 0.5 - center[0];
            let py = yy as f32 + 0.5 - center[1];
            let du = (px * u[0] + py * u[1]) / rx;
            let dv = (px * v[0] + py * v[1]) / ry;
            if du * du + dv * dv <= 1.0 {
                cov[yy * w + xx] = 255;
            }
        }
    }
}

/// Fold `region` into the accumulating `crisp` by the boolean op (`0` replace · `1` union · `2` subtract).
fn combine_into(crisp: &mut [u8], region: &[u8], op: u8) {
    let n = crisp.len().min(region.len());
    match op {
        1 => {
            for i in 0..n {
                crisp[i] = crisp[i].max(region[i]);
            }
        }
        2 => {
            for i in 0..n {
                crisp[i] = ((u16::from(crisp[i]) * u16::from(255 - region[i])) / 255) as u8;
            }
        }
        _ => crisp[..n].copy_from_slice(&region[..n]),
    }
}
