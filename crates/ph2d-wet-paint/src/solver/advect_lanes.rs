//! **O transporte em blocos de [`L`] células sem ramos** (n.º 7, `N7_ITEM1_sim6.md`): a mesma conta do
//! [`super::transport_row_escalar`] por célula, mas em duas fases por bloco — a RECOLHA (retro-traço, índices
//! e as leituras dos quatro cantos, com um índice seguro onde a célula não puxa) e a CONTA em arrays fixos
//! que o LLVM vetoriza; o «puxa ou não» (`active`, retro-traço fora da folha, `want > 0`) vira uma seleção no
//! fim em vez de três ramos mal previstos.
//!
//! ⚠️ **Ao bit:** cada lane faz as mesmas operações `f64` na mesma ordem da rota escalar (sem FMA: o Rust
//! nunca contrai; sem reassociar). As duas diferenças de forma são exatas: a escala da fonte aplica-se sempre
//! (`w · 1 == w` ao bit, o atalho escalar é só um teste a menos) e as contas de uma lane que não puxa são
//! descartadas pela seleção. O `km_mean` fica na rota escalar. Gate
//! `o_transporte_em_lanes_da_o_escalar_ao_bit`.

use super::Ctx;
use std::sync::atomic::AtomicU8;

/// `1` = o transporte em lanes (produto) · `0` = a rota escalar (o oráculo e a sonda `v0`).
#[doc(hidden)]
pub static TRANSPORTE_LANES: AtomicU8 = AtomicU8::new(1);

/// Células por bloco.
const L: usize = 8;

/// Lê `v[idx]` ou `0` fora da fatia — só lanes descartadas caem fora (a rota escalar indexa as válidas).
#[inline(always)]
fn ler(v: &[f32], idx: usize) -> f32 {
    v.get(idx).copied().unwrap_or(0.0)
}

#[inline(always)]
fn ld(v: &[[f32; 3]], idx: usize) -> [f32; 3] {
    v.get(idx).copied().unwrap_or([0.0; 3])
}

