//! **Selection shape list** (ADR-0103 Amendment 2) — the parametric source of truth behind the selection
//! mask. Every creation gesture appends a [`SelectionEntry`] (a parametric shape + a boolean op); the
//! `selection_mask` is a DERIVED cache produced by [`PainterTool::recompose_selection_mask`] (rasterize
//! each entry, combine by its op, feather). Editing a native gizmo in Edit mode rewrites ONE entry and
//! recomposites, so multiple shapes stay independently editable while the mask always reflects the whole
//! list. `Raster` carries a non-parametric coverage buffer (Automatic flood) that has no gizmo.

use super::PainterTool;
use super::curve_model::CurveModel;
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
    /// N-gon inscribed in the (`rx`,`ry`) ellipse about `center`, oriented by unit axis `u`. The vertex
    /// phase is CORNER-seeded (45°), so `sides = 4` renders an axis-aligned **rectangle** (the freshly-drawn
    /// rect marquee); the side count is editable (ADR-0103 Am.2, Enio 2026-07-03). Native gizmo = the
    /// Polygon gizmo (resize / rotate / sides).
    Polygon {
        center: [f32; 2],
        u: [f32; 2],
        rx: f32,
        ry: f32,
        sides: u32,
    },
    /// Closed editable curve backed by the SHARED [`CurveModel`] (the exact same editing core the stroke
    /// Curve editor owns — Enio's mandate: one system). A **raw lasso** carries a model with EMPTY handles
    /// (`model.is_curve() == false` ⇒ the transform box); a **converted** curve carries full handles/kinds
    /// (⇒ the per-anchor point editor). `u` is the transform-gizmo orientation (so the box rotates WITH the
    /// selection). Produced by the Free-Hand lasso (raw) and Convert-to-Curve (fitted).
    Freehand { model: CurveModel, u: [f32; 2] },
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
    /// Clone the parametric selection shape list for a structural undo snapshot — the source of truth the
    /// mask is a derived cache of, so undo/redo restores the editable SHAPES (curve anchors + handles, box
    /// params) in lock-step with the mask (see [`crate::undo::ModelSnapshot::selection_shapes`]). Cheap: a
    /// handful of small entries.
    pub(crate) fn selection_shapes_snapshot(&self) -> Vec<SelectionEntry> {
        self.paint.selection_shapes.clone()
    }

    /// Reinstate the parametric selection shape list from a structural snapshot (undo/redo), so the editable
    /// shapes track the mask that `restore_selection` reinstates alongside them.
    pub(crate) fn restore_selection_shapes(&mut self, shapes: Vec<SelectionEntry>) {
        self.paint.selection_shapes = shapes;
    }

    /// Append a parametric entry to the selection list, honouring the boolean op: **New** clears the list
    /// first (it replaces the whole selection), **Add**/**Remove** push onto the existing shapes. Does NOT
    /// recomposite — the caller drives the mask (creation already wrote a live preview).
    pub(super) fn push_selection_entry(&mut self, shape: SelectionShape, op: u8) {
        if op == 0 {
            self.paint.selection_shapes.clear();
            self.reset_selection_offset(); // a New selection starts from a clean offset (no rings)
        }
        self.paint
            .selection_shapes
            .push(SelectionEntry { shape, op });
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
            SelectionShape::Polygon {
                center,
                u,
                rx,
                ry,
                sides,
            } => self.raster_lasso(&sel_polygon_vertices(*center, *u, *rx, *ry, *sides)),
            SelectionShape::Freehand { model, .. } => {
                let spine = self.freehand_spine(&model.points, &model.handles);
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
        // Ring-stack mode: the shapes are nested ring curves → BAND-PARITY composite (not boolean ops).
        if self.paint.selection_ring_stack {
            self.recompose_ring_stack(n);
            return;
        }
        // Each gizmo edits its `SelectionShape` params IN PLACE (isolated from the stroke editors), so the
        // mask is always a straight rasterize+composite of the stored list — no separate live-editor path.
        let entries = self.paint.selection_shapes.clone();
        let mut crisp = vec![0u8; n];
        for e in &entries {
            let region = self.rasterize_selection_shape(&e.shape);
            if region.len() != n {
                continue;
            }
            combine_into(&mut crisp, &region, e.op);
        }
        // The recomposed crisp is the pre-offset SOURCE; the Offset stage (grow/shrink + concentric rings)
        // produces the effective mask installed via `set_selection_from_crisp`. Neutral offset ⇒ identity.
        self.set_selection_offset_source(&crisp);
        self.apply_selection_offset();
    }
}

