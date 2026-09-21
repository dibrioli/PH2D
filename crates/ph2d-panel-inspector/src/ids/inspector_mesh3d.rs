//! **Os ids da secção LIVE MESH** — o catavento (`docs/3D/02.2`, rota B).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — o mesmo corte do
//! [`super::inspector_weapon`] e das outras.
//!
//! # ⚠️ Três caixas, zero segmentados, e isso é o desenho
//!
//! O catavento são **três números** (dois ângulos e uma taxa): ⇒ **não há posição-no-array a ser
//! tag de clique aqui**, que é a armadilha dos segmentados e que o `SignalVerb::ALL` pagou em
//! 19/09 (o verbo existia, tinha lei e gates, e o artista não lhe chegava).

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// A volta em torno do eixo vertical do ecrã, em GRAUS — ⚠️ o componente guarda RADIANOS.
pub const INSP_MESH3D_YAW: NodeId = hash_node_id("insp_mesh3d_yaw");
/// A inclinação, em GRAUS.
pub const INSP_MESH3D_PITCH: NodeId = hash_node_id("insp_mesh3d_pitch");
/// ⭐ **Voltas por segundo** — `0` deixa a peça parada no ângulo autorado, ao bit.
pub const INSP_MESH3D_SPIN: NodeId = hash_node_id("insp_mesh3d_spin");
