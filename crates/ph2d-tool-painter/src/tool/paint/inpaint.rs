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
use ph2d_editor_core::tool::PanelEvent;
use ph2d_inpaint::{InpaintParams, InpaintRequest, inpaint_cpu};
use ph2d_painter_brush::Dab;

/// Live-feedback tint for a marked defect pixel (blended 50 % into the canvas). Cosmetic only — the heal
/// overwrites every marked pixel, so the colour is irrelevant to the result.
const TINT: [f32; 3] = [235.0, 60.0, 60.0];

impl PainterTool {
    /// Route the Inpaint card's three reconstruction sliders (`SetValue` on the `0..1` track). Returns
    /// `true` when handled — mirrors the other `route_*_event` early-dispatch helpers in
    /// [`crate::tool::PainterTool::handle_panel_event`]. The setters below store the raw track (mapped to
    /// engine values in [`Self::heal_inpaint`]).
    pub(crate) fn route_inpaint_event(&mut self, event: &PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        let PanelEvent::SetValue(id, v) = event else {
            return false;
        };
        let (id, v) = (*id, *v as f32);
        if id == core_ids::PAINTER_INPAINT_PATCH_SLIDER {
            self.set_inpaint_patch(v);
        } else if id == core_ids::PAINTER_INPAINT_QUALITY_SLIDER {
            self.set_inpaint_quality(v);
        } else if id == core_ids::PAINTER_INPAINT_SEARCH_SLIDER {
            self.set_inpaint_search(v);
        } else {
            return false;
        }
        true
    }

    /// Set the Inpaint **Patch Size** slider track (`0..1` → patch radius `2..=6`).
    pub fn set_inpaint_patch(&mut self, t: f32) {
        self.paint.inpaint_patch_norm = t.clamp(0.0, 1.0);
    }

    /// Set the Inpaint **Quality** slider track (`0..1` → EM iterations `3..=12`).
    pub fn set_inpaint_quality(&mut self, t: f32) {
        self.paint.inpaint_quality_norm = t.clamp(0.0, 1.0);
    }

    /// Set the Inpaint **Search** slider track (`0..1` → context-margin multiplier `0.5..3.0`).
    pub fn set_inpaint_search(&mut self, t: f32) {
        self.paint.inpaint_search_norm = t.clamp(0.0, 1.0);
    }

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
            let buf = crate::tool::paint::plane_fork::fork_canvas(
                &mut self.canvas_rgba,
                &self.undo.write_state,
                self.source_size.0,
            );
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
        // Reconstruction knobs from the Inpaint panel (`0..1` tracks → engine values). Patch Size + Quality
        // feed `InpaintParams`; Search scales the crop margin (how much context PatchMatch samples).
        let patch_radius = (2.0 + self.paint.inpaint_patch_norm * 4.0)
            .round()
            .clamp(2.0, 6.0) as u32;
        let em_iters = (3.0 + self.paint.inpaint_quality_norm * 9.0)
            .round()
            .clamp(3.0, 12.0) as u32;
        let mult = 0.5 + self.paint.inpaint_search_norm * 2.5;
        // Crop to the defect + a margin so PatchMatch has enough source context WITHOUT paying for the
        // whole layer (interactive on a big canvas). Margin scales with the hole × the Search multiplier.
        let hole = (maxx - minx + 1).max(maxy - miny + 1);
        let margin = ((hole / 2) as f32 * mult).round().clamp(24.0, 400.0) as usize;
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
        let result = run_inpaint(
            cw,
            ch,
            &InpaintRequest {
                width: cw as u32,
                height: ch as u32,
                rgba: &crop_rgba,
                mask: &crop_mask,
                params: InpaintParams {
                    patch_radius,
                    em_iters,
                    ..InpaintParams::default()
                },
            },
        );
        // Write only the reconstructed (marked) pixels back into the layer.
        {
            let dst = crate::tool::paint::plane_fork::fork_canvas(
                &mut self.canvas_rgba,
                &self.undo.write_state,
                self.source_size.0,
            );
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

/// Reconstruct the cropped `req` — GPU when the `gpu` feature is on, an adapter is available, and the
/// crop is large enough to amortise the upload/readback (below the threshold the CPU wins), otherwise
/// the CPU reference. This is the ADR-0102 "GPU with CPU fallback" contract at the call site.
#[cfg(feature = "gpu")]
fn run_inpaint(cw: usize, ch: usize, req: &InpaintRequest<'_>) -> ph2d_inpaint::InpaintResult {
    // GPU pays for its device round-trip only above ~128² pixels; small heals stay on the CPU.
    const GPU_MIN_PIXELS: usize = 128 * 128;
    if cw * ch >= GPU_MIN_PIXELS
        && let Some(gpu) = try_gpu()
    {
        ph2d_inpaint::inpaint(Some(gpu), req)
    } else {
        inpaint_cpu(req)
    }
}

/// CPU-only build: the heal always uses the reference path.
#[cfg(not(feature = "gpu"))]
fn run_inpaint(_cw: usize, _ch: usize, req: &InpaintRequest<'_>) -> ph2d_inpaint::InpaintResult {
    inpaint_cpu(req)
}

/// A process-wide headless [`GpuContext`](ph2d_gpu::GpuContext), created lazily on the first heal that
/// wants it (`None` when no adapter — the heal then falls back to CPU). Mirror of
/// `ph2d-tool-color-equalization`'s `try_headless_gpu`.
#[cfg(feature = "gpu")]
fn try_gpu() -> Option<&'static ph2d_gpu::GpuContext> {
    use std::sync::OnceLock;
    static GPU: OnceLock<Option<ph2d_gpu::GpuContext>> = OnceLock::new();
    GPU.get_or_init(|| {
        ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()
    })
    .as_ref()
}

/// Whether the GPU heal path can run on this machine (an adapter was acquired). Used by the parity test
/// to confirm the heal is exercising the GPU, not silently falling back to CPU.
#[cfg(all(test, feature = "gpu"))]
pub(crate) fn gpu_heal_available() -> bool {
    try_gpu().is_some()
}
