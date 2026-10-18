//! Os campos do `BrushSpec` que só a MACRO leva (W6c): os da aguada, do impasto, os flags dos meios e os
//! que o Wet Paint não lê mas o Digital lê. Mesmas leis do [`super`]: pelo NOME, `f32` pelo texto curto,
//! enums pelo nome da variante, chave ausente = o valor de `base`.

use super::{booleano, de_nome, f, nome, num, textura_de_json, textura_para_json};
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
        "pen_tilt" => {
            let t = b.pen_tilt;
            json!({"rotation": t.rotation as u8, "start_deg": num(t.start_deg), "full_deg": num(t.full_deg),
                   "width": num(t.width), "length": num(t.length), "smooth": num(t.smooth), "lag_px": num(t.lag_px)})
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
        "pen_tilt" => {
            let t = &mut b.pen_tilt;
            let c = |k: &str| v.and_then(|v| v.get(k));
            if let Some(r) = c("rotation").and_then(Value::as_f64) {
                t.rotation = ph2d_painter_brush::DabRotation::from_index(r);
            }
            t.start_deg = f(c("start_deg"), t.start_deg);
            t.full_deg = f(c("full_deg"), t.full_deg);
            t.width = f(c("width"), t.width);
            t.length = f(c("length"), t.length);
            t.smooth = f(c("smooth"), t.smooth);
            t.lag_px = f(c("lag_px"), t.lag_px);
        }
        outro => unreachable!("campo do pincel sem conversão: {outro}"),
    }
}
