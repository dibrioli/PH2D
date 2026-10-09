//! ⭐⭐⭐ **A ORDEM EM QUE A ONDA CORRE PELA FILA** — a porta é a da casa,
//! [`ph2d_ease::order`] (ciclo 17, doc 126 §3.2): `Index` · `From Center` · `Random` (ciclo 2,
//! doc 105, ao bit) e `Grid` · `From Point` (ciclo 17).
//!
//! ⚠️ O ciclo 2 recusou `By X`/`By Y` porque pediam um POSTO (uma ordenação, que derrubava o nó
//! para a CPU). O `From Point` com `axis` responde a mesma vontade sem o posto — a distância a um
//! ponto é por-elemento. O `motion.sort(key = X)` a montante continua a ser a ferramenta que
//! REORDENA as linhas.

pub use ph2d_ease::order::*;

/// A posição de `i` na onda pelas ordens que só leem `(i, n, seed)` — o atalho dos testes do
/// ciclo 2.
#[cfg(test)]
pub(crate) fn raw_at_index(order: i32, i: u32, n: u32, seed: u32) -> f32 {
    raw_at(
        &Order {
            kind: order,
            seed,
            ..Order::INDEX
        },
        i,
        n,
        [0.0, 0.0],
    )
}

#[cfg(test)]
#[path = "order_tests.rs"]
mod tests;
