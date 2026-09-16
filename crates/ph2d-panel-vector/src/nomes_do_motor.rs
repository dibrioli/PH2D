//! ⭐⭐⭐ **OS NOMES QUE O MOTOR PUBLICA PARA ESTE PAINEL — da crate do motor para a tabela de strings.**
//!
//! O painel de Vector auto-popula-se de tabelas do MOTOR: os tipos de efeito
//! (`PathEffect::KINDS`) e os parâmetros de cada um, os tipos de filtro raster (`FxOp::SPECS`) com
//! os rótulos dos controlos e dos modos, as leis de mistura (`BlendMode::name`) e os presets da
//! gaiola (`WarpStyle::label`). Até 2026-09-16 estes nomes eram pintados CRUS — e a régua do painel
//! não os via, porque o literal mora do outro lado da fronteira (a forma do `adjust_nomes` do
//! painel do Painter, §37.3 do handoff de 14/09).
//!
//! ⇒ aqui o rótulo do motor é um **identificador** (o braço de um `match`, que a régua lexical
//! isenta) e o que se pinta é a chave. ⚠️ **Um `match` POR FAMÍLIA**, porque o mesmo rótulo diz
//! coisas diferentes: `Color` é uma lei de mistura e o rótulo da cor de um filtro; `Amount` é um
//! parâmetro de efeito e um controlo de filtro.
//!
//! O gate `every_name_the_engine_publishes_has_a_key` varre as quatro tabelas do motor (e as
//! variantes que um interruptor acorda, como o `Roughen` do Zig Zag) e exige uma chave para cada
//! rótulo — o `None` nunca pode acontecer.

use ph2d_i18n::tr;

/// O nome a pintar para uma chave encontrada — ou o rótulo do motor, se a tabela não o conhece.
///
/// ⚠️ **O rótulo cru é a DEGRADAÇÃO, não o caminho:** o gate exige que o `None` nunca aconteça
/// para o que o motor publica, e um nome em inglês ainda diz mais ao artista do que uma linha sem
/// nome. ⛔ Sem `debug_assert`: os testes de costura publicam tabelas SINTÉTICAS (um efeito
/// chamado `Blur`), e a cobertura real é medida contra o motor, não contra quem publica.
fn pintar(chave: Option<&'static str>, fonte: &'static str) -> &'static str {
    chave.map_or(fonte, tr)
}

