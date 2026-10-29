//! As listas dos campos do `BrushSpec`: os que o preset leva e os que ficam fora (cada um com o porquê) — filho
//! de `wetpaint_preset` por LOC.

/// Os campos do `BrushSpec` que o preset leva, pelo nome do campo.
pub const CAMPOS_DO_PINCEL: [&str; 41] = [
    // O tecto de tamanho (J3, decisão 8): os quatro meios; gravado só quando o preset o fixa.
    "size_max_px",
    "hardness",
    "falloff",
    "custom_falloff",
    "spacing",
    "continuous_spacing",
    "jitter",
    "jitter_unit",
    "jitter_absolute_px",
    "jitter_scale",
    "jitter_rotate",
    "jitter_spacing",
    "spray_count",
    "dab_jitter",
    "hose",
    // O dual brush (L6): o Digital e o Wet o leem (gravado só armado); nos outros meios a linha esmaece.
    "dual",
    "input_samples",
    "stabilizer",
    "taper",
    "texture",
    "grain_depth",
    // O Grain «Same as Paper» fora da Aquarela (28/10): gravado só ligado.
    "grain_same_as_paper",
    "shape",
    "dab_flatten",
    "dab_angle_deg",
    "dab_quina",
    // As pontas rectas do traço (J6): gravadas só ligadas.
    "pontas_rectas",
    "pontas_alem",
    // As pontas que esbatem (J6t): gravadas só ligadas.
    "pontas_esbate",
    "dab_curve",
    "color_jitter_enabled",
    "color_jitter_hue",
    "color_jitter_sat",
    "color_jitter_val",
    "color_jitter_scope",
    "cor_mao",
    "blend",
    "blend_sobre_vazio",
    "pigment",
    "pigment_mix",
    // A lei espectral do pigmento (J4 efeitos): gravada só ligada.
    "pigment_espectral",
];

/// **O pincel INTEIRO, pelo nome de cada campo** — o que o estado inicial de uma MACRO leva de cada
/// pincel (`crate::macro_track::inicial`; W6c: os quatro meios, e cada um lê campos que o preset do Wet
/// Paint deixa de fora). A união das duas listas do censo, logo nenhum campo do `BrushSpec` falta.
#[must_use]
pub fn campos_da_macro() -> Vec<&'static str> {
    CAMPOS_DO_PINCEL
        .iter()
        .copied()
        // A pegada deitada de UM dab não é do pincel: a macro não a leva.
        .chain(
            CAMPOS_FORA
                .iter()
                .filter(|(_, porque)| *porque != POR_DAB)
                .map(|(c, _)| *c),
        )
        .collect()
}

pub(super) const VOLUME: &str = "volume do topo (Size px = 2·raio, Opacity = força)";
pub(super) const DA_OBRA: &str =
    "da obra, não do pincel (a cor e o papel são do documento; a simetria é da tela)";
const DO_GESTO: &str = "do gesto (o método do traço e o cartão Line), não do pincel";
pub(super) const DO_MEIO: &str = "a porta do meio (`set_paint_media`) é a dona";
pub(super) const DO_SCULPT: &str = "canal derivado do Sculpt (`sculpt_panel.rs`)";
pub(super) const POR_DAB: &str =
    "a pegada deitada de UM dab (`BrushSpec::at_dab`): `None` no pincel do artista";
pub(super) const AQUARELA: &str = "lido só pela Watercolor";
pub(super) const IMPASTO: &str = "lido só pelo Impasto";
pub(super) const DIGITAL: &str =
    "lido só pelo Digital (a ponta de píxel, L18; a orla em píxeis e o sub-passo fino, J3)";
pub(super) const DOS_KNOBS: &str = "a caneta deitada: no Wet Paint são os knobs do pincel (`dabRotation`…`tiltLag`, W6), no mapa `engine`";
const NAO_LIDO: &str = "o Wet Paint não o lê (deposita pela lei do fluido)";

