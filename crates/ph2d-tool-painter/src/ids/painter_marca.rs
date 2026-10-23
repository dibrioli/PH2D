//! As linhas da **marca** no cartão Jitter (L4, `docs/Painter/Pinceis/03_MOTORES.md` §4): uma por
//! [`crate::LinhaDaMarca`], pela ordem de [`crate::LINHAS_DA_MARCA`].

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// As linhas numéricas da marca — UMA lista para o reencaminhar do painel (`number_field::is_param_field`),
/// o `populate` e o encaminhamento do tool.
pub const PAINTER_MARCA_FIELDS: [NodeId; 13] = [
    hash_node_id("painter_brush.marca.opacity"),
    hash_node_id("painter_brush.marca.count"),
    hash_node_id("painter_brush.marca.roundness"),
    hash_node_id("painter_brush.marca.flipX"),
    hash_node_id("painter_brush.marca.flipY"),
    hash_node_id("painter_brush.marca.anglePerStroke"),
    hash_node_id("painter_brush.marca.across"),
    hash_node_id("painter_brush.marca.particle"),
    hash_node_id("painter_brush.marca.secondaryJitter"),
    hash_node_id("painter_brush.marca.pressureSecondary"),
    hash_node_id("painter_brush.marca.pressureHue"),
    hash_node_id("painter_brush.marca.pressureSat"),
    hash_node_id("painter_brush.marca.pressureVal"),
];

/// A amostra da cor SECUNDÁRIA (L11) na secção Randomize Color: o clique abre o seletor partilhado, que devolve
/// `SelectOption` «r,g,b».
pub const PAINTER_MARCA_SECONDARY: NodeId = hash_node_id("painter_brush.marca.secondary");

/// O chip numérico ligado à barra de cada linha da marca, pela ordem de [`PAINTER_MARCA_FIELDS`] — só as do cartão
/// Jitter se registam e pintam (a barra é o próprio id da linha; o chip mostra e aceita o valor VERDADEIRO dela).
pub const PAINTER_MARCA_CHIPS: [NodeId; 13] = [
    hash_node_id("painter_brush.marca.opacity.chip"),
    hash_node_id("painter_brush.marca.count.chip"),
    hash_node_id("painter_brush.marca.roundness.chip"),
    hash_node_id("painter_brush.marca.flipX.chip"),
    hash_node_id("painter_brush.marca.flipY.chip"),
    hash_node_id("painter_brush.marca.anglePerStroke.chip"),
    hash_node_id("painter_brush.marca.across.chip"),
    hash_node_id("painter_brush.marca.particle.chip"),
    hash_node_id("painter_brush.marca.secondaryJitter.chip"),
    hash_node_id("painter_brush.marca.pressureSecondary.chip"),
    hash_node_id("painter_brush.marca.pressureHue.chip"),
    hash_node_id("painter_brush.marca.pressureSat.chip"),
    hash_node_id("painter_brush.marca.pressureVal.chip"),
];