/// Per-side rotation step `(cos(2π/n), sin(2π/n))` for `n = 3..=12` (indexed by `n`), baked so the polygon
/// vertex generator stays transcendental-free (matches the brush's `POLY_STEP`).
const POLY_STEP: [[f32; 2]; 13] = [
    [1.0, 0.0],                 // 0 (unused)
    [1.0, 0.0],                 // 1 (unused)
    [1.0, 0.0],                 // 2 (unused)
    [-0.5, 0.866_025_4],        // 3  120°
    [0.0, 1.0],                 // 4  90°
    [0.309_017, 0.951_056_5],   // 5  72°
    [0.5, 0.866_025_4],         // 6  60°
    [0.623_489_8, 0.781_831_5], // 7
    [
        std::f32::consts::FRAC_1_SQRT_2,
        std::f32::consts::FRAC_1_SQRT_2,
    ], // 8  45°
    [0.766_044_4, 0.642_787_6], // 9
    [0.809_017, 0.587_785_25],  // 10 36°
    [0.841_253_5, 0.540_640_8], // 11
    [0.866_025_4, 0.5],         // 12 30°
];

/// Vertex-0 phase seed (45°, `cos = sin = √½`) — CORNER-seeded so `sides = 4` is an axis-aligned rectangle
/// (vertices at the box corners), not a diamond like the pointy-top brush polygon.
const CORNER_SEED: [f32; 2] = [
    std::f32::consts::FRAC_1_SQRT_2,
    std::f32::consts::FRAC_1_SQRT_2,
];

/// The vertices of a selection Polygon (`center`, unit axis `u`, semi-axes `rx`/`ry`, `sides`), CORNER-phase
/// seeded, as a closed polygon for the scanline fill. Transcendental-free (baked step table).
#[must_use]
pub(super) fn sel_polygon_vertices(
    center: [f32; 2],
    u: [f32; 2],
    rx: f32,
    ry: f32,
    sides: u32,
) -> Vec<[f32; 2]> {
    let n = sides.clamp(3, 12);
    let ul = {
        let m = (u[0] * u[0] + u[1] * u[1]).sqrt();
        if m > 1e-6 {
            [u[0] / m, u[1] / m]
        } else {
            [1.0, 0.0]
        }
    };
    let v = [-ul[1], ul[0]];
    let step = POLY_STEP[n as usize];
    let (mut cx, mut cy) = (CORNER_SEED[0], CORNER_SEED[1]);
    let mut out = Vec::with_capacity(n as usize);
    for _ in 0..n {
        out.push([
            center[0] + rx * cx * ul[0] + ry * cy * v[0],
            center[1] + rx * cx * ul[1] + ry * cy * v[1],
        ]);
        // Rotate (cx, cy) CCW by 2π/n via the baked step.
        let (ncx, ncy) = (cx * step[0] - cy * step[1], cx * step[1] + cy * step[0]);
        cx = ncx;
        cy = ncy;
    }
    out
}

/// Fill `cov` (0/255) inside the rotated ellipse (`center`, unit axis `u`, semi-axes `rx`/`ry`) via the
/// exact `(du/rx)² + (dv/ry)² ≤ 1` test — no transcendentals (HR-5-clean, though selection is view-side).
fn rasterize_ellipse(
    center: [f32; 2],
    u: [f32; 2],
    rx: f32,
    ry: f32,
    w: usize,
    h: usize,
    cov: &mut [u8],
) {
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
