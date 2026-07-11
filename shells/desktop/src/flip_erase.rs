//! ADR-0113 W2 T2.9 — the Flip eraser (clean-room of GP `erase.cc`, 3 modes).
//!
//! - **Soft** (default, most paint-like): reduces per-point opacity by
//!   `strength · falloff(dist)` within the brush radius; on pen-up, points that
//!   fell below the threshold are removed (splitting the stroke at the gap).
//! - **Hard**: cuts the stroke — removes points inside the circle and splits the
//!   survivors into separate strokes.
//! - **Stroke**: erases the whole stroke if any point is inside the circle.
//!
//! A LOCKED layer refuses all three (materials/strokes preserved). Erasing an
//! empty frame (no key yet) is a no-op — the eraser never creates a drawing.
//! HR-5: the falloff is a clamped linear ramp (no transcendentals).

use ph2d_core::Vec2;
use ph2d_flip::{FlipDrawing, FlipStroke, LayerId};
use ph2d_tool_flip::EraseMode;

/// Below this opacity a soft-erased point is dropped on pen-up (GP `erase.cc`).
const OPACITY_REMOVE_THRESHOLD: f32 = 0.05;

/// Resolve the active drawing for erasing (active layer → fallback top), unless
/// the layer is locked or the frame has no key. Returns `&mut FlipDrawing`.
fn active_drawing_mut<'a>(
    flip: &'a mut ph2d_flip::FlipDoc,
    playhead: &ph2d_core::Playhead,
    active_layer: Option<LayerId>,
) -> Option<&'a mut FlipDrawing> {
    let oid = flip.objects().first().map(|o| o.id)?;
    let obj = flip.object_mut(oid)?;
    let layer = active_layer
        .filter(|id| obj.layer(*id).is_some())
        .or_else(|| obj.layers().last().map(|l| l.id))?;
    if obj.layer(layer).is_some_and(|l| l.locked) {
        return None;
    }
    let frame = obj.frame_at(playhead);
    let did = obj.layer(layer).and_then(|l| l.drawing_at(frame))?;
    obj.drawing_mut(did)
}

/// A fresh stroke carrying `src`'s per-curve attributes (empty points).
fn new_like(src: &FlipStroke) -> FlipStroke {
    let mut s = FlipStroke::new();
    s.closed = src.closed;
    s.cap = src.cap;
    s.hardness = src.hardness;
    s.material = src.material;
    s.fill = src.fill;
    s
}

/// Whether any point of `s` lies within `radius` of `center`.
fn touches(s: &FlipStroke, center: Vec2, radius: f32) -> bool {
    let r2 = radius * radius;
    s.positions().iter().any(|p| {
        let d = *p - center;
        d.x * d.x + d.y * d.y <= r2
    })
}

/// Split `s` into runs of consecutive points that satisfy `keep`. Each run of
/// ≥2 points becomes a stroke (single-point runs are dropped — a dab, no line).
fn split_by<F: Fn(usize) -> bool>(s: &FlipStroke, keep: F) -> Vec<FlipStroke> {
    let mut out = Vec::new();
    let mut cur = new_like(s);
    for i in 0..s.len() {
        if keep(i) {
            if let Some(p) = s.point(i) {
                cur.push_point(p);
            }
        } else if cur.len() >= 2 {
            out.push(std::mem::replace(&mut cur, new_like(s)));
        } else {
            cur = new_like(s);
        }
    }
    if cur.len() >= 2 {
        out.push(cur);
    }
    out
}

