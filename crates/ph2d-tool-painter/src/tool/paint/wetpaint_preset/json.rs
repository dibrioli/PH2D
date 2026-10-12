//! Cada campo de [`super::CAMPOS_DO_PINCEL`] para JSON e de volta, pelo NOME do campo.
//!
//! Leis: `f32` escreve-se pelo menor texto que volta ao mesmo `f32` (`0.1`, nunca
//! `0.10000000149011612`); enums pelo NOME da variante (reordenar o enum não estraga um preset);
//! chave ausente ou ilegível = o valor de `base` (o preset diz o que fixa, o resto é o de fábrica).

use ph2d_painter_brush::{
    BrushBlend, BrushSpec, ColorJitterScope, Falloff, FalloffCurve, FalloffPoint, FootprintCurve,
    HandleType, JitterUnit, MAX_BRUSH_BLEND_MODES, MAX_TEX_PARAMS, TextureKind, TextureMapping,
    TextureSettings,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fmt::Debug;

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
    json!({
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
    })
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

/// O valor JSON de UM campo do pincel. Um nome fora de [`super::CAMPOS_DO_PINCEL`] é erro de quem chama.
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
        "input_samples" => json!(b.input_samples),
        "stabilizer" => num(b.stabilizer),
        "taper" => json!({
            "start": num(b.taper.start),
            "tip_start": num(b.taper.tip_start),
            "opacity": num(b.taper.opacity),
        }),
        "texture" => textura_para_json(&b.texture),
        "grain_depth" => num(b.grain_depth),
        "shape" => textura_para_json(&b.shape),
        "dab_flatten" => num(b.dab_flatten),
        "dab_angle_deg" => json!(b.dab_angle_deg),
        "dab_curve" => pegada_para_json(b.dab_curve),
        "color_jitter_enabled" => json!(b.color_jitter_enabled),
        "color_jitter_hue" => num(b.color_jitter_hue),
        "color_jitter_sat" => num(b.color_jitter_sat),
        "color_jitter_val" => num(b.color_jitter_val),
        "color_jitter_scope" => nome(b.color_jitter_scope),
        "blend" => nome(b.blend),
        "pigment" => json!(b.pigment),
        "pigment_mix" => num(b.pigment_mix),
        "paper_depth" => num(b.paper_depth),
        outro => unreachable!("campo do pincel sem conversão: {outro}"),
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
        "input_samples" => b.input_samples = inteiro(v, b.input_samples),
        "stabilizer" => b.stabilizer = f(v, b.stabilizer),
        "taper" => {
            let o = v.and_then(Value::as_object);
            let g = |k: &str, base: f32| f(o.and_then(|o| o.get(k)), base);
            b.taper.start = g("start", b.taper.start);
            b.taper.tip_start = g("tip_start", b.taper.tip_start);
            b.taper.opacity = g("opacity", b.taper.opacity);
        }
        "texture" => b.texture = textura_de_json(v, b.texture),
        "grain_depth" => b.grain_depth = f(v, b.grain_depth),
        "shape" => b.shape = textura_de_json(v, b.shape),
        "dab_flatten" => b.dab_flatten = f(v, b.dab_flatten),
        "dab_angle_deg" => b.dab_angle_deg = inteiro(v, b.dab_angle_deg),
        "dab_curve" => b.dab_curve = pegada_de_json(v, b.dab_curve),
        "color_jitter_enabled" => b.color_jitter_enabled = booleano(v, b.color_jitter_enabled),
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
        "pigment" => b.pigment = booleano(v, b.pigment),
        "pigment_mix" => b.pigment_mix = f(v, b.pigment_mix),
        "paper_depth" => b.paper_depth = f(v, b.paper_depth),
        outro => unreachable!("campo do pincel sem conversão: {outro}"),
    }
}

/// O sub-conjunto do pincel que o preset leva, por nome.
pub(super) fn pincel_para_json(b: &BrushSpec, campos: &[&str]) -> BTreeMap<String, Value> {
    campos
        .iter()
        .map(|&c| ((*c).to_owned(), ler_campo(b, c)))
        .collect()
}

/// `base` com os campos que o JSON fixa. Chaves fora de `campos` (de ondas futuras) são ignoradas
/// aqui e sobrevivem no [`ph2d_brush_presets::Parametros`].
pub(super) fn pincel_de_json(
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
