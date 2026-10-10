//! Os cartões deste nó — rótulos, gates e unidades (cortados do `lib.rs` pelo tecto de LOC,
//! ciclo 17). A lei do nó mora no `lib.rs`; aqui mora o que o painel mostra dela.

use super::*;

use ph2d_node_registry::{ParamUiHint, ParamWidget};

pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "rate",
        label: "node.value.time.param.rate",
        min: -8.0,
        max: 8.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset",
        label: "node.value.time.param.offset",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "stagger",
        // ⚠️ **`Phase Stagger`** (ciclo 6 W3): o `pulse.beat` pergunta a MESMA coisa — a fase
        // POR LINHA — e escrevia o nome inteiro. Três nós, uma palavra.
        label: "node.value.time.param.stagger",
        min: -2.0,
        max: 2.0,
        step: 0.02,
        widget: ParamWidget::Slider,
    },
    // ⭐ CICLO 17 — o ATRASO pela ordem e pela curva da casa (doc 126 §3.3).
    ParamUiHint {
        param: "delay",
        label: "node.value.time.param.delay",
        min: 0.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "order",
        label: "node.value.time.param.order",
        min: 0.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.value.time.param.order.0",
                "node.value.time.param.order.1",
                "node.value.time.param.order.2",
                "node.value.time.param.order.3",
                "node.value.time.param.order.4",
            ],
        },
    },
    ParamUiHint {
        param: "seed",
        label: "node.value.time.param.seed",
        min: 0.0,
        max: 999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
    ParamUiHint {
        param: "ease_curve",
        label: "node.value.time.param.ease_curve",
        min: 0.0,
        max: 14.0,
        step: 1.0,
        // ⭐ O GRÁFICO da curva (ciclo 17, doc 126 §6 — «com gráficos como na timeline»).
        widget: ParamWidget::Ease,
    },
    // A curva DESENHADA da `Custom` — um TEXT param, não um `ParamSpec`.
    ParamUiHint {
        param: CURVE_KEY,
        label: "node.value.time.param.curve",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Curve,
    },
    ParamUiHint {
        param: "ease_dir",
        label: "node.value.time.param.ease_dir",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::EasePart,
    },
    ParamUiHint {
        param: "overshoot",
        label: "node.value.time.param.overshoot",
        min: 0.0,
        max: 5.0,
        step: 0.05,
        widget: ParamWidget::EasePart,
    },
    ParamUiHint {
        param: "period",
        label: "node.value.time.param.period",
        min: 0.05,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::EasePart,
    },
    ParamUiHint {
        param: "steps",
        label: "node.value.time.param.steps",
        min: 1.0,
        max: 64.0,
        step: 1.0,
        widget: ParamWidget::EasePart,
    },
    ParamUiHint {
        param: "damping",
        label: "node.value.time.param.damping",
        min: 0.05,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::EasePart,
    },
    ParamUiHint {
        param: "bez_x1",
        label: "node.value.time.param.bez_x1",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::EasePart,
    },
    ParamUiHint {
        param: "bez_y1",
        label: "node.value.time.param.bez_y1",
        min: -1.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::EasePart,
    },
    ParamUiHint {
        param: "bez_x2",
        label: "node.value.time.param.bez_x2",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::EasePart,
    },
    ParamUiHint {
        param: "bez_y2",
        label: "node.value.time.param.bez_y2",
        min: -1.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::EasePart,
    },
    ParamUiHint {
        param: "grid_cols",
        label: "node.value.time.param.grid_cols",
        min: 1.0,
        max: 1000.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "grid_from",
        label: "node.value.time.param.grid_from",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.value.time.param.grid_from.0",
                "node.value.time.param.grid_from.1",
                "node.value.time.param.grid_from.2",
                "node.value.time.param.grid_from.3",
            ],
        },
    },
    ParamUiHint {
        param: "grid_cell",
        label: "node.value.time.param.grid_cell",
        min: 0.0,
        max: 10000.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "axis",
        label: "node.value.time.param.axis",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.value.time.param.axis.0",
                "node.value.time.param.axis.1",
                "node.value.time.param.axis.2",
            ],
        },
    },
    ParamUiHint {
        param: "origin_x",
        label: "node.value.time.param.origin_x",
        min: -50.0,
        max: 50.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "origin_y",
        label: "node.value.time.param.origin_y",
        min: -50.0,
        max: 50.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "reach",
        label: "node.value.time.param.reach",
        min: 0.0,
        max: 50.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
];