/// As células `x..x+L` da linha `y` (índice global `i` da primeira), escritas em `fr`/`sr`/`cr` a partir de `x`.
#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn bloco(
    c: &Ctx,
    y: i32,
    x: i32,
    i: usize,
    fr: &mut [f32],
    sr: &mut [f32],
    cr: &mut [[f32; 3]],
    outflow: &[f32],
    obase: usize,
    uv: &[[f32; 2]],
    ubase: usize,
) {
    let s = c.s;
    let (film, susp, rgb) = (c.film, c.susp, c.susp_rgb);
    let (f0, m0, of) = (
        &film[i..i + L],
        &susp[i..i + L],
        &outflow[i - obase..i - obase + L],
    );
    let (act, uvc) = (&c.active[i..i + L], &uv[i - ubase..i - ubase + L]);
    let xo = x as usize;
    if act.iter().all(|&a| a == 0) {
        // Ninguém puxa: só o que sai (a mesma conta da rota escalar sem o ramo ativo).
        for k in 0..L {
            let (f, m, o) = (f64::from(f0[k]), f64::from(m0[k]), f64::from(of[k]));
            let leave = if o > 1.0 { 1.0 } else { o };
            fr[xo + k] = (f - f * leave) as f32;
            sr[xo + k] = (m - m * leave) as f32;
            cr[xo + k] = rgb[i + k];
        }
        return;
    }
    let mut puxa = [false; L];
    let (mut fx, mut fy) = ([0.0f64; L], [0.0f64; L]);
    // Os quatro cantos × (film, susp, outflow, r, g, b).
    let mut cf = [[0.0f32; L]; 4];
    let mut cm = [[0.0f32; L]; 4];
    let mut co = [[0.0f32; L]; 4];
    let mut base = [0usize; L];
    let yf = f64::from(y);
    for k in 0..L {
        let sx = f64::from(x + k as i32) - f64::from(uvc[k][0]);
        let sy = yf - f64::from(uvc[k][1]);
        let dentro = !(sx < 1.0 || sx > c.wf || sy < 1.0 || sy > c.hf);
        let (x0, y0) = (sx as i64, sy as i64);
        fx[k] = sx - x0 as f64;
        fy[k] = sy - y0 as f64;
        let ok = act[k] != 0 && dentro;
        puxa[k] = ok;
        let i00 = if ok { x0 as usize + y0 as usize * s } else { i };
        base[k] = i00;
        let idx = [i00, i00 + 1, i00 + s, i00 + s + 1];
        for q in 0..4 {
            let j = idx[q];
            cf[q][k] = ler(film, j);
            cm[q][k] = ler(susp, j);
            co[q][k] = ler(outflow, j.wrapping_sub(obase));
        }
    }
    let mut nf = [0.0f64; L];
    let mut ns = [0.0f64; L];
    let mut nc = [[0.0f32; L]; 3];
    let mut troca = [false; L];
    let mut pk = [[0.0f64; 4]; L];
    for k in 0..L {
        let (f, m, o) = (f64::from(f0[k]), f64::from(m0[k]), f64::from(of[k]));
        let leave = if o > 1.0 { 1.0 } else { o };
        let new_film = f - f * leave;
        let new_susp = m - m * leave;
        let (ax, ay) = (fx[k], fy[k]);
        let w = [
            (1.0 - ax) * (1.0 - ay),
            ax * (1.0 - ay),
            (1.0 - ax) * ay,
            ax * ay,
        ];
        let mut kk = [0.0f64; 4];
        let mut p = [0.0f64; 4];
        for q in 0..4 {
            let oq = f64::from(co[q][k]);
            let escala = if oq > 1.0 { 1.0 / oq } else { 1.0 };
            kk[q] = w[q] * escala;
            p[q] = f64::from(cm[q][k]) * kk[q];
        }
        let want = p[0] + p[1] + p[2] + p[3];
        pk[k] = p;
        let film_in = f64::from(cf[0][k]) * kk[0]
            + f64::from(cf[1][k]) * kk[1]
            + f64::from(cf[2][k]) * kk[2]
            + f64::from(cf[3][k]) * kk[3];
        let ok = puxa[k];
        let entra = ok && want > 0.0;
        troca[k] = entra;
        nf[k] = if ok { new_film + film_in } else { new_film };
        ns[k] = if entra { new_susp + want } else { new_susp };
    }
    // A cor só onde alguma lane troca (a rota escalar só a lê com `want > 0`).
    if troca.iter().any(|&t| t) {
        for k in 0..L {
            let b = base[k];
            let cs = [
                ld(rgb, b),
                ld(rgb, b + 1),
                ld(rgb, b + s),
                ld(rgb, b + s + 1),
            ];
            let p = pk[k];
            let inv = 1.0 / (p[0] + p[1] + p[2] + p[3]);
            for ch in 0..3 {
                let v = (f64::from(cs[0][ch]) * p[0]
                    + f64::from(cs[1][ch]) * p[1]
                    + f64::from(cs[2][ch]) * p[2]
                    + f64::from(cs[3][ch]) * p[3])
                    * inv;
                nc[ch][k] = v as f32;
            }
        }
    }
    for k in 0..L {
        fr[xo + k] = nf[k] as f32;
        sr[xo + k] = ns[k] as f32;
        cr[xo + k] = if troca[k] {
            [nc[0][k], nc[1][k], nc[2][k]]
        } else {
            rgb[i + k]
        };
    }
}

/// O mesmo contrato do [`super::transport_row_escalar`] (sem `km_mean`): blocos de [`L`] por troço, a cauda
/// pela célula escalar.
#[allow(clippy::too_many_arguments)]
pub(super) fn transport_row_lanes(
    c: &Ctx,
    y: i32,
    fr: &mut [f32],
    sr: &mut [f32],
    cr: &mut [[f32; 3]],
    outflow: &[f32],
    obase: usize,
    uv: &[[f32; 2]],
    ubase: usize,
) {
    let (lo, hi) = c.span(y);
    if lo > hi {
        return;
    }
    let row0 = y as usize * c.s;
    let mut km = ([0.0f64; 12], [0.0f64; 4], [0.0f64; 3]);
    for (ra, rb) in c.runs(y) {
        let mut x = ra;
        while x + L as i32 - 1 <= rb {
            bloco(
                c,
                y,
                x,
                x as usize + row0,
                fr,
                sr,
                cr,
                outflow,
                obase,
                uv,
                ubase,
            );
            x += L as i32;
        }
        for xt in x..=rb {
            super::transport_cell(c, y, xt, fr, sr, cr, outflow, obase, uv, ubase, &mut km);
        }
    }
}