/// O nome de um efeito (ou preset da gaiola), traduzido.
pub(crate) fn efeito(fonte: &'static str) -> &'static str {
    pintar(chave_do_efeito(fonte), fonte)
}

/// O nome de um parâmetro de efeito, traduzido.
pub(crate) fn parametro(fonte: &'static str) -> &'static str {
    pintar(chave_do_parametro(fonte), fonte)
}

/// O nome de um tipo de filtro, traduzido.
pub(crate) fn filtro(fonte: &'static str) -> &'static str {
    pintar(chave_do_filtro(fonte), fonte)
}

/// Um rótulo de controlo de filtro, traduzido.
pub(crate) fn controlo(fonte: &'static str) -> &'static str {
    pintar(chave_do_controlo(fonte), fonte)
}

/// Um modo de filtro, traduzido.
pub(crate) fn modo(fonte: &'static str) -> &'static str {
    pintar(chave_do_modo(fonte), fonte)
}

/// Uma lei de mistura, traduzida.
pub(crate) fn mistura(fonte: &'static str) -> &'static str {
    pintar(chave_da_mistura(fonte), fonte)
}

/// A chave do nome de um tipo de EFEITO de caminho (e de um preset da gaiola, que são os mesmos
/// estilos de warp).
#[must_use]
pub fn chave_do_efeito(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Arc" => "panel.vector.engine.efeito.arc",
        "Arc Lower" => "panel.vector.engine.efeito.arc_lower",
        "Arc Upper" => "panel.vector.engine.efeito.arc_upper",
        "Bulge" => "panel.vector.engine.efeito.bulge",
        "Falloff Linear" => "panel.vector.engine.efeito.falloff_linear",
        "Falloff Radial" => "panel.vector.engine.efeito.falloff_radial",
        "Falloff Rect" => "panel.vector.engine.efeito.falloff_rect",
        "Falloff Sweep" => "panel.vector.engine.efeito.falloff_sweep",
        "Fisheye" => "panel.vector.engine.efeito.fisheye",
        "Flag" => "panel.vector.engine.efeito.flag",
        "Hatch" => "panel.vector.engine.efeito.hatch",
        "Knot" => "panel.vector.engine.efeito.knot",
        "Pucker & Bloat" => "panel.vector.engine.efeito.pucker_and_bloat",
        "Repeater" => "panel.vector.engine.efeito.repeater",
        "Rise" => "panel.vector.engine.efeito.rise",
        "Roughen" => "panel.vector.engine.efeito.roughen",
        "Sketch" => "panel.vector.engine.efeito.sketch",
        "Squeeze" => "panel.vector.engine.efeito.squeeze",
        "Trim Path" => "panel.vector.engine.efeito.trim_path",
        "Twist" => "panel.vector.engine.efeito.twist",
        "Wave" => "panel.vector.engine.efeito.wave",
        "Zig Zag" => "panel.vector.engine.efeito.zig_zag",
        _ => return None,
    })
}

/// A chave do nome de um PARÂMETRO de efeito de caminho.
#[must_use]
pub fn chave_do_parametro(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Amount" => "panel.vector.engine.parametro.amount",
        "Angle" => "panel.vector.engine.parametro.angle",
        "Bend" => "panel.vector.engine.parametro.bend",
        "Center X" => "panel.vector.engine.parametro.center_x",
        "Center Y" => "panel.vector.engine.parametro.center_y",
        "Copies X" => "panel.vector.engine.parametro.copies_x",
        "Copies Y" => "panel.vector.engine.parametro.copies_y",
        "Cross" => "panel.vector.engine.parametro.cross",
        "Curve" => "panel.vector.engine.parametro.curve",
        "Detail" => "panel.vector.engine.parametro.detail",
        "End" => "panel.vector.engine.parametro.end",
        "Gap" => "panel.vector.engine.parametro.gap",
        "Horizontal" => "panel.vector.engine.parametro.horizontal",
        "Invert" => "panel.vector.engine.parametro.invert",
        "Move X" => "panel.vector.engine.parametro.move_x",
        "Move Y" => "panel.vector.engine.parametro.move_y",
        "Offset" => "panel.vector.engine.parametro.offset",
        "Orbit" => "panel.vector.engine.parametro.orbit",
        "Passes" => "panel.vector.engine.parametro.passes",
        "Radius" => "panel.vector.engine.parametro.radius",
        "Ridges" => "panel.vector.engine.parametro.ridges",
        "Rough" => "panel.vector.engine.parametro.rough",
        "Roughness" => "panel.vector.engine.parametro.roughness",
        "Seed" => "panel.vector.engine.parametro.seed",
        "Size" => "panel.vector.engine.parametro.size",
        "Smooth" => "panel.vector.engine.parametro.smooth",
        "Softness" => "panel.vector.engine.parametro.softness",
        "Spacing" => "panel.vector.engine.parametro.spacing",
        "Spin" => "panel.vector.engine.parametro.spin",
        "Spread" => "panel.vector.engine.parametro.spread",
        "Start" => "panel.vector.engine.parametro.start",
        "Swap" => "panel.vector.engine.parametro.swap",
        "Vertical" => "panel.vector.engine.parametro.vertical",
        _ => return None,
    })
}

/// A chave do nome de um tipo de FILTRO raster.
#[must_use]
pub fn chave_do_filtro(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Bevel" => "panel.vector.engine.filtro.bevel",
        "Blur" => "panel.vector.engine.filtro.blur",
        "Color Adjust" => "panel.vector.engine.filtro.color_adjust",
        "Color Overlay" => "panel.vector.engine.filtro.color_overlay",
        "Drop Shadow" => "panel.vector.engine.filtro.drop_shadow",
        "Duotone" => "panel.vector.engine.filtro.duotone",
        "Feather" => "panel.vector.engine.filtro.feather",
        "Glow" => "panel.vector.engine.filtro.glow",
        "Gradient Map" => "panel.vector.engine.filtro.gradient_map",
        "Grow / Shrink" => "panel.vector.engine.filtro.grow_shrink",
        "Inner Glow" => "panel.vector.engine.filtro.inner_glow",
        "Inner Shadow" => "panel.vector.engine.filtro.inner_shadow",
        "Luma to Alpha" => "panel.vector.engine.filtro.luma_to_alpha",
        "Outline" => "panel.vector.engine.filtro.outline",
        "Turbulence" => "panel.vector.engine.filtro.turbulence",
        _ => return None,
    })
}

