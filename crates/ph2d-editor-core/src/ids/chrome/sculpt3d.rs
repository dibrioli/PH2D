//! **O painel da cena 3D** (`SCULPT3D_*`) — ADR-0150, W12.
//!
//! A família de ids do `ph2d-panel-sculpt3d`: a lista de ferramentas, os knobs
//! do pincel, o espelho, a topologia, o sombreamento e a lista de peças.
//!
//! Slug pontilhado (`sculpt3d.*`), como a família do painel de física. Hash de
//! string, então nenhum contador de id se move — o `node_id_collisions` varre
//! estas chaves como varre as outras.

use super::{NodeId, hash_node_id};

/// Retângulo externo do painel (para o `z_order` + a barreira de hit).
pub const SCULPT3D_PANEL: NodeId = hash_node_id("sculpt3d.panel");