/// Erase once at `center` (world) with `radius` (world) + `strength` (Soft only).
/// Returns `true` if the document changed.
pub(crate) fn erase_at(
    flip: &mut ph2d_flip::FlipDoc,
    playhead: &ph2d_core::Playhead,
    active_layer: Option<LayerId>,
    mode: EraseMode,
    center: Vec2,
    radius: f32,
    strength: f32,
) -> bool {
    let Some(dr) = active_drawing_mut(flip, playhead, active_layer) else {
        return false;
    };
    match mode {
        EraseMode::Stroke => {
            let before = dr.strokes.len();
            dr.strokes.retain(|s| !touches(s, center, radius));
            dr.strokes.len() != before
        }
        EraseMode::Soft => {
            let mut changed = false;
            let r = radius.max(f32::EPSILON);
            for s in dr.strokes.iter_mut() {
                let ps: Vec<Vec2> = s.positions().to_vec();
                let ops = s.opacities_mut();
                for (i, p) in ps.iter().enumerate() {
                    let d = ((*p - center).x.powi(2) + (*p - center).y.powi(2)).sqrt();
                    if d < radius {
                        let falloff = (1.0 - d / r).clamp(0.0, 1.0); // linear ramp (HR-5)
                        let reduced = (ops[i] - strength * falloff).max(0.0);
                        if reduced != ops[i] {
                            ops[i] = reduced;
                            changed = true;
                        }
                    }
                }
            }
            changed
        }
        EraseMode::Hard => {
            let r2 = radius * radius;
            let mut changed = false;
            let mut out: Vec<FlipStroke> = Vec::new();
            for s in std::mem::take(&mut dr.strokes) {
                let inside: Vec<bool> = s
                    .positions()
                    .iter()
                    .map(|p| {
                        let d = *p - center;
                        d.x * d.x + d.y * d.y <= r2
                    })
                    .collect();
                if inside.iter().any(|&b| b) {
                    changed = true;
                    out.extend(split_by(&s, |i| !inside[i]));
                } else {
                    out.push(s);
                }
            }
            dr.strokes = out;
            changed
        }
    }
}

/// Soft-erase pen-up cleanup: drop points below the opacity threshold, splitting
/// each stroke at the gaps. No-op for the other modes (caller gates by mode).
pub(crate) fn cleanup_soft(
    flip: &mut ph2d_flip::FlipDoc,
    playhead: &ph2d_core::Playhead,
    active_layer: Option<LayerId>,
) -> bool {
    let Some(dr) = active_drawing_mut(flip, playhead, active_layer) else {
        return false;
    };
    let mut changed = false;
    let mut out: Vec<FlipStroke> = Vec::new();
    for s in std::mem::take(&mut dr.strokes) {
        let low: Vec<bool> = s
            .opacities()
            .iter()
            .map(|o| *o < OPACITY_REMOVE_THRESHOLD)
            .collect();
        if low.iter().any(|&b| b) {
            changed = true;
            out.extend(split_by(&s, |i| !low[i]));
        } else {
            out.push(s);
        }
    }
    dr.strokes = out;
    changed
}

impl crate::App {
    /// The Flip tool wants the canvas for ERASING now (active + Erase mode). The
    /// `input_dispatch` reads the published style cache — no downcast.
    #[must_use]
    pub(crate) fn flip_wants_erase(&self) -> bool {
        self.flip_active
            && matches!(
                self.flip_style.map(|s| s.mode),
                Some(ph2d_tool_flip::FlipMode::Erase)
            )
    }

    /// Pen-down of the eraser: begin the gesture + erase at the cursor. Returns
    /// `true` if consumed (so the caller doesn't fall into the gizmo/pick).
    pub(crate) fn flip_erase_canvas_down(&mut self, x: f32, y: f32) -> bool {
        if !self.flip_wants_erase() {
            return false;
        }
        self.flip_erasing = true;
        self.flip_erase_apply(x, y);
        true
    }

    /// Move while erasing: erase at the cursor. `true` while a gesture is live.
    pub(crate) fn flip_erase_canvas_move(&mut self, x: f32, y: f32) -> bool {
        if !self.flip_erasing {
            return false;
        }
        self.flip_erase_apply(x, y);
        true
    }

    /// Pen-up: end the gesture + (Soft mode) drop the faded points.
    pub(crate) fn flip_erase_canvas_up(&mut self) -> bool {
        if !self.flip_erasing {
            return false;
        }
        self.flip_erasing = false;
        let soft = matches!(
            self.flip_style.map(|s| s.erase),
            Some(ph2d_tool_flip::EraseMode::Soft)
        );
        if soft {
            let active_layer = self.flip_active_layer;
            if let Some(gfx) = self.gfx.as_mut() {
                cleanup_soft(&mut gfx.flip, &self.playhead, active_layer);
            }
        }
        true
    }

