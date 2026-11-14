//! Os conversores de VALOR (número, par, enum pelo nome, textura, marca, cor pela mão) que as tabelas de
//! [`super`] usam. Leis do módulo: `f32` pelo menor texto que volta ao mesmo `f32`; enum pelo nome da variante;
//! chave ausente ou ilegível = o valor de `base`.

use ph2d_painter_brush::{
    CorDaMao, DabJitter, GrainLaw, GrainMode, HOSE_MAX, HoseMode, ImageHose, MAX_TEX_PARAMS,
    TextureKind, TextureMapping, TextureSettings,
};
use serde_json::{Value, json};
use std::fmt::Debug;

pub(super) fn num(v: f32) -> Value {
    format!("{v}")
        .parse::<f64>()
        .ok()
        .and_then(serde_json::Number::from_f64)
        .map_or(Value::Null, Value::Number)
}

pub(super) fn f(v: Option<&Value>, base: f32) -> f32 {
    v.and_then(Value::as_f64).map_or(base, |x| x as f32)
}

pub(super) fn par(v: [f32; 2]) -> Value {
    json!([num(v[0]), num(v[1])])
}

pub(super) fn f2(v: Option<&Value>, base: [f32; 2]) -> [f32; 2] {
    let a = v.and_then(Value::as_array);
    [
        f(a.and_then(|a| a.first()), base[0]),
        f(a.and_then(|a| a.get(1)), base[1]),
    ]
}

pub(super) fn inteiro<T: TryFrom<u64> + Copy>(v: Option<&Value>, base: T) -> T {
    v.and_then(Value::as_u64)
        .and_then(|x| T::try_from(x).ok())
        .unwrap_or(base)
}

pub(super) fn booleano(v: Option<&Value>, base: bool) -> bool {
    v.and_then(Value::as_bool).unwrap_or(base)
}

pub(super) fn nome<T: Debug>(v: T) -> Value {
    Value::String(format!("{v:?}"))
}

/// A variante cujo NOME é o texto; ausente ou desconhecido ⇒ `base`.
pub(super) fn de_nome<T: Debug + Copy>(
    v: Option<&Value>,
    todos: impl IntoIterator<Item = T>,
    base: T,
) -> T {
    let Some(s) = v.and_then(Value::as_str) else {
        return base;
    };
    todos
        .into_iter()
        .find(|t| format!("{t:?}") == s)
        .unwrap_or(base)
}

pub(super) fn textura_para_json(t: &TextureSettings) -> Value {
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
                "mode": nome(t.lei.mode),
                "depth_min": num(t.lei.depth_min),
                "depth_jitter": num(t.lei.depth_jitter),
                "depth_pressure": t.lei.depth_pressure,
            }),
        );
        // A imagem pela área (J4) só quando ligada: a lei gravada antes dela fica ao byte.
        if t.lei.area
            && let Some(l) = o.get_mut("lei").and_then(Value::as_object_mut)
        {
            l.insert("area".to_owned(), json!(true));
        }
        // A maciez e a ponta na profundidade do Height (pastel, 09/11) só quando ligadas: a lei de antes fica ao byte.
        if let Some(l) = o.get_mut("lei").and_then(Value::as_object_mut) {
            if t.lei.softness > 0.0 {
                l.insert("softness".to_owned(), json!(num(t.lei.softness)));
            }
            if t.lei.tip_depth {
                l.insert("tip_depth".to_owned(), json!(true));
            }
            if t.lei.fill > 0.0 {
                l.insert("fill".to_owned(), json!(num(t.lei.fill)));
            }
        }
    }
    v
}

/// A ponta de N imagens (L19) do JSON; AUSENTE = a imagem única.
pub(super) fn mangueira_de_json(v: Option<&Value>) -> ImageHose {
    let Some(o) = v.and_then(Value::as_object) else {
        return ImageHose::NEUTRA;
    };
    let mut h = ImageHose::NEUTRA;
    h.n = inteiro(o.get("n"), 1u8).clamp(1, HOSE_MAX as u8);
    h.mode = de_nome(
        o.get("mode"),
        (0..HoseMode::COUNT).map(HoseMode::from_u8),
        HoseMode::Random,
    );
    if let Some(a) = o.get("pesos").and_then(Value::as_array) {
        for (k, p) in h.pesos.iter_mut().enumerate() {
            *p = inteiro(a.get(k), *p);
        }
    }
    h
}

/// A lei do grão (L9) do JSON; AUSENTE = a de sempre (nunca a do pincel anterior: o preset sem ela é o de antes).
pub(super) fn lei_de_json(v: Option<&Value>) -> GrainLaw {
    let Some(o) = v.and_then(Value::as_object) else {
        return GrainLaw::NEUTRA;
    };
    let n = GrainLaw::NEUTRA;
    GrainLaw {
        mode: de_nome(
            o.get("mode"),
            (0..GrainMode::COUNT).map(GrainMode::from_u8),
            n.mode,
        ),
        depth_min: f(o.get("depth_min"), n.depth_min),
        depth_jitter: f(o.get("depth_jitter"), n.depth_jitter),
        depth_pressure: booleano(o.get("depth_pressure"), n.depth_pressure),
        area: booleano(o.get("area"), n.area),
        contraste: n.contraste,
        softness: f(o.get("softness"), n.softness).clamp(0.0, 1.0),
        tip_depth: booleano(o.get("tip_depth"), n.tip_depth),
        fill: f(o.get("fill"), n.fill).clamp(0.0, 1.0),
        leitura_dupla: n.leitura_dupla,
    }
}

