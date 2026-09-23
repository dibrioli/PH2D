//! **Os ids da secção PARALLAX** (plano 24, W7).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — o mesmo corte do [`super::inspector_ray`].
//!
//! # ⚠️ Oito números e ZERO segmentados, e isso é o desenho
//!
//! Os quatro componentes da paralaxe são pares de `f32` e mais nada: ⇒ **não há posição-no-array a
//! ser tag de clique aqui**, que é a armadilha que o `SignalVerb::ALL` desta linha pagou em 19/09
//! (o verbo existia, tinha lei e gates, e o artista não lhe chegava porque o array de ids ficou com
//! um a menos).

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// ⭐⭐⭐ **Quanto do movimento do mundo esta camada guarda** — o eixo X. `1` = o mundo; `0` = presa
/// ao ecrã.
pub const INSP_PARALLAX_K_X: NodeId = hash_node_id("insp_parallax_k_x");
/// …e o eixo Y.
pub const INSP_PARALLAX_K_Y: NodeId = hash_node_id("insp_parallax_k_y");
/// ⭐ **O ladrilho da repetição**, em METROS — o eixo X. ⚠️ `0` = sem repetição nesse eixo, e é a
/// convenção do componente (ver [`ph2d_ecs::ScrollRepeat`]).
pub const INSP_PARALLAX_TILE_X: NodeId = hash_node_id("insp_parallax_tile_x");
/// …e o eixo Y.
pub const INSP_PARALLAX_TILE_Y: NodeId = hash_node_id("insp_parallax_tile_y");
/// ⭐ **A deriva própria**, em metros por SEGUNDO — o eixo X. O eixo do tempo é o playhead, que é o
/// que a faz sobreviver a um scrub.
pub const INSP_PARALLAX_VEL_X: NodeId = hash_node_id("insp_parallax_vel_x");
/// …e o eixo Y.
pub const INSP_PARALLAX_VEL_Y: NodeId = hash_node_id("insp_parallax_vel_y");
/// ⭐ **A borda de que esta camada não sai** — o canto mínimo, eixo X.
pub const INSP_PARALLAX_MIN_X: NodeId = hash_node_id("insp_parallax_min_x");
/// …e o eixo Y.
pub const INSP_PARALLAX_MIN_Y: NodeId = hash_node_id("insp_parallax_min_y");
/// …o canto máximo, eixo X.
pub const INSP_PARALLAX_MAX_X: NodeId = hash_node_id("insp_parallax_max_x");
/// …e o eixo Y.
pub const INSP_PARALLAX_MAX_Y: NodeId = hash_node_id("insp_parallax_max_y");
