//! ⭐⭐ **A oclusão PRÓPRIA de uma peça** — por vértice, a fracção do céu, ponderada pelo cosseno,
//! que a própria peça tapa (os vincos onde peças fundidas encostam, o furo de um toro), por
//! [`RAIOS_PROPRIOS`] raios BINÁRIOS (bate / não bate) distribuídos pelo cosseno e marchados no
//! [`Volume`] dela. O conjunto é o MESMO em todos os vértices: sem ruído de vértice para vértice.
//!
//! ⛔ Recusados, medidos (03/10) contra o Cycles — peças fundidas (`L`, bola enterrada, toro) e as
//! malhas reais dos tubos das cenas `28` e `37`, médio / onde o Cycles `< 0,9`:
//!
//! | lei | fundidas | tubos |
//! |---|---|---|
//! | Quilez (`5` passos na normal, a que a casa assava) | `0,05–0,11` / `0,14–0,21` | `0,11–0,15` / `0,09–0,12` |
//! | `48` cones moles de `0,2` rad | `0,006–0,011` / `0,008–0,031` | `0,042–0,044` / `0,064–0,067` (escuros: leem DISTÂNCIA, e o campo por fórmula não é uma) |
//! | `128` raios binários no volume DIVIDIDO por `|∇f|` | `0,004` / `0,006–0,010` | `0,016–0,020` / `0,024–0,029` (claros, viés `+0,014`) |
//! | **`128` raios binários no volume CRU** | **`0,004`** / **`0,006–0,010`** | **`0,013–0,016`** / **`0,017–0,019`** (viés `−0,002..+0,003`) |

use crate::Volume;
use rayon::prelude::*;

/// Os raios por vértice. `64` dá `0,006` / `0,009–0,017` nas fundidas; `256` dá `0,003` / `0,004–0,006`.
pub const RAIOS_PROPRIOS: usize = 128;

/// ⭐⭐⭐ A visibilidade do céu em cada vértice (`1` = aberto). `alcance` = até onde um raio marcha
/// (a diagonal da peça chega). Só lê o SINAL do volume: um campo que não é distância só abranda a
/// marcha.
#[must_use]
pub fn visibilidade_propria(
    vol: &Volume,
    pos: &[[f32; 3]],
    nrm: &[[f32; 3]],
    alcance: f32,
) -> Vec<f32> {
    let h = vol.passo();
    let dirs: Vec<[f32; 3]> = (0..RAIOS_PROPRIOS)
        .map(|j| {
            let u = (j as f32 + 0.5) / RAIOS_PROPRIOS as f32;
            let phi = (j as f32 + 0.5) * 2.399_963;
            let r = u.sqrt();
            [r * phi.cos(), r * phi.sin(), (1.0 - u).sqrt()]
        })
        .collect();
    pos.par_iter()
        .zip(nrm)
        .map(|(p, n)| {
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2])
                .sqrt()
                .max(1.0e-12);
            let n = n.map(|c| c / l);
            let a = if n[0].abs() < 0.9 {
                [1.0, 0.0, 0.0]
            } else {
                [0.0, 1.0, 0.0]
            };
            let t = unit(cross(n, a));
            let b = cross(n, t);
            // Parte um quinto de passo FORA da superfície: o volume é trilinear e erra junto dela.
            let o: [f32; 3] = std::array::from_fn(|e| p[e] + n[e] * 0.2 * h);
            let livres = dirs
                .iter()
                .filter(|d| {
                    let w: [f32; 3] =
                        std::array::from_fn(|e| t[e] * d[0] + b[e] * d[1] + n[e] * d[2]);
                    !bate(vol, o, w, h, alcance)
                })
                .count();
            livres as f32 / RAIOS_PROPRIOS as f32
        })
        .collect()
}

/// O raio `o + s·w` toca a peça antes de `alcance`? ⚠️ Fora da caixa do volume (convexa, contém a
/// peça) ele não volta a entrar.
fn bate(vol: &Volume, o: [f32; 3], w: [f32; 3], h: f32, alcance: f32) -> bool {
    let mut s = 0.5 * h;
    while s < alcance {
        let q: [f32; 3] = std::array::from_fn(|e| o[e] + w[e] * s);
        if (0..3).any(|e| q[e] < vol.lo[e] || q[e] > vol.hi[e]) {
            return false;
        }
        let d = vol.distancia(q);
        if d < 0.05 * h {
            return true;
        }
        s += d.max(0.1 * h);
    }
    false
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn unit(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
        .sqrt()
        .max(1.0e-12);
    v.map(|c| c / l)
}
