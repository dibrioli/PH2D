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
    /// A faixa estreita ([`crate::extract_band`]) — `None` = a grade cheia.
    faixa: Option<crate::extract_band::Faixa>,
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
        com_faixa: bool,
    ) -> Self {
        let mut evals: Vec<Hybrid> = (0..threads.max(1)).map(|_| Hybrid::new(doc, reg)).collect();
        let faixa = com_faixa
            .then(|| crate::extract_band::Faixa::nova(&mut evals[0], doc, (lo, step, m)))
            .flatten();
        Self {
            evals,
            lo,
            step,
            m,
            next: 0,
            ready: VecDeque::new(),
            faixa,
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
        let faixa = self.faixa.as_ref();
        let out: Vec<Result<Vec<f32>, MeshError>> = std::thread::scope(|s| {
            let handles: Vec<_> = self
                .evals
                .iter_mut()
                .take(n)
                .enumerate()
                .map(|(t, h)| s.spawn(move || plane(h, lo, step, m, first + t, faixa)))
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
fn plane(
    h: &mut Hybrid,
    lo: [f64; 3],
    step: f64,
    m: usize,
    k: usize,
    faixa: Option<&crate::extract_band::Faixa>,
) -> Result<Vec<f32>, MeshError> {
    let coord = |axis: usize, i: usize| (i as f64).mul_add(step, lo[axis]);
    if let Some(f) = faixa {
        return f.camada(h, coord, m, k);
    }
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
        let um = crate::extract::sweep(&doc, &reg, 5, None, 1, true).expect("1 thread");
        let sete = crate::extract::sweep(&doc, &reg, 5, None, 7, true).expect("7 threads");
        assert_eq!(um.0, sete.0);
        assert_eq!(um.1, sete.1);
        assert!(!um.1.is_empty());
    }
}

#[cfg(test)]
mod band_tests {
    use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};

    fn combine(op: Op, kids: &[u32]) -> Node {
        Node::new(
            Xform::IDENTITY,
            NodeKind::Combine {
                op,
                children: kids.iter().copied().map(NodeId).collect(),
            },
        )
    }

    /// ⭐⭐ **A faixa estreita dá a MESMA malha que a grade cheia, ao bit** — num campo que não é
    /// distância (o nó de toro, `|∇f| ≤ 0,72`), numa caixa de quinas vivas, e numa união suave
    /// (`L = √2`), com o controlo de que a faixa de facto saltou amostras.
    #[test]
    fn the_band_gives_the_same_mesh_as_the_full_grid() {
        let no = crate::leaf(
            Primitive::TorusKnot {
                radius: 0.4,
                tube: 0.08,
                cord: ph2d_field::knot_cord_ceiling(0.4, 0.08, 2, 3) * 0.85,
                winds: 2,
                loops: 3,
            },
            Xform::IDENTITY,
        );
        let caixa = crate::leaf(
            Primitive::Box {
                half: [0.5, 0.3, 0.2],
                round: 0.0,
                chamfer: 0.0,
            },
            Xform::at(0.1, 0.0, 0.0),
        );
        let bola = crate::leaf(Primitive::Sphere { radius: 0.35 }, Xform::at(0.4, 0.2, 0.0));
        let docs = [
            FieldDoc::new(vec![no], NodeId(0)).expect("nó"),
            FieldDoc::new(vec![caixa.clone()], NodeId(0)).expect("caixa"),
            FieldDoc::new(
                vec![caixa, bola, combine(Op::Union(Blend::Exact { radius: 0.1 }), &[0, 1])],
                NodeId(2),
            )
            .expect("suave"),
        ];
        let reg = crate::hybrid::Registry::new();
        for (i, doc) in docs.iter().enumerate() {
            let cheia = crate::extract::sweep(doc, &reg, 7, None, 4, false).expect("cheia");
            let faixa = crate::extract::sweep(doc, &reg, 7, None, 4, true).expect("faixa");
            assert!(!cheia.1.is_empty(), "peça {i} sem faces");
            assert_eq!(cheia.0, faixa.0, "peça {i}: os vértices mudaram com a faixa");
            assert_eq!(cheia.1, faixa.1, "peça {i}: as faces mudaram com a faixa");
            // O controlo: a faixa existe e deixa blocos de fora.
            let mut h = crate::hybrid::Hybrid::new(doc, &reg);
            let ball = crate::bounds::bounding_ball(doc, &reg).expect("bola");
            let half = f64::from(ball.radius) * 1.05;
            let lo = ball.center.map(|c| f64::from(c) - half);
            let f = crate::extract_band::Faixa::nova(&mut h, doc, (lo, 2.0 * half / 128.0, 129));
            assert!(f.is_some_and(|f| f.inativos() > 0), "peça {i}: a faixa não saltou nada");
        }
    }
}
