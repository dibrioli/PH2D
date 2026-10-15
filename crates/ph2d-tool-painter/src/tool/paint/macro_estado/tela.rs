//! O estado do Painter que os meios LEEM e que não é de pincel nenhum (W6c): a pilha do Composite, as
//! duas rampas de cor (Grain e Shape), a luz e o «Show» do Impasto, o relevo da silhueta e a cor do
//! papel da aguada. Ao contrário dos slots do `BrushSpec`, é da TELA: um só para todos os modos.
//!
//! Escreve-se cru e invalida-se depois (as rampas re-assam o LUT, o composite refaz-se) — as mesmas
//! bandeiras que os setters do painel levantam.

use super::super::composite::{CompositeLayer, CompositeOp, EscopoDaBorracha};
use crate::tool::PainterTool;
use ph2d_color::ColorRamp;
use ph2d_painter_brush::RampAlphaMode;
use serde_json::{Value, json};

fn rgb(c: [f32; 3]) -> Value {
    json!([c[0], c[1], c[2]])
}

fn de_rgb(v: &Value, base: [f32; 3]) -> [f32; 3] {
    let mut c = base;
    if let Some(a) = v.as_array() {
        for (i, x) in c.iter_mut().enumerate() {
            *x = a.get(i).and_then(Value::as_f64).map_or(*x, |n| n as f32);
        }
    }
    c
}

fn f32_(v: &Value, base: f32) -> f32 {
    v.as_f64().map_or(base, |n| n as f32)
}

fn u8_(v: &Value) -> Option<u8> {
    v.as_u64().and_then(|n| u8::try_from(n).ok())
}

fn camada_para_json(c: &CompositeLayer) -> Value {
    json!({
        "op": c.op.to_u8(),
        "strength": c.strength,
        "erase_scope": c.erase_scope.to_u8(),
        "color": c.color.map(rgb),
        "hardness": c.hardness,
        "size": c.size,
    })
}

fn camada_de_json(v: &Value, base: CompositeLayer) -> CompositeLayer {
    CompositeLayer {
        op: u8_(&v["op"]).map_or(base.op, CompositeOp::from_u8),
        strength: f32_(&v["strength"], base.strength),
        erase_scope: match u8_(&v["erase_scope"]) {
            Some(1) => EscopoDaBorracha::Traco,
            Some(_) => EscopoDaBorracha::Tudo,
            None => base.erase_scope,
        },
        color: match &v["color"] {
            Value::Null => None,
            c => Some(de_rgb(c, base.color.unwrap_or_default())),
        },
        hardness: v["hardness"].as_f64().map(|n| n as f32),
        size: f32_(&v["size"], base.size),
    }
}

fn rampa_para_json(r: &ColorRamp, ligada: bool, bw: bool, alfa: RampAlphaMode) -> Value {
    json!({
        "rampa": serde_json::to_value(r).unwrap_or(Value::Null),
        "ligada": ligada,
        "bw": bw,
        "alfa": alfa.to_u8(),
    })
}

impl PainterTool {
    pub(super) fn macro_tela(&self) -> Value {
        let p = &self.paint;
        json!({
            "composite": {
                "ligado": p.composite_enabled,
                "n": p.composite_len,
                "op_do_mais": p.composite_add_op.to_u8(),
                "camadas": p.composite.iter().map(camada_para_json).collect::<Vec<_>>(),
            },
            "rampa_grao": rampa_para_json(
                &p.texture_ramp,
                p.texture_ramp_enabled,
                p.texture_ramp_bw,
                p.texture_ramp_alpha_mode,
            ),
            "rampa_forma": rampa_para_json(
                &p.shape_color_ramp,
                p.shape_color_ramp_enabled,
                p.shape_color_ramp_bw,
                p.shape_color_ramp_alpha_mode,
            ),
            "impasto": {
                "show": p.impasto_show,
                "live_edit": p.impasto_live_edit,
                "luzes": p.impasto_rig.lights.iter().map(|l| json!({
                    "on": l.on,
                    "angle_deg": l.angle_deg,
                    "elev_deg": l.elev_deg,
                    "intensity": l.intensity,
                    "color": rgb(l.color),
                })).collect::<Vec<_>>(),
                "selecionada": p.impasto_rig.selected,
            },
            "shape_relief": p.shape_relief,
            "paper_color": rgb(p.paper_color),
        })
    }

    pub(super) fn macro_tela_aplica(&mut self, v: &Value) {
        let p = &mut self.paint;
        let c = &v["composite"];
        p.composite_enabled = c["ligado"].as_bool().unwrap_or(p.composite_enabled);
        if let Some(n) = c["n"].as_u64().and_then(|n| usize::try_from(n).ok()) {
            p.composite_len = n.min(p.composite.len());
        }
        if let Some(op) = u8_(&c["op_do_mais"]) {
            p.composite_add_op = CompositeOp::from_u8(op);
        }
        if let Some(a) = c["camadas"].as_array() {
            for (camada, x) in p.composite.iter_mut().zip(a) {
                *camada = camada_de_json(x, *camada);
            }
        }
        let rampa = |x: &Value, r: &mut ColorRamp, ligada: &mut bool, bw: &mut bool| {
            if let Ok(nova) = serde_json::from_value::<ColorRamp>(x["rampa"].clone()) {
                *r = nova;
            }
            *ligada = x["ligada"].as_bool().unwrap_or(*ligada);
            *bw = x["bw"].as_bool().unwrap_or(*bw);
            u8_(&x["alfa"]).map(RampAlphaMode::from_u8)
        };
        let g = &v["rampa_grao"];
        if let Some(a) = rampa(
            g,
            &mut p.texture_ramp,
            &mut p.texture_ramp_enabled,
            &mut p.texture_ramp_bw,
        ) {
            p.texture_ramp_alpha_mode = a;
        }
        let s = &v["rampa_forma"];
        if let Some(a) = rampa(
            s,
            &mut p.shape_color_ramp,
            &mut p.shape_color_ramp_enabled,
            &mut p.shape_color_ramp_bw,
        ) {
            p.shape_color_ramp_alpha_mode = a;
        }
        p.texture_ramp_dirty = true;
        p.shape_ramp_dirty = true;
        p.shape_ramp_version = p.shape_ramp_version.wrapping_add(1);
        let i = &v["impasto"];
        p.impasto_show = i["show"].as_bool().unwrap_or(p.impasto_show);
        p.impasto_live_edit = i["live_edit"].as_bool().unwrap_or(p.impasto_live_edit);
        if let Some(a) = i["luzes"].as_array() {
            for (l, x) in p.impasto_rig.lights.iter_mut().zip(a) {
                l.on = x["on"].as_bool().unwrap_or(l.on);
                l.angle_deg = x["angle_deg"]
                    .as_u64()
                    .and_then(|n| u16::try_from(n).ok())
                    .unwrap_or(l.angle_deg);
                l.elev_deg = x["elev_deg"]
                    .as_u64()
                    .and_then(|n| u16::try_from(n).ok())
                    .unwrap_or(l.elev_deg);
                l.intensity = f32_(&x["intensity"], l.intensity);
                l.color = de_rgb(&x["color"], l.color);
            }
        }
        p.impasto_rig.selected = u8_(&i["selecionada"]).unwrap_or(p.impasto_rig.selected);
        p.shape_relief = f32_(&v["shape_relief"], p.shape_relief);
        p.paper_color = de_rgb(&v["paper_color"], p.paper_color);
        self.invalidate_composite();
    }
}

#[cfg(test)]
#[path = "tela_tests.rs"]
mod tests;