use ph2d_node_registry::{ParamGate, ParamGateAbove, ParamUnit, ParamUnitDecl};

/// **Um controlo que o cozimento não abre não é pintado.** A ordem e a curva escolhem-se como no
/// `motion.stagger`; e TODOS os números da onda só existem com o `delay` armado ([`WAVE_GATES`]).
pub(crate) static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: CURVE_KEY,
        when: "ease_curve",
        values: &[ph2d_ease::CUSTOM],
    },
    ParamGate {
        param: "ease_dir",
        when: "ease_curve",
        values: ph2d_ease::USES_DIRECTION,
    },
    ParamGate {
        param: "overshoot",
        when: "ease_curve",
        values: &[ph2d_ease::BACK],
    },
    ParamGate {
        param: "period",
        when: "ease_curve",
        values: &[ph2d_ease::ELASTIC],
    },
    ParamGate {
        param: "steps",
        when: "ease_curve",
        values: &[ph2d_ease::STEPS],
    },
    ParamGate {
        param: "damping",
        when: "ease_curve",
        values: &[ph2d_ease::SPRING],
    },
    ParamGate {
        param: "bez_x1",
        when: "ease_curve",
        values: &[ph2d_ease::BEZIER],
    },
    ParamGate {
        param: "bez_y1",
        when: "ease_curve",
        values: &[ph2d_ease::BEZIER],
    },
    ParamGate {
        param: "bez_x2",
        when: "ease_curve",
        values: &[ph2d_ease::BEZIER],
    },
    ParamGate {
        param: "bez_y2",
        when: "ease_curve",
        values: &[ph2d_ease::BEZIER],
    },
    ParamGate {
        param: "seed",
        when: "order",
        values: &[ph2d_ease::order::ORDER_RANDOM],
    },
    ParamGate {
        param: "grid_cols",
        when: "order",
        values: &[ph2d_ease::order::ORDER_GRID],
    },
    ParamGate {
        param: "grid_from",
        when: "order",
        values: &[ph2d_ease::order::ORDER_GRID],
    },
    ParamGate {
        param: "grid_cell",
        when: "order",
        values: &[ph2d_ease::order::ORDER_GRID],
    },
    ParamGate {
        param: "grid_cell",
        when: "grid_from",
        values: &[ph2d_ease::order::FROM_CELL],
    },
    ParamGate {
        param: "axis",
        when: "order",
        values: &[
            ph2d_ease::order::ORDER_GRID,
            ph2d_ease::order::ORDER_FROM_POINT,
        ],
    },
    ParamGate {
        param: "origin_x",
        when: "order",
        values: &[ph2d_ease::order::ORDER_FROM_POINT],
    },
    ParamGate {
        param: "origin_y",
        when: "order",
        values: &[ph2d_ease::order::ORDER_FROM_POINT],
    },
    ParamGate {
        param: "reach",
        when: "order",
        values: &[ph2d_ease::order::ORDER_FROM_POINT],
    },
];

/// ⭐ **A onda só aparece com o `delay` armado** (`delay > 0`, a faixa do slider começa em `0`):
/// com o atraso a zero o termo nem corre, e vinte controlos inertes no cartão seriam o knob morto
/// que o doc 90 caçou. Os gates SOMAM-SE aos de [`PARAM_GATES`].
pub(crate) static WAVE_GATES: &[ParamGateAbove] = &[
    ParamGateAbove {
        param: "order",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "seed",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "ease_curve",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: CURVE_KEY,
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "ease_dir",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "overshoot",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "period",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "steps",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "damping",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "bez_x1",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "bez_y1",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "bez_x2",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "bez_y2",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "grid_cols",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "grid_from",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "grid_cell",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "axis",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "origin_x",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "origin_y",
        when: "delay",
        above: 0.0,
    },
    ParamGateAbove {
        param: "reach",
        when: "delay",
        above: 0.0,
    },
];

/// O atraso é TEMPO; o ponto e o alcance são lugares do mundo.
pub(crate) static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "delay",
        unit: ParamUnit::Seconds,
    },
    ParamUnitDecl {
        param: "origin_x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "origin_y",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "reach",
        unit: ParamUnit::Length,
    },
];
