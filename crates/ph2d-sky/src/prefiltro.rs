//! ⭐⭐ **O PRÉ-FILTRO** — o atlas do [`crate::Ceu`], construído uma vez por panorama.
//!
//! - Nível `0` (`α = 0`, o espelho): o panorama lido bilinear no centro de cada texel.
//! - Nível `k > 0`: a média da radiância sob o lóbulo GGX de `α = (k/(NIVEIS−1))²`, com `N = V = R`,
//!   pesada por `N·L` — a MESMA pergunta do pré-filtro do estúdio (`BoxPrefilter`):
//!   - `α ≥` [`ALFA_EXACTA`]: a CONVOLUÇÃO EXACTA ([`convolucao`]) sobre o nível da pirâmide cujo
//!     texel é `≤ 1/20` do `α` — sem ruído e com a cauda longa do GGX inteira (um sol fora do
//!     lóbulo ainda brilha nele);
//!   - abaixo: amostragem de Hammersley FILTRADA (Colbert & Křivánek, GPU Gems 3, cap. 20) — cada
//!     amostra lê o nível cujo texel tem o ângulo sólido dela. ⛔ Medido (02/10): com o viés `+1`
//!     de quem a publicou o erro dobrava sobre o equiretangular; aqui é `0`.
//! - Irradiância: NÃO tem mapa próprio — é o nível `α = 1` (ver `Ceu::irradiance`).
//!
//! O erro de cada passo contra a quadratura crua mede-se nos gates (`crate::tests`).

use rayon::prelude::*;

use crate::{ATLAS_H, ATLAS_W, LADOS, NIVEIS, Panorama, Rgb, X0, Y0, de_oct};

/// Amostras do lóbulo por texel nos níveis amostrados, por `α`: um lóbulo de `0,3°` cabe num texel do
/// panorama e `64` amostras já o cobrem; os largos pedem mais. ⛔ Medido (02/10, floresta): `256` dava
/// `3,4 %` de mediana contra a convolução a `α = 0,03`, `1024` dá `0,4 %`; e `1024` em todo nível
/// custava `1,5 s` por céu.
pub(crate) fn amostras(alpha: f32) -> u32 {
    if alpha < 0.01 {
        256
    } else if alpha < 0.03 {
        512
    } else {
        1024
    }
}

/// O `α` a partir do qual o nível é a convolução exacta. ⛔ Medido (02/10, interior): a amostragem
/// deixava escapar as lâmpadas pequenas — `68 %` de erro máximo a `α = 0,14`.
pub(crate) const ALFA_EXACTA: f32 = 0.09;

/// A largura máxima do panorama sobre o qual a irradiância da régua é somada.
#[cfg(test)]
const LARGURA_IRR: u32 = 128;

/// A pirâmide do panorama: cada nível é o anterior reduzido `2×2`, pesado pelo ângulo sólido.
pub(crate) fn piramide(p: &Panorama) -> Vec<Panorama> {
    let mut v = vec![p.clone()];
    loop {
        let a = v.last().expect("o nível 0 existe");
        if a.altura < 2 || a.largura < 2 || a.altura % 2 != 0 || a.largura % 2 != 0 {
            break;
        }
        let (w, h) = (a.largura / 2, a.altura / 2);
        let mut rgb = Vec::with_capacity((w * h) as usize);
        for y in 0..h {
            let (w0, w1) = (a.angulo_solido(2 * y), a.angulo_solido(2 * y + 1));
            for x in 0..w {
                let t = |xx: u32, yy: u32| a.rgb[(yy * a.largura + xx) as usize];
                let (c00, c10) = (t(2 * x, 2 * y), t(2 * x + 1, 2 * y));
                let (c01, c11) = (t(2 * x, 2 * y + 1), t(2 * x + 1, 2 * y + 1));
                rgb.push([0, 1, 2].map(|i| {
                    (((f64::from(c00[i]) + f64::from(c10[i])) * w0
                        + (f64::from(c01[i]) + f64::from(c11[i])) * w1)
                        / (2.0 * (w0 + w1))) as f32
                }));
            }
        }
        v.push(Panorama {
            largura: w,
            altura: h,
            rgb,
        });
    }
    v
}