    /// Erase once under the cursor (screen coords → world + radius from the brush).
    fn flip_erase_apply(&mut self, x: f32, y: f32) {
        let Some(style) = self.flip_style else {
            return;
        };
        let active_layer = self.flip_active_layer;
        if let Some(gfx) = self.gfx.as_mut() {
            let win = gfx.surface.size();
            let w = gfx.camera.screen_to_world((x, y), win);
            let px_to_world = gfx.camera.height_world.max(f32::EPSILON) / win.height.max(1) as f32;
            // Eraser radius = the brush size (px → world); strength = brush opacity.
            let radius = (style.width_px as f32 * 0.5) * px_to_world;
            erase_at(
                &mut gfx.flip,
                &self.playhead,
                active_layer,
                style.erase,
                Vec2::new(w[0], w[1]),
                radius,
                style.opacity,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_core::Playhead;
    use ph2d_flip::{FlipDoc, Hold, KeyKind, Point, Rgba};

    fn doc_with_line() -> (FlipDoc, LayerId) {
        let mut doc = FlipDoc::new();
        let oid = doc.push_object("O");
        let obj = doc.object_mut(oid).unwrap();
        let l = obj.add_layer("L");
        let d = obj
            .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        let mut s = FlipStroke::new();
        for x in 0..5 {
            s.push_point(Point {
                pos: Vec2::new(x as f32, 0.0),
                width: 0.1,
                opacity: 1.0,
                color: Rgba::WHITE,
            });
        }
        obj.drawing_mut(d).unwrap().strokes.push(s);
        (doc, l)
    }

    #[test]
    fn stroke_mode_removes_the_whole_touched_stroke() {
        let (mut doc, l) = doc_with_line();
        let hit = erase_at(
            &mut doc,
            &Playhead::default(),
            Some(l),
            EraseMode::Stroke,
            Vec2::new(2.0, 0.0),
            0.5,
            1.0,
        );
        assert!(hit);
        let obj = doc.objects().first().unwrap();
        let did = obj.layer(l).unwrap().drawing_at(0).unwrap();
        assert_eq!(obj.drawing(did).unwrap().strokes.len(), 0);
    }

    #[test]
    fn hard_mode_splits_the_stroke_at_the_gap() {
        let (mut doc, l) = doc_with_line();
        // Erase the middle point (x=2) → two runs [0,1] and [3,4].
        let hit = erase_at(
            &mut doc,
            &Playhead::default(),
            Some(l),
            EraseMode::Hard,
            Vec2::new(2.0, 0.0),
            0.5,
            1.0,
        );
        assert!(hit);
        let obj = doc.objects().first().unwrap();
        let did = obj.layer(l).unwrap().drawing_at(0).unwrap();
        assert_eq!(obj.drawing(did).unwrap().strokes.len(), 2, "split into two");
    }

    #[test]
    fn soft_mode_reduces_opacity_then_cleanup_removes_faded() {
        let (mut doc, l) = doc_with_line();
        // Full-strength soft erase over the whole line → all points to ~0.
        for _ in 0..2 {
            erase_at(
                &mut doc,
                &Playhead::default(),
                Some(l),
                EraseMode::Soft,
                Vec2::new(2.0, 0.0),
                10.0,
                1.0,
            );
        }
        let obj = doc.objects().first().unwrap();
        let did = obj.layer(l).unwrap().drawing_at(0).unwrap();
        let min_op = obj.drawing(did).unwrap().strokes[0]
            .opacities()
            .iter()
            .copied()
            .fold(f32::INFINITY, f32::min);
        assert!(
            min_op < OPACITY_REMOVE_THRESHOLD,
            "center faded below threshold"
        );
        assert!(cleanup_soft(&mut doc, &Playhead::default(), Some(l)));
    }

    #[test]
    fn locked_layer_refuses_erase() {
        let (mut doc, l) = doc_with_line();
        doc.objects().first().unwrap(); // sanity
        let oid = doc.objects().first().unwrap().id;
        doc.object_mut(oid).unwrap().layer_mut(l).unwrap().locked = true;
        let hit = erase_at(
            &mut doc,
            &Playhead::default(),
            Some(l),
            EraseMode::Stroke,
            Vec2::new(2.0, 0.0),
            0.5,
            1.0,
        );
        assert!(!hit, "locked layer preserved");
    }
}
