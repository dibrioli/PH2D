//! **Reconstruct** sub-mode — the inverse of Reshape: drag to fade the deform out. It does NOT cross-fade
//! the original image over the deformed one (that just ghosts the original on top). Instead it REDUCES the
//! session displacement toward zero under the brush, then re-renders from `pre` — so the warped pixels
//! physically slide BACK to their original positions (a real un-warp), matching Procreate's Reconstruct.

use super::apply::bilinear_clamped;
use super::field::falloff;
use crate::tool::PainterTool;
use std::sync::Arc;

impl PainterTool {
    /// Reduce the session displacement under the dab by `falloff · pressure`, then re-render the bbox from
    /// `pre`. No-op before a session exists. Freeze holds the protected texels' displacement (they don't
    /// un-warp either, staying wherever they were — consistent with Freeze protecting a region from change).
    pub(super) fn warp_reconstruct_dab(&mut self, center: [f32; 2], radius: f32, pressure: f32) {
        let Some(bbox) = self.dab_bbox(center, radius) else {
            return;
        };
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if self.paint.warp.pre.len() != n * 4 || self.paint.warp.disp.len() != n {
            return; // no session to reconstruct
        }
        let r = radius.max(1.0);
        let inv_r2 = 1.0 / (r * r);
        let pressure = pressure.clamp(0.0, 1.0);
        // Confined to the SELECTED area (whole sprite when nothing is selected), like every Deform op.
        let restrict = self.deform_restricts_to_selection();

        // Pass 1 (immutable self): the per-texel amount of displacement to remove this dab (falloff·pressure,
        // scaled by the selection coverage). Collected first so the mutable pass doesn't fight the coverage borrow.
        let mut factors: Vec<f32> = Vec::with_capacity((bbox.w * bbox.h) as usize);
        for ry in 0..bbox.h {
            let dy = bbox.y + ry;
            for rx in 0..bbox.w {
                let dx = bbox.x + rx;
                let relx = dx as f32 - center[0];
                let rely = dy as f32 - center[1];
                let mut amt = falloff((relx * relx + rely * rely) * inv_r2) * pressure;
                if restrict {
                    amt *= f32::from(self.selection_coverage_at(dx, dy)) / 255.0;
                }
                factors.push(amt);
            }
        }

        // Pass 2 (mutable): shrink the displacement toward zero and re-render from `pre`.
        let src = Arc::clone(&self.paint.warp.pre);
        let disp = Arc::make_mut(&mut self.paint.warp.disp);
        let buf = crate::tool::paint::plane_fork::fork_par(&mut self.canvas_rgba);
        for ry in 0..bbox.h {
            let dy = bbox.y + ry;
            for rx in 0..bbox.w {
                let dx = bbox.x + rx;
                let gi = (dy * w + dx) as usize;
                let keepf = 1.0 - factors[(ry * bbox.w + rx) as usize];
                let d = &mut disp[gi];
                d[0] *= keepf;
                d[1] *= keepf;
                let px = bilinear_clamped(&src, w, h, dx as f32 - d[0], dy as f32 - d[1]);
                let b = gi * 4;
                buf[b..b + 4].copy_from_slice(&px);
            }
        }
        // W4: the un-warp slides the body back with the colour — the same one door the dab kernel uses.
        self.warp_render_relief(bbox);
        self.mark_dirty(bbox);
    }
}
