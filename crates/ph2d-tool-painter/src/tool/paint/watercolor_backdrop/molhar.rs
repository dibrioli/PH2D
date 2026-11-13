//! **Molhar a camada** (EDGE-1, doc 13 #10 + doc 14 #3) e as leituras da humidade que a shell pinta como véu
//! (#12a) — filho de `watercolor_backdrop`, cortado pelo tecto de 700 linhas.

use super::*;

impl PainterTool {
    /// EDGE-1 (doc 13 #10 + doc 14 #3): **Wet the layer** (Rebelle). Re-moisten the whole canvas WITHOUT
    /// depositing pigment AND re-open a wet session bound to the CURRENT canvas so strokes made now FUSE
    /// with the EXISTING paint — dropping water on a dry wash reactivates it. Fills `canvas_wet` = 255
    /// (visible damp #12a + the drying/fusion window), freezes the current pixels as the session base (what
    /// the rewet lifts) over the ground below (the lift target), starts the union buffers EMPTY (untouched
    /// paint stays exactly as-is until a stroke reaches it), and FORCES the rewet ([`WET_CANVAS_REWET`]) so
    /// the reactivation shows even with the brush's own Rewet at `0`. No pixel touch (the lift happens when
    /// you paint). Off the watercolor render-path it only arms the moisture (nothing to fuse).
    pub(in crate::tool::paint) fn wet_canvas_now(&mut self) {
        let (fw, fh) = self.source_size;
        let n = (fw as usize) * (fh as usize);
        if n == 0 {
            return;
        }
        self.paint.canvas_wet = vec![255u8; n];
        self.paint.canvas_wet_rect = Some((0, 0, fw as usize, fh as usize));
        self.paint.canvas_wet_rev = veu::nova_revisao();
        self.paint.canvas_wet_carry = 0.0;
        if !self.watercolor_render_active() {
            return;
        }
        // The existing pixels ARE the wet-session base (what the rewet reactivates); the ground below is
        // the lift target. Empty union buffers ⇒ `wet_session_continues` passes and the next stroke joins
        // this session (lifting the base) instead of starting fresh over the dry paint.
        self.paint.watercolor_base = Some(Arc::clone(&self.canvas_rgba));
        self.paint.wet_backdrop = Some(self.fundo_de_baixo());
        self.paint.wet_session_base = Some(Arc::clone(&self.canvas_rgba));
        self.arma_a_guarda_da_sessao();
        self.paint.stroke_coverage = vec![0u8; n];
        self.paint.stroke_color = vec![0u8; n * 4];
        self.paint.stroke_density = Vec::new();
        self.paint.stroke_deplete = Vec::new();
        self.paint.stroke_deplete_prox = Vec::new();
        self.paint.wet_reserve_cache = None;
        self.paint.wet_rewet_cache = None; // B16.5: idem
        self.paint.stroke_water = Vec::new();
        self.paint.wet_styles.clear();
        self.paint.wet_cum_dirty = None;
        self.paint.wet_stroke_dirty = None;
        self.paint.wet_pour_marca = None;
        self.paint.wet_session_wetness = WET_CANVAS_REWET;
    }

    /// #12a (doc 14): the canvas MOISTURE map + its live rect for the on-canvas wetness overlay —
    /// `(bytes, w, h, [x0, y0, x1, y1])`; each byte is the local wetness (`0` = dry, `255` = fully
    /// wet). `None` when the paper is dry. Read-only; the shell tints the wet region as a sheen.
    #[must_use]
    pub fn canvas_wet_view(&self) -> Option<CanvasWetView<'_>> {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let (x0, y0, x1, y1) = self.paint.canvas_wet_rect?;
        (n > 0 && self.paint.canvas_wet.len() == n).then_some((
            &self.paint.canvas_wet[..],
            w,
            h,
            [x0 as u32, y0 as u32, x1 as u32, y1 as u32],
        ))
    }

    /// #12a (doc 14): the on-canvas wetness PREVIEW strength (`0..1`) — the max veil alpha the shell
    /// paints over the wet paper. `0` = no preview. The Wetness card's slider drives it.
    #[must_use]
    pub fn wet_preview_intensity(&self) -> f32 {
        self.paint.wet_preview_intensity.clamp(0.0, 1.0)
    }
}
