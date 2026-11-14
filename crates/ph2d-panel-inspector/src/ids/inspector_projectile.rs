//! **Os ids da secção PROJECTILE MOTION** (TOP-20 #14, W3).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — o mesmo corte do
//! [`super::inspector_topdown`].
//!
//! Não há segmentado: o `ProjectileMotion` é nove números, uma caixa e um nome; as caixas Bounce e Homing do
//! cartão (GEF 06) escrevem esses números.

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
/// ⭐ GEF 06 (versão B) — a caixa «Bounce»: ligada mostra Max Bounces e Bounciness.
pub const INSP_PJ_BOUNCE: NodeId = hash_node_id("insp_pj_bounce");
/// ⭐ GEF 06 — a caixa «Homing»: ligada mostra Target e Pull.
pub const INSP_PJ_HOMING: NodeId = hash_node_id("insp_pj_homing");
/// ⭐ GEF 06 — o conta-gotas do Target: arma, e o próximo clique no palco escolhe o objecto perseguido.
pub const INSP_PJ_TARGET_PICK: NodeId = hash_node_id("insp_pj_target_pick");
/// ⭐ GEF 06 — a dobra «Speed change» (Acceleration, Max Speed): nasce FECHADA; o cabeçalho leva o resumo.
pub use ph2d_editor_core::ids::INSP_PJ_FOLD_SPEED_CHANGE;
