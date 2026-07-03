//! Selection **actions** (ADR-0103 Wave 5) — the panel's action buttons that operate on the active
//! selection: **Select layer contents** (mask ← the layer's opaque texels), **Color Fill** (flood the
//! selected region with the brush colour), and **Copy / Paste** (an in-memory clipboard of the selected
//! pixels). Every mutation that changes pixels or the mask records ONE structural undo entry, so it joins
//! the single interleaved queue. Split from `selection` for the LOC cap.

use super::selection_shapes::{SelectionEntry, SelectionShape};
use super::{PainterTool, Region};
use std::sync::Arc;

/// An in-memory clip of copied selection pixels: the source bounding box + its straight-RGBA texels
/// (already coverage-premultiplied against the selection, so Paste composites cleanly).
#[derive(Clone, Debug)]
pub(crate) struct SelectionClip {
    pub rect: Region,
    pub rgba: Vec<u8>,
}

impl PainterTool {
    /// **Select layer contents**: replace the selection with the active layer's opaque region (alpha > 0).
    /// Collapses the shape list to one `Raster` entry (the alpha silhouette is non-parametric). One undo entry.
    pub fn selection_from_layer_contents(&mut self) {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if w == 0 || h == 0 || self.canvas_rgba.len() != w * h * 4 {
            return;
        }
        let before = self.snapshot_model();
        let mut crisp = vec![0u8; w * h];
        for i in 0..w * h {
            if self.canvas_rgba[i * 4 + 3] > 0 {
                crisp[i] = 255;
            }
        }
        self.paint.selection_shapes = vec![SelectionEntry {
            shape: SelectionShape::Raster {
                crisp: Arc::new(crisp.clone()),
            },
            op: 0,
        }];
        self.set_selection_from_crisp(crisp);
        self.commit_structural_edit(before);
    }

    /// **Color Fill**: paint the brush colour into the selected region of the active layer, blended by the
    /// selection coverage (feathered edges partial). No-op without a live selection. One undo entry.
    pub fn selection_color_fill(&mut self) {
        if !self.selection_restricts_paint() {
            return;
        }
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if w == 0 || h == 0 || self.canvas_rgba.len() != w * h * 4 {
            return;
        }
        let before = self.snapshot_model();
        let color = [
            (self.paint.brush.color[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (self.paint.brush.color[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (self.paint.brush.color[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            255,
        ];
        let mask = Arc::clone(&self.paint.selection_mask);
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        let n = mask.len().min(buf.len() / 4);
        for i in 0..n {
            let cov = f32::from(mask[i]) / 255.0;
            if cov <= 0.0 {
                continue;
            }
            let b = i * 4;
            for c in 0..4 {
                let dst = f32::from(buf[b + c]);
                let src = f32::from(color[c]);
                buf[b + c] = (src * cov + dst * (1.0 - cov)).round().clamp(0.0, 255.0) as u8;
            }
        }
        self.mark_dirty(Region {
            x: 0,
            y: 0,
            w: w as u32,
            h: h as u32,
        });
        self.commit_structural_edit(before);
    }

    /// **Copy**: capture the selected pixels (coverage-premultiplied) into the in-memory clipboard. No undo
    /// entry (a read-only capture). No-op without a live selection.
    pub fn selection_copy(&mut self) {
        if !self.selection_restricts_paint() {
            return;
        }
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if w == 0 || h == 0 || self.canvas_rgba.len() != w * h * 4 {
            return;
        }
        let mask = &self.paint.selection_mask;
        // Tight bbox of the covered texels.
        let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0usize, 0usize);
        for y in 0..h {
            for x in 0..w {
                if mask[y * w + x] > 0 {
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x + 1);
                    y1 = y1.max(y + 1);
                }
            }
        }
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let (cw, ch) = (x1 - x0, y1 - y0);
        let mut rgba = vec![0u8; cw * ch * 4];
        for y in 0..ch {
            for x in 0..cw {
                let src = ((y0 + y) * w + (x0 + x)) as usize;
                let cov = f32::from(mask[src]) / 255.0;
                let s = src * 4;
                let d = (y * cw + x) * 4;
                for c in 0..3 {
                    rgba[d + c] = self.canvas_rgba[s + c];
                }
                // Premultiply the source alpha by coverage so Paste blends the selected shape only.
                rgba[d + 3] = (f32::from(self.canvas_rgba[s + 3]) * cov).round().clamp(0.0, 255.0) as u8;
            }
        }
        self.paint.selection_clipboard = Some(SelectionClip {
            rect: Region {
                x: x0 as u32,
                y: y0 as u32,
                w: cw as u32,
                h: ch as u32,
            },
            rgba,
        });
    }

    /// **Paste**: composite the clipboard back over the active layer at its original location (source-over
    /// by the clip's alpha). One undo entry. No-op with an empty clipboard.
    pub fn selection_paste(&mut self) {
        let Some(clip) = self.paint.selection_clipboard.clone() else {
            return;
        };
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if w == 0 || h == 0 || self.canvas_rgba.len() != w * h * 4 {
            return;
        }
        let before = self.snapshot_model();
        let (cx, cy, cw, ch) = (
            clip.rect.x as usize,
            clip.rect.y as usize,
            clip.rect.w as usize,
            clip.rect.h as usize,
        );
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        for y in 0..ch {
            for x in 0..cw {
                let (dx, dy) = (cx + x, cy + y);
                if dx >= w || dy >= h {
                    continue;
                }
                let s = (y * cw + x) * 4;
                let a = f32::from(clip.rgba[s + 3]) / 255.0;
                if a <= 0.0 {
                    continue;
                }
                let d = (dy * w + dx) * 4;
                for c in 0..3 {
                    let src = f32::from(clip.rgba[s + c]);
                    let dst = f32::from(buf[d + c]);
                    buf[d + c] = (src * a + dst * (1.0 - a)).round().clamp(0.0, 255.0) as u8;
                }
                let da = f32::from(buf[d + 3]) / 255.0;
                buf[d + 3] = ((a + da * (1.0 - a)) * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
        self.mark_dirty(clip.rect);
        self.commit_structural_edit(before);
    }
}
