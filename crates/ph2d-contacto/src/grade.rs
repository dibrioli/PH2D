//! ⭐⭐ **A grelha à volta de uma peça** — em cada ponto, a visibilidade da peça em harmónicos.

use crate::{ALCANCE, COEFS, LADO, MARGEM, RAIOS, Volume};
use rayon::prelude::*;
use std::f32::consts::PI;

/// A convolução com o cosseno truncado, por ordem (`π`, `2π/3`, `π/4`) — Ramamoorthi & Hanrahan 2001.
pub const A_CONVOLUCAO: [f32; COEFS] = [
    PI,
    2.0 * PI / 3.0,
    2.0 * PI / 3.0,
    2.0 * PI / 3.0,
    PI / 4.0,
    PI / 4.0,
    PI / 4.0,
    PI / 4.0,
    PI / 4.0,
];

/// Os harmónicos reais até à ordem `2` na direcção unitária `w`.
#[must_use]
pub fn base_sh(w: [f32; 3]) -> [f32; COEFS] {
    let [x, y, z] = w;
    [
        0.282_095,
        0.488_603 * y,
        0.488_603 * z,
        0.488_603 * x,
        1.092_548 * x * y,
        1.092_548 * y * z,
        0.315_392 * (3.0 * z * z - 1.0),
        1.092_548 * x * z,
        0.546_274 * (x * x - y * y),
    ]
}

/// As [`RAIOS`] direcções de cada ponto: a espiral de Fibonacci na esfera (área igual por raio).
#[must_use]
pub fn direcoes() -> [[f32; 3]; RAIOS] {
    std::array::from_fn(|m| {
        let z = 1.0 - 2.0 * (m as f32 + 0.5) / RAIOS as f32;
        let r = (1.0 - z * z).max(0.0).sqrt();
        let phi = (m as f32 + 0.5) * 2.399_963;
        [r * phi.cos(), r * phi.sin(), z]
    })
}

/// A fracção do céu, ponderada pelo cosseno, que os coeficientes `c` tapam a uma normal `n`.
#[must_use]
pub fn oclusao_de(c: &[f32; COEFS], n: [f32; 3]) -> f32 {
    let y = base_sh(n);
    let s: f32 = (0..COEFS).map(|k| c[k] * A_CONVOLUCAO[k] * y[k]).sum();
    (s / PI).clamp(0.0, 1.0)
}

/// ⭐⭐ **A grelha de uma peça**, no referencial dela: [`LADO`]`³` pontos de `lo` a `hi` (os cantos
/// são pontos), `x` mais depressa.
#[derive(Clone, Debug, PartialEq)]
pub struct Grade {
    pub lo: [f32; 3],
    pub hi: [f32; 3],
    pub coef: Vec<[f32; COEFS]>,
}

impl Grade {
    /// ⭐ Marcha os [`RAIOS`] de cada ponto no `vol` da peça (bola `centro`, `raio`).
    ///
    /// ⛔ Empurrar os pontos de DENTRO da peça para a superfície foi escrito e retirado por mutação:
    /// não mudou nenhuma resposta (nem a do Cycles, nem a da placa) na quarta casa.
    #[must_use]
    pub fn constroi(vol: &Volume, centro: [f32; 3], raio: f32) -> Self {
        let meia = raio.max(1.0e-6) * (1.0 + MARGEM);
        let lo = centro.map(|c| c - meia);
        let hi = centro.map(|c| c + meia);
        let dirs = direcoes();
        let base: Vec<[f32; COEFS]> = dirs.iter().map(|w| base_sh(*w)).collect();
        let h = vol.passo();
        let diag = (0..3)
            .map(|e| (vol.hi[e] - vol.lo[e]).powi(2))
            .sum::<f32>()
            .sqrt();
        let peso = 4.0 * PI / RAIOS as f32;
        let coef = (0..LADO * LADO * LADO)
            .into_par_iter()
            .map(|idx| {
                let ijk = [idx % LADO, (idx / LADO) % LADO, idx / (LADO * LADO)];
                let v: [f32; 3] =
                    std::array::from_fn(|e| lo[e] + 2.0 * meia * ijk[e] as f32 / (LADO - 1) as f32);
                let q = v;
                let longe = diag
                    + (0..3)
                        .map(|e| (q[e] - 0.5 * (vol.lo[e] + vol.hi[e])).powi(2))
                        .sum::<f32>()
                        .sqrt();
                let mut c = [0.0f32; COEFS];
                for (w, y) in dirs.iter().zip(&base) {
                    if tapa(vol, q, *w, h, longe) {
                        for k in 0..COEFS {
                            c[k] += y[k] * peso;
                        }
                    }
                }
                c
            })
            .collect();
        Self { lo, hi, coef }
    }

