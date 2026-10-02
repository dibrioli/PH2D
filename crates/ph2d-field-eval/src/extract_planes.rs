//! ⭐ **AS CAMADAS DA GRADE, avaliadas em PARALELO** — o preço da extração é avaliar o campo.
//!
//! Medido (02/10, release, `load 2,6`): a cena 28 do smoke (quatro nós de toro) a prof `7` levava
//! **`611 ms`** num núcleo, e era a avaliação — a montagem dos quads e o resto somavam `~70 ms`.
//! Cada camada é independente das outras, então `T` avaliadores (um JIT por thread, compilado UMA
//! vez) respondem `T` camadas de cada vez, e a varredura consome-as pela ordem de sempre.
//!
//! ⚠️ **A resposta é BIT A BIT a mesma** com qualquer `T`: o mesmo código compilado sobre os mesmos
//! pontos, só noutra thread (gate `the_planes_are_the_same_with_any_thread_count`).

use std::collections::VecDeque;

use crate::MeshError;
use crate::hybrid::{Hybrid, Registry};

/// As camadas `z = 0..m` da grade, entregues pela ordem.
pub(crate) struct Planes {
    evals: Vec<Hybrid>,
    lo: [f64; 3],
    step: f64,
    m: usize,
    next: usize,
    ready: VecDeque<Vec<f32>>,
}

/// Quantas threads por omissão: as do aparelho, e nunca mais que as camadas.
pub(crate) fn threads_for(m: usize) -> usize {
    std::thread::available_parallelism()
        .map_or(1, std::num::NonZeroUsize::get)
        .min(m)
        .max(1)
}

impl Planes {
    pub(crate) fn new(
        doc: &ph2d_field::FieldDoc,
        reg: &Registry,
        (lo, step, m): ([f64; 3], f64, usize),
        threads: usize,
    ) -> Self {
        Self {
            evals: (0..threads.max(1)).map(|_| Hybrid::new(doc, reg)).collect(),
            lo,
            step,
            m,
            next: 0,
            ready: VecDeque::new(),
        }
    }

    /// A próxima camada, pela ordem de `z`.
    pub(crate) fn next(&mut self) -> Result<Vec<f32>, MeshError> {
        if self.ready.is_empty() {
            self.batch()?;
        }
        self.ready
            .pop_front()
            .ok_or_else(|| MeshError::Rejected("a grade acabou antes da varredura".into()))
    }

    fn batch(&mut self) -> Result<(), MeshError> {
        let n = self.evals.len().min(self.m - self.next);
        let (lo, step, m, first) = (self.lo, self.step, self.m, self.next);
        let out: Vec<Result<Vec<f32>, MeshError>> = std::thread::scope(|s| {
            let handles: Vec<_> = self
                .evals
                .iter_mut()
                .take(n)
                .enumerate()
                .map(|(t, h)| s.spawn(move || plane(h, lo, step, m, first + t)))
                .collect();
            handles
                .into_iter()
                .map(|h| {
                    h.join()
                        .unwrap_or_else(|_| Err(MeshError::Rejected("camada em pânico".into())))
                })
                .collect()
        });
        for r in out {
            self.ready.push_back(r?);
        }
        self.next += n;
        Ok(())
    }
}

/// Uma camada. ⚠️ A coordenada é a MESMA conta do `Grid::coord` do extrator (`i·step + lo`, em
/// `mul_add`) — uma segunda forma dela daria pontos diferentes no último bit.
fn plane(h: &mut Hybrid, lo: [f64; 3], step: f64, m: usize, k: usize) -> Result<Vec<f32>, MeshError> {
    let coord = |axis: usize, i: usize| (i as f64).mul_add(step, lo[axis]);
    let (mut xs, mut ys, mut zs) = (
        Vec::with_capacity(m * m),
        Vec::with_capacity(m * m),
        Vec::with_capacity(m * m),
    );
    let z = coord(2, k) as f32;
    for j in 0..m {
        let y = coord(1, j) as f32;
        for i in 0..m {
            xs.push(coord(0, i) as f32);
            ys.push(y);
            zs.push(z);
        }
    }
    Ok(h.eval(&xs, &ys, &zs)?.to_vec())
}

#[cfg(test)]
mod tests {
    use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};

    #[test]
    fn the_planes_are_the_same_with_any_thread_count() {
        let doc = FieldDoc::new(
            vec![crate::leaf(
                Primitive::Torus {
                    major: 0.5,
                    minor: 0.2,
                },
                Xform::IDENTITY,
            )],
            NodeId(0),
        )
        .expect("doc");
        let reg = crate::hybrid::Registry::new();
        let um = crate::extract::sweep(&doc, &reg, 5, None, 1).expect("1 thread");
        let sete = crate::extract::sweep(&doc, &reg, 5, None, 7).expect("7 threads");
        assert_eq!(um.0, sete.0);
        assert_eq!(um.1, sete.1);
        assert!(!um.1.is_empty());
    }
}
