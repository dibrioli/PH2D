//! Watercolor **ground** (real backdrop) + water **soak** (dwell) — the per-stroke state the optical
//! composite reads its Beer–Lambert base / rewet reference / lift target from, and the "the longer
//! the water sits, the more it dissolves" field. Split from `watercolor_render.rs` for the LOC cap.
//!
//! The ground REPLACES the old virtual-cream constant (`PAPER`): where the active layer is
//! transparent, the optics now see the **composite of the layers below it** over the document
//! [`paper colour`](super::PaintState::paper_color) — painting over a white layer no longer pulls
//! toward beige, and the rewet's "what is paint" question is answered against the true local
//! ground (Enio 2026-07-06, doc `11_aquarela_avaliacao_padrao_ouro.md` §3).

use super::*;
use crate::compositor::composite_below;

/// How fast the parked brush pours dwell into [`PaintState::wet_soak`]: a held nib saturates its
/// disc in ~2 s (`255 / 2 s`). Moving strokes soak too (each tick pours at the current dab), but the
/// dt spreads across positions, so a sweep stays light — only LINGERING builds a deep soak.
const SOAK_RATE_PER_S: f32 = 127.5;

impl PainterTool {
    /// Freeze the watercolor ground for a beginning stroke: the pre-stroke `canvas_rgba` as the
    /// optical base (shared `Arc`, O(1)) + the real **backdrop** under the active layer. Also resets
    /// the per-stroke soak. No-op (clears both) when the watercolor render-path is off.
    pub(super) fn freeze_watercolor_ground(&mut self) {
        if !self.watercolor_render_active() {
            self.paint.watercolor_base = None;
            self.paint.wet_backdrop = None;
            return;
        }
        self.paint.watercolor_base = Some(Arc::clone(&self.canvas_rgba));
        self.paint.wet_backdrop = Some(Arc::new(self.build_wet_backdrop()));
        self.paint.wet_soak.iter_mut().for_each(|s| *s = 0);
        self.paint.wet_soak_pos = None;
        self.paint.wet_soak_active = false;
    }

    /// Composite the layers strictly BELOW the anchor over the document paper colour → the opaque
    /// RGBA8 ground. The anchor is the active layer; painting a MASK anchors at the mask's PARENT
    /// (the mask shows through its parent, so the ground under the parent is what the eye sees).
    /// Runs once per stroke (pen-down); `composite_below` handles an unknown anchor (plain paper).
    fn build_wet_backdrop(&self) -> Vec<u8> {
        let (w, h) = self.source_size;
        let paper = self.paper_color_rgb8();
        let anchor = self.layers.active().and_then(|id| {
            if self.layers.is_mask(id) {
                // The mask isn't in the z-order — resolve the raster that owns it.
                self.layers
                    .all_ids()
                    .find(|&pid| self.layers.get(pid).is_some_and(|l| l.mask == Some(id)))
            } else {
                Some(id)
            }
        });
        let Some(anchor) = anchor else {
            let mut ground = vec![0u8; (w as usize) * (h as usize) * 4];
            for px in ground.chunks_exact_mut(4) {
                px.copy_from_slice(&[paper[0], paper[1], paper[2], 255]);
            }
            return ground;
        };
        let src = ToolPixelSource {
            active_id: anchor,
            active_rgba: &self.canvas_rgba,
            images: &self.images,
        };
        composite_below(&self.layers, &src, w, h, anchor, paper)
    }

    /// The document paper colour as straight sRGB8 bytes.
    pub(super) fn paper_color_rgb8(&self) -> [u8; 3] {
        let c = self.paint.paper_color;
        [
            (c[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (c[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (c[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        ]
    }

    /// Set one paper-colour channel (`0..1` straight sRGB); the shared-picker read-back drives all
    /// three per frame via `SelectOption(PAINTER_WATERCOLOR_PAPER_COLOR_THUMB, "r,g,b")`.
    pub(crate) fn set_paper_color_rgb8(&mut self, r: u8, g: u8, b: u8) {
        self.paint.paper_color = [
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
        ];
    }

    /// Pour `dt_s` of water dwell into the soak disc at the last dab position (the tick heartbeat,
    /// moving or parked). Returns the poured disc's region when the soak actually grew AND the
    /// rewet reads it (`wet_rewet > 0`), so the caller can fold it into the frame dirty rect and
    /// recomposite — a parked wet brush visibly deepens its bleed while held.
    pub(super) fn grow_wet_soak(&mut self, dt_s: f32) -> Option<Region> {
        let wet = self.paint.brush.wet_rewet;
        let (center, radius) = self.paint.wet_soak_pos?;
        if wet <= 0.0 || dt_s <= 0.0 || radius <= 0.0 {
            return None;
        }
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        if fw == 0 || fh == 0 {
            return None;
        }
        if self.paint.wet_soak.len() != fw * fh {
            self.paint.wet_soak = vec![0u8; fw * fh];
        }
        // Byte units per tick; at least 1 step so slow frames still soak.
        let add = (SOAK_RATE_PER_S * dt_s).clamp(1.0, 255.0) as u16;
        let (cx, cy) = (center[0], center[1]);
        // A sitting puddle SPREADS: the pour disc grows with the centre's own saturation (up to
        // 2× the nib) — holding the brush pushes the dwell past the disc edge, so the widened
        // dissolve visibly creeps outward instead of stopping at the footprint.
        let ci = (cy.clamp(0.0, (fh - 1) as f32) as usize) * fw
            + (cx.clamp(0.0, (fw - 1) as f32) as usize);
        let center_soak = if self.paint.wet_soak.len() == fw * fh {
            f32::from(self.paint.wet_soak[ci]) / 255.0
        } else {
            0.0
        };
        let r = radius * (1.0 + center_soak);
        let inv_r = 1.0 / r;
        let x0 = (cx - r).floor().max(0.0) as usize;
        let y0 = (cy - r).floor().max(0.0) as usize;
        let x1 = ((cx + r).ceil() as i64).clamp(0, fw as i64) as usize;
        let y1 = ((cy + r).ceil() as i64).clamp(0, fh as i64) as usize;
        if x0 >= x1 || y0 >= y1 {
            return None;
        }
        let soak = &mut self.paint.wet_soak;
        let mut grew = false;
        for y in y0..y1 {
            let dy = (y as f32 + 0.5) - cy;
            let base = y * fw;
            for x in x0..x1 {
                let dx = (x as f32 + 0.5) - cx;
                let dn = (dx * dx + dy * dy).sqrt() * inv_r;
                if dn >= 1.0 {
                    continue;
                }
                // Full pour inside the core, fading to the rim (the water pools under the nib).
                let w = (1.0 - dn).min(0.6) / 0.6;
                let idx = base + x;
                let cur = soak[idx];
                let next = (u16::from(cur) + (f32::from(add) * w) as u16).min(255) as u8;
                if next != cur {
                    soak[idx] = next;
                    grew = true;
                }
            }
        }
        if grew {
            self.paint.wet_soak_active = true;
        }
        grew.then(|| Region {
            x: x0 as u32,
            y: y0 as u32,
            w: (x1 - x0) as u32,
            h: (y1 - y0) as u32,
        })
    }
}
