//! **O relevo segue o alfa que a Borracha do Wet deixou** (B06.1, BUGS #70): a altura e a cobertura da camada activa
//! escalam pela razão alfa novo / alfa antigo, píxel a píxel, na caixa de escrita. A tinta que sai leva o corpo, na viva
//! (a `base` da sessão) e na morta (a tela), pela mesma lei gradual que tira a cor.

use super::borracha_morta::Caixa;
use super::*;

/// O alfa de `rgba` (largura `w`) dentro da `caixa`, linha a linha.
pub(super) fn alfa_na_caixa(rgba: &[u8], caixa: Caixa, w: usize) -> Vec<u8> {
    (caixa.y0..caixa.y1)
        .flat_map(|y| (caixa.x0..caixa.x1).map(move |x| rgba[(y * w + x) * 4 + 3]))
        .collect()
}

/// A razão alfa depois / alfa antes de cada píxel da `caixa` (1 onde o alfa não desceu); `None` se nenhum desceu.
pub(super) fn razao_do_alfa(
    antes: &[u8],
    depois: &[u8],
    caixa: Caixa,
    w: usize,
) -> Option<Vec<f32>> {
    let mut desceu = false;
    let razao = (caixa.y0..caixa.y1)
        .flat_map(|y| (caixa.x0..caixa.x1).map(move |x| (y, x)))
        .zip(antes)
        .map(|((y, x), &a0)| {
            let a1 = depois[(y * w + x) * 4 + 3];
            if a1 < a0 {
                desceu = true;
                f32::from(a1) / f32::from(a0)
            } else {
                1.0
            }
        })
        .collect();
    desceu.then_some(razao)
}

impl PainterTool {
    /// Escala a altura e a cobertura da camada activa pela `razao` da `caixa` ([`razao_do_alfa`]). Sem relevo sob a
    /// Borracha nenhum plano se copia.
    pub(super) fn relevo_segue_o_alfa(&mut self, caixa: Caixa, razao: &[f32]) {
        let Some(layer) = self.layers.active() else {
            return;
        };
        if !crate::tool::paint::impasto::apagar_segue_a_camada() {
            return;
        }
        let (w, h) = self.source_size;
        let (wu, n) = (w as usize, w as usize * h as usize);
        let regiao = crate::compositor::Region {
            x: caixa.x0 as u32,
            y: caixa.y0 as u32,
            w: (caixa.x1 - caixa.x0) as u32,
            h: (caixa.y1 - caixa.y0) as u32,
        };
        let pixeis = || {
            (caixa.y0..caixa.y1)
                .flat_map(move |y| (caixa.x0..caixa.x1).map(move |x| y * wu + x))
                .zip(razao.iter().copied())
                .filter(|&(_, k)| k < 1.0)
        };
        let mut mudou = false;
        if let Some(e) = self.heights.get_mut(&layer)
            && e.len() == n
            && pixeis().any(|(i, _)| e[i] != 0.0)
        {
            let dst = crate::tool::paint::plane_fork::fork_heights(
                e,
                &self.undo.write_state,
                layer,
                (w, h),
                Some(regiao),
            );
            for (i, k) in pixeis() {
                dst[i] *= k;
            }
            self.declare_wrote(Some(regiao));
            mudou = true;
        }
        if let Some(e) = self.covers.get_mut(&layer)
            && e.len() == n
            && pixeis().any(|(i, _)| e[i] != 0)
        {
            let dst = crate::tool::paint::plane_fork::fork_covers(
                e,
                &self.undo.write_state,
                layer,
                (w, h),
                Some(regiao),
            );
            for (i, k) in pixeis() {
                // Para baixo, como a cobertura da Borracha normal (`erase_dab_height`).
                dst[i] = (f32::from(dst[i]) * k) as u8;
            }
            self.declare_wrote(Some(regiao));
            mudou = true;
        }
        // A caixa já é suja de quem chama (a óptica da sessão, ou o `mark_dirty` da tinta morta).
        if mudou {
            self.sync_relief_flags();
        }
    }
}
