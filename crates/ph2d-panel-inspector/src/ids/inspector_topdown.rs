//! **Os ids da secção TOP-DOWN PLAYER** (TOP-20 #13, W3).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — o mesmo corte do [`super::inspector_camera`]
//! e do [`super::inspector_factory`].
//!
//! # ⚠️ Não há lista, e por isso não há estado de painel
//!
//! Um objecto tem **um** mover. ⇒ esta secção lê o snapshot e pinta os campos, como a da câmera.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// Velocidade máxima, m/s.
pub const INSP_TD_SPEED: NodeId = hash_node_id("insp_td_speed");
/// Rampa de arranque, m/s². ⚠️ `0` é **instantâneo**, não parado.
pub const INSP_TD_ACCEL: NodeId = hash_node_id("insp_td_accel");
/// Rampa de travagem, m/s². ⚠️ `0` é instantâneo.
pub const INSP_TD_DECEL: NodeId = hash_node_id("insp_td_decel");
/// O segmentado das DIRECÇÕES — **um id por opção**.
///
/// ⚠️ **A POSIÇÃO no array é a tag do clique**, como em todo segmentado desta casa: reordenar faria
/// um clique escrever outro modo — e compila.
pub const INSP_TD_DIRECTIONS: [NodeId; 5] = [
    hash_node_id("insp_td_dir_free"),
    hash_node_id("insp_td_dir_eight"),
    hash_node_id("insp_td_dir_four"),
    hash_node_id("insp_td_dir_x"),
    hash_node_id("insp_td_dir_y"),
];
/// O segmentado do VIEWPOINT — um id por opção.
pub const INSP_TD_VIEWPOINT: [NodeId; 4] = [
    hash_node_id("insp_td_view_top"),
    hash_node_id("insp_td_view_iso21"),
    hash_node_id("insp_td_view_iso30"),
    hash_node_id("insp_td_view_custom"),
];
/// A elevação do tabuleiro, graus — ⚠️ **só pintada em `Custom`**.
pub const INSP_TD_VIEW_ANGLE: NodeId = hash_node_id("insp_td_view_angle");
/// O segmentado do FACING — um id por opção.
pub const INSP_TD_FACING: [NodeId; 4] = [
    hash_node_id("insp_td_face_none"),
    hash_node_id("insp_td_face_move"),
    hash_node_id("insp_td_face_snap4"),
    hash_node_id("insp_td_face_snap8"),
];
/// Graus por segundo da viragem — ⚠️ **só pintada quando ele roda**.
pub const INSP_TD_TURN_SPEED: NodeId = hash_node_id("insp_td_turn_speed");
/// Abaixo deste ângulo de incidência ele pára em vez de deslizar.
pub const INSP_TD_MIN_SLIDE: NodeId = hash_node_id("insp_td_min_slide");
/// Quantas vezes o orçamento pode mudar de direcção num tique.
pub const INSP_TD_MAX_SLIDES: NodeId = hash_node_id("insp_td_max_slides");
/// Lê as setas do Input Map? Desligado, ele é motor puro.
pub const INSP_TD_DEFAULT_CONTROLS: NodeId = hash_node_id("insp_td_default_controls");
