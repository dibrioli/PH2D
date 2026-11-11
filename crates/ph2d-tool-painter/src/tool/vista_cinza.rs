//! **A vista cinza da máscara** (o olho da linha da máscara aberto) — o ramo `Gray` do dreno, partido do
//! `runtime.rs` (B52.R4) com a cura B10.A1 da [missão 4K](../../../../docs/Painter/missao_4k/BACKLOG.md):
//! com a vista quente (o `composited` é o cinza DESTA máscara) só a caixa suja é re-cinzada e sobe; a tela
//! inteira (16,7 M píxeis a 4K) só no 1.º quadro do olho aberto. Gate: `vista_cinza_tests`.

use super::*;

impl PainterTool {
    /// O dreno com o olho de uma máscara aberto: `Some` = o quadro é o cinza dela (e o ramo é `Gray`);
    /// `None` = a composição normal segue.
    pub(crate) fn drena_a_vista_cinza(&mut self) -> Option<(Arc<Vec<u8>>, u32, u32)> {
        let mv = self.mask_view_grayscale?;
        let (w, h) = self.source_size;
        let quente = self.composited_cinza == Some(mv)
            && self
                .composited
                .as_ref()
                .is_some_and(|c| c.len() == (w as usize) * (h as usize) * 4);
        self.mask_grayscale_view_source()?;
        let caixa = self.dirty_rect.take();
        // Os campos à mão (o `mask_grayscale_view_source` empresta o `self` todo): a fonte é o
        // `canvas_rgba`/`images`, o destino o `composited`.
        let src: &[u8] = if self.layers.active() == Some(mv) {
            &self.canvas_rgba
        } else {
            &self.images.get(&mv)?.rgba8
        };
        match (quente, caixa) {
            (true, Some(r)) => {
                let cache = Arc::make_mut(self.composited.as_mut().expect("quente"));
                cinza_na_caixa(src, w, cache, r);
                self.publica_o_envio(Some(r), Vec::new());
            }
            // Sem caixa (uma mudança que não marcou região) a tela inteira, como sempre foi.
            _ => {
                let mut out = vec![0u8; (w as usize) * (h as usize) * 4];
                cinza_na_caixa(src, w, &mut out, Region { x: 0, y: 0, w, h });
                self.composited = Some(Arc::new(out));
                self.publica_o_envio(None, Vec::new());
            }
        }
        self.composited_cinza = Some(mv);
        self.last_drain_branch = crate::tool::DrainBranch::Gray;
        Some((
            Arc::clone(self.composited.as_ref().expect("acabado de pôr")),
            w,
            h,
        ))
    }

    /// O cinza da máscara como um quadro RGBA opaco, a tela inteira (o `current_preview` do trait). Mostra
    /// a máscara CRUA (sem o Invert).
    #[must_use]
    pub(crate) fn mask_grayscale_view_pixels(&self) -> Option<Vec<u8>> {
        let src = self.mask_grayscale_view_source()?;
        let (w, h) = self.source_size;
        let mut out = vec![0u8; (w as usize) * (h as usize) * 4];
        cinza_na_caixa(src, w, &mut out, Region { x: 0, y: 0, w, h });
        Some(out)
    }
}

/// Escreve em `out` (RGBA da tela, largura `w`) o cinza opaco da máscara `src` dentro de `r`.
pub(crate) fn cinza_na_caixa(src: &[u8], w: u32, out: &mut [u8], r: Region) {
    let w = w as usize;
    for y in r.y as usize..(r.y + r.h) as usize {
        let linha = y * w;
        for x in r.x as usize..(r.x + r.w) as usize {
            let i = linha + x;
            let g = (crate::compositor::mask_value(src, i) * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8;
            out[i * 4..i * 4 + 4].copy_from_slice(&[g, g, g, 255]);
        }
    }
}
