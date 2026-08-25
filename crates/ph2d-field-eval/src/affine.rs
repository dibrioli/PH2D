//! ⭐ **O MAPA AFIM mundo→local de um nó** — a cadeia de poses da árvore, composta uma vez.
//!
//! Irmão do [`crate`] por responsabilidade (teto de LOC): ali mora *o que se especializa*, aqui *em
//! que espaço a região chega a cada folha*.

use ph2d_field::Xform;

use crate::inverse_rotation_matrix;

/// Um mapa afim `p ↦ M·p + c` — a composição de poses que leva o mundo ao plano de um perfil.
///
/// ⚠️ **A escala é uniforme** por decisão do módulo (ADR-0161 §6), então `M` continua a ser uma
/// rotação escalada, e a caixa transformada por cantos é conservadora sem folga inventada.
#[derive(Clone, Copy)]
pub(crate) struct Affine {
    m: [[f64; 3]; 3],
    c: [f64; 3],
}

impl Affine {
    /// O que a [`place`] faz às coordenadas: `local = R⁻¹·(p − t)/s`.
    pub(crate) fn of(x: Xform) -> Self {
        let inv_s = 1.0 / f64::from(x.scale);
        let r = inverse_rotation_matrix(x.rotation);
        let t = x.translation.map(f64::from);
        let mut m = [[0.0; 3]; 3];
        let mut c = [0.0; 3];
        for (i, row) in m.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                *cell = r[i][j] * inv_s;
            }
            c[i] = -(r[i][0] * t[0] + r[i][1] * t[1] + r[i][2] * t[2]) * inv_s;
        }
        Self { m, c }
    }

    /// `self ∘ outer` — primeiro o de fora (o pai), depois este.
    pub(crate) fn after(self, outer: Self) -> Self {
        let mut m = [[0.0; 3]; 3];
        let mut c = self.c;
        for (i, row) in m.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                *cell = (0..3).map(|k| self.m[i][k] * outer.m[k][j]).sum();
            }
            c[i] += (0..3).map(|k| self.m[i][k] * outer.c[k]).sum::<f64>();
        }
        Self { m, c }
    }

    /// A imagem de um conjunto de pontos — um mapa afim leva ponto a ponto, e o casco da imagem é a
    /// imagem do casco.
    pub(crate) fn points_of(self, pts: &[[f32; 3]]) -> Vec<[f32; 3]> {
        pts.iter()
            .map(|p| {
                let mut out = [0.0f32; 3];
                for (i, o) in out.iter_mut().enumerate() {
                    *o = ((0..3).map(|j| self.m[i][j] * f64::from(p[j])).sum::<f64>() + self.c[i])
                        as f32;
                }
                out
            })
            .collect()
    }

    /// A caixa local que contém a imagem da caixa do mundo — pelos **oito cantos**, que é exacto
    /// para um mapa afim.
    pub(crate) fn box_of(self, lo: [f32; 3], hi: [f32; 3]) -> ([f32; 3], [f32; 3]) {
        let (mut out_lo, mut out_hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
        for k in 0..8u8 {
            let p = [
                if k & 1 == 0 { lo[0] } else { hi[0] },
                if k & 2 == 0 { lo[1] } else { hi[1] },
                if k & 4 == 0 { lo[2] } else { hi[2] },
            ];
            for i in 0..3 {
                let v = (0..3).map(|j| self.m[i][j] * f64::from(p[j])).sum::<f64>() + self.c[i];
                out_lo[i] = out_lo[i].min(v as f32);
                out_hi[i] = out_hi[i].max(v as f32);
            }
        }
        (out_lo, out_hi)
    }
}
