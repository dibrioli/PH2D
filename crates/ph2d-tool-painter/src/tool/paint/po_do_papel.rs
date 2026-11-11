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
    let px = rgba.as_chunks::<4>().0;
    if w == 0 || !px.par_chunks(w).any(|l| l.iter().any(|p| p[3] != 0)) {
        return Vec::new();
    }
    // Cada linha converte o seu alfa e corre as três caixas em pingue-pongue (a → tmp → a → tmp, tmp volta à linha).
    let mut a = vec![0.0f32; w * h];
    a.par_chunks_mut(w).zip(px.par_chunks(w)).for_each_init(
        || vec![0.0f32; w],
        |tmp, (linha, src)| {
            if src.iter().all(|p| p[3] == 0) {
                return; // a caixa de zeros dá +0.0: o que a linha já tem
            }
            for (d, p) in linha.iter_mut().zip(src) {
                *d = f32::from(p[3]) / 255.0;
            }
            caixa(linha, tmp, RAIO_DA_CAIXA);
            caixa(tmp, linha, RAIO_DA_CAIXA);
            caixa(linha, tmp, RAIO_DA_CAIXA);
            linha.copy_from_slice(tmp);
        },
    );
    // Cada faixa copia as suas colunas (linha a linha, contíguas), corre as três caixas verticais e quantiza.
    let mut faixas = vec![0u8; w * h];
    faixas.par_chunks_mut(h * FAIXA).enumerate().for_each_init(
        || (vec![0.0f32; h * FAIXA], vec![0.0f32; h * FAIXA]),
        |(f, g), (s, saida)| {
            let x0 = s * FAIXA;
            let l = saida.len() / h;
            let (f, g) = (&mut f[..h * l], &mut g[..h * l]);
            for (y, dst) in f.chunks_exact_mut(l).enumerate() {
                dst.copy_from_slice(&a[y * w + x0..y * w + x0 + l]);
            }
            caixa_em_faixa(f, g, l, RAIO_DA_CAIXA);
            caixa_em_faixa(g, f, l, RAIO_DA_CAIXA);
            caixa_em_faixa(f, g, l, RAIO_DA_CAIXA);
            for (o, v) in saida.iter_mut().zip(g.iter()) {
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

/// Passo 2 do B49.1 (lei por JANELA, sem história): a média de cada caixa soma os `≤ 2r+1` vizinhos em ordem crescente,
/// de `0.0` — o bit de um texel não depende de onde a janela começa, e um ladrilho com apron `3r` dá o mesmo que a
/// tela inteira. Muda raros `u8` face à soma corrida de [`po_da_tela`]: liga-se a `prepara_o_po` com a prancha.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "B49.1 passo 2: liga-se a prepara_o_po com a prancha"
    )
)]
pub(super) fn po_na_janela(
    rgba: &[u8],
    w: usize,
    h: usize,
    (x0, y0, x1, y1): (usize, usize, usize, usize),
) -> Vec<u8> {
    let r = RAIO_DA_CAIXA;
    let (x1, y1) = (x1.min(w), y1.min(h));
    if x0 >= x1 || y0 >= y1 {
        return Vec::new();
    }
    let px = rgba.as_chunks::<4>().0;
    // Linhas e colunas que a janela lê: 3 caixas = apron 3r, cortado à tela.
    let (ly0, ly1) = (y0.saturating_sub(3 * r), (y1 + 3 * r).min(h));
    let (lx0, lx1) = (x0.saturating_sub(3 * r), (x1 + 3 * r).min(w));
    let lw = x1 - x0;
    // Passe horizontal: cada linha lida → as colunas [x0, x1).
    let mut a = vec![0.0f32; (ly1 - ly0) * lw];
    a.par_chunks_mut(lw).enumerate().for_each_init(
        || (Vec::new(), Vec::new()),
        |(f, g), (i, saida)| {
            let src = &px[(ly0 + i) * w + lx0..(ly0 + i) * w + lx1];
            if src.iter().all(|p| p[3] == 0) {
                return;
            }
            f.clear();
            f.extend(src.iter().map(|p| f32::from(p[3]) / 255.0));
            // Cada caixa estreita o domínio válido em r de cada lado (salvo na borda da tela).
            let mut ini = lx0;
            for passo in 0..3 {
                let (a0, a1) = if passo == 2 {
                    (x0, x1)
                } else {
                    let m = 2 - passo;
                    (x0.saturating_sub(m * r), (x1 + m * r).min(w))
                };
                media_em_janela(f, ini, w, a0, a1, 1, r, g);
                std::mem::swap(f, g);
                ini = a0;
            }
            saida.copy_from_slice(f);
        },
    );
    // Passe vertical: as `lw` colunas lado a lado, linha a linha.
    let (mut f, mut g) = (a, Vec::new());
    let mut ini = ly0;
    for passo in 0..3 {
        let (a0, a1) = if passo == 2 {
            (y0, y1)
        } else {
            let m = 2 - passo;
            (y0.saturating_sub(m * r), (y1 + m * r).min(h))
        };
        media_em_janela(&f, ini, h, a0, a1, lw, r, &mut g);
        std::mem::swap(&mut f, &mut g);
        ini = a0;
    }
    f.iter()
        .map(|v| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8)
        .collect()
}

/// A média em caixa de raio `r` (cortada a `[0, n)`, renormalizada) das posições `[a0, a1)` de `src`, que guarda as
/// posições `ini..` com `l` faixas por posição; a soma corre os vizinhos em ordem crescente, de `0.0`.
#[expect(clippy::too_many_arguments, reason = "o domínio da janela num eixo")]
fn media_em_janela(
    src: &[f32],
    ini: usize,
    n: usize,
    a0: usize,
    a1: usize,
    l: usize,
    r: usize,
    dst: &mut Vec<f32>,
) {
    dst.clear();
    dst.resize((a1 - a0) * l, 0.0);
    let uma = |(i, d): (usize, &mut [f32])| {
        let p = a0 + i;
        let (lo, hi) = (p.saturating_sub(r), (p + r).min(n - 1));
        for q in lo..=hi {
            for (d, v) in d.iter_mut().zip(&src[(q - ini) * l..(q - ini + 1) * l]) {
                *d += v;
            }
        }
        let k = (hi - lo + 1) as f32;
        for d in d.iter_mut() {
            *d /= k;
        }
    };
    if l == 1 {
        dst.chunks_mut(1).enumerate().for_each(uma); // a linha do passe horizontal já é um job
    } else {
        dst.par_chunks_mut(l)
            .with_min_len(16)
            .enumerate()
            .for_each(uma);
    }
}

#[cfg(test)]
#[path = "po_do_papel_tests.rs"]
mod tests;