/// A chave de um rótulo de CONTROLO de filtro (raio, offset, cor, ruído, crescimento, ajuste).
#[must_use]
pub fn chave_do_controlo(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Amount" => "panel.vector.engine.controlo.amount",
        "Brightness" => "panel.vector.engine.controlo.brightness",
        "Color" => "panel.vector.engine.controlo.color",
        "Depth" => "panel.vector.engine.controlo.depth",
        "Detail" => "panel.vector.engine.controlo.detail",
        "Feather" => "panel.vector.engine.controlo.feather",
        "Highlights" => "panel.vector.engine.controlo.highlights",
        "Hue" => "panel.vector.engine.controlo.hue",
        "Light X" => "panel.vector.engine.controlo.light_x",
        "Light Y" => "panel.vector.engine.controlo.light_y",
        "Offset X" => "panel.vector.engine.controlo.offset_x",
        "Offset Y" => "panel.vector.engine.controlo.offset_y",
        "Radius" => "panel.vector.engine.controlo.radius",
        "Saturation" => "panel.vector.engine.controlo.saturation",
        "Seed" => "panel.vector.engine.controlo.seed",
        "Shadow" => "panel.vector.engine.controlo.shadow",
        "Shadows" => "panel.vector.engine.controlo.shadows",
        "Size" => "panel.vector.engine.controlo.size",
        "Width" => "panel.vector.engine.controlo.width",
        _ => return None,
    })
}

/// A chave de um MODO de filtro (os chips de escolha de um tipo).
#[must_use]
pub fn chave_do_modo(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Contour" => "panel.vector.engine.modo.contour",
        "Creased" => "panel.vector.engine.modo.creased",
        "Linear" => "panel.vector.engine.modo.linear",
        "Proximity" => "panel.vector.engine.modo.proximity",
        "Smooth" => "panel.vector.engine.modo.smooth",
        _ => return None,
    })
}

/// A chave de uma LEI DE MISTURA.
#[must_use]
pub fn chave_da_mistura(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Add" => "panel.vector.engine.mistura.add",
        "Color" => "panel.vector.engine.mistura.color",
        "Color Burn" => "panel.vector.engine.mistura.color_burn",
        "Color Dodge" => "panel.vector.engine.mistura.color_dodge",
        "Darken" => "panel.vector.engine.mistura.darken",
        "Difference" => "panel.vector.engine.mistura.difference",
        "Exclusion" => "panel.vector.engine.mistura.exclusion",
        "Hard Light" => "panel.vector.engine.mistura.hard_light",
        "Hue" => "panel.vector.engine.mistura.hue",
        "Lighten" => "panel.vector.engine.mistura.lighten",
        "Linear Burn" => "panel.vector.engine.mistura.linear_burn",
        "Linear Light" => "panel.vector.engine.mistura.linear_light",
        "Luminosity" => "panel.vector.engine.mistura.luminosity",
        "Multiply" => "panel.vector.engine.mistura.multiply",
        "Normal" => "panel.vector.engine.mistura.normal",
        "Overlay" => "panel.vector.engine.mistura.overlay",
        "Saturation" => "panel.vector.engine.mistura.saturation",
        "Screen" => "panel.vector.engine.mistura.screen",
        "Soft Light" => "panel.vector.engine.mistura.soft_light",
        "Vivid Light" => "panel.vector.engine.mistura.vivid_light",
        _ => return None,
    })
}
