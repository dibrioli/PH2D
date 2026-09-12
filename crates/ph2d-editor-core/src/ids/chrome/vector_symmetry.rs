//! **Os ids da seção SYMMETRY** — a simetria de desenho (plano 25 §9, W6.3) — irmão de `vector`
//! pelo teto de 700 LOC, e o corte é por responsabilidade: aqui moram os controles de um MODO de
//! desenho (o outro lado do traço é derivado enquanto ele está ligado), e não os do documento.
//!
//! ⚠️ **Isto nasceu como um `PathEffect` da pilha e foi REPROVADO** (Enio, 2026-08-01: *"funciona
//! bem mas não é legal como um efeito; melhor como uma opção para as tools de desenho exatamente
//! como o modo painter, morando em uma seção específica para isso"*). Os ids de efeito
//! desapareceram com ele; estes são de uma seção própria.

use ph2d_a11y::NodeId;

use super::hash_node_id;

/// **Apply** — consolida as cópias em geometria de documento e desarma a simetria.
///
/// ⚠️ Oferecido só quando há simetria VIVA na seleção: sem ela não há o que consolidar, e um botão
/// que não faz nada é pior que botão que falta.
pub const VECTOR_SYM_APPLY: NodeId = hash_node_id("vector.sym.apply");
