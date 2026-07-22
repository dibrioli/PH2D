//! The watercolor composite's **window arithmetic** (child of
//! [`super`], split for the workspace file-LOC cap): resolve the frozen
//! session base + ground, consume the frame/cumulative dirty rects, and
//! derive the padded OUTPUT region plus the twice-padded READ window that
//! gives every warped blur sample full support. Pure bookkeeping, moved
//! verbatim — every early-out returns `None` and the caller performs the
//! `commit`-time base drop exactly as the inline code did.

use super::super::*;

/// Everything the composite needs to know about WHERE it runs: the frozen
/// bases, the output region, the read window, and the style maxima that
/// sized the padding (the per-pixel loop reads several of them again).
pub(super) struct WashWindow {
    pub fw: usize,
    pub fh: usize,
    pub n: usize,
    pub base_arc: Arc<Vec<u8>>,
    pub backdrop_arc: Arc<Vec<u8>>,
    pub spread: usize,
    pub warp_amp: f32,
    pub wet: f32,
    pub core_r: usize,
    pub wet_any: f32,
    pub spread_any: usize,
    pub soaked: bool,
    pub watered: bool,
    pub x0: usize,
    pub y0: usize,
    pub y1: usize,
    pub bw: usize,
    pub bh: usize,
    pub region: Region,
    pub rx0: usize,
    pub ry0: usize,
    pub rx1: usize,
    pub ry1: usize,
    pub rw: usize,
    pub rh: usize,
}

impl PainterTool {
    /// Build the composite window, or `None` when there is nothing to do
    /// (missing buffers, empty dirty rect, degenerate region). The caller
    /// owns the `commit`-time `watercolor_base` drop on `None`.
    pub(super) fn wash_window(&mut self, commit: bool) -> Option<WashWindow> {
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        let n = fw * fh;
        if n == 0 || self.paint.stroke_coverage.len() != n || self.canvas_rgba.len() != n * 4 {
            return None;
        }
        // The frozen base (own the Arc so the canvas make_mut doesn't alias the field).
        // EDGE-1: the composite reads the SESSION base — frozen at the session's FIRST stroke —
        // so a continuing stroke re-renders the whole UNION from scratch (one wash, one rim)
        // instead of double-counting its own bake. Per-stroke fallback = first-stroke identical.
        let base_arc = match self.paint.wet_session_base.as_ref() {
            Some(b) if b.len() == n * 4 => Arc::clone(b),
            _ => self.paint.watercolor_base.as_ref().map(Arc::clone)?,
        };
        if base_arc.len() != n * 4 {
            return None;
        }
        // The frozen GROUND ([`super::super::watercolor_backdrop`]): the Beer–Lambert base where
        // the layer is transparent, the rewet's "what is paint" reference and the lift's target.
        // A missing/mis-sized one (defensive) degrades to a uniform paper-colour ground.
        let backdrop_arc = match self.paint.wet_backdrop.as_ref() {
            Some(b) if b.len() == n * 4 => Arc::clone(b),
            _ => {
                let p = self.paper_color_rgb8();
                let mut g = vec![0u8; n * 4];
                for px in g.chunks_exact_mut(4) {
                    px.copy_from_slice(&[p[0], p[1], p[2], 255]);
                }
                Arc::new(g)
            }
        };

        // Pick the recomposite rect and CONSUME the frame one (wet_edges `resetFrame`): live =
        // this frame's dabs; commit = the whole stroke (`paint_end`'s finish dabs folded in).
        let frame = self.paint.wet_frame_dirty.take();
        let dirty = if commit {
            match (self.paint.wet_cum_dirty, frame) {
                (Some(c), Some(f)) => Some(union_region(c, f)),
                (c, f) => c.or(f),
            }
        } else {
            frame
        };
        let dirty = dirty?;

        let spread = self.paint.brush.edge_spread.round().clamp(0.0, 48.0) as usize;
        let warp_amp = self.paint.brush.warp.max(0.0);
        let wet = self.paint.brush.wet_rewet.clamp(0.0, 1.0);
        // Silhouette-feather radius (`inner`), capped so a pool keeps a saturated core.
        let core_r = spread.min(((self.paint.brush.radius_px * 0.5).round() as usize).max(1));
        // EDGE-1 per-stroke style (doc 13 topo): geometry/field paths take the SESSION MAXIMA
        // (see `session_maxima`) — per-pixel terms resolve the OWNER's values in the loop.
        let (wet_any, warp_any, spread_any, core_any) = self
            .paint
            .wet_styles
            .session_maxima(wet, warp_amp, spread, core_r);
        // Influence radius of a dab beyond its own disc (blur reach + warp + rounding slack):
        // dry = the capped feather only (a tight window); Wet = the dissolve's spread; a session
        // that poured dwell reads a 2× blur, so the reach doubles.
        let soaked = wet_any > 0.0 && self.paint.wet_soak_active && self.paint.wet_soak.len() == n;
        // EDGE-2: carried water poured this session (Dilution) — builds the rewet fields (and its
        // own halo) regardless of Rewet, so pure water blooms against the paint beneath.
        let watered = self.paint.stroke_water.len() == n;
        let reach = if soaked || watered {
            spread_any * 2
        } else if wet_any > 0.0 {
            spread_any
        } else {
            core_any
        };
        let pad = reach + warp_any.ceil() as usize + 2;
        let x0 = (dirty.x as usize).saturating_sub(pad);
        let y0 = (dirty.y as usize).saturating_sub(pad);
        let x1 = ((dirty.x as usize) + (dirty.w as usize) + pad).min(fw);
        let y1 = ((dirty.y as usize) + (dirty.h as usize) + pad).min(fh);
        if x0 >= x1 || y0 >= y1 {
            return None;
        }
        let (bw, bh) = (x1 - x0, y1 - y0);
        let region = Region {
            x: x0 as u32,
            y: y0 as u32,
            w: bw as u32,
            h: bh as u32,
        };

        // READ window = the output region padded by the influence radius AGAIN, so the blur under
        // every warped sample position inside the output has full support. (A window clamped at
        // the output's edge would misread the old coverage there — a darkened seam at each
        // frame-region boundary.)
        let rx0 = x0.saturating_sub(pad);
        let ry0 = y0.saturating_sub(pad);
        let rx1 = (x1 + pad).min(fw);
        let ry1 = (y1 + pad).min(fh);
        let (rw, rh) = (rx1 - rx0, ry1 - ry0);

        Some(WashWindow {
            fw,
            fh,
            n,
            base_arc,
            backdrop_arc,
            spread,
            warp_amp,
            wet,
            core_r,
            wet_any,
            spread_any,
            soaked,
            watered,
            x0,
            y0,
            y1,
            bw,
            bh,
            region,
            rx0,
            ry0,
            rx1,
            ry1,
            rw,
            rh,
        })
    }
}
