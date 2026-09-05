//! As peças de prova — e **uma delas é CURVA de propósito**.
//!
//! ⚠️ Uma grade plana não contém o fenômeno que decidiu o modelo de dobra: com
//! repouso plano, o modelo quadrático (que foi refutado) e o do ângulo diedro
//! concordam. *Uma fixtura plana aprovaria o modelo errado.*

use crate::{ClothTopology, V3};

/// Um triângulo qualquer — nada de retângulo, para não esconder cisalhamento.
pub(crate) fn triangle() -> (Vec<V3>, [u32; 3]) {
    (
        vec![[0.1, -0.2, 0.3], [1.3, 0.1, -0.2], [0.4, 1.1, 0.6]],
        [0, 1, 2],
    )
}

/// Duas faces numa aresta — a dobradiça, já **dobrada** (não plana).
pub(crate) fn hinge_pair() -> (Vec<V3>, Vec<[u32; 3]>) {
    (
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.1, 0.0],
            [0.4, 0.9, 0.35],
            [0.6, -0.8, 0.25],
        ],
        vec![[0, 1, 2], [1, 0, 3]],
    )
}

/// Uma grade `n×n` no plano `xy`, com passo `1/n`.
pub(crate) fn grid(n: usize) -> (Vec<V3>, Vec<[u32; 3]>) {
    let s = 1.0 / n as f64;
    let mut x = Vec::with_capacity((n + 1) * (n + 1));
    for j in 0..=n {
        for i in 0..=n {
            x.push([i as f64 * s, j as f64 * s, 0.0]);
        }
    }
    let id = |i: usize, j: usize| u32::try_from(j * (n + 1) + i).unwrap_or(u32::MAX);
    let mut t = Vec::with_capacity(n * n * 2);
    for j in 0..n {
        for i in 0..n {
            t.push([id(i, j), id(i + 1, j), id(i + 1, j + 1)]);
            t.push([id(i, j), id(i + 1, j + 1), id(i, j + 1)]);
        }
    }
    (x, t)
}

/// A mesma grade, **abaulada** — o repouso curvo que uma escultura de facto tem.
pub(crate) fn dome(n: usize) -> (Vec<V3>, Vec<[u32; 3]>) {
    let (mut x, t) = grid(n);
    for p in &mut x {
        let (u, v) = (p[0] - 0.5, p[1] - 0.5);
        p[2] = 0.35 * (0.25 - u * u - v * v).max(0.0);
    }
    (x, t)
}

/// A região pronta, com o repouso já medido.
pub(crate) fn region(x: &[V3], t: &[[u32; 3]]) -> ClothTopology {
    ClothTopology::build(t, x.len())
}

/// O anel de fora da grade — o que o pincel prega.
pub(crate) fn border(n: usize) -> Vec<bool> {
    let mut p = vec![false; (n + 1) * (n + 1)];
    for j in 0..=n {
        for i in 0..=n {
            if i == 0 || j == 0 || i == n || j == n {
                p[j * (n + 1) + i] = true;
            }
        }
    }
    p
}
