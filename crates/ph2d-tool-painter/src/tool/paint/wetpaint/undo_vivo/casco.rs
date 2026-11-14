//! **O casco de cada passo** (B02.5): a caixa dos dabs do passo sobrevive ao tecto do undo vivo. Desfazer ou refazer um
//! passo que já saiu do histórico só enterra dentro do casco dele; fora, a tela fica a de antes do restauro (a da tinta
//! viva, que a água longe continua): a tela de antes de cada restauro é sempre a que a tinta viva mostra, e o enterro
//! seguinte compara com ela.

use super::*;

/// Os cascos dos passos fechados, pela chave do passo: uma caixa cada, só os que o documento alcança.
#[derive(Clone, Default)]
pub(crate) struct Cascos(Vec<(usize, CaixaDeCelulas)>);

impl Cascos {
    fn de(&self, chave: usize) -> Option<CaixaDeCelulas> {
        self.0.iter().find(|(c, _)| *c == chave).map(|(_, k)| *k)
    }
}

/// Guarda o casco do passo de cima (o do registo aberto), com o alcance da orla; larga os que o documento já não alcança
/// (`chave < desde`) e os de um ramo abandonado (`chave >= ate`).
pub(super) fn guarda_o_casco(sess: &mut WetSession, desde: usize, ate: usize) {
    let (gw, gh) = (sess.grid.0 as i64, sess.grid.1 as i64);
    let passo = sess
        .antes_do_passo
        .last()
        .filter(|p| p.depois.is_none())
        .map(|p| p.chave);
    if let (Some(chave), Some([x0, y0, x1, y1])) = (passo, sess.engine.casco_dos_dabs_do_aberto()) {
        let r = ALCANCE_DA_ORLA as i64;
        let k = (
            (x0 - r).clamp(1, gw) as usize,
            (y0 - r).clamp(1, gh) as usize,
            (x1 - 1 + r).clamp(1, gw) as usize,
            (y1 - 1 + r).clamp(1, gh) as usize,
        );
        sess.cascos.0.retain(|(c, _)| *c != chave);
        sess.cascos.0.push((chave, k));
    }
    sess.cascos.0.retain(|(c, _)| (desde..ate).contains(c));
}

/// O casco do passo `chave`, se ele pintou com esta sessão: só nele um restauro sem passo enterra.
pub(in crate::tool::paint::wetpaint) fn casco_de(
    sess: &WetSession,
    chave: usize,
) -> Option<CaixaDeCelulas> {
    #[cfg(test)]
    if super::casco_tests::sem_casco::get() {
        return None;
    }
    sess.cascos.de(chave)
}

impl PainterTool {
    /// Fora do `casco`, a tela volta a `velha` (a de antes do restauro): o restauro só muda o que o passo pintou.
    pub(in crate::tool::paint::wetpaint) fn wet_fora_do_casco_fica(
        &mut self,
        sess: &WetSession,
        velha: &[u8],
        (x0, y0, x1, y1): CaixaDeCelulas,
    ) {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        if velha.len() != w * h * 4 || self.canvas_rgba.len() != velha.len() {
            return;
        }
        let q = usize::from(sess.ratio);
        let (px0, px1) = (((x0 - 1) * q).min(w), (x1 * q).min(w));
        let (py0, py1) = (((y0 - 1) * q).min(h), (y1 * q).min(h));
        let fora = |y: usize| {
            if (py0..py1).contains(&y) {
                [(0, px0), (px1, w)]
            } else {
                [(0, w), (w, w)]
            }
        };
        let faixa = |y: usize, (a, b): (usize, usize)| (y * w + a) * 4..(y * w + b) * 4;
        let difere = (0..h).any(|y| {
            fora(y)
                .into_iter()
                .any(|s| velha[faixa(y, s)] != self.canvas_rgba[faixa(y, s)])
        });
        if !difere {
            return;
        }
        // Dentro do undo/redo (o journal reseta no fim): a cópia é paralela e a tela entra pela porta do restauro.
        let mut nova = Arc::clone(&self.canvas_rgba);
        let tela = crate::tool::paint::plane_fork::fork_plano_de_trabalho(&mut nova);
        for y in 0..h {
            for s in fora(y) {
                tela[faixa(y, s)].copy_from_slice(&velha[faixa(y, s)]);
            }
        }
        self.instala_a_tela_do_restauro(nova);
    }
}
