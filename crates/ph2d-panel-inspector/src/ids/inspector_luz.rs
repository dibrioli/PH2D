//! **Os ids da secção LUZ** (Luz 2D, W4 — `docs/Luz2D/01_plano.md` §4).

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// Os chips do TIPO — a posição É a tag (`luz_edits::TIPOS`), com gate de ida-e-volta.
pub const INSP_LUZ_TIPO: [NodeId; 2] = [
    hash_node_id("insp_luz_tipo_0"),
    hash_node_id("insp_luz_tipo_1"),
];
/// A amostra da COR (abre o seletor; a escolha volta pelo `sync_luz`).
pub const INSP_LUZ_COR: NodeId = hash_node_id("insp_luz_cor");
/// A fluência a 1 m.
pub const INSP_LUZ_INTENSIDADE: NodeId = hash_node_id("insp_luz_intensidade");
/// O raio da fonte, em metros.
pub const INSP_LUZ_TAMANHO: NodeId = hash_node_id("insp_luz_tamanho");
/// «Limitar o alcance» — desligada, o alcance é `∞` (física pura).
pub const INSP_LUZ_ALCANCE_LIGADO: NodeId = hash_node_id("insp_luz_alcance_ligado");
/// O alcance da queda autoral, em metros.
pub const INSP_LUZ_ALCANCE: NodeId = hash_node_id("insp_luz_alcance");
