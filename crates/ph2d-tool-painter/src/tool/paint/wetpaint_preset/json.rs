//! Cada campo do `BrushSpec` para JSON e de volta, pelo NOME do campo: os do preset
//! ([`super::CAMPOS_DO_PINCEL`]) aqui, e os que só a macro leva aqui e em [`meio`].
//!
//! Leis: `f32` escreve-se pelo menor texto que volta ao mesmo `f32` (`0.1`, nunca
//! `0.10000000149011612`); enums pelo NOME da variante (reordenar o enum não estraga um preset);
//! chave ausente ou ilegível = o valor de `base` (o preset diz o que fixa, o resto é o de fábrica).

use ph2d_painter_brush::line_kind::LineKind;
use ph2d_painter_brush::{
    BrushBlend, BrushSpec, ColorJitterScope, CorDaMao, DabJitter, GrainLaw, GrainMode, HOSE_MAX, HoseMode, ImageHose, PixelTip, Falloff, FalloffCurve, FalloffPoint, FootprintCurve,
    HandleType, JitterUnit, MAX_BRUSH_BLEND_MODES, MAX_TEX_PARAMS, MirrorAxis, StrokeMethod,
    TextureKind, TextureMapping, TextureSettings,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fmt::Debug;

mod meio;

fn num(v: f32) -> Value {
    format!("{v}")
        .parse::<f64>()
        .ok()
        .and_then(serde_json::Number::from_f64)
        .map_or(Value::Null, Value::Number)
}

fn f(v: Option<&Value>, base: f32) -> f32 {
    v.and_then(Value::as_f64).map_or(base, |x| x as f32)
}

fn par(v: [f32; 2]) -> Value {
    json!([num(v[0]), num(v[1])])
}

fn f2(v: Option<&Value>, base: [f32; 2]) -> [f32; 2] {
    let a = v.and_then(Value::as_array);
    [
        f(a.and_then(|a| a.first()), base[0]),
        f(a.and_then(|a| a.get(1)), base[1]),
    ]
}

fn inteiro<T: TryFrom<u64> + Copy>(v: Option<&Value>, base: T) -> T {
    v.and_then(Value::as_u64)
        .and_then(|x| T::try_from(x).ok())
        .unwrap_or(base)
}

fn booleano(v: Option<&Value>, base: bool) -> bool {
    v.and_then(Value::as_bool).unwrap_or(base)
}

fn nome<T: Debug>(v: T) -> Value {
    Value::String(format!("{v:?}"))
}

/// A variante cujo NOME é o texto; ausente ou desconhecido ⇒ `base`.
fn de_nome<T: Debug + Copy>(v: Option<&Value>, todos: impl IntoIterator<Item = T>, base: T) -> T {
    let Some(s) = v.and_then(Value::as_str) else {
        return base;
    };
    todos
        .into_iter()
        .find(|t| format!("{t:?}") == s)
        .unwrap_or(base)
}

fn textura_para_json(t: &TextureSettings) -> Value {
    let mut v = json!({
        "kind": nome(t.kind),
        "mapping": nome(t.mapping),
        "angle_deg": t.angle_deg,
        "rake": t.rake,
        "flow": t.flow,
        "offset": par(t.offset),
        "size": par(t.size),
        "stencil_offset": par(t.stencil_offset),
        "stencil_size": par(t.stencil_size),
        "stencil_angle_deg": t.stencil_angle_deg,
        "params": t.params.iter().map(|&p| num(p)).collect::<Vec<_>>(),
    });
    // A lei do grão (L9) só quando não é a de sempre: o preset sem ela é o de antes, ao byte.
    if t.lei != GrainLaw::NEUTRA
        && let Some(o) = v.as_object_mut()
    {
        o.insert(
            "lei".to_owned(),
            json!({
                "mode": t.lei.mode.to_u8(),
                "depth_min": num(t.lei.depth_min),
                "depth_jitter": num(t.lei.depth_jitter),
                "depth_pressure": t.lei.depth_pressure,
            }),
        );
    }
    v
}

/// A ponta de N imagens (L19) do JSON; AUSENTE = a imagem única.
fn mangueira_de_json(v: Option<&Value>) -> ImageHose {
    let Some(o) = v.and_then(Value::as_object) else {
        return ImageHose::NEUTRA;
    };
    let mut h = ImageHose::NEUTRA;
    h.n = inteiro(o.get("n"), 1u8).clamp(1, HOSE_MAX as u8);
    h.mode = HoseMode::from_u8(inteiro(o.get("mode"), 0u8));
    if let Some(a) = o.get("pesos").and_then(Value::as_array) {
        for (k, p) in h.pesos.iter_mut().enumerate() {
            *p = inteiro(a.get(k), *p);
        }
    }
    h
}

/// A lei do grão (L9) do JSON; AUSENTE = a de sempre (nunca a do pincel anterior: o preset sem ela é o de antes).
fn lei_de_json(v: Option<&Value>) -> GrainLaw {
    let Some(o) = v.and_then(Value::as_object) else {
        return GrainLaw::NEUTRA;
    };
    let n = GrainLaw::NEUTRA;
    GrainLaw {
        mode: GrainMode::from_u8(inteiro(o.get("mode"), n.mode.to_u8())),
        depth_min: f(o.get("depth_min"), n.depth_min),
        depth_jitter: f(o.get("depth_jitter"), n.depth_jitter),
        depth_pressure: booleano(o.get("depth_pressure"), n.depth_pressure),
    }
}

fn textura_de_json(v: Option<&Value>, base: TextureSettings) -> TextureSettings {
    let Some(o) = v.and_then(Value::as_object) else {
        return base;
    };
    let mut params = base.params;
    if let Some(a) = o.get("params").and_then(Value::as_array) {
        for (i, p) in params.iter_mut().enumerate().take(MAX_TEX_PARAMS) {
            *p = f(a.get(i), *p);
        }
    }
    TextureSettings {
        kind: de_nome(
            o.get("kind"),
            (0..TextureKind::COUNT).map(TextureKind::from_u8),
            base.kind,
        ),
        mapping: de_nome(
            o.get("mapping"),
            (0..TextureMapping::COUNT).map(TextureMapping::from_u8),
            base.mapping,
        ),
        angle_deg: inteiro(o.get("angle_deg"), base.angle_deg),
        rake: booleano(o.get("rake"), base.rake),
        flow: booleano(o.get("flow"), base.flow),
        offset: f2(o.get("offset"), base.offset),
        size: f2(o.get("size"), base.size),
        stencil_offset: f2(o.get("stencil_offset"), base.stencil_offset),
        stencil_size: f2(o.get("stencil_size"), base.stencil_size),
        stencil_angle_deg: inteiro(o.get("stencil_angle_deg"), base.stencil_angle_deg),
        params,
        lei: lei_de_json(o.get("lei")),
    }
}

fn marca_para_json(j: &DabJitter) -> Value {
    json!({
        "opacity": num(j.opacity),
        "count": num(j.count),
        "flip_x": num(j.flip_x),
        "flip_y": num(j.flip_y),
        "roundness": num(j.roundness),
        "angle_per_stroke": num(j.angle_per_stroke),
        "across": num(j.across),
        "particle": num(j.particle),
    })
}

/// O jitter da marca (L4) do JSON; uma chave ausente (um preset de antes) = a de `base`.
fn marca_de_json(v: Option<&Value>, base: DabJitter) -> DabJitter {
    let Some(o) = v.and_then(Value::as_object) else {
        return base;
    };
    DabJitter {
        opacity: f(o.get("opacity"), base.opacity),
        count: f(o.get("count"), base.count),
        flip_x: f(o.get("flip_x"), base.flip_x),
        flip_y: f(o.get("flip_y"), base.flip_y),
        roundness: f(o.get("roundness"), base.roundness),
        angle_per_stroke: f(o.get("angle_per_stroke"), base.angle_per_stroke),
        across: f(o.get("across"), base.across),
        particle: f(o.get("particle"), base.particle),
    }
}

/// A cor pela mão (L11) do JSON; uma chave ausente = a de `base`.
fn cor_mao_de_json(v: Option<&Value>, base: CorDaMao) -> CorDaMao {
    let Some(o) = v.and_then(Value::as_object) else {
        return base;
    };
    let sec = o.get("secondary").and_then(Value::as_array);
    CorDaMao {
        secondary: [0, 1, 2].map(|k| f(sec.and_then(|a| a.get(k)), base.secondary[k])),
        secondary_jitter: f(o.get("secondary_jitter"), base.secondary_jitter),
        pressure_secondary: f(o.get("pressure_secondary"), base.pressure_secondary),
        pressure_hue: f(o.get("pressure_hue"), base.pressure_hue),
        pressure_sat: f(o.get("pressure_sat"), base.pressure_sat),
        pressure_val: f(o.get("pressure_val"), base.pressure_val),
    }
}

fn curva_para_json(c: &FalloffCurve) -> Value {
    Value::Array(
        c.points()
            .iter()
            .map(|p| json!({"id": p.id, "x": num(p.x), "y": num(p.y), "handle": nome(p.handle)}))
            .collect(),
    )
}

fn curva_de_json(v: Option<&Value>, base: FalloffCurve) -> FalloffCurve {
    let Some(a) = v.and_then(Value::as_array) else {
        return base;
    };
    let pts: Vec<FalloffPoint> = a
        .iter()
        .filter_map(Value::as_object)
        .map(|o| FalloffPoint {
            id: inteiro(o.get("id"), 0u8),
            x: f(o.get("x"), 0.0),
            y: f(o.get("y"), 0.0),
            handle: de_nome(
                o.get("handle"),
                [HandleType::Auto, HandleType::Vector],
                HandleType::Auto,
            ),
        })
        .collect();
    FalloffCurve::from_points(&pts).unwrap_or(base)
}

fn pegada_para_json(c: FootprintCurve) -> Value {
    Value::Array(
        c.rows()
            .iter()
            .map(|r| Value::Array(r.iter().map(|&x| num(x)).collect()))
            .collect(),
    )
}

fn pegada_de_json(v: Option<&Value>, base: FootprintCurve) -> FootprintCurve {
    let Some(a) = v.and_then(Value::as_array) else {
        return base;
    };
    let mut rows = base.rows();
    for (r, linha) in rows.iter_mut().zip(a) {
        let l = linha.as_array();
        for (i, x) in r.iter_mut().enumerate() {
            *x = f(l.and_then(|l| l.get(i)), *x);
        }
    }
    FootprintCurve::from_rows(rows)
}

/// O taper: as três chaves de sempre, e a cauda e o fall-off (L5) só quando armados — o preset sem eles é o de
/// antes, ao byte.
fn taper_para_json(t: &ph2d_painter_brush::taper::Taper) -> Value {
    let mut v = json!({
        "start": num(t.start),
        "tip_start": num(t.tip_start),
        "opacity": num(t.opacity),
    });
    if let Some(o) = v.as_object_mut() {
        for (k, x) in [
            ("end", t.end),
            ("tip_end", t.tip_end),
            ("fall_off", t.fall_off),
            ("fall_off_start", t.fall_off_start),
        ] {
            if x != 0.0 {
                o.insert(k.to_owned(), num(x));
            }
        }
    }
    v
}

/// O valor JSON de UM campo do pincel. Um nome que não é campo do `BrushSpec` é erro de quem chama.
fn ler_campo(b: &BrushSpec, campo: &str) -> Value {
    match campo {
        "hardness" => num(b.hardness),
        "falloff" => nome(b.falloff),
        "custom_falloff" => curva_para_json(&b.custom_falloff),
        "spacing" => num(b.spacing),
        "jitter" => num(b.jitter),
        "jitter_unit" => nome(b.jitter_unit),
        "jitter_absolute_px" => num(b.jitter_absolute_px),
        "jitter_scale" => num(b.jitter_scale),
        "jitter_rotate" => num(b.jitter_rotate),
        "jitter_spacing" => num(b.jitter_spacing),
        "spray_count" => json!(b.spray_count),
        "dab_jitter" => marca_para_json(&b.dab_jitter),
        "pixel" => json!(b.pixel.to_u8()),
        "hose" => json!({
            "n": b.hose.n,
            "mode": b.hose.mode.to_u8(),
            "pesos": b.hose.pesos.to_vec(),
        }),
        "cor_mao" => json!({
            "secondary": [num(b.cor_mao.secondary[0]), num(b.cor_mao.secondary[1]), num(b.cor_mao.secondary[2])],
            "secondary_jitter": num(b.cor_mao.secondary_jitter),
            "pressure_secondary": num(b.cor_mao.pressure_secondary),
            "pressure_hue": num(b.cor_mao.pressure_hue),
            "pressure_sat": num(b.cor_mao.pressure_sat),
            "pressure_val": num(b.cor_mao.pressure_val),
        }),
        "input_samples" => json!(b.input_samples),
        "stabilizer" => num(b.stabilizer),
        "taper" => taper_para_json(&b.taper),
        "texture" => textura_para_json(&b.texture),
        "grain_depth" => num(b.grain_depth),
        "shape" => textura_para_json(&b.shape),
        "dab_flatten" => num(b.dab_flatten),
        "dab_angle_deg" => json!(b.dab_angle_deg),
        "dab_quina" => num(b.dab_quina),
        "dab_curve" => pegada_para_json(b.dab_curve),
        "color_jitter_enabled" => json!(b.color_jitter_enabled),
        "continuous_spacing" => json!(b.continuous_spacing),
        "color_jitter_hue" => num(b.color_jitter_hue),
        "color_jitter_sat" => num(b.color_jitter_sat),
        "color_jitter_val" => num(b.color_jitter_val),
        "color_jitter_scope" => nome(b.color_jitter_scope),
        "blend" => nome(b.blend),
        "blend_sobre_vazio" => json!(b.blend_sobre_vazio),
        "pigment" => json!(b.pigment),
        "pigment_mix" => num(b.pigment_mix),
        "paper_depth" => num(b.paper_depth),
        outro => ler_campo_da_obra(b, outro),
    }
}

/// Os campos da obra e do gesto, que o preset deixa de fora e a MACRO leva ([`super::campos_da_macro`]).
fn ler_campo_da_obra(b: &BrushSpec, campo: &str) -> Value {
    match campo {
        "radius_px" => num(b.radius_px),
        "strength" => num(b.strength),
        "color" => json!([num(b.color[0]), num(b.color[1]), num(b.color[2])]),
        "paper" => textura_para_json(&b.paper),
        "symmetry" => json!({
            "enabled": b.symmetry.enabled,
            "circular": b.symmetry.circular,
            "axis": nome(b.symmetry.axis),
            "radial_segments": b.symmetry.radial_segments,
            "center": par(b.symmetry.center),
            "custom_dir": par(b.symmetry.custom_dir),
        }),
        "stroke_method" => nome(b.stroke_method),
        "grid_cell_px" => par(b.grid_cell_px),
        "grid_offset_px" => par(b.grid_offset_px),
        "grid_fit" => num(b.grid_fit),
        "edge_to_edge" => json!(b.edge_to_edge),
        "line_kind" => nome(b.line_kind),
        "style_solid" => json!(b.style_solid),
        "sketchy_reach" => num(b.sketchy_reach),
        "sketchy_density" => num(b.sketchy_density),
        "thread_width_px" => num(b.thread_width_px),
        "thread_opacity" => num(b.thread_opacity),
        "sketchy_magnetify" => json!(b.sketchy_magnetify),
        "wire_history" => num(b.wire_history),
        "wire_connection_line" => json!(b.wire_connection_line),
        "ribbon_weight" => num(b.ribbon_weight),
        "ribbon_friction" => num(b.ribbon_friction),
        "ribbon_gravity" => num(b.ribbon_gravity),
        "ribbon_rungs" => num(b.ribbon_rungs),
        "rough_amount" => num(b.rough_amount),
        "rough_bowing" => num(b.rough_bowing),
        "rough_passes" => json!(b.rough_passes),
        "dash_ratio" => num(b.dash_ratio),
        "dash_samples" => json!(b.dash_samples),
        outro => meio::ler(b, outro),
    }
}

fn escrever_campo_da_obra(b: &mut BrushSpec, campo: &str, v: Option<&Value>) {
    match campo {
        "radius_px" => b.radius_px = f(v, b.radius_px),
        "strength" => b.strength = f(v, b.strength),
        "color" => {
            let a = v.and_then(Value::as_array);
            for (i, c) in b.color.iter_mut().enumerate() {
                *c = f(a.and_then(|a| a.get(i)), *c);
            }
        }
        "paper" => b.paper = textura_de_json(v, b.paper),
        "symmetry" => {
            let o = v.and_then(Value::as_object);
            let g = |k: &str| o.and_then(|o| o.get(k));
            let s = &mut b.symmetry;
            s.enabled = booleano(g("enabled"), s.enabled);
            s.circular = booleano(g("circular"), s.circular);
            s.axis = de_nome(g("axis"), (0..3).map(MirrorAxis::from_u8), s.axis);
            s.radial_segments = inteiro(g("radial_segments"), s.radial_segments);
            s.center = f2(g("center"), s.center);
            s.custom_dir = f2(g("custom_dir"), s.custom_dir);
        }
        "stroke_method" => {
            b.stroke_method = de_nome(
                v,
                (0..StrokeMethod::COUNT).map(StrokeMethod::from_u8),
                b.stroke_method,
            );
        }
        "grid_cell_px" => b.grid_cell_px = f2(v, b.grid_cell_px),
        "grid_offset_px" => b.grid_offset_px = f2(v, b.grid_offset_px),
        "grid_fit" => b.grid_fit = f(v, b.grid_fit),
        "edge_to_edge" => b.edge_to_edge = booleano(v, b.edge_to_edge),
        "line_kind" => b.line_kind = de_nome(v, (0..6).map(LineKind::from_wire), b.line_kind),
        "style_solid" => b.style_solid = booleano(v, b.style_solid),
        "sketchy_reach" => b.sketchy_reach = f(v, b.sketchy_reach),
        "sketchy_density" => b.sketchy_density = f(v, b.sketchy_density),
        "thread_width_px" => b.thread_width_px = f(v, b.thread_width_px),
        "thread_opacity" => b.thread_opacity = f(v, b.thread_opacity),
        "sketchy_magnetify" => b.sketchy_magnetify = booleano(v, b.sketchy_magnetify),
        "wire_history" => b.wire_history = f(v, b.wire_history),
        "wire_connection_line" => b.wire_connection_line = booleano(v, b.wire_connection_line),
        "ribbon_weight" => b.ribbon_weight = f(v, b.ribbon_weight),
        "ribbon_friction" => b.ribbon_friction = f(v, b.ribbon_friction),
        "ribbon_gravity" => b.ribbon_gravity = f(v, b.ribbon_gravity),
        "ribbon_rungs" => b.ribbon_rungs = f(v, b.ribbon_rungs),
        "rough_amount" => b.rough_amount = f(v, b.rough_amount),
        "rough_bowing" => b.rough_bowing = f(v, b.rough_bowing),
        "rough_passes" => b.rough_passes = inteiro(v, b.rough_passes),
        "dash_ratio" => b.dash_ratio = f(v, b.dash_ratio),
        "dash_samples" => b.dash_samples = inteiro(v, b.dash_samples),
        outro => meio::escrever(b, outro, v),
    }
}

/// Escreve UM campo a partir do JSON; ilegível ⇒ fica o de `b`.
fn escrever_campo(b: &mut BrushSpec, campo: &str, v: Option<&Value>) {
    match campo {
        "hardness" => b.hardness = f(v, b.hardness),
        "falloff" => b.falloff = de_nome(v, Falloff::ALL, b.falloff),
        "custom_falloff" => b.custom_falloff = curva_de_json(v, b.custom_falloff),
        "spacing" => b.spacing = f(v, b.spacing),
        "jitter" => b.jitter = f(v, b.jitter),
        "jitter_unit" => {
            b.jitter_unit = de_nome(v, [JitterUnit::Brush, JitterUnit::View], b.jitter_unit);
        }
        "jitter_absolute_px" => b.jitter_absolute_px = f(v, b.jitter_absolute_px),
        "jitter_scale" => b.jitter_scale = f(v, b.jitter_scale),
        "jitter_rotate" => b.jitter_rotate = f(v, b.jitter_rotate),
        "jitter_spacing" => b.jitter_spacing = f(v, b.jitter_spacing),
        "spray_count" => b.spray_count = inteiro(v, b.spray_count),
        "dab_jitter" => b.dab_jitter = marca_de_json(v, b.dab_jitter),
        // AUSENTE = desligada (só se grava ligada), nunca a do pincel anterior.
        "pixel" => b.pixel = PixelTip::from_u8(inteiro(v, 0)),
        // AUSENTE = a imagem única (só se grava com várias), nunca a ponta do pincel anterior.
        "hose" => b.hose = mangueira_de_json(v),
        "cor_mao" => b.cor_mao = cor_mao_de_json(v, b.cor_mao),
        "input_samples" => b.input_samples = inteiro(v, b.input_samples),
        "stabilizer" => b.stabilizer = f(v, b.stabilizer),
        "taper" => {
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
        }
        "texture" => b.texture = textura_de_json(v, b.texture),
        "grain_depth" => b.grain_depth = f(v, b.grain_depth),
        "shape" => b.shape = textura_de_json(v, b.shape),
        "dab_flatten" => b.dab_flatten = f(v, b.dab_flatten),
        "dab_angle_deg" => b.dab_angle_deg = inteiro(v, b.dab_angle_deg),
        "dab_quina" => b.dab_quina = f(v, b.dab_quina).clamp(0.0, 1.0),
        "dab_curve" => b.dab_curve = pegada_de_json(v, b.dab_curve),
        "color_jitter_enabled" => b.color_jitter_enabled = booleano(v, b.color_jitter_enabled),
        "continuous_spacing" => b.continuous_spacing = booleano(v, b.continuous_spacing),
        "color_jitter_hue" => b.color_jitter_hue = f(v, b.color_jitter_hue),
        "color_jitter_sat" => b.color_jitter_sat = f(v, b.color_jitter_sat),
        "color_jitter_val" => b.color_jitter_val = f(v, b.color_jitter_val),
        "color_jitter_scope" => {
            b.color_jitter_scope = de_nome(
                v,
                [ColorJitterScope::PerDab, ColorJitterScope::PerStroke],
                b.color_jitter_scope,
            );
        }
        "blend" => {
            b.blend = de_nome(
                v,
                (0..MAX_BRUSH_BLEND_MODES).map(BrushBlend::from_u8),
                b.blend,
            );
        }
        "blend_sobre_vazio" => b.blend_sobre_vazio = booleano(v, b.blend_sobre_vazio),
        "pigment" => b.pigment = booleano(v, b.pigment),
        "pigment_mix" => b.pigment_mix = f(v, b.pigment_mix),
        "paper_depth" => b.paper_depth = f(v, b.paper_depth),
        outro => escrever_campo_da_obra(b, outro, v),
    }
}

/// O sub-conjunto do pincel que o preset leva, por nome.
pub(crate) fn pincel_para_json(b: &BrushSpec, campos: &[&str]) -> BTreeMap<String, Value> {
    campos
        .iter()
        // A ponta de píxel (L18) só quando ligada: o preset sem ela é o de antes, ao byte.
        .filter(|&&c| !(c == "pixel" && b.pixel == PixelTip::Off))
        // A ponta de N imagens (L19) só quando não é a de sempre: o preset de imagem única é o de antes, ao byte.
        .filter(|&&c| !(c == "hose" && b.hose == ImageHose::NEUTRA))
        .map(|&c| ((*c).to_owned(), ler_campo(b, c)))
        .collect()
}

/// `base` com os campos que o JSON fixa. Chaves fora de `campos` (de ondas futuras) são ignoradas
/// aqui e sobrevivem no [`ph2d_brush_presets::Parametros`].
pub(crate) fn pincel_de_json(
    m: &BTreeMap<String, Value>,
    campos: &[&str],
    base: BrushSpec,
) -> BrushSpec {
    let mut b = base;
    for &c in campos {
        escrever_campo(&mut b, c, m.get(c));
    }
    b
}
