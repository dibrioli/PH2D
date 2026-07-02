//! The **Inpaint** heal brush (ADR-0102): paint over a defect to mark it, and on pen-up reconstruct the
//! marked region from the surrounding pixels with the `ph2d-inpaint` engine (multi-scale PatchMatch).
//!
//! Two phases:
//! * [`PainterTool::stamp_dabs_inpaint`] — per dab, mark a hard disc into `inpaint_mask` and tint the
//!   canvas red for live feedback. No colour/blend is applied (the marked pixels are the hole, which the
//!   heal overwrites).
//! * [`PainterTool::heal_inpaint`] — on pen-up, crop to the mask's bounding box + a margin (so a big
//!   layer stays interactive — PatchMatch runs on the defect neighbourhood, not the whole canvas), run
//!   the engine, write the reconstructed hole pixels back, and clear the mask. Runs BEFORE `close_stroke`
//!   so the structural-undo entry captures pre-stroke → healed as one step.

use super::Region;
use crate::tool::PainterTool;
use ph2d_inpaint::{InpaintParams, InpaintRequest, inpaint_cpu};
use ph2d_painter_brush::Dab;
use std::sync::Arc;

/// Live-feedback tint for a marked defect pixel (blended 50 % into the canvas). Cosmetic only — the heal
/// overwrites every marked pixel, so the colour is irrelevant to the result.
const TINT: [f32; 3] = [235.0, 60.0, 60.0];

impl PainterTool {
    /// Mark each dab's hard disc into the heal mask + tint the canvas under it.
    pub(super) fn stamp_dabs_inpaint(&mut self, dabs: &[Dab]) {
        let (w, h) = self.source_size;
        let (w, h) = (w as usize, h as usize);
        if w == 0 || h == 0 || dabs.is_empty() {
            return;
        }
        if self.paint.inpaint_mask.len() != w * h {
            self.paint.inpaint_mask = vec![0u8; w * h];
        }
        let mut touched: Option<Region> = None;
        {
            let mask = &mut self.paint.inpaint_mask;
            let buf = Arc::make_mut(&mut self.canvas_rgba);
            if buf.len() != w * h * 4 {
                return;
            }
            for d in dabs {
                let (cx, cy) = (d.center[0], d.center[1]);
                let r = d.radius_px.max(0.5);
                let r2 = r * r;
                let x0 = (cx - r).floor().max(0.0) as usize;
                let x1 = ((cx + r).ceil() as usize).min(w - 1);
                let y0 = (cy - r).floor().max(0.0) as usize;
                let y1 = ((cy + r).ceil() as usize).min(h - 1);
                if x0 > x1 || y0 > y1 {
                    continue;
                }
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        let dx = x as f32 + 0.5 - cx;
                        let dy = y as f32 + 0.5 - cy;
                        if dx * dx + dy * dy > r2 {
                            continue;
                        }
                        let idx = y * w + x;
                        mask[idx] = 255;
                        let o = idx * 4;
                        for k in 0..3 {
                            buf[o + k] = (f32::from(buf[o + k]) * 0.5 + TINT[k] * 0.5) as u8;
                        }
                    }
                }
                let rect = Region {
                    x: x0 as u32,
                    y: y0 as u32,
                    w: (x1 - x0 + 1) as u32,
                    h: (y1 - y0 + 1) as u32,
                };
                touched = Some(touched.map_or(rect, |acc| super::union_region(acc, rect)));
            }
        }
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }

    /// Reconstruct the marked region and clear the heal mask. No-op when nothing is marked.
    pub(super) fn heal_inpaint(&mut self) {
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        if fw == 0 || fh == 0 || self.paint.inpaint_mask.len() != fw * fh {
            self.paint.inpaint_mask.clear();
            return;
        }
        // Bounding box of the marked pixels.
        let (mut minx, mut miny, mut maxx, mut maxy) = (fw, fh, 0usize, 0usize);
        let mut any = false;
        for y in 0..fh {
            for x in 0..fw {
                if self.paint.inpaint_mask[y * fw + x] >= 128 {
                    any = true;
                    minx = minx.min(x);
                    maxx = maxx.max(x);
                    miny = miny.min(y);
                    maxy = maxy.max(y);
                }
            }
        }
        if !any {
            self.paint.inpaint_mask.iter_mut().for_each(|m| *m = 0);
            return;
        }
        // Crop to the defect + a margin so PatchMatch has enough source context WITHOUT paying for the
        // whole layer (interactive on a big canvas). Margin scales with the hole, clamped.
        let hole = (maxx - minx + 1).max(maxy - miny + 1);
        let margin = (hole / 2).clamp(24, 128);
        let x0 = minx.saturating_sub(margin);
        let y0 = miny.saturating_sub(margin);
        let x1 = (maxx + margin + 1).min(fw);
        let y1 = (maxy + margin + 1).min(fh);
        let (cw, ch) = (x1 - x0, y1 - y0);

        if self.canvas_rgba.len() != fw * fh * 4 {
            self.paint.inpaint_mask.iter_mut().for_each(|m| *m = 0);
            return;
        }
        // Crop the canvas RGBA (straight) + the mask into the working rectangle.
        let mut crop_rgba = vec![0u8; cw * ch * 4];
        let mut crop_mask = vec![0u8; cw * ch];
        {
            let src = &**self.canvas_rgba;
            for cy in 0..ch {
                for cx in 0..cw {
                    let (sx, sy) = (x0 + cx, y0 + cy);
                    let so = (sy * fw + sx) * 4;
                    let co = (cy * cw + cx) * 4;
                    crop_rgba[co..co + 4].copy_from_slice(&src[so..so + 4]);
                    crop_mask[cy * cw + cx] = self.paint.inpaint_mask[sy * fw + sx];
                }
            }
        }
        let result = inpaint_cpu(&InpaintRequest {
            width: cw as u32,
            height: ch as u32,
            rgba: &crop_rgba,
            mask: &crop_mask,
            params: InpaintParams::default(),
        });
        // Write only the reconstructed (marked) pixels back into the layer.
        {
            let dst = Arc::make_mut(&mut self.canvas_rgba);
            for cy in 0..ch {
                for cx in 0..cw {
                    if crop_mask[cy * cw + cx] < 128 {
                        continue;
                    }
                    let (sx, sy) = (x0 + cx, y0 + cy);
                    let so = (sy * fw + sx) * 4;
                    let co = (cy * cw + cx) * 4;
                    dst[so..so + 4].copy_from_slice(&result.rgba[co..co + 4]);
                }
            }
        }
        self.mark_dirty(Region {
            x: x0 as u32,
            y: y0 as u32,
            w: cw as u32,
            h: ch as u32,
        });
        self.paint.inpaint_mask.iter_mut().for_each(|m| *m = 0);
    }
}
