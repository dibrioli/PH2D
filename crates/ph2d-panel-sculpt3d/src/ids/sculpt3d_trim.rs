//! Os ids do **BOX TRIM** — irmão (`#[path]`) do [`super::sculpt3d`], cortado
//! por assunto, como os do tecido, da pose, do contorno e do esfregão.
//!
//! Ordem do dono (2026-09-15): *«Nos parâmetros botões box e circle (novo) e
//! laço. Em laço um parâmetro para suavizar o traço.»*

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// **A FORMA que ele corta** — `ph2d_sculpt3d::TrimForma::ALL`.
///
/// ⚠️ **O tamanho CONTA-SE, não se escolhe:** o censo compara este array com o
/// `ALL` do motor, logo uma forma nova que não passe por aqui nasce
/// inalcançável e o gate fica vermelho — em vez de o chip sumir em silêncio.
pub const SCULPT3D_TRIM_FORMA: [NodeId; 3] = [
    hash_node_id("sculpt3d.trim_forma.0"),
    hash_node_id("sculpt3d.trim_forma.1"),
    hash_node_id("sculpt3d.trim_forma.2"),
];

/// A pista da suavização do traço — só o LAÇO a lê.
pub const SCULPT3D_TRIM_SMOOTH: NodeId = hash_node_id("sculpt3d.trim_smooth");
/// Chip ligado a [`SCULPT3D_TRIM_SMOOTH`].
pub const SCULPT3D_TRIM_SMOOTH_NUM: NodeId = hash_node_id("sculpt3d.trim_smooth_num");
