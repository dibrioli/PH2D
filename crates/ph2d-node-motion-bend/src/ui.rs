//! **A SUPERFÍCIE DE PAINEL do `motion.bend`** — os hints e as unidades.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o corte é por
//! RESPONSABILIDADE: o `lib.rs` responde *como a dobra funciona* e este responde *como ela se
//! apresenta*. É o mesmo corte que o `motion.spline_wrap` e o `motion.trail` já fizeram.

use super::{DIRECTION, LIMITS, MODE, MODE_LABELS};
use ph2d_node_registry::{ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget};

pub(super) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "angle",
        label: "Angle",
        min: -270.0,
        max: 270.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    // ⚠️ Um `Angle`, e a volta INTEIRA: a direção da dobra é um eixo, e um eixo tem 360° de
    // resposta distinta (a `−90` a dobra corre para baixo, e isso não é o mesmo que `+90`).
    ParamUiHint {
        param: DIRECTION,
        label: "Direction",
        min: -180.0,
        max: 180.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: MODE,
        label: "Mode",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: MODE_LABELS,
        },
    },
    // ⚠️ **O curso é `−1..1` e o step `0,01`**: a fatia é uma FRAÇÃO do extent medido, e o
    // curso INTEIRO tem de ser alcançável — um `min` acima de `−1` esconderia a ponta.
    ParamUiHint {
        param: LIMITS.0,
        label: "Limit Lower",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: LIMITS.1,
        label: "Limit Upper",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
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
        param: "pivot_x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "pivot_y",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "angle",
        unit: ParamUnit::Angle,
    },
];
