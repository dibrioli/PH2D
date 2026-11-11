//! Os campos que o preset leva ([`super::super::CAMPOS_DO_PINCEL`]): uma linha por campo, com a ida, a volta e,
//! quando o campo só se grava ligado, a condição ao lado (o preset sem ele é o de antes, ao byte).

use super::curvas::{
    curva_de_json, curva_para_json, pegada_de_json, pegada_para_json, taper_para_json,
};
use super::valores::{
    booleano, cor_mao_de_json, cor_mao_para_json, de_nome, f, inteiro, mangueira_de_json,
    mangueira_para_json, marca_de_json, marca_para_json, nome, num, textura_de_json,
    textura_para_json,
};
use super::{Campo, c, ligado, so};
use ph2d_painter_brush::{
    BrushBlend, ColorJitterScope, Falloff, ImageHose, JitterUnit, MAX_BRUSH_BLEND_MODES, PixelTip,
    TextureSettings,
};
use serde_json::{Value, json};

pub(super) static CAMPOS: &[Campo] = &[
    c(
        "hardness",
        |b| num(b.hardness),
        |b, v| b.hardness = f(v, b.hardness),
    ),
    c(
        "falloff",
        |b| nome(b.falloff),
        |b, v| b.falloff = de_nome(v, Falloff::ALL, b.falloff),
    ),
    c(
        "custom_falloff",
        |b| curva_para_json(&b.custom_falloff),
        |b, v| b.custom_falloff = curva_de_json(v, b.custom_falloff),
    ),
    c(
        "spacing",
        |b| num(b.spacing),
        |b, v| b.spacing = f(v, b.spacing),
    ),
    c(
        "jitter",
        |b| num(b.jitter),
        |b, v| b.jitter = f(v, b.jitter),
    ),
    c(
        "jitter_unit",
        |b| nome(b.jitter_unit),
        |b, v| b.jitter_unit = de_nome(v, [JitterUnit::Brush, JitterUnit::View], b.jitter_unit),
    ),
    c(
        "jitter_absolute_px",
        |b| num(b.jitter_absolute_px),
        |b, v| b.jitter_absolute_px = f(v, b.jitter_absolute_px),
    ),
    c(
        "jitter_scale",
        |b| num(b.jitter_scale),
        |b, v| b.jitter_scale = f(v, b.jitter_scale),
    ),
    c(
        "jitter_rotate",
        |b| num(b.jitter_rotate),
        |b, v| b.jitter_rotate = f(v, b.jitter_rotate),
    ),
    c(
        "jitter_spacing",
        |b| num(b.jitter_spacing),
        |b, v| b.jitter_spacing = f(v, b.jitter_spacing),
    ),
    c(
        "spray_count",
        |b| json!(b.spray_count),
        |b, v| b.spray_count = inteiro(v, b.spray_count),
    ),
    c(
        "dab_jitter",
        |b| marca_para_json(&b.dab_jitter),
        |b, v| b.dab_jitter = marca_de_json(v, b.dab_jitter),
    ),
    // A ponta de píxel (L18) só ligada; AUSENTE = desligada, nunca a do pincel anterior.
    so(
        "pixel",
        |b| nome(b.pixel),
        |b, v| {
            b.pixel = de_nome(
                v,
                (0..PixelTip::COUNT).map(PixelTip::from_u8),
                PixelTip::Off,
            )
        },
        |b| b.pixel != PixelTip::Off,
    ),
    // A ponta de N imagens (L19) só quando não é a de sempre; AUSENTE = a imagem única.
    so(
        "hose",
        |b| mangueira_para_json(&b.hose),
        |b, v| b.hose = mangueira_de_json(v),
        |b| b.hose != ImageHose::NEUTRA,
    ),
    c(
        "cor_mao",
        |b| cor_mao_para_json(&b.cor_mao),
        |b, v| b.cor_mao = cor_mao_de_json(v, b.cor_mao),
    ),
    c(
        "input_samples",
        |b| json!(b.input_samples),
        |b, v| b.input_samples = inteiro(v, b.input_samples),
    ),
    c(
        "stabilizer",
        |b| num(b.stabilizer),
        |b, v| b.stabilizer = f(v, b.stabilizer),
    ),
    // O tecto de tamanho (J3) só fixado; AUSENTE = sem tecto, nunca o do pincel anterior.
    so(
        "size_max_px",
        |b| b.size_max_px.map_or(Value::Null, num),
        |b, v| {
            b.size_max_px = v
                .and_then(Value::as_f64)
                .filter(|x| x.is_finite() && *x >= 1.0)
                .map(|x| x as f32);
        },
        |b| b.size_max_px.is_some(),
    ),
    c(
        "taper",
        |b| taper_para_json(&b.taper),
        |b, v| {
            let o = v.and_then(Value::as_object);
            let g = |k: &str, base: f32| f(o.and_then(|o| o.get(k)), base);
            b.taper.start = g("start", b.taper.start);
            b.taper.tip_start = g("tip_start", b.taper.tip_start);
            b.taper.opacity = g("opacity", b.taper.opacity);
            // A cauda e o fall-off (L5): sub-chave AUSENTE = a de fábrica (o preset de antes não os tinha).
            let z = |k: &str| f(o.and_then(|o| o.get(k)), 0.0);
            b.taper.end = z("end");
            b.taper.tip_end = z("tip_end");
            b.taper.fall_off = z("fall_off");
            b.taper.fall_off_start = z("fall_off_start");
        },
    ),
    c(
        "texture",
        |b| textura_para_json(&b.texture),
        |b, v| b.texture = textura_de_json(v, b.texture),
    ),
    c(
        "grain_depth",
        |b| num(b.grain_depth),
        |b, v| b.grain_depth = f(v, b.grain_depth),
    ),
    // Ausente = desligado (a de fábrica): o preset de antes lê o grão de antes.
    so(
        "grain_same_as_paper",
        |b| json!(b.grain_same_as_paper),
        |b, v| b.grain_same_as_paper = booleano(v, false),
        |b| b.grain_same_as_paper,
    ),
    // A referência do Grain (Bug #61) só quando o Grain já não é ela: a ausência diz «o Grain do ficheiro é ela»
    // (`pincel_de_json`).
    so(
        "grao_aplicado",
        |b| {
            let mut v = textura_para_json(&b.grao_aplicado.grao);
            v["media"] = json!(b.grao_aplicado.media);
            v
        },
        |b, v| {
            if v.is_some() {
                b.grao_aplicado = ph2d_painter_brush::GraoAplicado {
                    grao: textura_de_json(v, TextureSettings::default()),
                    media: v
                        .and_then(|o| o.get("media"))
                        .and_then(Value::as_f64)
                        .and_then(media_do_ficheiro),
                };
            }
        },
        |b| b.grao_trocado(),
    ),
    c(
        "shape",
        |b| textura_para_json(&b.shape),
        |b, v| b.shape = textura_de_json(v, b.shape),
    ),
    c(
        "dab_flatten",
        |b| num(b.dab_flatten),
        |b, v| b.dab_flatten = f(v, b.dab_flatten),
    ),
    c(
        "dab_angle_deg",
        |b| json!(b.dab_angle_deg),
        |b, v| b.dab_angle_deg = inteiro(v, b.dab_angle_deg),
    ),
    c(
        "dab_quina",
        |b| num(b.dab_quina),
        |b, v| b.dab_quina = f(v, b.dab_quina).clamp(0.0, 1.0),
    ),
    // Ausente = sem inércia (a de fábrica): o preset de antes lê o rumo de antes.
    so(
        "dab_rotation_lag",
        |b| num(b.dab_rotation_lag),
        |b, v| {
            b.dab_rotation_lag =
                f(v, 0.0).clamp(0.0, ph2d_painter_brush::heading::ROTATION_LAG_MAX);
        },
        |b| ligado(b.dab_rotation_lag),
    ),
    // As pontas rectas (J6): ausentes = desligadas (a de fábrica), como a lei espectral.
    so(
        "pontas_rectas",
        |b| json!(b.pontas_rectas),
        |b, v| b.pontas_rectas = booleano(v, false),
        |b| b.pontas_rectas,
    ),
    so(
        "pontas_alem",
        |b| num(b.pontas_alem),
        |b, v| b.pontas_alem = f(v, 0.0).clamp(0.0, ph2d_painter_brush::PONTAS_ALEM_MAX),
        |b| ligado(b.pontas_alem),
    ),
    // O esbate negativo (as pontas que CARREGAM, R2-A) viaja: só o `0` sai.
    so(
        "pontas_esbate",
        |b| num(b.pontas_esbate),
        |b, v| {
            b.pontas_esbate = f(v, 0.0).clamp(
                crate::ids::PAINTER_BRUSH_PONTAS_ESBATE_MIN,
                ph2d_painter_brush::PONTAS_ESBATE_MAX,
            );
        },
        |b| b.pontas_esbate != 0.0,
    ),
    c(
        "dab_curve",
        |b| pegada_para_json(b.dab_curve),
        |b, v| b.dab_curve = pegada_de_json(v, b.dab_curve),
    ),
    c(
        "color_jitter_enabled",
        |b| json!(b.color_jitter_enabled),
        |b, v| b.color_jitter_enabled = booleano(v, b.color_jitter_enabled),
    ),
    c(
        "continuous_spacing",
        |b| json!(b.continuous_spacing),
        |b, v| b.continuous_spacing = booleano(v, b.continuous_spacing),
    ),
    c(
        "color_jitter_hue",
        |b| num(b.color_jitter_hue),
        |b, v| b.color_jitter_hue = f(v, b.color_jitter_hue),
    ),
    c(
        "color_jitter_sat",
        |b| num(b.color_jitter_sat),
        |b, v| b.color_jitter_sat = f(v, b.color_jitter_sat),
    ),
    c(
        "color_jitter_val",
        |b| num(b.color_jitter_val),
        |b, v| b.color_jitter_val = f(v, b.color_jitter_val),
    ),
    c(
        "color_jitter_scope",
        |b| nome(b.color_jitter_scope),
        |b, v| {
            b.color_jitter_scope = de_nome(
                v,
                [ColorJitterScope::PerDab, ColorJitterScope::PerStroke],
                b.color_jitter_scope,
            );
        },
    ),
    c(
        "blend",
        |b| nome(b.blend),
        |b, v| {
            b.blend = de_nome(
                v,
                (0..MAX_BRUSH_BLEND_MODES).map(BrushBlend::from_u8),
                b.blend,
            );
        },
    ),
    c(
        "blend_sobre_vazio",
        |b| json!(b.blend_sobre_vazio),
        |b, v| b.blend_sobre_vazio = booleano(v, b.blend_sobre_vazio),
    ),
    c(
        "pigment",
        |b| json!(b.pigment),
        |b, v| b.pigment = booleano(v, b.pigment),
    ),
    c(
        "pigment_mix",
        |b| num(b.pigment_mix),
        |b, v| b.pigment_mix = f(v, b.pigment_mix),
    ),
    c(
        "paper_depth",
        |b| num(b.paper_depth),
        |b, v| b.paper_depth = f(v, b.paper_depth),
    ),
];

/// A média do Grain aplicado que o ficheiro traz: abaixo do piso de `media_do_grao` (ou não finita) o pincel não pintava
/// nada; vale a ausência, e diz-se (Bug #65 D4).
fn media_do_ficheiro(m: f64) -> Option<f32> {
    let media = m as f32;
    if media.is_finite() && media >= 1e-3 {
        return Some(media);
    }
    eprintln!("[painter-preset] grao_aplicado.media inválida ({m}): ignorada");
    None
}
