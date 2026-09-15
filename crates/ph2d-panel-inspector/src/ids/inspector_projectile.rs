//! **Os ids da secção PROJECTILE MOTION** (TOP-20 #14, W3).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — o mesmo corte do
//! [`super::inspector_topdown`].
//!
//! # ⚠️ Não há segmentado nenhum, e isso é o desenho
//!
//! O `ProjectileMotion` não tem um único enum: ele é nove números, uma caixa e um nome. ⇒ não há
//! posição-no-array a ser tag de clique aqui, que é a armadilha que os três segmentados do irmão
//! carregam.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// A rapidez com que ele nasce, m/s.
pub const INSP_PJ_SPEED: NodeId = hash_node_id("insp_pj_speed");
/// Aceleração ao longo da direcção de voo, m/s². Negativa trava.
pub const INSP_PJ_ACCEL: NodeId = hash_node_id("insp_pj_accel");
/// Tecto de rapidez, m/s. ⚠️ `0` é **sem tecto**.
pub const INSP_PJ_MAX_SPEED: NodeId = hash_node_id("insp_pj_max_speed");
/// A gravidade que faz o arco, m/s².
pub const INSP_PJ_GRAVITY: NodeId = hash_node_id("insp_pj_gravity");
/// A fracção da rapidez que sobrevive a um ricochete.
pub const INSP_PJ_BOUNCINESS: NodeId = hash_node_id("insp_pj_bounciness");
/// Quantos ricochetes o voo aguenta. ⚠️ `0` = acaba no primeiro toque.
pub const INSP_PJ_MAX_BOUNCES: NodeId = hash_node_id("insp_pj_max_bounces");
/// Metros percorridos até o voo acabar. ⚠️ `0` é **sem limite**.
pub const INSP_PJ_RANGE: NodeId = hash_node_id("insp_pj_range");
/// A flecha aponta para onde voa.
pub const INSP_PJ_FACE_VELOCITY: NodeId = hash_node_id("insp_pj_face_velocity");
/// Aceleração de perseguição, m/s². `0` = não persegue.
pub const INSP_PJ_HOMING_ACCEL: NodeId = hash_node_id("insp_pj_homing_accel");
/// **O NOME de quem ele persegue** — ⚠️ só pintado quando a perseguição está ligada.
pub const INSP_PJ_HOMING_TARGET: NodeId = hash_node_id("insp_pj_homing_target");
