//! ⭐⭐ **A FAIXA ESTREITA** — a grade fina só é avaliada PERTO da superfície.
//!
//! Uma grade grossa (blocos de [`BLOCO`]³ amostras) pergunta ao campo no centro de cada bloco. Com o
//! limite do gradiente `L` ([`crate::gradient_bound`]), `|f(x) − f(c)| ≤ L·|x − c|`: um bloco cujo
//! centro vale mais que `L·(meia diagonal + uma diagonal de célula)` não tem superfície, nem a uma
//! célula dele. As amostras dele herdam o valor do centro (o SINAL certo) e não são avaliadas.
//!
//! ⭐ **A malha sai BIT A BIT a mesma** (gate `the_band_gives_the_same_mesh_as_the_full_grid`): toda
//! célula que cruza a superfície tem os oito cantos a menos de uma diagonal de célula dela — logo
//! dentro de blocos activos, com os valores verdadeiros. O que a faixa troca são só amostras cujo
//! único papel era dizer «fora» ou «dentro».
//!
//! Medido (02/10, cena 28, prof `8`, release): a extracção inteira levava `969 ms` com a grade cheia.

use crate::hybrid::Hybrid;

/// O lado de um bloco, em amostras.
pub(crate) const BLOCO: usize = 8;

pub(crate) struct Faixa {
    nb: usize,
    valor: Vec<f32>,
    ativo: Vec<bool>,
}

impl Faixa {
    /// `None` quando a grade é pequena demais para valer a pena (menos de `4` blocos por eixo).
    pub(crate) fn nova(
        h: &mut Hybrid,
        doc: &ph2d_field::FieldDoc,
        (lo, step, m): ([f64; 3], f64, usize),
    ) -> Option<Self> {
        let n = m - 1;
        if n < 4 * BLOCO || n % BLOCO != 0 {
            return None;
        }
        let nb = n / BLOCO;
        let centro = |b: usize, eixo: usize| ((b * BLOCO) as f64 + BLOCO as f64 * 0.5).mul_add(step, lo[eixo]);
        let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
        for bk in 0..nb {
            for bj in 0..nb {
                for bi in 0..nb {
                    xs.push(centro(bi, 0) as f32);
                    ys.push(centro(bj, 1) as f32);
                    zs.push(centro(bk, 2) as f32);
                }
            }
        }
        let valor = h.eval(&xs, &ys, &zs).ok()?.to_vec();
        let l = f64::from(crate::gradient_bound(doc)).max(1.0);
        let raiz3 = 3.0f64.sqrt();
        // Meia diagonal do bloco + uma diagonal de célula, com 1 % de folga para o `f32`.
        let margem = (raiz3 * BLOCO as f64 * 0.5 * step + raiz3 * step) * l * 1.01;
        let ativo = valor
            .iter()
            .map(|f| !f.is_finite() || f64::from(f.abs()) <= margem)
            .collect();
        Some(Self { nb, valor, ativo })
    }

    /// Quantos blocos ficam de fora (o controlo do gate).
    #[cfg(test)]
    pub(crate) fn inativos(&self) -> usize {
        self.ativo.iter().filter(|a| !**a).count()
    }

    fn bloco(&self, i: usize, j: usize, k: usize) -> usize {
        let c = |x: usize| (x / BLOCO).min(self.nb - 1);
        (c(k) * self.nb + c(j)) * self.nb + c(i)
    }

    /// A camada `k`: avalia só as amostras dos blocos activos; as outras herdam o centro.
    pub(crate) fn camada(
        &self,
        h: &mut Hybrid,
        coord: impl Fn(usize, usize) -> f64,
        m: usize,
        k: usize,
    ) -> Result<Vec<f32>, crate::MeshError> {
        let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
        let z = coord(2, k) as f32;
        for j in 0..m {
            let y = coord(1, j) as f32;
            for i in 0..m {
                if self.ativo[self.bloco(i, j, k)] {
                    xs.push(coord(0, i) as f32);
                    ys.push(y);
                    zs.push(z);
                }
            }
        }
        let avaliados = if xs.is_empty() { &[][..] } else { h.eval(&xs, &ys, &zs)? };
        let mut a = avaliados.iter();
        let mut out = Vec::with_capacity(m * m);
        for j in 0..m {
            for i in 0..m {
                let b = self.bloco(i, j, k);
                out.push(if self.ativo[b] {
                    *a.next().ok_or_else(|| crate::MeshError::Rejected("faixa: avaliação curta".into()))?
                } else {
                    self.valor[b]
                });
            }
        }
        Ok(out)
    }
}
