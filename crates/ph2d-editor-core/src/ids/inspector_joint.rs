//! **Os ids da §12 — Physics Joint.**
//!
//! Irmão de `inspector.rs`, separado dele quando os dois juntos passaram do cap
//! de 700 LOC, e o corte é por SEÇÃO: tudo aqui descreve a seção que fala do
//! objeto-joint selecionado (tipo, as duas pontas que ele nomeia, os parâmetros
//! do tipo escolhido, e o teto sob o qual ele parte).
//!
//! ⚠️ **O gesto de CRIAR não está aqui** — ele mora nos ids da §11
//! (`INSP_PHYS_JOIN*`), porque um joint não existe ainda quando você quer fazer
//! um: o botão tem de estar onde você já está, olhando dois corpos selecionados.

use super::hash_node_id;
use ph2d_a11y::NodeId;

/// The §12 section header (collapse state owner) and its colour circle.
pub const INSP_LIVE_JOINT_SECTION: NodeId = hash_node_id("insp_live_joint_section");
pub const INSP_LIVE_JOINT_COLOR: NodeId = hash_node_id("insp_live_joint_color");

/// **O cabeçalho da §13 — Pulley Wheel** (dono do estado de colapso) e o
/// círculo de cor dele.
///
/// ⚠️ Seção PRÓPRIA, e não mais rows na §12: uma roldana é uma ENTIDADE (a
/// espinha do W1), então ela é o objeto SELECIONADO quando estas rows importam,
/// e a §12 só existe com a CORDA selecionada. Uma seção que trocasse de assunto
/// conforme o que está selecionado teria um estado de colapso descrevendo dois
/// objetos diferentes.
pub const INSP_LIVE_WHEEL_SECTION: NodeId = hash_node_id("insp_live_wheel_section");
pub const INSP_LIVE_WHEEL_COLOR: NodeId = hash_node_id("insp_live_wheel_color");
