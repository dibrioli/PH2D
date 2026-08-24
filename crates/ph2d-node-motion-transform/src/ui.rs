//! **A SUPERFÍCIE DE PAINEL do `motion.transform`** — os hints, as unidades, os gates de
//! visibilidade e o piso digitável do flip.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o corte é por
//! RESPONSABILIDADE: o `lib.rs` responde *como o afim funciona* e este responde *como ele se
//! apresenta*. É o mesmo corte que o `motion.spline_wrap`, o `motion.trail` e o `motion.bend`
//! já fizeram.

use super::{SCALE_Y, UNIFORM};
use ph2d_node_registry::{ParamGate, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget};

/// Param UI hints (M1.P1) for the transform rows.
pub(super) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "pivot_mode",
        label: "Scale About",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["World Origin", "Point", "Centroid"],
        },
    },
    ParamUiHint {
        param: "pivot_x",
        label: "Pivot X",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "pivot_y",
        label: "Pivot Y",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "scale",
        label: "Scale",
        min: 0.0,
        max: 5.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **O link nasce LIGADO**, e é o que faz o nó continuar a ser o de sempre até alguém
    // desligar a corrente.
    ParamUiHint {
        param: UNIFORM,
        label: "Uniform",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    ParamUiHint {
        param: SCALE_Y,
        label: "Scale Y",
        min: 0.0,
        max: 5.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset_x",
        label: "Offset X",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset_y",
        label: "Offset Y",
        min: -10.0,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
];

/// **What each of this node's numbers IS** (doc 88, Wave A) — never how it is
/// shown. A `Length` is stored in world METRES and the panel resolves the face
/// the artist reads (`px` or `m`) from `ProjectSettings::display_unit`; a node
/// that could pin one would be overriding a setting it does not own.
///
/// Only params whose value is a world COORDINATE or a world DISTANCE are declared
/// here. A weight, a fraction, a rate and a count are left bare on purpose: a unit
/// that is wrong is worse than a unit that is missing, because the artist can read
/// a bare number but a mislabelled one teaches them something false.
pub(super) static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "offset_x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "offset_y",
        unit: ParamUnit::Length,
    },
];

/// **O FLIP é uma escala NEGATIVA, e ele entra pela caixa de texto** (doc 89 folha 05 — Blender
/// *Transform Geometry*: o espelho de um layout é `scale = −1` num eixo).
///
/// ⚠️ **O slider fica em `0..5` de propósito, e o piso digitável é que desce.** Alargar o CURSO
/// para `−5..5` gastaria metade do percurso do knob no caso raro e apertaria o comum — e o que
/// um `ParamHardMin` faz é exactamente isto: ele **ALARGA a caixa de texto para fora do
/// slider**, sem mexer no que o dedo alcança. (A célula `from`/`to` do `motion.spline_wrap`
/// recusou o mesmo mecanismo, e por uma razão que aqui não vale: lá o MOTOR clampa em `[0,1]`,
/// então um número fora seria aceite e desmentido em silêncio. Aqui uma escala negativa é
/// honrada pela aritmética tal como foi digitada.)
///
/// ⚠️ **`−5` e não `−∞`**: é o espelho do teto do slider, e um layout espelhado e ampliado
/// cinco vezes já é o dobro do que o curso positivo alcança.
pub(super) static PARAM_HARD_MIN: &[ph2d_node_registry::ParamHardMin] = &[
    ph2d_node_registry::ParamHardMin {
        param: "scale",
        min: -5.0,
    },
    ph2d_node_registry::ParamHardMin {
        param: SCALE_Y,
        min: -5.0,
    },
];

/// The two coordinates belong to the mode that reads them: at the origin they are
/// zero by definition, and on a centroid the layout answers — so a pair of number
/// rows in either would be two knobs the cook never opens.
pub(super) static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: "pivot_x",
        when: "pivot_mode",
        values: &[1],
    },
    ParamGate {
        param: "pivot_y",
        when: "pivot_mode",
        values: &[1],
    },
    // Com o link LIGADO o `scale_y` não é lido, então ele não é pintado — o mesmo gate que o
    // irmão `motion.scale` declara sobre o `amount_y`.
    ParamGate {
        param: SCALE_Y,
        when: UNIFORM,
        values: &[0],
    },
];
