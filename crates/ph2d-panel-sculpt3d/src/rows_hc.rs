//! **OS DOIS KNOBS DO HC** (o *Surface Smooth*) — irmão (`#[path]`) do
//! [`super::rows`], cortado por ASSUNTO como os do plano, do tecido e da pose.
//!
//! ⚠️ **A pergunta é `verb == SurfaceSmooth`, e não `uses_neighbours()`** — o
//! irmão `Smooth` também lê o anel e não tem `b` nenhum para devolver; oferecer-
//! lhe estes dois seria um par de sliders que não move um vértice.

use super::types::{Place, Row};
use crate::state::UiLevel;
use ph2d_sculpt3d::Verb;

// **OS DOIS KNOBS DO HC** — o que ele DEVOLVE, e de onde a devolução vem.
//
// ⚠️ **A pergunta é `verb == SurfaceSmooth`, e não `uses_neighbours()`** — o
// irmão [`Verb::Smooth`] também lê o anel e não tem `b` nenhum para
// devolver; oferecer-lhe estes dois seria um par de sliders que não move um
// vértice, que é exactamente o knob morto que este arquivo evita.
pub(super) const HC_SHAPE: Row = Row {
    label: "panel.sculpt3d.hc_shape",
    slider: crate::ids::SCULPT3D_HC_SHAPE,
    chip: crate::ids::SCULPT3D_HC_SHAPE_NUM,
    min: 0.0,
    max: 1.0,
    step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
    decimals: 2,
    get: |u| u.brush.hc_shape,
    set: |u, v| u.brush.hc_shape = v,
    show: |u| u.brush.verb == Verb::SurfaceSmooth,
    // ⚠️ **Pro, e o default está no MEIO de um fator** — a varredura mediu
    // uma troca monótona e SUAVE em toda a faixa (segurar mais a pose custa
    // 1,6× menos deriva por 7% menos alisamento, sem joelho e sem cliff),
    // então não há óptimo a escolher: o `0,5` é o meio, dito e não vestido
    // de medição. Ver `ph2d_sculpt3d::HC_SHAPE_DEFAULT`.
    level: UiLevel::Pro,
    place: Place::Knobs,
};
pub(super) const HC_VERTEX: Row = Row {
    label: "panel.sculpt3d.hc_vertex",
    slider: crate::ids::SCULPT3D_HC_VERTEX,
    chip: crate::ids::SCULPT3D_HC_VERTEX_NUM,
    // ⚠️ **O piso é `0,5` e ele NÃO é gosto:** abaixo dele o operador
    // AMPLIFICA em vez de contrair, e a faixa do Blender (`[0, 1]`) alcança
    // o disfuncional. A forma fechada e a tabela medida vivem em
    // `ph2d_sculpt3d::HC_VERTEX_DEFAULT`; aqui fica só a consequência — o
    // dedo não chega lá, e o clamp do MOTOR cobre o que um documento traga.
    min: ph2d_sculpt3d::HC_VERTEX_MIN,
    max: 1.0,
    step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
    decimals: 2,
    get: |u| u.brush.hc_vertex,
    set: |u, v| u.brush.hc_vertex = v,
    show: |u| u.brush.verb == Verb::SurfaceSmooth,
    // ⚠️ **O default É o piso**, e é onde ele alisa MAIS (0,6806 da
    // rugosidade removível contra 0,6903 em 0,650). O outro extremo, `1`, é
    // o `strength = 0` deste verbo escrito no outro eixo: a correção passa a
    // ser exactamente o que o passo laplaciano somou.
    level: UiLevel::Pro,
    place: Place::Knobs,
};
