//! **Os ids da janela GRID & BACKGROUND.** ⚠️ APPEND-ONLY: um id é o hash de uma STRING.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

use crate::canal::{Caixa, Cor, Numero};

pub const PANEL: NodeId = ph2d_editor_core::ids::PIXEL_PREFS_PANEL;
pub const PREFS_DRAG: NodeId = hash_node_id("pixel_prefs.drag");
pub const PREFS_EDGE: NodeId = hash_node_id("pixel_prefs.edge");
pub const SEC_SCOPE: NodeId = hash_node_id("pixel_prefs.sec.scope");
pub const PREFS_SEC_GRID: NodeId = hash_node_id("pixel_prefs.sec.grid");
pub const SEC_PIXEL_GRID: NodeId = hash_node_id("pixel_prefs.sec.pixel_grid");
pub const SEC_CHECKER: NodeId = hash_node_id("pixel_prefs.sec.checker");
pub const SEC_ONION: NodeId = hash_node_id("pixel_prefs.sec.onion");
/// Os segmentos `Behind` / `In Front` e `Merge` / `Red/Blue Tint` do Onion Skin.
pub const ONION_BEHIND: NodeId = hash_node_id("pixel_prefs.onion.behind");
pub const ONION_MERGE: NodeId = hash_node_id("pixel_prefs.onion.merge");
pub const SCOPE_DOC: NodeId = hash_node_id("pixel_prefs.scope.doc");
pub const SCOPE_NEW: NodeId = hash_node_id("pixel_prefs.scope.new");
pub const SET_DEFAULTS: NodeId = hash_node_id("pixel_prefs.set_defaults");
pub const PREFS_RESET: NodeId = hash_node_id("pixel_prefs.reset");

/// Os números, cada um com o seu campo.
pub const NUMEROS: [(NodeId, Numero); 12] = [
    (hash_node_id("pixel_prefs.grid.x"), Numero::GridX),
    (hash_node_id("pixel_prefs.grid.y"), Numero::GridY),
    (hash_node_id("pixel_prefs.grid.w"), Numero::GridW),
    (hash_node_id("pixel_prefs.grid.h"), Numero::GridH),
    (
        hash_node_id("pixel_prefs.grid.opacity"),
        Numero::GridOpacity,
    ),
    (
        hash_node_id("pixel_prefs.pixel_grid.opacity"),
        Numero::PixelOpacity,
    ),
    (hash_node_id("pixel_prefs.checker.w"), Numero::CheckerW),
    (hash_node_id("pixel_prefs.checker.h"), Numero::CheckerH),
    (hash_node_id("pixel_prefs.onion.prev"), Numero::OnionPrev),
    (hash_node_id("pixel_prefs.onion.next"), Numero::OnionNext),
    (
        hash_node_id("pixel_prefs.onion.opacity"),
        Numero::OnionOpacity,
    ),
    (hash_node_id("pixel_prefs.onion.step"), Numero::OnionStep),
];

/// As caixas de marcar.
pub const CAIXAS: [(NodeId, Caixa); 9] = [
    (hash_node_id("pixel_prefs.grid.visible"), Caixa::GridVisible),
    (hash_node_id("pixel_prefs.grid.auto"), Caixa::GridAuto),
    (
        hash_node_id("pixel_prefs.pixel_grid.visible"),
        Caixa::PixelVisible,
    ),
    (
        hash_node_id("pixel_prefs.pixel_grid.auto"),
        Caixa::PixelAuto,
    ),
    (hash_node_id("pixel_prefs.checker.zoom"), Caixa::CheckerZoom),
    (
        hash_node_id("pixel_prefs.onion.loop_tag"),
        Caixa::OnionLoopTag,
    ),
    (
        hash_node_id("pixel_prefs.onion.current_layer"),
        Caixa::OnionCurrentLayer,
    ),
    (hash_node_id("pixel_prefs.onion.front"), Caixa::OnionFront),
    (hash_node_id("pixel_prefs.onion.tint"), Caixa::OnionTint),
];

/// As amostras de cor.
pub const CORES: [(NodeId, Cor); 4] = [
    (hash_node_id("pixel_prefs.grid.color"), Cor::Grid),
    (hash_node_id("pixel_prefs.pixel_grid.color"), Cor::PixelGrid),
    (hash_node_id("pixel_prefs.checker.color_a"), Cor::CheckerA),
    (hash_node_id("pixel_prefs.checker.color_b"), Cor::CheckerB),
];

/// O id de um número.
#[must_use]
pub fn numero(n: Numero) -> NodeId {
    NUMEROS
        .iter()
        .find(|(_, x)| *x == n)
        .map_or(PANEL, |(id, _)| *id)
}

/// O id de uma caixa.
#[must_use]
pub fn caixa(c: Caixa) -> NodeId {
    CAIXAS
        .iter()
        .find(|(_, x)| *x == c)
        .map_or(PANEL, |(id, _)| *id)
}

/// O id de uma amostra.
#[must_use]
pub fn cor(c: Cor) -> NodeId {
    CORES
        .iter()
        .find(|(_, x)| *x == c)
        .map_or(PANEL, |(id, _)| *id)
}