    /// A mesma grelha com os coeficientes arredondados a `f16` — o que a placa guarda.
    #[must_use]
    pub fn em_f16(&self) -> Self {
        Self {
            coef: self
                .coef
                .iter()
                .map(|c| c.map(|x| half::f16::from_f32(x).to_f32()))
                .collect(),
            ..self.clone()
        }
    }

    /// Os coeficientes em `u ∈ [0, 1]³` (trilinear entre os pontos — o que a placa faz).
    #[must_use]
    pub fn coef_em(&self, u: [f32; 3]) -> [f32; COEFS] {
        let mut i0 = [0usize; 3];
        let mut f = [0.0f32; 3];
        for e in 0..3 {
            let t = u[e].clamp(0.0, 1.0) * (LADO - 1) as f32;
            let i = (t.floor() as usize).min(LADO - 2);
            i0[e] = i;
            f[e] = t - i as f32;
        }
        let mut s = [0.0f32; COEFS];
        for dz in 0..2 {
            for dy in 0..2 {
                for dx in 0..2 {
                    let w = (if dx == 1 { f[0] } else { 1.0 - f[0] })
                        * (if dy == 1 { f[1] } else { 1.0 - f[1] })
                        * (if dz == 1 { f[2] } else { 1.0 - f[2] });
                    let c = &self.coef[((i0[2] + dz) * LADO + i0[1] + dy) * LADO + i0[0] + dx];
                    for k in 0..COEFS {
                        s[k] += w * c[k];
                    }
                }
            }
        }
        s
    }

    /// ⭐⭐⭐ **O céu que esta peça tapa ao ponto `p` de normal `n`** (no referencial da peça).
    /// Fora do cubo, a da borda na mesma direcção a partir do centro, vezes `1/s²` (o ângulo sólido
    /// de uma peça vista de longe); além de [`ALCANCE`], zero.
    #[must_use]
    pub fn oclusao(&self, p: [f32; 3], n: [f32; 3]) -> f32 {
        let mut u: [f32; 3] =
            std::array::from_fn(|e| (p[e] - self.lo[e]) / (self.hi[e] - self.lo[e]));
        let s = (0..3).map(|e| (u[e] - 0.5).abs() * 2.0).fold(0.0, f32::max);
        if s > ALCANCE {
            return 0.0;
        }
        let mut f = 1.0;
        if s > 1.0 {
            u = u.map(|c| 0.5 + (c - 0.5) / s);
            f = 1.0 / (s * s);
        }
        f * oclusao_de(&self.coef_em(u), n)
    }
}

/// O raio `q + s·w` toca a peça?
fn tapa(vol: &Volume, q: [f32; 3], w: [f32; 3], h: f32, longe: f32) -> bool {
    let mut s = 0.05 * h;
    for _ in 0..96 {
        let d = vol.distancia([q[0] + w[0] * s, q[1] + w[1] * s, q[2] + w[2] * s]);
        if d < 0.05 * h {
            return true;
        }
        s += d.max(0.1 * h);
        if s > longe {
            return false;
        }
    }
    false
}
