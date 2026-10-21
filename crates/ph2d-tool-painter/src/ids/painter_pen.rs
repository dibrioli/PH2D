//! O cartão **Pen** do pincel (L2, `docs/Painter/Pinceis/03_MOTORES.md` §4): a mão — pressão e inclinação —
//! nos QUATRO meios, uma linha por [`crate::LinhaDaMao`] (a ordem de [`crate::LINHAS_DA_MAO`]). No Wet Paint
//! as linhas escrevem os knobs da caneta (as mesmas do painel Tuning); nos outros meios os campos do pincel.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

/// O cabeçalho do cartão Pen (secção do corpo que se arrasta). Click dobra.
pub const PAINTER_PEN_SECTION: NodeId = hash_node_id("painter_brush.pen_section");

/// O Reset do cartão — a mão de fábrica do meio de agora.
pub const PAINTER_PEN_RESET: NodeId = hash_node_id("painter_brush.pen_reset");

/// As linhas numéricas do cartão, pela ordem de [`crate::LINHAS_DA_MAO`] — UMA lista para o reencaminhar
/// do painel (`number_field::is_param_field`), o `populate` e o encaminhamento do tool.
pub const PAINTER_PEN_FIELDS: [NodeId; 20] = [
    hash_node_id("painter_brush.pen.pressureSize"),
    hash_node_id("painter_brush.pen.pressureSizeCurve"),
    hash_node_id("painter_brush.pen.pressureOpacity"),
    hash_node_id("painter_brush.pen.pressureOpacityCurve"),
    hash_node_id("painter_brush.pen.pressureFloor"),
    hash_node_id("painter_brush.pen.pressureCeiling"),
    hash_node_id("painter_brush.pen.pressureSmoothing"),
    hash_node_id("painter_brush.pen.pulledString"),
    hash_node_id("painter_brush.pen.dabRotation"),
    hash_node_id("painter_brush.pen.tiltStart"),
    hash_node_id("painter_brush.pen.tiltFull"),
    hash_node_id("painter_brush.pen.tiltWidth"),
    hash_node_id("painter_brush.pen.tiltLength"),
    hash_node_id("painter_brush.pen.tiltSmooth"),
    hash_node_id("painter_brush.pen.tiltLag"),
    hash_node_id("painter_brush.pen.tiltOpacity"),
    hash_node_id("painter_brush.pen.speedSize"),
    hash_node_id("painter_brush.pen.speedOpacity"),
    hash_node_id("painter_brush.pen.pressureSCurve"),
    hash_node_id("painter_brush.pen.pressureJitter"),
];

/// A linha da rotação do carimbo é um MENU (None · Follow · Pen tilt), não um número: o id da linha 8.
pub const PAINTER_PEN_ROTATION: NodeId = PAINTER_PEN_FIELDS[8];

/// Quantas opções o menu da rotação tem (`DabRotation`: None · FollowTrajectory · PenTilt).
pub const PAINTER_PEN_ROTATION_COUNT: u8 = 3;

/// A opção `i` do menu da rotação.
#[must_use]
pub fn painter_pen_rotation_option_id(i: u8) -> NodeId {
    hash_node_id_runtime(&format!("painter_brush.pen.rotation_opt.{i}"))
}
