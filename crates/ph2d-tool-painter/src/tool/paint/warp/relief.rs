//! **W4 — the advective family.** The warp carries the impasto planes (`heights` + `covers` + `mats`)
//! along the SAME displacement as the pixels: one sampler, four channels, five brushes at once
//! (Push · Twist · Pinch · Wrinkle · Fold — and Reconstruct un-warps the body with the colour).
//!
//! Doc 18 §5's exception, arrived in full: *"empurrar tinta move a matéria, e matéria carrega cor"* — and
//! body. A Grab that slid the colour out from under its own thickness would leave the light shading a
//! ridge of paint that is no longer there.
//!
//! ## One door
//!
//! Every warp render calls [`PainterTool::warp_render_relief`] with the bbox it just re-rendered — the dab
//! kernel and the Reconstruct resampler alike. The relief read is `bilinear(pre_plane, dst − disp[dst])`,
//! the exact inverse-warp the pixels take (same `disp`, same window, same clamp), so the body and the
//! colour cannot disagree about where anything went. The planes were frozen TOGETHER at session start
//! (`ensure_deform_session`), so mid-session toggle flips re-render coherently from one baseline.

use super::super::Region;
use super::super::impasto_ceiling::H_MAX;
use super::super::region::grow_region;
use crate::tool::PainterTool;
use ph2d_painter_brush::material::MaterialBytes;
use std::sync::Arc;

/// Bilinear-sample an `f32` plane at fractional `(x, y)`, clamping to the edge — the heights' twin of
/// `apply::bilinear_clamped` (no premultiply: a height is not a colour).
#[inline]
fn bilinear_f32(field: &[f32], w: u32, h: u32, x: f32, y: f32) -> f32 {
    let (x0, y0, x1, y1, fx, fy) = corners(w, h, x, y);
    let stride = w as usize;
    let at = |xi: usize, yi: usize| field[yi * stride + xi];
    let top = at(x0, y0) + (at(x1, y0) - at(x0, y0)) * fx;
    let bot = at(x0, y1) + (at(x1, y1) - at(x0, y1)) * fx;
    top + (bot - top) * fy
}

/// Bilinear-sample a `u8` plane (coverage) at fractional `(x, y)`, edge-clamped, rounded to nearest.
#[inline]
fn bilinear_u8(field: &[u8], w: u32, h: u32, x: f32, y: f32) -> u8 {
    let (x0, y0, x1, y1, fx, fy) = corners(w, h, x, y);
    let stride = w as usize;
    let at = |xi: usize, yi: usize| f32::from(field[yi * stride + xi]);
    let top = at(x0, y0) + (at(x1, y0) - at(x0, y0)) * fx;
    let bot = at(x0, y1) + (at(x1, y1) - at(x0, y1)) * fx;
    (top + (bot - top) * fy).round().clamp(0.0, 255.0) as u8
}

/// Bilinear-sample the 7-byte material plane, per channel — the material blends across a warp the same
/// way the colour does (a stretched fosco↔brilhante boundary smears, exactly like its pixels).
#[inline]
fn bilinear_mat(field: &[MaterialBytes], w: u32, h: u32, x: f32, y: f32) -> MaterialBytes {
    let (x0, y0, x1, y1, fx, fy) = corners(w, h, x, y);
    let stride = w as usize;
    let mut out = [0u8; 7];
    for (c, o) in out.iter_mut().enumerate() {
        let at = |xi: usize, yi: usize| f32::from(field[yi * stride + xi][c]);
        let top = at(x0, y0) + (at(x1, y0) - at(x0, y0)) * fx;
        let bot = at(x0, y1) + (at(x1, y1) - at(x0, y1)) * fx;
        *o = (top + (bot - top) * fy).round().clamp(0.0, 255.0) as u8;
    }
    out
}

/// The shared corner/fraction resolution — one clamp policy for all three samplers (and it is the SAME
/// policy as the pixels': extend, never wrap).
#[inline]
#[allow(clippy::type_complexity)]
fn corners(w: u32, h: u32, x: f32, y: f32) -> (usize, usize, usize, usize, f32, f32) {
    let x0f = x.floor();
    let y0f = y.floor();
    let (wi, hi) = (w as i64, h as i64);
    let x0 = (x0f as i64).clamp(0, wi - 1);
    let y0 = (y0f as i64).clamp(0, hi - 1);
    let x1 = (x0 + 1).clamp(0, wi - 1);
    let y1 = (y0 + 1).clamp(0, hi - 1);
    (
        x0 as usize,
        y0 as usize,
        x1 as usize,
        y1 as usize,
        x - x0f,
        y - y0f,
    )
}

