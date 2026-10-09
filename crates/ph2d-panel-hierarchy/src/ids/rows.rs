//! Os ids SINTÉTICOS das linhas novas — a Cena, o conteúdo e os ícones de componente.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// ⚠️ **Um id de linha nunca tem os bits dos companheiros** (58–62, `ph2d_editor_core::ids::*_BIT`):
/// `linha | EXPAND_TOGGLE_BIT` é o galo dela, e um id que já os tivesse seria lido como o galo de
/// OUTRA linha. Os ids sintéticos abaixo vivem nos 58 bits de baixo, como os da ponte.
pub(super) const ROW_ID_MASK: u64 = (1u64 << 58) - 1;

/// A raiz CENA ([`ph2d_editor_core::hier_rows::RowRole::Scene`]).
pub const HIER_SCENE_ROW: NodeId = NodeId(hash_node_id("hier_scene_row").0 & ROW_ID_MASK);

/// SplitMix64 — mistura estável de um `u64` (os ids sintéticos têm de sobreviver entre quadros).
pub(super) const fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A linha de um grupo de conteúdo (`key = None`) ou de uma peça dele, sob a linha `owner`.
#[must_use]
pub fn hier_content_row(owner: NodeId, group: u8, key: Option<u64>) -> NodeId {
    let g = mix(owner.0 ^ mix(u64::from(group) + 1));
    let h = match key {
        None => mix(g ^ 0xC0),
        Some(k) => mix(g ^ mix(k ^ 0xC1)),
    };
    NodeId((h & ROW_ID_MASK).max(1))
}

/// O ícone da família `category` na linha `row` — o clique traz o cartão dela à vista.
#[must_use]
pub fn hier_component_icon(
    row: NodeId,
    category: ph2d_component_desc::ComponentCategory,
) -> NodeId {
    NodeId((mix(row.0 ^ mix(0xA0 + category as u64)) & ROW_ID_MASK).max(1))
}
