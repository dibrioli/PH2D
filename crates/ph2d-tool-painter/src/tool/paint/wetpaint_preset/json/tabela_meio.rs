//! Os campos do `BrushSpec` que só a MACRO leva (W6c): os da aguada, do impasto, os flags dos meios e os
//! que o Wet Paint não lê mas o Digital lê. Mesmas leis do [`super`].

use super::valores::{
    booleano, de_nome, f, inteiro, nome, num, textura_de_json, textura_para_json,
};
use super::{Campo, c, ligado, so};
use ph2d_painter_brush::height_modes::{DepthSource, DrawTo};
use ph2d_painter_brush::{BrushSpec, DabRotation, DualMode};
use serde_json::{Value, json};

pub(super) static CAMPOS: &[Campo] = &[
    c(
        "needs_heading",
        |b| json!(b.needs_heading),
        |b, v| b.needs_heading = booleano(v, b.needs_heading),
    ),
    c(
        "watercolor",
        |b| json!(b.watercolor),
        |b, v| b.watercolor = booleano(v, b.watercolor),
    ),
    c(
        "impasto",
        |b| json!(b.impasto),
        |b, v| b.impasto = booleano(v, b.impasto),
    ),
    c(
        "fora_do_digital",
        |b| json!(b.fora_do_digital),
        |b, v| b.fora_do_digital = booleano(v, b.fora_do_digital),
    ),
    c(
        "edge_gain",
        |b| num(b.edge_gain),
        |b, v| b.edge_gain = f(v, b.edge_gain),
    ),
    c(
        "edge_spread",
        |b| num(b.edge_spread),
        |b, v| b.edge_spread = f(v, b.edge_spread),
    ),
    c(
        "smooth_edges",
        |b| json!(b.smooth_edges),
        |b, v| b.smooth_edges = booleano(v, b.smooth_edges),
    ),
    c(
        "granulation",
        |b| num(b.granulation),
        |b, v| b.granulation = f(v, b.granulation),
    ),
    c("fill", |b| num(b.fill), |b, v| b.fill = f(v, b.fill)),
    c("depth", |b| num(b.depth), |b, v| b.depth = f(v, b.depth)),
    c(
        "opacity",
        |b| num(b.opacity),
        |b, v| b.opacity = f(v, b.opacity),
    ),
    c("warp", |b| num(b.warp), |b, v| b.warp = f(v, b.warp)),
    c(
        "edge_flow",
        |b| textura_para_json(&b.edge_flow),
        |b, v| b.edge_flow = textura_de_json(v, b.edge_flow),
    ),
    c(
        "paper_edge",
        |b| num(b.paper_edge),
        |b, v| b.paper_edge = f(v, b.paper_edge),
    ),
    c(
        "wet_smudge",
        |b| num(b.wet_smudge),
        |b, v| b.wet_smudge = f(v, b.wet_smudge),
    ),
    c(
        "wet_rewet",
        |b| num(b.wet_rewet),
        |b, v| b.wet_rewet = f(v, b.wet_rewet),
    ),
    c(
        "wet_charge",
        |b| num(b.wet_charge),
        |b, v| b.wet_charge = f(v, b.wet_charge),
    ),
    c(
        "wet_dilution",
        |b| num(b.wet_dilution),
        |b, v| b.wet_dilution = f(v, b.wet_dilution),
    ),
    c(
        "wet_pull",
        |b| num(b.wet_pull),
        |b, v| b.wet_pull = f(v, b.wet_pull),
    ),
    c(
        "granulation_use_paper",
        |b| json!(b.granulation_use_paper),
        |b, v| b.granulation_use_paper = booleano(v, b.granulation_use_paper),
    ),
    c(
        "watercolor_shape_auto",
        |b| json!(b.watercolor_shape_auto),
        |b, v| b.watercolor_shape_auto = booleano(v, b.watercolor_shape_auto),
    ),
    c(
        "impasto_smooth_edges",
        |b| json!(b.impasto_smooth_edges),
        |b, v| b.impasto_smooth_edges = booleano(v, b.impasto_smooth_edges),
    ),
    c(
        "impasto_depth",
        |b| num(b.impasto_depth),
        |b, v| b.impasto_depth = f(v, b.impasto_depth),
    ),
    c(
        "film_depth",
        |b| num(b.film_depth),
        |b, v| b.film_depth = f(v, b.film_depth),
    ),
    c(
        "impasto_source",
        |b| nome(b.impasto_source),
        |b, v| {
            b.impasto_source = de_nome(
                v,
                (0..DepthSource::COUNT).map(DepthSource::from_u8),
                b.impasto_source,
            );
        },
    ),
    c(
        "impasto_draw_to",
        |b| nome(b.impasto_draw_to),
        |b, v| {
            b.impasto_draw_to = de_nome(
                v,
                (0..DrawTo::COUNT).map(DrawTo::from_u8),
                b.impasto_draw_to,
            );
        },
    ),
    c(
        "impasto_smoothing",
        |b| num(b.impasto_smoothing),
        |b, v| b.impasto_smoothing = f(v, b.impasto_smoothing),
    ),
    c(
        "impasto_body",
        |b| num(b.impasto_body),
        |b, v| b.impasto_body = f(v, b.impasto_body),
    ),
    c(
        "impasto_plow",
        |b| num(b.impasto_plow),
        |b, v| b.impasto_plow = f(v, b.impasto_plow),
    ),
    c(
        "impasto_push",
        |b| num(b.impasto_push),
        |b, v| b.impasto_push = f(v, b.impasto_push),
    ),
    c(
        "impasto_shine",
        |b| num(b.impasto_shine),
        |b, v| b.impasto_shine = f(v, b.impasto_shine),
    ),
    c(
        "impasto_roughness",
        |b| num(b.impasto_roughness),
        |b, v| b.impasto_roughness = f(v, b.impasto_roughness),
    ),
    c(
        "impasto_metallic",
        |b| num(b.impasto_metallic),
        |b, v| b.impasto_metallic = f(v, b.impasto_metallic),
    ),
    c(
        "impasto_wax",
        |b| num(b.impasto_wax),
        |b, v| b.impasto_wax = f(v, b.impasto_wax),
    ),
    c(
        "impasto_wax_color",
        |b| {
            json!([
                num(b.impasto_wax_color[0]),
                num(b.impasto_wax_color[1]),
                num(b.impasto_wax_color[2])
            ])
        },
        |b, v| {
            let a = v.and_then(Value::as_array);
            for (i, c) in b.impasto_wax_color.iter_mut().enumerate() {
                *c = f(a.and_then(|a| a.get(i)), *c);
            }
        },
    ),
    c("flow", |b| num(b.flow), |b, v| b.flow = f(v, b.flow)),
    c(
        "accumulate",
        |b| json!(b.accumulate),
        |b, v| b.accumulate = booleano(v, b.accumulate),
    ),
    c(
        "airbrush_rate_s",
        |b| num(b.airbrush_rate_s),
        |b, v| b.airbrush_rate_s = f(v, b.airbrush_rate_s),
    ),
    c(
        "space_attenuation",
        |b| json!(b.space_attenuation),
        |b, v| b.space_attenuation = booleano(v, b.space_attenuation),
    ),
    // J3: só se gravam ligados; ausentes = desligados, nunca o do pincel anterior.
    // Sem tecto: acima do raio a dureza do dab já é 0 (`dureza_do_dab`).
    so(
        "borda_px",
        |b| num(b.borda_px),
        |b, v| b.borda_px = f(v, 0.0).max(0.0),
        |b| ligado(b.borda_px),
    ),
    so(
        "sub_passo_fino",
        |b| json!(b.sub_passo_fino),
        |b, v| b.sub_passo_fino = booleano(v, false),
        |b| b.sub_passo_fino,
    ),
    // O passo pela pressão (J4) só ligado.
    so(
        "passo_pela_pressao",
        |b| json!(b.passo_pela_pressao),
        |b, v| b.passo_pela_pressao = booleano(v, false),
        |b| b.passo_pela_pressao,
    ),
    so(
        "parado_s",
        |b| num(b.parado_s),
        |b, v| b.parado_s = f(v, 0.0).max(0.0),
        |b| ligado(b.parado_s),
    ),
    // O tecto por dab (J3) só ligado; o legado `traco.tecto_por_dab`, em `pincel_de_json`.
    so(
        "tecto_por_dab",
        |b| json!(b.tecto_por_dab),
        |b, v| b.tecto_por_dab = booleano(v, false),
        |b| b.tecto_por_dab,
    ),
    so(
        "wash",
        |b| json!(b.wash),
        |b, v| b.wash = booleano(v, false),
        |b| b.wash,
    ),
    // O fio do arrasto (J6) só ligado; ausente = desligado.
    so(
        "smear_fio",
        |b| json!(b.smear_fio),
        |b, v| b.smear_fio = booleano(v, false),
        |b| b.smear_fio,
    ),
    // A mistura opaca do Digital só ligada; ausente = `Off` (o pincel de antes, ao bit).
    so(
        "mistura_opaca",
        ler_mistura_opaca,
        escrever_mistura_opaca,
        |b| b.mistura_opaca.ligada(),
    ),
    // O dual brush (L6) só ARMADO; AUSENTE = desligado, nunca a 2.ª ponta do pincel anterior; sub-chave ausente =
    // a de fábrica.
    so("dual", ler_dual, escrever_dual, |b| b.dual.is_some()),
    c("pen_tilt", ler_pen_tilt, escrever_pen_tilt),
];

