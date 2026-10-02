//! ⭐ **O campo avaliado em PARALELO** — valor e gradiente de muitos pontos, um avaliador (um JIT)
//! por thread. A resposta é BIT A BIT a mesma com qualquer número de threads: o mesmo código
//! compilado sobre os mesmos pontos (gate `the_parallel_answer_is_the_serial_one`).
//!
//! Medido (02/10, cena 28, prof `8`): preparar UMA peça do Render por malha — Newton, normais,
//! oclusão — eram `~1,5 M` avaliações do nó de toro num núcleo, `376 ms`.

use ph2d_field::FieldDoc;

use crate::MeshError;
use crate::hybrid::{Hybrid, Registry};

/// Abaixo disto um pedaço não paga a compilação de mais um avaliador.
pub const PEDACO_MIN: usize = 4096;

fn pedacos(n: usize) -> usize {
    std::thread::available_parallelism()
        .map_or(1, std::num::NonZeroUsize::get)
        .min(n.div_ceil(PEDACO_MIN))
        .max(1)
}

fn em_paralelo<T: Send + Clone + Default>(
    doc: &FieldDoc,
    reg: &Registry,
    pts: &[[f32; 3]],
    f: impl Fn(&mut Hybrid, &[f32], &[f32], &[f32], &mut Vec<T>) -> Result<(), MeshError> + Sync,
) -> Result<Vec<T>, MeshError> {
    let k = pedacos(pts.len());
    let tam = pts.len().div_ceil(k).max(1);
    let partes: Vec<Result<Vec<T>, MeshError>> = std::thread::scope(|s| {
        let hs: Vec<_> = pts
            .chunks(tam)
            .map(|c| {
                let f = &f;
                s.spawn(move || {
                    let mut h = Hybrid::new(doc, reg);
                    let xs: Vec<f32> = c.iter().map(|p| p[0]).collect();
                    let ys: Vec<f32> = c.iter().map(|p| p[1]).collect();
                    let zs: Vec<f32> = c.iter().map(|p| p[2]).collect();
                    let mut out = Vec::with_capacity(c.len());
                    f(&mut h, &xs, &ys, &zs, &mut out)?;
                    Ok(out)
                })
            })
            .collect();
        hs.into_iter()
            .map(|h| h.join().unwrap_or_else(|_| Err(MeshError::Rejected("avaliação em pânico".into()))))
            .collect()
    });
    let mut out = Vec::with_capacity(pts.len());
    for p in partes {
        out.extend(p?);
    }
    Ok(out)
}

/// `f` em cada ponto.
///
/// # Errors
/// Os do [`Hybrid::eval`].
pub fn valores(doc: &FieldDoc, reg: &Registry, pts: &[[f32; 3]]) -> Result<Vec<f32>, MeshError> {
    em_paralelo(doc, reg, pts, |h, xs, ys, zs, out| {
        out.extend_from_slice(h.eval(xs, ys, zs)?);
        Ok(())
    })
}

/// `∇f` em cada ponto (`eps` é o passo da diferença central, só usado com escultura).
///
/// # Errors
/// Os do [`Hybrid::gradients`].
pub fn gradientes(
    doc: &FieldDoc,
    reg: &Registry,
    pts: &[[f32; 3]],
    eps: f32,
) -> Result<Vec<[f32; 3]>, MeshError> {
    em_paralelo(doc, reg, pts, |h, xs, ys, zs, out| h.gradients(xs, ys, zs, eps, out))
}

/// `(f, ∇f)` em cada ponto, numa passada.
///
/// # Errors
/// Os do [`Hybrid::eval`] e do [`Hybrid::gradients`].
pub fn valores_e_gradientes(
    doc: &FieldDoc,
    reg: &Registry,
    pts: &[[f32; 3]],
    eps: f32,
) -> Result<Vec<(f32, [f32; 3])>, MeshError> {
    em_paralelo(doc, reg, pts, |h, xs, ys, zs, out| {
        let f = h.eval(xs, ys, zs)?.to_vec();
        let mut g = Vec::with_capacity(xs.len());
        h.gradients(xs, ys, zs, eps, &mut g)?;
        out.extend(f.into_iter().zip(g));
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};

    #[test]
    fn the_parallel_answer_is_the_serial_one() {
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
        let pts: Vec<[f32; 3]> = (0..20_000)
            .map(|i| {
                let t = i as f32 * 0.001;
                [t.sin() * 0.7, (t * 1.3).cos() * 0.4, (t * 0.7).sin() * 0.6]
            })
            .collect();
        let mut h = crate::hybrid::Hybrid::new(&doc, &reg);
        let xs: Vec<f32> = pts.iter().map(|p| p[0]).collect();
        let ys: Vec<f32> = pts.iter().map(|p| p[1]).collect();
        let zs: Vec<f32> = pts.iter().map(|p| p[2]).collect();
        let serie = h.eval(&xs, &ys, &zs).expect("série").to_vec();
        let mut gserie = Vec::new();
        h.gradients(&xs, &ys, &zs, 1e-4, &mut gserie).expect("grad");
        assert_eq!(super::valores(&doc, &reg, &pts).expect("par"), serie);
        assert_eq!(super::gradientes(&doc, &reg, &pts, 1e-4).expect("par"), gserie);
        assert!(super::pedacos(pts.len()) > 1 || std::thread::available_parallelism().map_or(1, |n| n.get()) == 1);
    }
}
