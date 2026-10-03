//! ⭐⭐ **A oclusão PRÓPRIA de uma peça** — por vértice, a fracção do céu, ponderada pelo cosseno,
//! que a própria peça tapa (os vincos onde peças fundidas encostam, o furo de um toro), por
//! [`CONES`] cones de abertura [`ALFA`] distribuídos pelo cosseno, marchados no [`Volume`] dela.
//! Cada cone vê `min_t clamp(½ + ½·d/(t·tan α))` — contínuo, sem os anéis de raios binários.

use crate::Volume;
use rayon::prelude::*;

/// Os cones por vértice.
pub const CONES: usize = 48;
/// A meia-abertura de cada cone (rad).
pub const ALFA: f32 = 0.2;
/// A razão entre dois passos de um cone.
pub const RAZAO: f32 = 1.3;

/// As direcções dos cones no referencial `(t, b, n)`: Fibonacci distribuída pelo cosseno.
fn direcoes() -> [[f32; 3]; CONES] {
    std::array::from_fn(|k| {
        let u = (k as f32 + 0.5) / CONES as f32;
        let phi = (k as f32 + 0.5) * 2.399_963;
        let r = u.sqrt();
        [r * phi.cos(), r * phi.sin(), (1.0 - u).sqrt()]
    })
}

/// ⭐⭐⭐ A visibilidade do céu em cada vértice (`1` = aberto). `alcance` = até onde um cone marcha
/// (a diagonal da peça chega: depois dela nada da peça tapa).
#[must_use]
pub fn visibilidade_propria(
    vol: &Volume,
    pos: &[[f32; 3]],
    nrm: &[[f32; 3]],
    alcance: f32,
) -> Vec<f32> {
    let dirs = direcoes();
    let h = vol.passo();
    let ta = ALFA.tan();
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
            let mut vis = 0.0;
            for d in &dirs {
                let w: [f32; 3] = std::array::from_fn(|e| t[e] * d[0] + b[e] * d[1] + n[e] * d[2]);
                let mut v = 1.0f32;
                let mut s = 0.5 * h;
                while s < alcance && v > 0.0 {
                    let q: [f32; 3] = std::array::from_fn(|e| o[e] + w[e] * s);
                    // ⚠️ Saiu da caixa (convexa, contém a peça): não volta a entrar. O limite
                    // inferior de fora escurecia os cones que saem rasantes (viés `−0,025`).
                    if (0..3).any(|e| q[e] < vol.lo[e] || q[e] > vol.hi[e]) {
                        break;
                    }
                    let dd = vol.distancia(q);
                    v = v.min((0.5 + 0.5 * dd / (s * ta)).clamp(0.0, 1.0));
                    s *= RAZAO;
                }
                vis += v;
            }
            vis / CONES as f32
        })
        .collect()
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
