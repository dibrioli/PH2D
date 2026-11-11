//! Os campos da obra e do gesto, que o preset deixa de fora e a MACRO leva ([`super::super::campos_da_macro`]).

use super::valores::{
    booleano, de_nome, f, f2, inteiro, nome, num, par, textura_de_json, textura_para_json,
};
use super::{Campo, c, so};
use ph2d_painter_brush::line_kind::LineKind;
use ph2d_painter_brush::{MirrorAxis, StrokeMethod};
use serde_json::{Value, json};

pub(super) static CAMPOS: &[Campo] = &[
    c(
        "radius_px",
        |b| num(b.radius_px),
        |b, v| b.radius_px = f(v, b.radius_px),
    ),
    c(
        "strength",
        |b| num(b.strength),
        |b, v| b.strength = f(v, b.strength),
    ),
    c(
        "color",
        |b| json!([num(b.color[0]), num(b.color[1]), num(b.color[2])]),
        |b, v| {
            let a = v.and_then(Value::as_array);
            for (i, c) in b.color.iter_mut().enumerate() {
                *c = f(a.and_then(|a| a.get(i)), *c);
            }
        },
    ),
    c(
        "paper",
        |b| textura_para_json(&b.paper),
        |b, v| b.paper = textura_de_json(v, b.paper),
    ),
    c(
        "symmetry",
        |b| {
            json!({
                "enabled": b.symmetry.enabled,
                "circular": b.symmetry.circular,
                "axis": nome(b.symmetry.axis),
                "radial_segments": b.symmetry.radial_segments,
                "center": par(b.symmetry.center),
                "custom_dir": par(b.symmetry.custom_dir),
            })
        },
        |b, v| {
            let o = v.and_then(Value::as_object);
            let g = |k: &str| o.and_then(|o| o.get(k));
            let s = &mut b.symmetry;
            s.enabled = booleano(g("enabled"), s.enabled);
            s.circular = booleano(g("circular"), s.circular);
            s.axis = de_nome(g("axis"), (0..3).map(MirrorAxis::from_u8), s.axis);
            s.radial_segments = inteiro(g("radial_segments"), s.radial_segments);
            s.center = f2(g("center"), s.center);
            s.custom_dir = f2(g("custom_dir"), s.custom_dir);
        },
    ),
    c(
        "stroke_method",
        |b| nome(b.stroke_method),
        |b, v| {
            b.stroke_method = de_nome(
                v,
                (0..StrokeMethod::COUNT).map(StrokeMethod::from_u8),
                b.stroke_method,
            );
        },
    ),
    c(
        "grid_cell_px",
        |b| par(b.grid_cell_px),
        |b, v| b.grid_cell_px = f2(v, b.grid_cell_px),
    ),
    c(
        "grid_offset_px",
        |b| par(b.grid_offset_px),
        |b, v| b.grid_offset_px = f2(v, b.grid_offset_px),
    ),
    c(
        "grid_fit",
        |b| num(b.grid_fit),
        |b, v| b.grid_fit = f(v, b.grid_fit),
    ),
    c(
        "edge_to_edge",
        |b| json!(b.edge_to_edge),
        |b, v| b.edge_to_edge = booleano(v, b.edge_to_edge),
    ),
    c(
        "line_kind",
        |b| nome(b.line_kind),
        |b, v| {
            b.line_kind = de_nome(v, (0..6).map(LineKind::from_wire), b.line_kind);
        },
    ),
    c(
        "style_solid",
        |b| json!(b.style_solid),
        |b, v| b.style_solid = booleano(v, b.style_solid),
    ),
    c(
        "sketchy_reach",
        |b| num(b.sketchy_reach),
        |b, v| b.sketchy_reach = f(v, b.sketchy_reach),
    ),
    c(
        "sketchy_density",
        |b| num(b.sketchy_density),
        |b, v| b.sketchy_density = f(v, b.sketchy_density),
    ),
    c(
        "thread_width_px",
        |b| num(b.thread_width_px),
        |b, v| b.thread_width_px = f(v, b.thread_width_px),
    ),
    c(
        "thread_opacity",
        |b| num(b.thread_opacity),
        |b, v| b.thread_opacity = f(v, b.thread_opacity),
    ),
    c(
        "sketchy_magnetify",
        |b| json!(b.sketchy_magnetify),
        |b, v| b.sketchy_magnetify = booleano(v, b.sketchy_magnetify),
    ),
    c(
        "wire_history",
        |b| num(b.wire_history),
        |b, v| b.wire_history = f(v, b.wire_history),
    ),
    c(
        "wire_connection_line",
        |b| json!(b.wire_connection_line),
        |b, v| b.wire_connection_line = booleano(v, b.wire_connection_line),
    ),
    // As cordas sorteadas do Wire (J4 efeitos) só ligadas; ausente = desligado, nunca o do pincel anterior.
    so(
        "wire_sorteio",
        |b| json!(b.wire_sorteio),
        |b, v| b.wire_sorteio = booleano(v, false),
        |b| b.wire_sorteio,
    ),
    so(
        "wire_cordas",
        |b| json!(b.wire_cordas),
        |b, v| {
            b.wire_cordas = v
                .and_then(Value::as_u64)
                .map_or(0, |c| c.min(u64::from(u8::MAX)) as u8)
        },
        |b| b.wire_cordas != 0,
    ),
    c(
        "ribbon_weight",
        |b| num(b.ribbon_weight),
        |b, v| b.ribbon_weight = f(v, b.ribbon_weight),
    ),
    c(
        "ribbon_friction",
        |b| num(b.ribbon_friction),
        |b, v| b.ribbon_friction = f(v, b.ribbon_friction),
    ),
    c(
        "ribbon_gravity",
        |b| num(b.ribbon_gravity),
        |b, v| b.ribbon_gravity = f(v, b.ribbon_gravity),
    ),
    c(
        "ribbon_rungs",
        |b| num(b.ribbon_rungs),
        |b, v| b.ribbon_rungs = f(v, b.ribbon_rungs),
    ),
    c(
        "rough_amount",
        |b| num(b.rough_amount),
        |b, v| b.rough_amount = f(v, b.rough_amount),
    ),
    c(
        "rough_bowing",
        |b| num(b.rough_bowing),
        |b, v| b.rough_bowing = f(v, b.rough_bowing),
    ),
    c(
        "rough_passes",
        |b| json!(b.rough_passes),
        |b, v| b.rough_passes = inteiro(v, b.rough_passes),
    ),
    c(
        "dash_ratio",
        |b| num(b.dash_ratio),
        |b, v| b.dash_ratio = f(v, b.dash_ratio),
    ),
    c(
        "dash_samples",
        |b| json!(b.dash_samples),
        |b, v| b.dash_samples = inteiro(v, b.dash_samples),
    ),
];
