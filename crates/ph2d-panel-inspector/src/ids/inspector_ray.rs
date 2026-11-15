//! **Os ids da secção RAY SENSOR** (suplente #21).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — o mesmo corte do
//! [`super::inspector_projectile`].
//!
//! # ⚠️ Não há segmentado nenhum, e isso é o desenho
//!
//! O `RaySensor` são seis números e o `RaySignals` dois nomes: ⇒ **não há posição-no-array a ser
//! tag de clique aqui**, que é a armadilha de todo segmentado indexado, e que o
//! `SignalVerb::ALL` desta linha pagou em 19/09 (o verbo existia, tinha lei e
//! gates, e o artista não lhe chegava porque o array de ids ficou com um a menos).

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// Para onde o raio aponta, em graus locais (GEF 07: a origem é a posição do objeto).
pub const INSP_RAY_ANGLE: NodeId = hash_node_id("insp_ray_angle");
/// A caixa «Turns with facing» (dono 15/11).
pub const INSP_RAY_TURNS: NodeId = hash_node_id("insp_ray_turns");
/// Até onde ele enxerga, em metros.
pub const INSP_RAY_REACH: NodeId = hash_node_id("insp_ray_reach");
/// A camada de colisão pela qual ele pergunta — a matriz do mundo decide o que ele vê. É a lista suspensa
/// da camada (a mesma do collider) e o canal do losango (`component.field.ray_sensor.2`).
pub const INSP_RAY_LAYER: NodeId = hash_node_id("insp_ray_layer");
/// As opções da lista da camada, uma por camada.
pub static INSP_RAY_LAYER_OPTS: [NodeId; super::INSP_PHYS_LAYERS] =
    super::inspector_physics_body::tabela_por_camada(b"insp_ray_layer_opt_");
/// «Sees»: um quadro por camada, a linha da matriz do mundo da camada do raio; o «+» no fim.
pub static INSP_RAY_SEES: [NodeId; super::INSP_PHYS_LAYERS] =
    super::inspector_physics_body::tabela_por_camada(b"insp_ray_sees_");
pub const INSP_RAY_SEES_ADD: NodeId = hash_node_id("insp_ray_sees_add");
/// A frase debaixo da fila «Sees»: só a dica, com o nome inteiro.
pub const INSP_RAY_SEES_APPLIES: NodeId = hash_node_id("insp_ray_sees_applies");
/// O nome publicado quando ele **passa a ver** — ⚠️ vazio = calado, a regra do `SignalOnHit`.
pub const INSP_RAY_ON_ENTER: NodeId = hash_node_id("insp_ray_on_enter");
/// …e quando ele **deixa de ver**.
pub const INSP_RAY_ON_EXIT: NodeId = hash_node_id("insp_ray_on_exit");
/// «Rename layers…» e «Open in tab…» do Ray (emenda 4, dono 08/11): ids próprios porque o collider pinta os dele no
/// mesmo Inspector (hover e prima vivem no store por id); os campos do renomear são os do collider, e só um cartão os
/// mostra de cada vez (`INSP_RAY_LAYER_RENAMING`).
pub const INSP_RAY_LAYER_RENAME: NodeId = hash_node_id("insp_ray_layer_rename");
pub const INSP_RAY_LAYER_RENAMING: NodeId = hash_node_id("insp_ray_layer_renaming");
pub const INSP_RAY_LAYER_RENAME_DONE: NodeId = hash_node_id("insp_ray_layer_rename_done");
pub const INSP_RAY_EDIT_MATRIX: NodeId = hash_node_id("insp_ray_edit_matrix");

pub use ph2d_editor_core::ids::{
    INSP_RAY_FOLD_FILTER, INSP_RAY_FOLD_LAYERS, INSP_RAY_FOLD_SIGNALS,
};

/// O grupo «Filter» (GEF 07, versão C): as caixas, a escolha «Only see» e a tag.
pub const INSP_RAY_IGNORE_BODY: NodeId = hash_node_id("insp_ray_ignore_body");
pub const INSP_RAY_IGNORE_PARTS: NodeId = hash_node_id("insp_ray_ignore_parts");
/// «Ignore up to» (C8): a caixa do nome e o conta-gotas.
pub const INSP_RAY_UP_TO: NodeId = hash_node_id("insp_ray_up_to");
pub const INSP_RAY_UP_TO_PICK: NodeId = hash_node_id("insp_ray_up_to_pick");
/// «Only see»: Everything · Only tag · All except tag (a ordem do `RaySee`).
pub const INSP_RAY_ONLY_SEE: [NodeId; 3] = [
    hash_node_id("insp_ray_see_everything"),
    hash_node_id("insp_ray_see_only_tag"),
    hash_node_id("insp_ray_see_except_tag"),
];
/// O chip da tag do filtro — as opções são o [`super::INSP_RAY_SEE_TAG_OPT`].
pub const INSP_RAY_SEE_TAG: NodeId = hash_node_id("insp_ray_see_tag");
pub const INSP_RAY_WALLS_BLOCK: NodeId = hash_node_id("insp_ray_walls_block");
pub const INSP_RAY_HIT_TRIGGERS: NodeId = hash_node_id("insp_ray_hit_triggers");
pub const INSP_RAY_STARTS_INSIDE: NodeId = hash_node_id("insp_ray_starts_inside");
pub const INSP_RAY_ALL_HITS: NodeId = hash_node_id("insp_ray_all_hits");
/// As melhorias (RS, dono 15/11): a caixa «Enabled», a forma do feixe, o alvo e a lista pelo conta-gotas.
pub const INSP_RAY_ENABLED: NodeId = hash_node_id("insp_ray_enabled");
pub const INSP_RAY_THICKNESS: NodeId = hash_node_id("insp_ray_thickness");
pub const INSP_RAY_CONE: NodeId = hash_node_id("insp_ray_cone");
/// «Aim at»: a caixa do nome e o conta-gotas.
pub const INSP_RAY_AIM: NodeId = hash_node_id("insp_ray_aim");
pub const INSP_RAY_AIM_PICK: NodeId = hash_node_id("insp_ray_aim_pick");
/// «Also ignore»: o conta-gotas que junta um objecto; os chips são o [`super::INSP_RAY_IGNORE_CHIP`].
pub const INSP_RAY_IGNORE_PICK: NodeId = hash_node_id("insp_ray_ignore_pick");
/// «Shape» (dono 15/11): a lista suspensa Line · Thick · Cone e as opções dela, na ordem do `RayShapeKind::ALL`.
pub const INSP_RAY_SHAPE: NodeId = hash_node_id("insp_ray_shape");
pub const INSP_RAY_SHAPE_OPTS: [NodeId; 3] = [
    hash_node_id("insp_ray_shape_line"),
    hash_node_id("insp_ray_shape_thick"),
    hash_node_id("insp_ray_shape_cone"),
];
