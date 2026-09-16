//! **OS TRÊS NÚMEROS DO PINCEL DE PLANO** — irmão (`#[path]`) do [`super::rows`],
//! cortado por ASSUNTO, como os do tecido, da pose e do contorno.
//!
//! ⚠️⚠️ **Os dois primeiros NÃO são afinação: são a ferramenta.** `Height 1 /
//! Depth 0` apara, `0 / 1` enche, `1 / 1` achata — medido, `151`, `114` e
//! `265 = 151 + 114` vértices tocados. É por isso que os dois nascem no nível
//! BÁSICO: *esconder um deles é esconder metade do pincel*.
//!
//! ⚠️ **E `0` apaga aquele lado inteiro**, com os dois a zero a deixarem o pincel
//! **inerte sem deixar de existir** — o *nada* do controlo, alcançável de
//! propósito (espec §3.1, gate G-8).

use super::show::e_pincel_de_plano;
use super::types::{Place, Row};
use crate::state::UiLevel;

pub(super) const PLANO_ALTURA: Row = Row {
    label: "panel.sculpt3d.plano_altura",
    slider: crate::ids::SCULPT3D_PLANO_ALTURA,
    chip: crate::ids::SCULPT3D_PLANO_ALTURA_NUM,
    min: 0.0,
    max: 1.0,
    step: 0.05, // LITERAL-PX-OK: fracao do curso, nao metrica de layout
    decimals: 2,
    get: |u| u.brush.plano_altura,
    set: |u, v| u.brush.plano_altura = v,
    show: e_pincel_de_plano,
    level: UiLevel::Basic,
    place: Place::Knobs,
};

pub(super) const PLANO_PROFUNDIDADE: Row = Row {
    label: "panel.sculpt3d.plano_profundidade",
    slider: crate::ids::SCULPT3D_PLANO_PROFUNDIDADE,
    chip: crate::ids::SCULPT3D_PLANO_PROFUNDIDADE_NUM,
    min: 0.0,
    max: 1.0,
    step: 0.05, // LITERAL-PX-OK: fracao do curso, nao metrica de layout
    decimals: 2,
    get: |u| u.brush.plano_profundidade,
    set: |u, v| u.brush.plano_profundidade = v,
    show: e_pincel_de_plano,
    level: UiLevel::Basic,
    place: Place::Knobs,
};

pub(super) const PLANO_AREA: Row = Row {
    label: "panel.sculpt3d.plano_area",
    slider: crate::ids::SCULPT3D_PLANO_AREA,
    chip: crate::ids::SCULPT3D_PLANO_AREA_NUM,
    min: 0.0,
    max: 2.0,
    step: 0.05, // LITERAL-PX-OK: fracao do curso, nao metrica de layout
    decimals: 2,
    get: |u| u.brush.area_radius_frac,
    set: |u, v| u.brush.area_radius_frac = v,
    show: e_pincel_de_plano,
    level: UiLevel::Pro,
    place: Place::Knobs,
};
