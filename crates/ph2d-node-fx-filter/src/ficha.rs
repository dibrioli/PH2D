//! **A ficha do cartão** do `fx.filter` — as linhas (rótulo, faixa, controlo), as unidades e os
//! tectos do valor digitado. Saiu do `lib.rs` pelo tecto de LOC (ciclo 20 W3b): o `lib.rs` é a LEI
//! (o manifesto, os gates derivados da `SPECS`, o grafo lido); isto é a APRESENTAÇÃO dela.

use ph2d_fx_op::FxOp;
use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget};

use crate::{
    COMPRIMENTO_DIGITADO, COMPRIMENTO_MAX, FALLOFF_LABELS, KIND_LABELS, NOISE_LABELS, RAMP_LABELS,
    SCOPE_LABELS, param,
};

const fn hint(
    param: &'static str,
    label: &'static str,
    min: f32,
    max: f32,
    step: f32,
    widget: ParamWidget,
) -> ParamUiHint {
    ParamUiHint {
        param,
        label,
        min,
        max,
        step,
        widget,
    }
}

const fn slider(param: &'static str, label: &'static str, min: f32, max: f32) -> ParamUiHint {
    hint(param, label, min, max, 0.005, ParamWidget::Slider)
}

const fn modos(
    param: &'static str,
    label: &'static str,
    labels: &'static [&'static str],
) -> ParamUiHint {
    hint(param, label, 0.0, 1.0, 1.0, ParamWidget::Enum { labels })
}

const L: f32 = COMPRIMENTO_MAX;

pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    hint(
        param::KIND,
        "node.fx.filter.param.kind",
        0.0,
        (FxOp::KINDS - 1) as f32,
        1.0,
        ParamWidget::Enum {
            labels: &KIND_LABELS,
        },
    ),
    modos(param::SCOPE, "node.fx.filter.param.scope", &SCOPE_LABELS),
    slider(param::RADIUS, "node.fx.filter.param.radius", 0.0, L),
    slider(param::OFFSET_X, "node.fx.filter.param.offset_x", -L, L),
    slider(param::OFFSET_Y, "node.fx.filter.param.offset_y", -L, L),
    hint(
        param::COLOR[0],
        "node.fx.filter.param.color_r",
        0.0,
        1.0,
        0.01,
        ParamWidget::Color {
            channels: param::COLOR,
        },
    ),
    hint(
        param::COLOR2[0],
        "node.fx.filter.param.color2_r",
        0.0,
        1.0,
        0.01,
        ParamWidget::Color {
            channels: param::COLOR2,
        },
    ),
    hint(
        param::OPACITY,
        "node.fx.filter.param.opacity",
        0.0,
        1.0,
        0.01,
        ParamWidget::Slider,
    ),
    modos(
        param::FALLOFF,
        "node.fx.filter.param.falloff",
        &FALLOFF_LABELS,
    ),
    modos(
        param::NOISE_MODE,
        "node.fx.filter.param.noise_mode",
        &NOISE_LABELS,
    ),
    modos(
        param::RAMP_MODE,
        "node.fx.filter.param.ramp_mode",
        &RAMP_LABELS,
    ),
    slider(param::SIZE, "node.fx.filter.param.size", 0.0, L),
    hint(
        param::DETAIL,
        "node.fx.filter.param.detail",
        1.0,
        FxOp::MAX_DETAIL as f32,
        1.0,
        ParamWidget::IntSlider,
    ),
    hint(
        param::SEED,
        "node.fx.filter.param.seed",
        0.0,
        u8::MAX as f32,
        1.0,
        ParamWidget::Seed,
    ),
    slider(param::GROW, "node.fx.filter.param.grow", -L, L),
    // Meia volta para cada lado cobre todas as matizes: é o PERÍODO, não um tecto.
    hint(
        param::HUE,
        "node.fx.filter.param.hue",
        -180.0,
        180.0,
        1.0,
        ParamWidget::Slider,
    ),
    // `-1` = cinzento / preto exacto, `+1` = croma dobrado / branco exacto: a faixa da LEI.
    hint(
        param::SATURATION,
        "node.fx.filter.param.saturation",
        -1.0,
        1.0,
        0.01,
        ParamWidget::Slider,
    ),
    hint(
        param::BRIGHTNESS,
        "node.fx.filter.param.brightness",
        -1.0,
        1.0,
        0.01,
        ParamWidget::Slider,
    ),
];

const fn digitado(param: &'static str) -> ParamHardMax {
    ParamHardMax {
        param,
        max: COMPRIMENTO_DIGITADO,
    }
}

/// O valor digitado vai além do arrasto nos comprimentos (o passe honra-o, [`COMPRIMENTO_DIGITADO`]).
pub(crate) static PARAM_HARD_MAX: &[ParamHardMax] = &[
    digitado(param::RADIUS),
    digitado(param::OFFSET_X),
    digitado(param::OFFSET_Y),
    digitado(param::SIZE),
    digitado(param::GROW),
];

const fn unidade(param: &'static str, unit: ParamUnit) -> ParamUnitDecl {
    ParamUnitDecl { param, unit }
}

pub(crate) static PARAM_UNITS: &[ParamUnitDecl] = &[
    unidade(param::RADIUS, ParamUnit::Length),
    unidade(param::OFFSET_X, ParamUnit::Length),
    unidade(param::OFFSET_Y, ParamUnit::Length),
    unidade(param::SIZE, ParamUnit::Length),
    unidade(param::GROW, ParamUnit::Length),
    unidade(param::HUE, ParamUnit::Angle),
];
