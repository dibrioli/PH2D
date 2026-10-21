//! Os campos do `BrushSpec` que só a MACRO leva (W6c): os da aguada, do impasto, os flags dos meios e os
//! que o Wet Paint não lê mas o Digital lê. Mesmas leis do [`super`]: pelo NOME, `f32` pelo texto curto,
//! enums pelo nome da variante, chave ausente = o valor de `base`.

use super::{booleano, de_nome, f, inteiro, nome, num, textura_de_json, textura_para_json};
use ph2d_painter_brush::BrushSpec;
use ph2d_painter_brush::height_modes::{DepthSource, DrawTo};
use serde_json::{Value, json};

pub(super) fn ler(b: &BrushSpec, campo: &str) -> Value {
    match campo {
        "needs_heading" => json!(b.needs_heading),
        "watercolor" => json!(b.watercolor),
        "impasto" => json!(b.impasto),
        "edge_gain" => num(b.edge_gain),
        "edge_spread" => num(b.edge_spread),
        "smooth_edges" => json!(b.smooth_edges),
        "granulation" => num(b.granulation),
        "fill" => num(b.fill),
        "depth" => num(b.depth),
        "opacity" => num(b.opacity),
        "warp" => num(b.warp),
        "edge_flow" => textura_para_json(&b.edge_flow),
        "paper_edge" => num(b.paper_edge),
        "wet_smudge" => num(b.wet_smudge),
        "wet_rewet" => num(b.wet_rewet),
        "wet_charge" => num(b.wet_charge),
        "wet_dilution" => num(b.wet_dilution),
        "wet_pull" => num(b.wet_pull),
        "granulation_use_paper" => json!(b.granulation_use_paper),
        "watercolor_shape_auto" => json!(b.watercolor_shape_auto),
        "impasto_smooth_edges" => json!(b.impasto_smooth_edges),
        "impasto_depth" => num(b.impasto_depth),
        "film_depth" => num(b.film_depth),
        "impasto_source" => nome(b.impasto_source),
        "impasto_draw_to" => nome(b.impasto_draw_to),
        "impasto_smoothing" => num(b.impasto_smoothing),
        "impasto_body" => num(b.impasto_body),
        "impasto_plow" => num(b.impasto_plow),
        "impasto_push" => num(b.impasto_push),
        "impasto_shine" => num(b.impasto_shine),
        "impasto_roughness" => num(b.impasto_roughness),
        "impasto_metallic" => num(b.impasto_metallic),
        "impasto_wax" => num(b.impasto_wax),
        "impasto_wax_color" => json!([
            num(b.impasto_wax_color[0]),
            num(b.impasto_wax_color[1]),
            num(b.impasto_wax_color[2])
        ]),
        "flow" => num(b.flow),
        "accumulate" => json!(b.accumulate),
        "airbrush_rate_s" => num(b.airbrush_rate_s),
        "space_attenuation" => json!(b.space_attenuation),
        "borda_px" => num(b.borda_px),
        "sub_passo_fino" => json!(b.sub_passo_fino),
        "passo_pela_pressao" => json!(b.passo_pela_pressao),
        "parado_s" => num(b.parado_s),
        "traco" => {
            let mut v = json!({
                "tecto": num(b.traco.tecto),
                "molhada": num(b.traco.molhada),
                "queimada": num(b.traco.queimada),
                "largura": num(b.traco.largura),
            });
            // O tecto por dab (J3) só gravado LIGADO: o preset sem ele é o de antes, ao byte.
            if b.traco.tecto_por_dab
                && let Some(o) = v.as_object_mut()
            {
                o.insert("tecto_por_dab".to_owned(), json!(true));
            }
            v
        }
        // O dual brush (L6): gravado só ARMADO (`pincel_para_json`).
        "dual" => b.dual.map_or(Value::Null, |d| {
            json!({
                "modo": d.modo.to_u8(),
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
        }),
        "pen_tilt" => {
            let t = b.pen_tilt;
            json!({"rotation": t.rotation as u8, "start_deg": num(t.start_deg), "full_deg": num(t.full_deg),
                   "width": num(t.width), "length": num(t.length), "smooth": num(t.smooth), "lag_px": num(t.lag_px),
                   "opacity": num(t.opacity)})
        }
        outro => unreachable!("campo do pincel sem conversão: {outro}"),
    }
}

pub(super) fn escrever(b: &mut BrushSpec, campo: &str, v: Option<&Value>) {
    match campo {
        "needs_heading" => b.needs_heading = booleano(v, b.needs_heading),
        "watercolor" => b.watercolor = booleano(v, b.watercolor),
        "impasto" => b.impasto = booleano(v, b.impasto),
        "edge_gain" => b.edge_gain = f(v, b.edge_gain),
        "edge_spread" => b.edge_spread = f(v, b.edge_spread),
        "smooth_edges" => b.smooth_edges = booleano(v, b.smooth_edges),
        "granulation" => b.granulation = f(v, b.granulation),
        "fill" => b.fill = f(v, b.fill),
        "depth" => b.depth = f(v, b.depth),
        "opacity" => b.opacity = f(v, b.opacity),
        "warp" => b.warp = f(v, b.warp),
        "edge_flow" => b.edge_flow = textura_de_json(v, b.edge_flow),
        "paper_edge" => b.paper_edge = f(v, b.paper_edge),
        "wet_smudge" => b.wet_smudge = f(v, b.wet_smudge),
        "wet_rewet" => b.wet_rewet = f(v, b.wet_rewet),
        "wet_charge" => b.wet_charge = f(v, b.wet_charge),
        "wet_dilution" => b.wet_dilution = f(v, b.wet_dilution),
        "wet_pull" => b.wet_pull = f(v, b.wet_pull),
        "granulation_use_paper" => {
            b.granulation_use_paper = booleano(v, b.granulation_use_paper);
        }
        "watercolor_shape_auto" => {
            b.watercolor_shape_auto = booleano(v, b.watercolor_shape_auto);
        }
        "impasto_smooth_edges" => b.impasto_smooth_edges = booleano(v, b.impasto_smooth_edges),
        "impasto_depth" => b.impasto_depth = f(v, b.impasto_depth),
        "film_depth" => b.film_depth = f(v, b.film_depth),
        "impasto_source" => {
            b.impasto_source = de_nome(
                v,
                (0..DepthSource::COUNT).map(DepthSource::from_u8),
                b.impasto_source,
            );
        }
        "impasto_draw_to" => {
            b.impasto_draw_to = de_nome(
                v,
                (0..DrawTo::COUNT).map(DrawTo::from_u8),
                b.impasto_draw_to,
            );
        }
        "impasto_smoothing" => b.impasto_smoothing = f(v, b.impasto_smoothing),
        "impasto_body" => b.impasto_body = f(v, b.impasto_body),
        "impasto_plow" => b.impasto_plow = f(v, b.impasto_plow),
        "impasto_push" => b.impasto_push = f(v, b.impasto_push),
        "impasto_shine" => b.impasto_shine = f(v, b.impasto_shine),
        "impasto_roughness" => b.impasto_roughness = f(v, b.impasto_roughness),
        "impasto_metallic" => b.impasto_metallic = f(v, b.impasto_metallic),
        "impasto_wax" => b.impasto_wax = f(v, b.impasto_wax),
        "impasto_wax_color" => {
            let a = v.and_then(Value::as_array);
            for (i, c) in b.impasto_wax_color.iter_mut().enumerate() {
                *c = f(a.and_then(|a| a.get(i)), *c);
            }
        }
        "flow" => b.flow = f(v, b.flow),
        "accumulate" => b.accumulate = booleano(v, b.accumulate),
        "airbrush_rate_s" => b.airbrush_rate_s = f(v, b.airbrush_rate_s),
        "space_attenuation" => b.space_attenuation = booleano(v, b.space_attenuation),
        // J3: ausentes = desligados (só se gravam ligados), nunca o do pincel anterior.
        // Sem tecto: acima do raio a dureza do dab já é 0 (`dureza_do_dab`).
        "borda_px" => b.borda_px = f(v, 0.0).max(0.0),
        "sub_passo_fino" => b.sub_passo_fino = booleano(v, false),
        "passo_pela_pressao" => b.passo_pela_pressao = booleano(v, false),
        "parado_s" => b.parado_s = f(v, 0.0).max(0.0),
        // O plano do traço (L8): sub-chave ausente = a de fábrica.
        "traco" if v.is_some() => {
            let n = ph2d_painter_brush::TracoComposto::NEUTRO;
            let c = |k: &str| v.and_then(|v| v.get(k));
            b.traco = ph2d_painter_brush::TracoComposto {
                tecto: f(c("tecto"), n.tecto),
                molhada: f(c("molhada"), n.molhada),
                queimada: f(c("queimada"), n.queimada),
                largura: f(c("largura"), n.largura),
                tecto_por_dab: booleano(c("tecto_por_dab"), n.tecto_por_dab),
            };
        }
        // Campo ausente = o pincel de `b` (a lei do módulo); campo PRESENTE com uma sub-chave ausente (gravado
        // antes dela, a `opacity` da L2) = a de fábrica dessa sub-chave, nunca a de agora.
        "pen_tilt" if v.is_some() => {
            let fab = ph2d_painter_brush::PenTilt::default();
            let t = &mut b.pen_tilt;
            let c = |k: &str| v.and_then(|v| v.get(k));
            t.rotation = c("rotation")
                .and_then(Value::as_f64)
                .map_or(fab.rotation, ph2d_painter_brush::DabRotation::from_index);
            t.start_deg = f(c("start_deg"), fab.start_deg);
            t.full_deg = f(c("full_deg"), fab.full_deg);
            t.width = f(c("width"), fab.width);
            t.length = f(c("length"), fab.length);
            t.smooth = f(c("smooth"), fab.smooth);
            t.lag_px = f(c("lag_px"), fab.lag_px);
            t.opacity = f(c("opacity"), fab.opacity);
        }
        // O dual brush (L6): AUSENTE = desligado (só se grava armado), nunca a 2.ª ponta do pincel anterior;
        // sub-chave ausente = a de fábrica.
        "dual" => {
            let n = ph2d_painter_brush::DualTip::DE_FABRICA;
            b.dual = v.filter(|v| v.is_object()).map(|v| {
                let c = |k: &str| v.get(k);
                ph2d_painter_brush::DualTip {
                    modo: ph2d_painter_brush::DualMode::from_u8(inteiro(c("modo"), 0)),
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
        "pen_tilt" | "traco" => {}
        outro => unreachable!("campo do pincel sem conversão: {outro}"),
    }
}
