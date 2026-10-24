//! The **Eyedropper** — an on-canvas colour pick. The left-rail Eyedropper arms it (no colour wheel);
//! the next canvas Down samples the pixel the user SEES (the flattened composite, falling back to the
//! working canvas for a trivial stack) into the brush colour, then disarms so painting resumes as Brush.
//!
//! Mirrors the Clone "Set Source" / Symmetry pick modes: it runs before the paintable-target gate in
//! [`super::PainterTool::on_canvas_pointer`] (it samples a pixel, so it works on any active layer).

use super::PainterTool;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};

impl PainterTool {
    /// Whether the on-canvas Eyedropper pick is armed (the next Down samples a colour). The shell reads
    /// this to keep the rail Eyedropper button checked, and to snap it back to Brush when it clears.
    #[must_use]
    pub fn eyedropper_armed(&self) -> bool {
        self.paint.eyedropper_armed
    }

    /// **A amostra do conta-gotas** no ponto `pos` (px de imagem): o pixel da composição achatada que o
    /// artista VÊ (a de cache; sem ela, a tela de trabalho — pilha trivial), straight-sRGB8. É o MESMO
    /// leitor do clique ([`Self::eyedropper_pointer`]) e do cursor que mostra a cor antes do clique, por
    /// isso os dois nunca discordam. Um índice num buffer: O(1), sem compor (medido 24/10: 5,1 ns por chamada, perfil debug, `load 18`).
    /// Fora da tela o ponto encosta na borda, como o clique sempre fez; `None` sem tela.
    #[must_use]
    pub fn eyedropper_amostra(&self, pos: [f32; 2]) -> Option<[u8; 4]> {
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return None;
        }
        // Clamp the sample to the canvas without `.clamp` (swap-free): floor, floor-to-0, cap at edge.
        let x = (pos[0].max(0.0) as usize).min(w as usize - 1);
        let y = (pos[1].max(0.0) as usize).min(h as usize - 1);
        let idx = (y * (w as usize) + x) * 4;
        let buf = self
            .composited
            .as_deref()
            .map(|c| c.as_slice())
            .filter(|c| c.len() >= idx + 4)
            .or_else(|| (self.canvas_rgba.len() >= idx + 4).then(|| self.canvas_rgba.as_slice()))?;
        Some([buf[idx], buf[idx + 1], buf[idx + 2], buf[idx + 3]])
    }

    /// Consume a canvas event while the Eyedropper is armed: the Down samples the composited pixel under
    /// the cursor into the brush colour and disarms; Move/Up are swallowed (no paint / no sprite move)
    /// during the armed window. Always returns `true` so the shell never falls through to move the sprite.
    pub(super) fn eyedropper_pointer(&mut self, ev: CanvasPointer) -> bool {
        if ev.phase != PointerPhase::Down {
            return true;
        }
        if let Some([r, g, b, _]) = self.eyedropper_amostra(ev.pos) {
            self.paint.brush.color = [
                f32::from(r) / 255.0,
                f32::from(g) / 255.0,
                f32::from(b) / 255.0,
            ];
            self.sync_brush_color_across_modes(); // the sampled colour is the shared paint colour (all modes)
        }
        self.paint.eyedropper_armed = false; // one-shot → back to Brush
        true
    }
}