pub(super) fn textura_de_json(v: Option<&Value>, base: TextureSettings) -> TextureSettings {
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

pub(super) fn marca_para_json(j: &DabJitter) -> Value {
    let mut v = json!({
        "opacity": num(j.opacity),
        "count": num(j.count),
        "flip_x": num(j.flip_x),
        "flip_y": num(j.flip_y),
        "roundness": num(j.roundness),
        "angle_per_stroke": num(j.angle_per_stroke),
        "across": num(j.across),
        "particle": num(j.particle),
    });
    // A partícula em píxeis (J4) só ligada: o preset sem ela é o de antes, ao byte.
    if j.particle_px > 0.0 {
        v["particle_px"] = num(j.particle_px);
        // A contagem que cresce com o pincel (dono 08/11) só ligada: o Pastel é o de antes, ao byte.
        if j.particle_ref_px > 0.0 {
            v["particle_ref_px"] = num(j.particle_ref_px);
        }
    }
    // O tecto da partícula e o véu (J4) só ligados: idem.
    if j.particle_max_px > 0.0 {
        v["particle_max_px"] = num(j.particle_max_px);
    }
    if j.particle_min_px > 0.0 {
        v["particle_min_px"] = num(j.particle_min_px);
    }
    if j.po > 0.0 {
        v["po"] = num(j.po);
        v["po_alcance"] = num(j.po_alcance);
        v["po_px"] = num(j.po_px);
        v["po_opacidade"] = num(j.po_opacidade);
        // O lado do pó só ligado: o preset com pó de antes é o de antes, ao byte.
        if j.po_lado != 0.0 {
            v["po_lado"] = num(j.po_lado);
        }
    }
    if j.veu > 0.0 {
        v["veu"] = num(j.veu);
        v["veu_escala"] = num(j.veu_escala);
        v["veu_opacidade"] = num(j.veu_opacidade);
    }
    // A marca abaixo de meio píxel pela área (J6) só ligada: idem.
    if j.subpixel_area {
        v["subpixel_area"] = json!(true);
    }
    v
}

/// O jitter da marca (L4) do JSON; uma chave ausente (um preset de antes) = a de `base`.
pub(super) fn marca_de_json(v: Option<&Value>, base: DabJitter) -> DabJitter {
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
        // AUSENTE = desligada (nunca a do pincel anterior: o preset sem ela é o de antes).
        particle_px: f(o.get("particle_px"), 0.0).max(0.0),
        particle_ref_px: f(o.get("particle_ref_px"), 0.0).max(0.0),
        particle_max_px: f(o.get("particle_max_px"), 0.0).max(0.0),
        particle_min_px: f(o.get("particle_min_px"), 0.0).max(0.0),
        po: f(o.get("po"), 0.0).max(0.0),
        po_alcance: f(o.get("po_alcance"), 1.0).max(0.0),
        po_px: f(o.get("po_px"), 2.0).max(0.5),
        po_opacidade: f(o.get("po_opacidade"), 0.3).clamp(0.0, 1.0),
        po_lado: f(o.get("po_lado"), 0.0).clamp(-1.0, 1.0),
        veu: f(o.get("veu"), 0.0).clamp(0.0, 1.0),
        veu_escala: f(o.get("veu_escala"), 1.0).max(0.05),
        veu_opacidade: f(o.get("veu_opacidade"), 1.0)
            .clamp(0.0, ph2d_painter_brush::DabJitter::VEU_OPACIDADE_MAX),
        subpixel_area: o
            .get("subpixel_area")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }
}

/// A cor pela mão (L11) do JSON; uma chave ausente = a de `base`.
pub(super) fn cor_mao_de_json(v: Option<&Value>, base: CorDaMao) -> CorDaMao {
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
        // AUSENTE = desligado (nunca o do pincel anterior: o preset sem ele é o de antes).
        vies: {
            let a = o.get("vies").and_then(Value::as_array);
            [0, 1, 2].map(|k| f(a.and_then(|a| a.get(k)), 0.0))
        },
    }
}

pub(super) fn mangueira_para_json(h: &ImageHose) -> Value {
    json!({
        "n": h.n,
        "mode": nome(h.mode),
        "pesos": h.pesos.to_vec(),
    })
}

pub(super) fn cor_mao_para_json(c: &CorDaMao) -> Value {
    let mut v = json!({
        "secondary": [num(c.secondary[0]), num(c.secondary[1]), num(c.secondary[2])],
        "secondary_jitter": num(c.secondary_jitter),
        "pressure_secondary": num(c.pressure_secondary),
        "pressure_hue": num(c.pressure_hue),
        "pressure_sat": num(c.pressure_sat),
        "pressure_val": num(c.pressure_val),
    });
    // O viés (J4) só ligado: o preset sem ele é o de antes, ao byte.
    if c.vies != [0.0; 3] {
        v["vies"] = json!(c.vies.map(num));
    }
    v
}
