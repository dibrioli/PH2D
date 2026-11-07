//! ⭐⭐⭐ **O COLISOR DE UM OBJECTO DA CENA** — doc 115 §33 (decisão do dono de 2026-10-11:
//! *«automático é melhor»*).
//!
//! A declaração da caixa é do CARTÃO: o `source.object` tem o próprio `Collide` (o espelho do da
//! `source.shape`), a lei da caixa mora com ele ([`ph2d_node_source_object::colisor`]) e o passe do
//! fim arma-se sozinho quando alguma peça declara. ⇒ a aparência que a membrana publica não leva
//! colisor nenhum. Os gates estão no irmão de testes.

#[cfg(test)]
#[path = "motion_bridge_objects_collider_tests.rs"]
mod tests;
