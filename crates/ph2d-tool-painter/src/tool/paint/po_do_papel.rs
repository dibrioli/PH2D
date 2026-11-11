//! **O pó pousado** do traço ([`ph2d_painter_brush::texture::PoDoPapel`], dono 09/11): no 1.º lote de um traço cujo
//! grão enche o dente ([`ph2d_painter_brush::GrainLaw::enche_o_dente`]), o alfa da camada no pen-down — a tela que o
//! `before` do desfazer já segura — desfocado à escala de um vale ([`RAIO_DA_CAIXA`]). Imutável até o pen-up: a borracha, o desfazer e mover a camada
//! ficam coerentes sem plano novo, e o que o próprio traço pousa só conta no traço seguinte.

use crate::tool::PainterTool;
use ph2d_painter_brush::BrushSpec;
use ph2d_painter_brush::texture::{PoDoPapel, RAIO_DA_CAIXA, caixa};
use rayon::prelude::*;

impl PainterTool {
    /// Prepara o pó do traço para o pincel do lote, uma vez por traço; nada se o pincel não enche o dente.
    pub(super) fn prepara_o_po(&mut self, brush: &BrushSpec, w: u32, h: u32) {
        if self.paint.po_do_papel.is_some()
            || !brush.texture.is_active()
            || !brush.texture.lei.enche_o_dente()
        {
            return;
        }
        let (w, h) = (w as usize, h as usize);
        let base = self
            .tela_de_antes_do_traco(true)
            .filter(|b| w > 0 && b.len() == w * h * 4);
        let plano = base.map_or_else(Vec::new, |b| po_da_tela(&b, w, h));
        self.paint.po_do_papel = Some(plano);
    }
}

/// O pó do traço (o campo `po_do_papel`) numa tela de largura `w` (`None` = sem pó: o papel de sempre). Função do
/// CAMPO, não do tool: as rotas seguram-no ao lado dos empréstimos mutáveis da tela.
pub(super) fn po(plano: Option<&Vec<u8>>, w: u32) -> Option<PoDoPapel<'_>> {
    plano.filter(|p| !p.is_empty()).map(|plano| PoDoPapel {
        plano,
        largura: w as usize,
    })
}

/// Colunas por faixa do passe vertical: 32 somas corridas lado a lado (128 B por linha), sem transpor a tela.
const FAIXA: usize = 32;

/// O pó da tela `rgba` (`w`×`h`): o alfa desfocado por três caixas em cada eixo; vazio se a camada não tem tinta.
fn po_da_tela(rgba: &[u8], w: usize, h: usize) -> Vec<u8> {
    if !rgba.as_chunks::<4>().0.iter().any(|p| p[3] != 0) {
        return Vec::new();
    }
    let mut a: Vec<f32> = rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| f32::from(p[3]) / 255.0)
        .collect();
    a.par_chunks_mut(w)
        .for_each_init(|| vec![0.0f32; w], |tmp, linha| {
            for _ in 0..3 {
                caixa(linha, tmp, RAIO_DA_CAIXA);
                linha.copy_from_slice(tmp);
            }
        });
    // Cada faixa copia as suas colunas (linha a linha, contíguas), corre as três caixas verticais e quantiza.
    let mut faixas = vec![0u8; w * h];
    faixas
        .par_chunks_mut(h * FAIXA)
        .enumerate()
        .for_each_init(
            || (vec![0.0f32; h * FAIXA], vec![0.0f32; h * FAIXA]),
            |(f, g), (s, saida)| {
                let x0 = s * FAIXA;
                let l = saida.len() / h;
                let (f, g) = (&mut f[..h * l], &mut g[..h * l]);
                for (y, dst) in f.chunks_exact_mut(l).enumerate() {
                    dst.copy_from_slice(&a[y * w + x0..y * w + x0 + l]);
                }
                for _ in 0..3 {
                    caixa_em_faixa(f, g, l, RAIO_DA_CAIXA);
                    f.copy_from_slice(g);
                }
                for (o, v) in saida.iter_mut().zip(f.iter()) {
                    *o = (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                }
            },
        );
    let mut out = vec![0u8; w * h];
    out.par_chunks_mut(w).enumerate().for_each(|(y, linha)| {
        for (s, faixa) in faixas.chunks(h * FAIXA).enumerate() {
            let l = faixa.len() / h;
            linha[s * FAIXA..s * FAIXA + l].copy_from_slice(&faixa[y * l..(y + 1) * l]);
        }
    });
    out
}

/// [`caixa`] em cada uma das `l` colunas de `src` (`n`×`l`, linha a linha): a mesma soma corrida, na mesma ordem de
/// `y`, por coluna — o mesmo bit que a caixa sobre a coluna transposta.
fn caixa_em_faixa(src: &[f32], dst: &mut [f32], l: usize, r: usize) {
    let n = src.len() / l;
    if n == 0 || dst.len() != src.len() {
        return;
    }
    let mut soma = [-0.0f32; FAIXA];
    let soma = &mut soma[..l];
    for linha in src[..(r.min(n - 1) + 1) * l].chunks_exact(l) {
        for (s, v) in soma.iter_mut().zip(linha) {
            *s += v;
        }
    }
    let mut lo = 0usize;
    let mut hi = r.min(n - 1);
    for (y, d) in dst.chunks_exact_mut(l).enumerate() {
        let k = (hi - lo + 1) as f32;
        for (d, s) in d.iter_mut().zip(soma.iter()) {
            *d = s / k;
        }
        if y + r + 1 < n {
            hi += 1;
            for (s, v) in soma.iter_mut().zip(&src[hi * l..(hi + 1) * l]) {
                *s += v;
            }
        }
        if y >= r {
            for (s, v) in soma.iter_mut().zip(&src[lo * l..(lo + 1) * l]) {
                *s -= v;
            }
            lo += 1;
        }
    }
}

#[cfg(test)]
#[path = "po_do_papel_tests.rs"]
mod tests;
