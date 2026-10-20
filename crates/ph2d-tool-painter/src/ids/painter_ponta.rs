//! As linhas da **ponta** (L9 · L18 · L19, `docs/Painter/Pinceis/03_MOTORES.md` §4): uma por
//! [`crate::LinhaDaPonta`], pela ordem de [`crate::LINHAS_DA_PONTA`].

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// As linhas numéricas da ponta — UMA lista para o reencaminhar do painel (`number_field::is_param_field`), o
/// `populate` e o encaminhamento do tool.
pub const PAINTER_PONTA_FIELDS: [NodeId; 5] = [
    hash_node_id("painter_brush.ponta.pixel"),
    hash_node_id("painter_brush.ponta.grainMode"),
    hash_node_id("painter_brush.ponta.depthMin"),
    hash_node_id("painter_brush.ponta.depthJitter"),
    hash_node_id("painter_brush.ponta.depthPressure"),
];

/// A opção `i` do menu da linha `linha` da ponta (as linhas de MODO: a ponta de píxel, o modo do grão, o controlo
/// da profundidade).
#[must_use]
pub fn painter_ponta_option_id(linha: usize, i: u8) -> NodeId {
    ph2d_tool_registry::hash_node_id_runtime(&format!("painter_brush.ponta.opt.{linha}.{i}"))
}