/// A leitura trilinear na pirâmide, no nível contínuo `lod`.
fn trilinear(pir: &[Panorama], d: [f32; 3], lod: f32) -> Rgb {
    let topo = (pir.len() - 1) as f32;
    let lod = lod.clamp(0.0, topo);
    let l0 = lod.floor() as usize;
    let t = lod - l0 as f32;
    let a = pir[l0].radiancia(d);
    if t <= 0.0 || l0 + 1 >= pir.len() {
        return a;
    }
    let b = pir[l0 + 1].radiancia(d);
    [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t)
}

/// Uma base ortonormada em torno de `n` (Duff et al. 2017).
fn base(n: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let s = if n[2] >= 0.0 { 1.0 } else { -1.0 };
    let a = -1.0 / (s + n[2]);
    let b = n[0] * n[1] * a;
    (
        [1.0 + s * n[0] * n[0] * a, s * b, -s * n[0]],
        [b, s + n[1] * n[1] * a, -n[1]],
    )
}

/// ⭐ **O lóbulo GGX de `α` em torno de `r`**, com `m` amostras — a média pesada por `N·L`.
/// `filtrada = false` lê sempre o nível `0` (a quadratura crua dos gates).
pub(crate) fn lobo(pir: &[Panorama], r: [f32; 3], alpha: f32, m: u32, filtrada: bool) -> Rgb {
    let rn = {
        let l = f64::from(r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
        [f64::from(r[0]) / l, f64::from(r[1]) / l, f64::from(r[2]) / l]
    };
    let (t, b) = base(rn);
    let a2 = f64::from(alpha) * f64::from(alpha);
    let p0 = &pir[0];
    let pi = std::f64::consts::PI;
    let omega_texel = 2.0 * pi / f64::from(p0.largura) * (pi / f64::from(p0.altura));
    let (mut soma, mut peso) = ([0.0f64; 3], 0.0f64);
    for i in 0..m {
        let u1 = (f64::from(i) + 0.5) / f64::from(m);
        let u2 = f64::from(i.reverse_bits()) / f64::from(u32::MAX);
        let ch2 = (1.0 - u1) / (1.0 + (a2 - 1.0) * u1);
        let ch = ch2.max(0.0).sqrt();
        let sh = (1.0 - ch2).max(0.0).sqrt();
        let phi = 2.0 * pi * u2;
        let h = [sh * phi.cos(), sh * phi.sin(), ch];
        let l = [2.0 * ch * h[0], 2.0 * ch * h[1], 2.0 * ch * h[2] - 1.0];
        if l[2] <= 0.0 {
            continue;
        }
        let lw = [0, 1, 2].map(|k| (t[k] * l[0] + b[k] * l[1] + rn[k] * l[2]) as f32);
        let c = if filtrada {
            let den = ch2 * (a2 - 1.0) + 1.0;
            let pdf = a2 / (pi * den * den) / 4.0;
            let omega_s = 1.0 / (f64::from(m) * pdf);
            let sin_t = (1.0 - f64::from(lw[1]) * f64::from(lw[1]))
                .max(0.0)
                .sqrt()
                .max(1.0 / f64::from(p0.altura));
            let lod = (0.5 * (omega_s / (omega_texel * sin_t)).log2()).max(0.0);
            trilinear(pir, lw, lod as f32)
        } else {
            p0.radiancia(lw)
        };
        for k in 0..3 {
            soma[k] += f64::from(c[k]) * l[2];
        }
        peso += l[2];
    }
    soma.map(|s| (s / peso.max(1.0e-30)) as f32)
}

/// ⭐ **A convolução EXACTA** do lóbulo GGX de `α` em torno de `r` sobre as `fontes`
/// (direcção, `L·Ω`): `Σ L·Ω·K / Σ Ω·K`, `K(l) = (r·l)·D(h)`. É o limite do [`lobo`].
pub(crate) fn convolucao(fontes: &[([f64; 3], [f64; 4])], r: [f32; 3], alpha: f32) -> Rgb {
    let l = f64::from(r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
    let r = r.map(|c| f64::from(c) / l);
    let a2 = f64::from(alpha) * f64::from(alpha);
    let (mut s, mut w) = ([0.0f64; 3], 0.0f64);
    for (d, c) in fontes {
        let rl = r[0] * d[0] + r[1] * d[1] + r[2] * d[2];
        if rl <= 0.0 {
            continue;
        }
        // `(r·h)² = (1 + r·l)/2` com `h = (r + l)/|r + l|`.
        let nh2 = (1.0 + rl) * 0.5;
        let den = nh2 * (a2 - 1.0) + 1.0;
        let k = rl / (den * den);
        for i in 0..3 {
            s[i] += c[i] * k;
        }
        w += c[3] * k;
    }
    s.map(|v| (v / w) as f32)
}

/// As fontes da convolução a `α`: o nível da pirâmide com `altura ≥ 20/α` (o texel `≤ α/20`), e
/// nunca abaixo de `64` linhas (uma lâmpada pequena num texel de `6°` erra `5 %`, medido no interior),
/// cada texel com `(direcção, [L·Ω, Ω])`.
pub(crate) fn fontes_conv(pir: &[Panorama], alpha: f32) -> Vec<([f64; 3], [f64; 4])> {
    let alvo = ((20.0 / alpha).ceil() as u32).max(64);
    let p = pir
        .iter()
        .rev()
        .find(|p| p.altura >= alvo)
        .unwrap_or(&pir[0]);
    let mut v = Vec::with_capacity(p.rgb.len());
    for y in 0..p.altura {
        let om = p.angulo_solido(y);
        for x in 0..p.largura {
            let c = p.rgb[(y * p.largura + x) as usize];
            let d = p.direcao(x, y).map(f64::from);
            let n = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            v.push((
                d.map(|k| k / n),
                [f64::from(c[0]) * om, f64::from(c[1]) * om, f64::from(c[2]) * om, om],
            ));
        }
    }
    v
}

/// A RÉGUA da irradiância (os gates): direcção e `L·Ω/π` de cada texel do panorama reduzido.
#[cfg(test)]
pub(crate) fn fontes_irr(pir: &[Panorama]) -> Vec<([f32; 3], [f64; 3])> {
    let p = pir
        .iter()
        .find(|p| p.largura <= LARGURA_IRR)
        .unwrap_or_else(|| pir.last().expect("o nível 0 existe"));
    let mut v = Vec::with_capacity(p.rgb.len());
    for y in 0..p.altura {
        let w = p.angulo_solido(y) / std::f64::consts::PI;
        for x in 0..p.largura {
            let c = p.rgb[(y * p.largura + x) as usize];
            v.push((p.direcao(x, y), c.map(|k| f64::from(k) * w)));
        }
    }
    v
}

/// `E(n)/π` sobre as fontes.
#[cfg(test)]
pub(crate) fn irradiancia(fontes: &[([f32; 3], [f64; 3])], n: [f32; 3]) -> Rgb {
    let mut s = [0.0f64; 3];
    for (d, c) in fontes {
        let cn = f64::from(n[0] * d[0] + n[1] * d[1] + n[2] * d[2]);
        if cn > 0.0 {
            for k in 0..3 {
                s[k] += c[k] * cn;
            }
        }
    }
    s.map(|v| v as f32)
}

/// A direcção do centro do texel `(i, j)` de um mapa de lado `n` — com a margem: `i, j ∈ [0, n+2)`.
pub(crate) fn centro(i: u32, j: u32, n: u32) -> [f32; 3] {
    let c = |k: u32| ((k as f32 - 0.5) / n as f32) * 2.0 - 1.0;
    de_oct(c(i), c(j))
}

/// ⭐ O atlas inteiro.
pub(crate) fn atlas(p: &Panorama) -> Vec<[f32; 4]> {
    let pir = piramide(p);
    let mut atlas = vec![[0.0, 0.0, 0.0, 1.0]; (ATLAS_W * ATLAS_H) as usize];
    for k in 0..NIVEIS {
        let n = LADOS[k];
        let r = k as f32 / (NIVEIS - 1) as f32;
        let alpha = r * r;
        let conv = (k > 0 && alpha >= ALFA_EXACTA).then(|| fontes_conv(&pir, alpha));
        let linhas: Vec<Vec<Rgb>> = (0..n + 2)
            .into_par_iter()
            .map(|j| {
                (0..n + 2)
                    .map(|i| {
                        let d = centro(i, j, n);
                        if k == 0 {
                            pir[0].radiancia(d)
                        } else if let Some(f) = &conv {
                            convolucao(f, d, alpha)
                        } else {
                            lobo(&pir, d, alpha, amostras(alpha), true)
                        }
                    })
                    .collect()
            })
            .collect();
        for (j, linha) in linhas.iter().enumerate() {
            for (i, c) in linha.iter().enumerate() {
                atlas[((Y0[k] + j as u32) * ATLAS_W + X0[k] + i as u32) as usize] =
                    [c[0], c[1], c[2], 1.0];
            }
        }
    }
    atlas
}
