//! **Vários [`super::box_blur`] do MESMO raio numa só passagem** (ADR-0173) — os canais viajam juntos
//! como `[f32; N]` (uma soma vectorial por texel em vez de `N` passagens), e por fora saem os mesmos
//! planos que os consumidores já leem:
//!
//! - [`box_blur4`]: os quatro campos `near` (e os quatro `far`) do rewet — presença e as três cores
//!   pesadas por ela;
//! - [`box_blur2`]: o campo molhado do Rewet por dono e a massa dele (`blur(v·m)` e `blur(m)`,
//!   [`super::super::watercolor_rewet_px::build_wet_field`]) — eram dois `box_blur` sobre a MESMA
//!   janela, e a aquarela da foto do dono (2026-09-23) punha-os em `~12 %` do quadro.
//!
//! ⭐ **Byte-idênticos a `N` `box_blur`** — cada canal faz as MESMAS somas `f32` pela MESMA ordem
//! (linha a linha a partir de `x = 0`, coluna a coluna a partir de `y = 0`), e a MESMA divisão, só que
//! lado a lado: as duas passagens são UMA função genérica em `N`, logo os dois tamanhos não podem
//! divergir um do outro. Os gates `quatro_borroes_juntos_dao_o_byte_de_quatro_separados` e
//! `dois_borroes_juntos_dao_o_byte_de_dois_separados` usam o [`super::box_blur`] como oráculo.

use rayon::prelude::*;

use super::FAIXA;

/// Os dois planos de rascunho de um borrão de `N` canais (o transposto e o prefixo).
type Rascunho<const N: usize> = std::cell::RefCell<(Vec<[f32; N]>, Vec<[f32; N]>)>;
/// Os quatro planos de um borrão de quatro canais, em pares (é a forma do `unzip` encaixado).
type QuatroPlanos = ((Vec<f32>, Vec<f32>), (Vec<f32>, Vec<f32>));

thread_local! {
    /// O rascunho do [`box_blur4`] — o irmão de quatro canais do rascunho do `box_blur`, pelas mesmas
    /// razões (reusado entre borrões e quadros, sem o `memset` de um plano novo).
    static RASCUNHO4: Rascunho<4> =
        const { std::cell::RefCell::new((Vec::new(), Vec::new())) };
    /// O rascunho do [`box_blur2`].
    static RASCUNHO2: Rascunho<2> =
        const { std::cell::RefCell::new((Vec::new(), Vec::new())) };
}

/// Quatro `box_blur` do mesmo raio. ⚠️ `try_borrow_mut`: numa espera do rayon esta thread pode roubar
/// outra tarefa que também borra — essa paga um rascunho novo em vez de entrar em pânico.
pub(in crate::tool::paint) fn box_blur4(
    src: [&[f32]; 4],
    w: usize,
    h: usize,
    radius: usize,
) -> [Vec<f32>; 4] {
    if radius == 0 || w == 0 || h == 0 {
        return src.map(<[f32]>::to_vec);
    }
    RASCUNHO4.with(|r| match r.try_borrow_mut() {
        Ok(mut g) => {
            let (tmp, pref) = &mut *g;
            box_blur4_com(src, w, h, radius, tmp, pref)
        }
        Err(_) => box_blur4_com(src, w, h, radius, &mut Vec::new(), &mut Vec::new()),
    })
}

/// Dois `box_blur` do mesmo raio (ver o cabeçalho).
pub(in crate::tool::paint) fn box_blur2(
    src: [&[f32]; 2],
    w: usize,
    h: usize,
    radius: usize,
) -> [Vec<f32>; 2] {
    if radius == 0 || w == 0 || h == 0 {
        return src.map(<[f32]>::to_vec);
    }
    RASCUNHO2.with(|r| match r.try_borrow_mut() {
        Ok(mut g) => {
            let (tmp, pref) = &mut *g;
            box_blur2_com(src, w, h, radius, tmp, pref)
        }
        Err(_) => box_blur2_com(src, w, h, radius, &mut Vec::new(), &mut Vec::new()),
    })
}

