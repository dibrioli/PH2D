//! Os ids do **PINCEL DE PLANO** — irmão (`#[path]`) do [`super::sculpt3d`],
//! cortado por assunto, como os do Box Trim, do tecido, da pose e do contorno.
//!
//! Report do dono (2026-09-15): *«o Trim Brush do próprio blender que faz o trim
//! esfregando o pincel como massinha, modelando»*. A espec é
//! `docs/3D/cleanroom/SPEC_pincel_de_plano.md`.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// ⭐⭐⭐ **O TECTO DE CIMA** — quanto acima do plano o pincel alcança.
pub const SCULPT3D_PLANO_ALTURA: NodeId = hash_node_id("sculpt3d.plano_altura");
/// Chip ligado a [`SCULPT3D_PLANO_ALTURA`].
pub const SCULPT3D_PLANO_ALTURA_NUM: NodeId = hash_node_id("sculpt3d.plano_altura_num");

/// ⭐⭐⭐ **O TECTO DE BAIXO** — o espelho do de cima.
///
/// ⚠️ **Os dois juntos são a ferramenta**: `1/0` apara, `0/1` enche, `1/1`
/// achata. Um sozinho é meia pergunta.
pub const SCULPT3D_PLANO_PROFUNDIDADE: NodeId = hash_node_id("sculpt3d.plano_profundidade");
/// Chip ligado a [`SCULPT3D_PLANO_PROFUNDIDADE`].
pub const SCULPT3D_PLANO_PROFUNDIDADE_NUM: NodeId = hash_node_id("sculpt3d.plano_profundidade_num");

/// **A EXTENSÃO com que o CENTRO do plano é lido**, em fracção do raio.
pub const SCULPT3D_PLANO_AREA: NodeId = hash_node_id("sculpt3d.plano_area");
/// Chip ligado a [`SCULPT3D_PLANO_AREA`].
pub const SCULPT3D_PLANO_AREA_NUM: NodeId = hash_node_id("sculpt3d.plano_area_num");

/// ⭐⭐⭐ **A FIRMEZA DA NORMAL** — quanto o plano guarda a inclinação ao longo do traço (a alavanca
/// do aparar, espec §14.7).
pub const SCULPT3D_PLANO_FIRMEZA_NORMAL: NodeId = hash_node_id("sculpt3d.plano_firmeza_normal");
/// Chip ligado a [`SCULPT3D_PLANO_FIRMEZA_NORMAL`].
pub const SCULPT3D_PLANO_FIRMEZA_NORMAL_NUM: NodeId =
    hash_node_id("sculpt3d.plano_firmeza_normal_num");

/// **A FIRMEZA DO CENTRO** — quanto o plano guarda a altura ao longo do traço (espec §6.2).
pub const SCULPT3D_PLANO_FIRMEZA_CENTRO: NodeId = hash_node_id("sculpt3d.plano_firmeza_centro");
/// Chip ligado a [`SCULPT3D_PLANO_FIRMEZA_CENTRO`].
pub const SCULPT3D_PLANO_FIRMEZA_CENTRO_NUM: NodeId =
    hash_node_id("sculpt3d.plano_firmeza_centro_num");

/// **O que o `Ctrl` faz** — `ph2d_sculpt3d::PlanoInversao::ALL`.
///
/// ⚠️ **O tamanho CONTA-SE, não se escolhe:** o censo compara este array com o
/// `ALL` do motor, logo uma lei nova que não passe por aqui nasce inalcançável e
/// o gate fica vermelho — em vez de o chip sumir em silêncio.
pub const SCULPT3D_PLANO_INVERSAO: [NodeId; 2] = [
    hash_node_id("sculpt3d.plano_inversao.0"),
    hash_node_id("sculpt3d.plano_inversao.1"),
];
