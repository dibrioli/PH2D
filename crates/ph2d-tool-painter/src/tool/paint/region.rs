//! Pixel-region helpers for the interactive drag-preview: the touchable dab bbox + save/restore of a
//! `canvas_rgba` rectangle. Split from `paint.rs` for the workspace file-LOC cap (a child module, so it
//! keeps access to `PaintState`'s private fields). Used by `paint::stamp_drag_preview`.

use super::Region;
use crate::tool::PainterTool;
use std::sync::Arc;

impl PainterTool {
    /// Pixel bbox a dab of `radius` at `center` can touch — the drag-preview **save/restore + dirty-upload**
    /// region; MUST superset the blit/accumulate write bounds (`floor(c−r) .. ceil(c+r)+1`). A
    /// `round(c)±(ceil(r)+1)` box misses the high edge 1px for `frac(c) < 0.5` → stale un-restored edge row
    /// (thin horizontal trails). Enio 2026-06-27.
    pub(super) fn dab_bbox(&self, center: [f32; 2], radius: f32) -> Option<Region> {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return None;
        }
        let x0 = (center[0] - radius).floor().max(0.0) as i64;
        let y0 = (center[1] - radius).floor().max(0.0) as i64;
        let x1 = ((center[0] + radius).ceil() as i64 + 1).min(w as i64);
        let y1 = ((center[1] + radius).ceil() as i64 + 1).min(h as i64);
        if x1 <= x0 || y1 <= y0 {
            return None;
        }
        Some(Region {
            x: x0 as u32,
            y: y0 as u32,
            w: (x1 - x0) as u32,
            h: (y1 - y0) as u32,
        })
    }

    /// Copy the RGBA8 pixels of `rect` out of `canvas_rgba` (row-major over the region).
    pub(super) fn save_region(&self, rect: &Region) -> Vec<u8> {
        let stride = self.source_size.0 as usize * 4;
        let rw = rect.w as usize * 4;
        let mut out = Vec::with_capacity(rw * rect.h as usize);
        for row in 0..rect.h {
            let start = (rect.y + row) as usize * stride + rect.x as usize * 4;
            out.extend_from_slice(&self.canvas_rgba[start..start + rw]);
        }
        out
    }

    /// Write `pixels` (from [`Self::save_region`]) back into `rect` and flag it dirty.
    pub(super) fn restore_region(&mut self, rect: &Region, pixels: &[u8]) {
        let stride = self.source_size.0 as usize * 4;
        let rw = rect.w as usize * 4;
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        for row in 0..rect.h {
            let dst = (rect.y + row) as usize * stride + rect.x as usize * 4;
            let src = row as usize * rw;
            buf[dst..dst + rw].copy_from_slice(&pixels[src..src + rw]);
        }
        self.mark_dirty(*rect);
    }
}

/// Smallest region covering both `a` and `b` — the routes' dirty-rect fold (every stamp route
/// imports it as `super::union_region`; moved here from `paint.rs` for the workspace file-LOC cap).
pub(super) fn union_region(a: Region, b: Region) -> Region {
    let x0 = a.x.min(b.x);
    let y0 = a.y.min(b.y);
    let x1 = (a.x + a.w).max(b.x + b.w);
    let y1 = (a.y + a.h).max(b.y + b.h);
    Region {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    }
}
