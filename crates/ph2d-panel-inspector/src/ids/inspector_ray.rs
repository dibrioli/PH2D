//! **Os ids da secção RAY SENSOR** (suplente #21).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — o mesmo corte do
//! [`super::inspector_projectile`].
//!
//! # ⚠️ Não há segmentado nenhum, e isso é o desenho
//!
//! O `RaySensor` são seis números e o `RaySignals` dois nomes: ⇒ **não há posição-no-array a ser
//! tag de clique aqui**, que é a armadilha que os três segmentados do `SCULPT3D_POSE_MODE`
//! carregam, e que o `SignalVerb::ALL` desta linha pagou em 19/09 (o verbo existia, tinha lei e
//! gates, e o artista não lhe chegava porque o array de ids ficou com um a menos).

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// Onde o raio nasce, em coordenadas **locais** — o eixo X.
pub const INSP_RAY_ORIGIN_X: NodeId = hash_node_id("insp_ray_origin_x");
/// …e o eixo Y.
pub const INSP_RAY_ORIGIN_Y: NodeId = hash_node_id("insp_ray_origin_y");
/// Para onde ele aponta, em coordenadas **locais** — o eixo X.
///
/// ⚠️ **Não precisa de vir normalizado** (a porta do motor normaliza-o), mas ⛔ **nulo não é um
/// raio**: a porta devolve `None`, e a secção diz-o na primeira linha da queixa.
pub const INSP_RAY_DIR_X: NodeId = hash_node_id("insp_ray_dir_x");
/// …e o eixo Y.
pub const INSP_RAY_DIR_Y: NodeId = hash_node_id("insp_ray_dir_y");
/// Até onde ele enxerga, em metros.
pub const INSP_RAY_REACH: NodeId = hash_node_id("insp_ray_reach");
/// A camada de colisão pela qual ele pergunta — a matriz do mundo decide o que ele vê.
pub const INSP_RAY_LAYER: NodeId = hash_node_id("insp_ray_layer");
/// O nome publicado quando ele **passa a ver** — ⚠️ vazio = calado, a regra do `SignalOnHit`.
pub const INSP_RAY_ON_ENTER: NodeId = hash_node_id("insp_ray_on_enter");
/// …e quando ele **deixa de ver**.
pub const INSP_RAY_ON_EXIT: NodeId = hash_node_id("insp_ray_on_exit");