fn box_blur4_com(
    src: [&[f32]; 4],
    w: usize,
    h: usize,
    radius: usize,
    tmp: &mut Vec<[f32; 4]>,
    pref: &mut Vec<[f32; 4]>,
) -> [Vec<f32>; 4] {
    let bloco = passagens(src, w, h, radius, tmp, pref);
    // Os quatro planos nascem num `collect` só de um iterador INDEXADO (o `unzip` encaixado): cada
    // texel é escrito directamente na memória por iniciar dos quatro, sem `memset`.
    let pref = &pref[..];
    let ((c0, c1), (c2, c3)): QuatroPlanos = (0..w * h)
        .into_par_iter()
        .with_min_len(4096)
        .map(|i| {
            let v: [f32; 4] = coluna(pref, bloco, (w, h), radius, i);
            ((v[0], v[1]), (v[2], v[3]))
        })
        .unzip();
    [c0, c1, c2, c3]
}

fn box_blur2_com(
    src: [&[f32]; 2],
    w: usize,
    h: usize,
    radius: usize,
    tmp: &mut Vec<[f32; 2]>,
    pref: &mut Vec<[f32; 2]>,
) -> [Vec<f32>; 2] {
    let bloco = passagens(src, w, h, radius, tmp, pref);
    let pref = &pref[..];
    let (c0, c1): (Vec<f32>, Vec<f32>) = (0..w * h)
        .into_par_iter()
        .with_min_len(4096)
        .map(|i| {
            let v: [f32; 2] = coluna(pref, bloco, (w, h), radius, i);
            (v[0], v[1])
        })
        .unzip();
    [c0, c1]
}

/// A soma de dois vectores de canais, canal a canal (a MESMA soma `f32` que o `box_blur` faz).
#[inline]
fn soma<const N: usize>(a: [f32; N], b: [f32; N]) -> [f32; N] {
    std::array::from_fn(|k| a[k] + b[k])
}

/// A passagem HORIZONTAL (para `tmp`) e os prefixos da VERTICAL em FAIXAS de colunas (para `pref`);
/// devolve o tamanho de um bloco de faixa. A saída de cada texel sai do [`coluna`].
fn passagens<const N: usize>(
    src: [&[f32]; N],
    w: usize,
    h: usize,
    radius: usize,
    tmp: &mut Vec<[f32; N]>,
    pref: &mut Vec<[f32; N]>,
) -> usize {
    tmp.resize(w * h, [0.0; N]);
    tmp.par_chunks_mut(w).enumerate().for_each_init(
        || vec![[0.0f32; N]; w + 1],
        |p, (y, trow)| {
            let base = y * w;
            for x in 0..w {
                let i = base + x;
                p[x + 1] = soma(p[x], std::array::from_fn(|k| src[k][i]));
            }
            for (x, t) in trow.iter_mut().enumerate() {
                let lo = x.saturating_sub(radius);
                let hi = (x + radius).min(w - 1);
                let cnt = (hi - lo + 1) as f32;
                let (b, a) = (p[hi + 1], p[lo]);
                *t = std::array::from_fn(|k| (b[k] - a[k]) / cnt);
            }
        },
    );
    let nf = w.div_ceil(FAIXA);
    let bloco = (h + 1) * FAIXA;
    pref.resize(nf * bloco, [0.0; N]);
    let tmp = &tmp[..];
    pref.par_chunks_mut(bloco).enumerate().for_each(|(s, blk)| {
        let x0 = s * FAIXA;
        let sw = FAIXA.min(w - x0);
        blk[..sw].fill([0.0; N]);
        for y in 0..h {
            let (prev, next) = blk.split_at_mut((y + 1) * FAIXA);
            let prev = &prev[y * FAIXA..y * FAIXA + sw];
            let trow = &tmp[y * w + x0..y * w + x0 + sw];
            for ((n, p), t) in next[..sw].iter_mut().zip(prev).zip(trow) {
                *n = soma(*p, *t);
            }
        }
    });
    bloco
}

/// A caixa VERTICAL do texel `i`, lida dos prefixos das faixas.
#[inline]
fn coluna<const N: usize>(
    pref: &[[f32; N]],
    bloco: usize,
    (w, h): (usize, usize),
    radius: usize,
    i: usize,
) -> [f32; N] {
    let (y, x) = (i / w, i % w);
    let lo = y.saturating_sub(radius);
    let hi = (y + radius).min(h - 1);
    let cnt = (hi - lo + 1) as f32;
    let blk = &pref[(x / FAIXA) * bloco..];
    let j = x % FAIXA;
    let (b, a) = (blk[(hi + 1) * FAIXA + j], blk[lo * FAIXA + j]);
    std::array::from_fn(|k| (b[k] - a[k]) / cnt)
}
