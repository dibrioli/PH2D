//! Selection **creation input** (ADR-0103) — the mode / boolean-op / threshold setters and the on-canvas
//! pointer gesture (rectangle / ellipse marquee, freehand lasso with brush stabilization, Automatic flood).
//! On pen-up each gesture (a) combines its region into the mask and (b) appends a parametric
//! [`super::selection_shapes::SelectionShape`] to the shape list, so Edit mode can reinstate the native
//! gizmo. Split from `selection` for the LOC cap.

use super::PainterTool;
use super::selection_shapes::SelectionShape;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};
use std::sync::Arc;

/// The in-progress selection gesture — the overlay draws its rubber-band; the mask is rasterized on Up
/// (Automatic writes it live). Held in [`super::PaintState`].
#[derive(Clone, Debug)]
pub(crate) enum SelectionDrag {
    /// Rectangle / Ellipse marquee: a rubber-band from `anchor` to the current point. `ellipse` picks the
    /// shape.
    Marquee {
        anchor: [f32; 2],
        cur: [f32; 2],
        ellipse: bool,
    },
    /// Freehand lasso: the captured (brush-stabilized) path + the running stabilizer point.
    Lasso {
        points: Vec<[f32; 2]>,
        stab: [f32; 2],
    },
    /// Automatic flood-select: the seed + the threshold at gesture start (horizontal drag adjusts it live,
    /// Procreate-style). The mask is written on every change.
    Auto { seed: [f32; 2], thresh0: f32 },
}

impl PainterTool {
    /// Set the Selection sub-mode (`0` Automatic · `1` Freehand · `2` Rectangle · `3` Ellipse).
    pub fn set_selection_mode(&mut self, m: u8) {
        self.paint.selection_mode = m.min(3);
    }
    /// The active Selection sub-mode discriminant.
    #[must_use]
    pub fn selection_mode(&self) -> u8 {
        self.paint.selection_mode
    }
    /// Set the boolean operator for the next gesture (`0` New · `1` Add · `2` Remove).
    pub fn set_selection_bool_op(&mut self, op: u8) {
        self.paint.selection_bool_op = op.min(2);
    }
    /// The active boolean operator discriminant.
    #[must_use]
    pub fn selection_bool_op(&self) -> u8 {
        self.paint.selection_bool_op
    }
    /// Set the Automatic-mode threshold (`0..1`); re-floods live if an Automatic gesture is open.
    pub fn set_selection_threshold(&mut self, t: f32) {
        self.paint.selection_threshold = t.clamp(0.0, 1.0);
        let auto_seed = match &self.paint.selection_drag {
            Some(SelectionDrag::Auto { seed, .. }) => Some(*seed),
            _ => None,
        };
        if let Some(seed) = auto_seed {
            self.auto_reflood(seed);
        }
    }
    /// The Automatic threshold (`0..1`).
    #[must_use]
    pub fn selection_threshold(&self) -> f32 {
        self.paint.selection_threshold
    }

    /// **Invert** the whole selection (`255 − coverage`), sizing the mask to the canvas first. One undo entry.
    /// The inverse is non-parametric, so the shape list collapses to a single `Raster` entry.
    pub fn invert_selection(&mut self) {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        let n = (w as usize) * (h as usize);
        let before = self.snapshot_model();
        let mut crisp = if self.paint.selection_crisp.len() == n {
            (*self.paint.selection_crisp).clone()
        } else {
            vec![0u8; n]
        };
        for v in crisp.iter_mut() {
            *v = 255 - *v;
        }
        self.paint.selection_shapes = vec![super::selection_shapes::SelectionEntry {
            shape: SelectionShape::Raster {
                crisp: Arc::new(crisp.clone()),
            },
            op: 0,
        }];
        self.set_selection_from_crisp(crisp);
        self.commit_structural_edit(before);
    }

    /// Route a Selection-mode canvas pointer to the active sub-mode (called from `on_canvas_pointer`).
    pub(super) fn selection_pointer(&mut self, ev: CanvasPointer) -> bool {
        match ev.phase {
            PointerPhase::Down => self.selection_down(ev.pos),
            PointerPhase::Move => self.selection_move(ev.pos),
            PointerPhase::Up => self.selection_up(ev.pos),
            PointerPhase::Hover => false,
        }
    }

    /// Begin a selection gesture: snapshot for undo, capture the base selection this gesture combines
    /// against, and start the sub-mode's drag. Automatic floods immediately.
    fn selection_down(&mut self, pos: [f32; 2]) -> bool {
        self.paint.stroke_undo = Some(self.snapshot_model());
        self.ensure_selection_mask();
        // Add/Remove combine against the CRISP base (not the Feathered mask), so feathered edges never
        // re-seed the accumulator.
        self.paint.selection_base = Arc::clone(&self.paint.selection_crisp);
        match self.paint.selection_mode {
            0 => {
                let thresh0 = self.paint.selection_threshold;
                self.paint.selection_drag = Some(SelectionDrag::Auto { seed: pos, thresh0 });
                self.auto_reflood(pos);
            }
            1 => {
                self.paint.selection_drag = Some(SelectionDrag::Lasso {
                    points: vec![pos],
                    stab: pos,
                });
            }
            3 => {
                self.paint.selection_drag = Some(SelectionDrag::Marquee {
                    anchor: pos,
                    cur: pos,
                    ellipse: true,
                });
            }
            _ => {
                self.paint.selection_drag = Some(SelectionDrag::Marquee {
                    anchor: pos,
                    cur: pos,
                    ellipse: false,
                });
            }
        }
        true
    }