impl PainterTool {
    /// Re-render the impasto planes over `bbox` from the session's frozen baselines through the TOTAL
    /// displacement — the body's edition of the pixels' single-resample render. No-op when the toggle is
    /// off, when the session froze no relief, or when the frozen planes describe a layer that is no
    /// longer active (bits are recycled; the wrong layer must not wear this body).
    pub(super) fn warp_render_relief(&mut self, bbox: Region) {
        if !self.paint.warp.affect_relief {
            return;
        }
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let Some(layer) = self.paint.warp.relief_layer else {
            return;
        };
        if self.layers.active() != Some(layer) {
            return;
        }
        let pre_h = Arc::clone(&self.paint.warp.pre_h);
        let disp = Arc::clone(&self.paint.warp.disp);
        // How far the body rides along the pixels' displacement (see `WarpSession::relief_disp_scale`).
        // `1.0` — the default, and Deform always — makes the two sample the identical coordinate.
        let ds = self.paint.warp.relief_disp_scale;
        if ds <= 0.0 {
            return;
        }
        if pre_h.len() != n || disp.len() != n {
            return;
        }
        if let Some(entry) = self.heights.get_mut(&layer)
            && entry.len() == n
        {
            let dst = Arc::make_mut(entry);
            for ry in 0..bbox.h {
                let dy = bbox.y + ry;
                for rx in 0..bbox.w {
                    let dx = bbox.x + rx;
                    let gi = (dy * w + dx) as usize;
                    let d = [disp[gi][0] * ds, disp[gi][1] * ds];
                    let v = bilinear_f32(&pre_h, w, h, dx as f32 - d[0], dy as f32 - d[1]);
                    dst[gi] = v.clamp(-H_MAX, H_MAX);
                }
            }
        }
        let pre_cover = Arc::clone(&self.paint.warp.pre_cover);
        if pre_cover.len() == n
            && let Some(entry) = self.covers.get_mut(&layer)
            && entry.len() == n
        {
            let dst = Arc::make_mut(entry);
            for ry in 0..bbox.h {
                let dy = bbox.y + ry;
                for rx in 0..bbox.w {
                    let dx = bbox.x + rx;
                    let gi = (dy * w + dx) as usize;
                    let d = [disp[gi][0] * ds, disp[gi][1] * ds];
                    dst[gi] = bilinear_u8(&pre_cover, w, h, dx as f32 - d[0], dy as f32 - d[1]);
                }
            }
        }
        let pre_mats = Arc::clone(&self.paint.warp.pre_mats);
        if pre_mats.len() == n
            && let Some(entry) = self.mats.get_mut(&layer)
            && entry.len() == n
        {
            let dst = Arc::make_mut(entry);
            for ry in 0..bbox.h {
                let dy = bbox.y + ry;
                for rx in 0..bbox.w {
                    let dx = bbox.x + rx;
                    let gi = (dy * w + dx) as usize;
                    let d = [disp[gi][0] * ds, disp[gi][1] * ds];
                    dst[gi] = bilinear_mat(&pre_mats, w, h, dx as f32 - d[0], dy as f32 - d[1]);
                }
            }
        }
        // Grow by one: the light reads a texel's NEIGHBOURS (the normal is a central difference), so a
        // texel just outside the changed box is lit by a slope that changed inside it. The pixel path's
        // own `mark_dirty(bbox)` unions with this.
        if let Some(g) = grow_region(bbox, 1, w, h) {
            self.mark_dirty(g);
        }
    }

    /// Put the impasto planes back to the session's frozen baselines — the whole canvas of them (Reset is
    /// a wholesale restore, mirroring `canvas_rgba = pre`). Guarded like the render: the planes must
    /// describe the still-active layer.
    pub(super) fn deform_restore_relief_planes(&mut self) {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let Some(layer) = self.paint.warp.relief_layer else {
            return;
        };
        if self.layers.active() != Some(layer) {
            return;
        }
        let pre_h = Arc::clone(&self.paint.warp.pre_h);
        if pre_h.len() == n {
            self.heights.insert(layer, pre_h);
        }
        let pre_cover = Arc::clone(&self.paint.warp.pre_cover);
        if pre_cover.len() == n {
            self.covers.insert(layer, pre_cover);
        }
        let pre_mats = Arc::clone(&self.paint.warp.pre_mats);
        if pre_mats.len() == n {
            self.mats.insert(layer, pre_mats);
        }
        self.sync_relief_flags();
    }
}