fn ler_mistura_opaca(b: &BrushSpec) -> Value {
    let m = &b.mistura_opaca;
    let mut o = json!({
        "modo": nome(m.modo),
        "paint_amount": num(m.paint_amount),
        "paint_density": num(m.paint_density),
        "color_stretch": num(m.color_stretch),
        "amount_pressure": num(m.amount_pressure),
        "density_pressure": num(m.density_pressure),
    });
    // «Sample» só se grava fora do default: os pincéis de antes guardam o mesmo byte.
    if m.amostra != Default::default() {
        o["sample"] = json!(m.amostra.nome());
    }
    if m.meia_definicao {
        o["meia_definicao"] = json!(true);
    }
    o
}

fn escrever_mistura_opaca(b: &mut BrushSpec, v: Option<&Value>) {
    use ph2d_painter_brush::mistura_opaca::{MisturaOpaca, ModoMisturaOpaca};
    let fab = MisturaOpaca::default();
    let c = |k: &str| v.and_then(|v| v.get(k));
    b.mistura_opaca = MisturaOpaca {
        // Os nomes de antes de 07/11 (`Blend`, `RunningColor`) carregam como `WetBlend`.
        modo: c("modo")
            .and_then(Value::as_str)
            .and_then(ModoMisturaOpaca::do_nome)
            .unwrap_or(fab.modo),
        paint_amount: f(c("paint_amount"), fab.paint_amount).clamp(0.0, 1.0),
        paint_density: f(c("paint_density"), fab.paint_density).clamp(0.0, 1.0),
        color_stretch: f(c("color_stretch"), fab.color_stretch).clamp(0.0, 1.0),
        amount_pressure: f(c("amount_pressure"), fab.amount_pressure).clamp(0.0, 1.0),
        density_pressure: f(c("density_pressure"), fab.density_pressure).clamp(0.0, 1.0),
        // «Sample» (#79): ausente = This layer (gravado antes de 08/11).
        amostra: c("sample")
            .and_then(Value::as_str)
            .and_then(ph2d_painter_brush::mistura_opaca::AmostraDaMistura::do_nome)
            .unwrap_or(fab.amostra),
        meia_definicao: c("meia_definicao")
            .and_then(Value::as_bool)
            .unwrap_or(fab.meia_definicao),
    };
}

