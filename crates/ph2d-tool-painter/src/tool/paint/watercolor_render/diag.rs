//! **O envelope de DIAGNÓSTICO do composite da aquarela** (filho de [`super`], split pelo teto de LOC
//! do workspace).
//!
//! Medir e compor são responsabilidades diferentes, e é por isso que o corte cai aqui: o corpo
//! (`apply_watercolor_inner`) fica no arquivo do composite, e a porta pública é este envelope, que
//! cronometra a passada e lê o DELTA da janela que ela somou.
//!
//! ⚠️ **Envelope, e não um `Instant` no meio do corpo:** o composite tem saídas antecipadas (janela
//! vazia, base ausente, região degenerada), e um `note` esquecido num desses ramos publica a média
//! sobre uma amostra menor **sem dizer que o fez** — que é a forma exata do instrumento mudo que este
//! módulo já pagou uma vez (`sim media 0.00ms x0`, doc 28 §5.40).

use super::super::*;

impl PainterTool {
    /// Composite the watercolor wash over the frozen base ([`PaintState::watercolor_base`](super::PaintState)).
    ///
    /// Reads the coverage + deposited-colour buffers, reconstructs the optical density `D`, and applies
    /// per-channel Beer–Lambert in linear light — see the module docs. The base is a separate `Arc` (each
    /// frame recomposites from the pristine pre-stroke pixels, no overlay peel); `commit` drops it at the
    /// pen-up bake (inside the undo transaction), the live passes keep it for the next frame.
    ///
    /// **Dirty-rect (wet_edges `renderFrame`/`endStroke`):** live passes recomposite ONLY the frame
    /// dirty rect (dabs since the last composite) padded by the influence radius; the pen-up bake makes
    /// one cumulative pass from a tracked bbox — never a full scan. Returns the region (`None` = no-op).
    pub(in crate::tool::paint) fn apply_watercolor(&mut self, commit: bool) -> Option<Region> {
        // Envelope de diagnóstico (`crate::wash_diag`): mede o composite e o DELTA da janela que ele
        // somou, para o log do produto poder publicar `ns/texel`. O corpo é o `_inner`; um envelope
        // em vez de um `Instant` no meio do corpo porque a função tem saídas antecipadas, e um
        // `note` esquecido num ramo mede a janela errada em silêncio.
        let t0 = std::time::Instant::now();
        let px0 = self.wash.window_px;
        let out = self.apply_watercolor_inner(commit);
        crate::wash_diag::note_composite(
            t0.elapsed().as_secs_f32() * 1e3,
            self.wash.window_px.saturating_sub(px0),
        );
        out
    }
}
