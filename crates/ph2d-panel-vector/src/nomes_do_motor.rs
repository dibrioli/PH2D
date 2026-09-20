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

// ⭐⭐⭐ **AS CINCO FAMÍLIAS DO CATÁLOGO** (2026-09-17) — o que faltava do motor do vector.
// ⚠️ Cinco e não uma, com a colisão MEDIDA: `Corner` e `Curve` são um campo de FORMA e um campo de
// CONECTOR, e uma só tabela daria a uma delas a palavra da outra.
//
// ⛔⛔ **A sexta família NÃO entra, e a razão é uma medição que eu devia ter feito primeiro:** as
// FAMÍLIAS do catálogo (`Basic`/`Round`/`Arrows`/…) já têm ponte desde antes — o
// [`crate::state::group_i18n_key`], chaveado pela **VARIANTE do enum** e não pela palavra inglesa,
// que é a forma MAIS forte. Eu contei-as no censo lexical da crate do motor e ia construir a
// segunda resposta para a mesma pergunta. *Antes de migrar um rótulo do motor, pergunte se o painel
// já o traduz* (`CLAUDE.md` §5.0).

/// As FORMAS do catálogo, traduzido.
pub(crate) fn forma(fonte: &'static str) -> &'static str {
    pintar(chave_da_forma(fonte), fonte)
}

/// Os PARÂMETROS de uma forma, traduzido.
pub(crate) fn campo(fonte: &'static str) -> &'static str {
    pintar(chave_do_campo(fonte), fonte)
}

/// O conector (campos e rotas), traduzido.
pub(crate) fn conector(fonte: &'static str) -> &'static str {
    pintar(chave_do_conector(fonte), fonte)
}

/// Os presets de MOLDURA, traduzido.
pub(crate) fn moldura(fonte: &'static str) -> &'static str {
    pintar(chave_da_moldura(fonte), fonte)
}

/// As pontas de traço (campos e estados), traduzido.
pub(crate) fn marcador(fonte: &'static str) -> &'static str {
    pintar(chave_do_marcador(fonte), fonte)
}

/// Uma PONTA de traço, traduzida.
pub(crate) fn ponta(fonte: &'static str) -> &'static str {
    pintar(chave_da_ponta(fonte), fonte)
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
/// A chave de as FORMAS do catálogo.
pub fn chave_da_forma(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Arc" => "panel.vector.engine.forma.arc",
        "Arrow" => "panel.vector.engine.forma.arrow",
        "Banner" => "panel.vector.engine.forma.banner",
        "Bent" => "panel.vector.engine.forma.bent",
        "Bolt" => "panel.vector.engine.forma.bolt",
        "Bone" => "panel.vector.engine.forma.bone",
        "Brace" => "panel.vector.engine.forma.brace",
        "Burst" => "panel.vector.engine.forma.burst",
        "Check" => "panel.vector.engine.forma.check",
        "Chevron" => "panel.vector.engine.forma.chevron",
        "Cloud" => "panel.vector.engine.forma.cloud",
        "Cone" => "panel.vector.engine.forma.cone",
        "Cross" => "panel.vector.engine.forma.cross",
        "Cube" => "panel.vector.engine.forma.cube",
        "Data" => "panel.vector.engine.forma.data",
        "Database" => "panel.vector.engine.forma.database",
        "Decision" => "panel.vector.engine.forma.decision",
        "Delay" => "panel.vector.engine.forma.delay",
        "Display" => "panel.vector.engine.forma.display",
        "Document" => "panel.vector.engine.forma.document",
        "Double" => "panel.vector.engine.forma.double",
        "Drop" => "panel.vector.engine.forma.drop",
        "Gear" => "panel.vector.engine.forma.gear",
        "Heart" => "panel.vector.engine.forma.heart",
        "Junction" => "panel.vector.engine.forma.junction",
        "Line" => "panel.vector.engine.forma.line",
        "Manual in" => "panel.vector.engine.forma.manual_in",
        "Manual op" => "panel.vector.engine.forma.manual_op",
        "Moon" => "panel.vector.engine.forma.moon",
        "Note" => "panel.vector.engine.forma.note",
        "Off-page" => "panel.vector.engine.forma.off_page",
        "Oval" => "panel.vector.engine.forma.oval",
        "Oval say" => "panel.vector.engine.forma.oval_say",
        "Pie" => "panel.vector.engine.forma.pie",
        "Poly" => "panel.vector.engine.forma.poly",
        "Prepare" => "panel.vector.engine.forma.prepare",
        "Pyramid" => "panel.vector.engine.forma.pyramid",
        "Rect" => "panel.vector.engine.forma.rect",
        "Round" => "panel.vector.engine.forma.round",
        "Rope Segment" => "panel.vector.engine.forma.rope_segment",
        "Segment" => "panel.vector.engine.forma.segment",
        "Shield" => "panel.vector.engine.forma.shield",
        "Speech" => "panel.vector.engine.forma.speech",
        "Spiral" => "panel.vector.engine.forma.spiral",
        "Star" => "panel.vector.engine.forma.star",
        "Subroutine" => "panel.vector.engine.forma.subroutine",
        "Tag" => "panel.vector.engine.forma.tag",
        "Terminal" => "panel.vector.engine.forma.terminal",
        "Thought" => "panel.vector.engine.forma.thought",
        _ => return None,
    })
}
/// A chave de os PARÂMETROS de uma forma.
pub fn chave_do_campo(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Bubbles" => "panel.vector.engine.campo.bubbles",
        "Smooth" => "panel.vector.engine.campo.smooth",
        "Spikes" => "panel.vector.engine.campo.spikes",
        "Teeth" => "panel.vector.engine.campo.teeth",
        "Wedge" => "panel.vector.engine.campo.wedge",
        "From above" => "panel.vector.engine.campo.from_above",
        "From below" => "panel.vector.engine.campo.from_below",
        "Angular" => "panel.vector.engine.campo.angular",
        "Arm" => "panel.vector.engine.campo.arm",
        "BL offset" => "panel.vector.engine.campo.bl_offset",
        "BR offset" => "panel.vector.engine.campo.br_offset",
        "Bars" => "panel.vector.engine.campo.bars",
        "Base" => "panel.vector.engine.campo.base",
        "Bumps" => "panel.vector.engine.campo.bumps",
        "Cleft" => "panel.vector.engine.campo.cleft",
        "Cord" => "panel.vector.engine.campo.cord",
        "Corner" => "panel.vector.engine.campo.corner",
        "Curve" => "panel.vector.engine.campo.curve",
        "Cut" => "panel.vector.engine.campo.cut",
        "Depth" => "panel.vector.engine.campo.depth",
        "Eye" => "panel.vector.engine.campo.eye",
        "Head len" => "panel.vector.engine.campo.head_len",
        "Head width" => "panel.vector.engine.campo.head_width",
        "Hole" => "panel.vector.engine.campo.hole",
        "Inner" => "panel.vector.engine.campo.inner",
        "Irregular" => "panel.vector.engine.campo.irregular",
        "Lip" => "panel.vector.engine.campo.lip",
        "Nose" => "panel.vector.engine.campo.nose",
        "Node" => "panel.vector.engine.campo.node",
        "Notch" => "panel.vector.engine.campo.notch",
        "Notch round" => "panel.vector.engine.campo.notch_round",
        "Phase" => "panel.vector.engine.campo.phase",
        "Pinch" => "panel.vector.engine.campo.pinch",
        "Point" => "panel.vector.engine.campo.point",
        "Points" => "panel.vector.engine.campo.points",
        "Puff" => "panel.vector.engine.campo.puff",
        "Radial" => "panel.vector.engine.campo.radial",
        "Radius" => "panel.vector.engine.campo.radius",
        "Rise" => "panel.vector.engine.campo.rise",
        "Room" => "panel.vector.engine.campo.room",
        "Sides" => "panel.vector.engine.campo.sides",
        "Skew" => "panel.vector.engine.campo.skew",
        "Slant" => "panel.vector.engine.campo.slant",
        "Smoothing" => "panel.vector.engine.campo.smoothing",
        "Start" => "panel.vector.engine.campo.start",
        "Stem" => "panel.vector.engine.campo.stem",
        "Sweep" => "panel.vector.engine.campo.sweep",
        "TR offset" => "panel.vector.engine.campo.tr_offset",
        "Tail" => "panel.vector.engine.campo.tail",
        "Tip" => "panel.vector.engine.campo.tip",
        "Tip round" => "panel.vector.engine.campo.tip_round",
        "Tip x" => "panel.vector.engine.campo.tip_x",
        "Tip y" => "panel.vector.engine.campo.tip_y",
        "Turns" => "panel.vector.engine.campo.turns",
        "Viewed" => "panel.vector.engine.campo.viewed",
        "Waist" => "panel.vector.engine.campo.waist",
        "Wave" => "panel.vector.engine.campo.wave",
        "Weight" => "panel.vector.engine.campo.weight",
        _ => return None,
    })
}
/// A chave de o conector (campos e rotas).
pub fn chave_do_conector(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Corner" => "panel.vector.engine.conector.corner",
        "Curve" => "panel.vector.engine.conector.curve",
        "Curved" => "panel.vector.engine.conector.curved",
        "Jetty" => "panel.vector.engine.conector.jetty",
        "Orthogonal" => "panel.vector.engine.conector.orthogonal",
        "Route" => "panel.vector.engine.conector.route",
        "Spread" => "panel.vector.engine.conector.spread",
        "Straight" => "panel.vector.engine.conector.straight",
        _ => return None,
    })
}
/// A chave de os presets de MOLDURA.
pub fn chave_da_moldura(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Desktop" => "panel.vector.engine.moldura.desktop",
        "Phone" => "panel.vector.engine.moldura.phone",
        "Square" => "panel.vector.engine.moldura.square",
        "Tablet" => "panel.vector.engine.moldura.tablet",
        _ => return None,
    })
}
/// A chave de as pontas de traço (campos e estados).
pub fn chave_do_marcador(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Both Ends" => "panel.vector.engine.marcador.both_ends",
        "Head Round" => "panel.vector.engine.marcador.head_round",
        "Head Size" => "panel.vector.engine.marcador.head_size",
        "Off" => "panel.vector.engine.marcador.off",
        "On" => "panel.vector.engine.marcador.on",
        _ => return None,
    })
}

/// A chave de uma PONTA de traço (`ph2d_vec_scene::Marker::label`).
pub fn chave_da_ponta(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "None" => "panel.vector.engine.ponta.none",
        "Arrow" => "panel.vector.engine.ponta.arrow",
        "Open" => "panel.vector.engine.ponta.open",
        "Diamond" => "panel.vector.engine.ponta.diamond",
        "Diamond (hollow)" => "panel.vector.engine.ponta.diamond_hollow",
        "Circle" => "panel.vector.engine.ponta.circle",
        "Circle (hollow)" => "panel.vector.engine.ponta.circle_hollow",
        "Bar" => "panel.vector.engine.ponta.bar",
        _ => return None,
    })
}