/// Os campos do `BrushSpec` que o preset NÃO leva, e porquê.
pub const CAMPOS_FORA: [(&str, &str); 80] = [
    ("radius_px", VOLUME),
    ("strength", VOLUME),
    ("color", DA_OBRA),
    ("paper", DA_OBRA),
    // O dente do papel é da TELA (dono 27/10, W3b): o pincel não o grava, e o de um preset antigo é ignorado.
    ("paper_depth", DA_OBRA),
    ("symmetry", DA_OBRA),
    ("stroke_method", DO_GESTO),
    ("grid_cell_px", DO_GESTO),
    ("grid_offset_px", DO_GESTO),
    ("grid_fit", DO_GESTO),
    ("edge_to_edge", DO_GESTO),
    ("line_kind", DO_GESTO),
    ("style_solid", DO_GESTO),
    ("sketchy_reach", DO_GESTO),
    ("sketchy_density", DO_GESTO),
    ("thread_width_px", DO_GESTO),
    ("thread_opacity", DO_GESTO),
    ("sketchy_magnetify", DO_GESTO),
    ("wire_history", DO_GESTO),
    ("wire_connection_line", DO_GESTO),
    ("wire_sorteio", DO_GESTO),
    ("wire_cordas", DO_GESTO),
    ("ribbon_weight", DO_GESTO),
    ("ribbon_friction", DO_GESTO),
    ("ribbon_gravity", DO_GESTO),
    ("ribbon_rungs", DO_GESTO),
    ("rough_amount", DO_GESTO),
    ("rough_bowing", DO_GESTO),
    ("rough_passes", DO_GESTO),
    ("dash_ratio", DO_GESTO),
    ("dash_samples", DO_GESTO),
    ("needs_heading", DO_SCULPT),
    ("dab_tilt", POR_DAB),
    ("pixel", DIGITAL),
    ("pen_tilt", DOS_KNOBS),
    ("watercolor", DO_MEIO),
    ("impasto", DO_MEIO),
    ("fora_do_digital", DO_MEIO),
    ("edge_gain", AQUARELA),
    ("edge_spread", AQUARELA),
    ("smooth_edges", AQUARELA),
    ("granulation", AQUARELA),
    ("fill", AQUARELA),
    ("depth", AQUARELA),
    ("opacity", AQUARELA),
    ("warp", AQUARELA),
    ("edge_flow", AQUARELA),
    ("paper_edge", AQUARELA),
    ("wet_smudge", AQUARELA),
    ("wet_rewet", AQUARELA),
    ("wet_charge", AQUARELA),
    ("wet_dilution", AQUARELA),
    ("wet_pull", AQUARELA),
    ("granulation_use_paper", AQUARELA),
    ("watercolor_shape_auto", AQUARELA),
    ("impasto_smooth_edges", IMPASTO),
    ("impasto_depth", IMPASTO),
    ("film_depth", IMPASTO),
    ("impasto_source", IMPASTO),
    ("impasto_draw_to", IMPASTO),
    ("impasto_smoothing", IMPASTO),
    ("impasto_body", IMPASTO),
    ("impasto_plow", IMPASTO),
    ("impasto_push", IMPASTO),
    ("impasto_shine", IMPASTO),
    ("impasto_roughness", IMPASTO),
    ("impasto_metallic", IMPASTO),
    ("impasto_wax", IMPASTO),
    ("impasto_wax_color", IMPASTO),
    ("flow", NAO_LIDO),
    ("accumulate", NAO_LIDO),
    ("airbrush_rate_s", NAO_LIDO),
    ("space_attenuation", NAO_LIDO),
    ("tecto_por_dab", DIGITAL),
    ("wash", DIGITAL),
    ("smear_fio", DIGITAL),
    ("borda_px", DIGITAL),
    ("sub_passo_fino", DIGITAL),
    ("passo_pela_pressao", DIGITAL),
    ("parado_s", DIGITAL),
];