    /// Extend the gesture: marquee tracks the corner, lasso appends a (stabilized) point, Automatic drags the
    /// threshold (horizontal delta from the seed, Procreate-style). Live preview each Move; committed on Up.
    fn selection_move(&mut self, pos: [f32; 2]) -> bool {
        // Extract the Copy data (and drop the borrow) before touching other `self` fields / methods.
        enum Move {
            Marquee([f32; 2], bool),
            Lasso,
            Auto([f32; 2], f32),
        }
        let kind = match &self.paint.selection_drag {
            Some(SelectionDrag::Marquee {
                anchor, ellipse, ..
            }) => Move::Marquee(*anchor, *ellipse),
            Some(SelectionDrag::Lasso { .. }) => Move::Lasso,
            Some(SelectionDrag::Auto { seed, thresh0 }) => Move::Auto(*seed, *thresh0),
            None => return false,
        };
        match kind {
            Move::Marquee(anchor, ellipse) => {
                if let Some(SelectionDrag::Marquee { cur, .. }) = &mut self.paint.selection_drag {
                    *cur = pos;
                }
                let region = self.raster_marquee(anchor, pos, ellipse);
                self.apply_selection_region(&region);
                self.invalidate_composite();
            }
            Move::Lasso => {
                // Fold the raw sample through the brush **stabilizer** (same `lazy_mouse_step` the FreeHand
                // stroke uses) before capturing it, so the lasso is smoothed like a Free Hand stroke.
                let stabilizer = self.paint.brush.stabilizer;
                if let Some(SelectionDrag::Lasso { points, stab }) = &mut self.paint.selection_drag
                {
                    *stab = ph2d_painter_brush::lazy_mouse_step(*stab, pos, stabilizer);
                    points.push(*stab);
                }
                let pts = match &self.paint.selection_drag {
                    Some(SelectionDrag::Lasso { points, .. }) => points.clone(),
                    _ => Vec::new(),
                };
                let region = self.raster_lasso(&pts);
                self.apply_selection_region(&region);
                self.invalidate_composite();
            }
            Move::Auto(seed, thresh0) => {
                let w = self.source_size.0;
                if w > 0 {
                    let delta = (pos[0] - seed[0]) / w as f32;
                    self.paint.selection_threshold = (thresh0 + delta).clamp(0.0, 1.0);
                }
                self.auto_reflood(seed);
            }
        }
        true
    }

    /// Finish the gesture: rasterize the region, combine into the mask, append the parametric shape to the
    /// list, and commit ONE structural undo entry so the selection joins the single interleaved queue.
    fn selection_up(&mut self, pos: [f32; 2]) -> bool {
        let op = self.paint.selection_bool_op;
        let (region, shape) = match self.paint.selection_drag.take() {
            Some(SelectionDrag::Marquee {
                anchor, ellipse, ..
            }) => {
                let region = self.raster_marquee(anchor, pos, ellipse);
                let shape = if ellipse {
                    let center = [(anchor[0] + pos[0]) * 0.5, (anchor[1] + pos[1]) * 0.5];
                    SelectionShape::Ellipse {
                        center,
                        u: [1.0, 0.0],
                        rx: ((pos[0] - anchor[0]).abs() * 0.5).max(0.5),
                        ry: ((pos[1] - anchor[1]).abs() * 0.5).max(0.5),
                    }
                } else {
                    // Rect → a corner-phase 4-gon (Polygon gizmo, editable side count); rx/ry = half-extent
                    // · √2 so the 4 vertices land on the drawn box corners.
                    let center = [(anchor[0] + pos[0]) * 0.5, (anchor[1] + pos[1]) * 0.5];
                    SelectionShape::Polygon {
                        center,
                        u: [1.0, 0.0],
                        rx: ((pos[0] - anchor[0]).abs() * 0.5 * std::f32::consts::SQRT_2).max(0.5),
                        ry: ((pos[1] - anchor[1]).abs() * 0.5 * std::f32::consts::SQRT_2).max(0.5),
                        sides: 4,
                    }
                };
                (region, Some(shape))
            }
            Some(SelectionDrag::Lasso { mut points, .. }) => {
                points.push(pos);
                let region = self.raster_lasso(&points);
                let shape = SelectionShape::Freehand {
                    points,
                    handles: Vec::new(),
                    u: [1.0, 0.0],
                };
                (region, Some(shape))
            }
            Some(SelectionDrag::Auto { seed, .. }) => {
                let region = self.raster_flood(seed);
                // Automatic has no parametric form — freeze its coverage as a Raster entry.
                let shape = SelectionShape::Raster {
                    crisp: Arc::new(region.clone()),
                };
                (region, Some(shape))
            }
            None => return false,
        };
        self.apply_selection_region(&region);
        if let Some(shape) = shape {
            self.push_selection_entry(shape, op);
        }
        if let Some(before) = self.paint.stroke_undo.take() {
            self.commit_structural_edit(before);
        }
        self.paint.selection_base = Arc::new(Vec::new());
        self.invalidate_composite();
        true
    }

    /// Re-flood the Automatic selection from `seed` at the current threshold, writing the combined mask live.
    fn auto_reflood(&mut self, seed: [f32; 2]) {
        let region = self.raster_flood(seed);
        self.apply_selection_region(&region);
        self.invalidate_composite();
    }
}