fn ler_dual(b: &BrushSpec) -> Value {
    b.dual.map_or(Value::Null, |d| {
        json!({
            "modo": nome(d.modo),
            "forma": textura_para_json(&d.forma),
            "escala": num(d.escala),
            "passo": num(d.passo),
            "espalhar": num(d.espalhar),
            "atraves": num(d.atraves),
            "contagem": d.contagem,
            "angulo": d.angulo,
            "rotacao": num(d.rotacao),
            "dureza": num(d.dureza),
        })
    })
}

fn escrever_dual(b: &mut BrushSpec, v: Option<&Value>) {
    let n = ph2d_painter_brush::DualTip::DE_FABRICA;
    b.dual = v.filter(|v| v.is_object()).map(|v| {
        let c = |k: &str| v.get(k);
        ph2d_painter_brush::DualTip {
            modo: de_nome(
                c("modo"),
                (0..DualMode::COUNT).map(DualMode::from_u8),
                DualMode::Multiply,
            ),
            forma: textura_de_json(c("forma"), n.forma),
            escala: f(c("escala"), n.escala),
            passo: f(c("passo"), n.passo),
            espalhar: f(c("espalhar"), n.espalhar),
            atraves: f(c("atraves"), n.atraves),
            contagem: inteiro(c("contagem"), n.contagem),
            angulo: inteiro(c("angulo"), n.angulo),
            rotacao: f(c("rotacao"), n.rotacao),
            dureza: f(c("dureza"), n.dureza),
        }
    });
}

fn ler_pen_tilt(b: &BrushSpec) -> Value {
    let t = b.pen_tilt;
    json!({"rotation": nome(t.rotation), "start_deg": num(t.start_deg), "full_deg": num(t.full_deg),
           "width": num(t.width), "length": num(t.length), "smooth": num(t.smooth), "lag_px": num(t.lag_px),
           "opacity": num(t.opacity)})
}

/// Campo ausente = o pincel de `b` (a lei do módulo); campo PRESENTE com uma sub-chave ausente (gravado antes dela, a
/// `opacity` da L2) = a de fábrica dessa sub-chave, nunca a de agora.
fn escrever_pen_tilt(b: &mut BrushSpec, v: Option<&Value>) {
    if v.is_none() {
        return;
    }
    let fab = ph2d_painter_brush::PenTilt::default();
    let t = &mut b.pen_tilt;
    let c = |k: &str| v.and_then(|v| v.get(k));
    t.rotation = de_nome(
        c("rotation"),
        [0.0, 1.0, 2.0].map(DabRotation::from_index),
        fab.rotation,
    );
    t.start_deg = f(c("start_deg"), fab.start_deg);
    t.full_deg = f(c("full_deg"), fab.full_deg);
    t.width = f(c("width"), fab.width);
    t.length = f(c("length"), fab.length);
    t.smooth = f(c("smooth"), fab.smooth);
    t.lag_px = f(c("lag_px"), fab.lag_px);
    t.opacity = f(c("opacity"), fab.opacity);
}
