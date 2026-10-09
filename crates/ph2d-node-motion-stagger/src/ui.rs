//! Os cartões deste nó — rótulos, gates e unidades (cortados do `lib.rs` pelo tecto de LOC,
//! ciclo 17). A lei do nó mora no `lib.rs`; aqui mora o que o painel mostra dela.

use super::*;

use ph2d_node_registry::{ParamGate, ParamUiHint, ParamWidget};

/// **A direção do easing não existe numa reta** (doc 90 §2, caça aos knobs mortos).
///
/// ⚠️ `ease_curve` nasce em `Linear`, e o `Linear` devolve `t` **antes** de olhar para a
/// direção — In, Out e In-Out dão a mesma saída, ao bit. Era o pior dos dezanove pela posição:
/// um Stagger recém-largado, o artista gira o seletor de direção à procura do que ele promete,
/// e nada se move. *É o primeiro gesto que qualquer pessoa faz neste nó.*
///
/// `0 = Linear` · `1..7 = Quad · Cubic · Quart · Quint · Circ · Back · Bounce`.
pub(crate) static PARAM_GATES: &[ParamGate] = &[
    // ⛔ **A FORMA só aparece na ease que a LÊ** — o irmão exacto do `motion.oscillator`,
    // achado pelo mesmo censo e no mesmo dia. Fora da `Custom` o `ease` devolve a família
    // enumerada sem olhar para a curva, e o editor era oferecido em todas as nove.
    ParamGate {
        param: CURVE_KEY,
        when: "ease_curve",
        values: &[ease::EASE_CUSTOM],
    },
    ParamGate {
        param: "ease_dir",
        when: "ease_curve",
        values: ph2d_ease::USES_DIRECTION,
    },
    // ⭐ CICLO 17 — cada número de forma só aparece na família que o LÊ.
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
    // A grelha só no `Grid`; a célula só no `Grid` E com `From = Cell` (os gates somam-se).
    ParamGate {
        param: "grid_cols",
        when: "order",
        values: &[order::ORDER_GRID],
    },
    ParamGate {
        param: "grid_from",
        when: "order",
        values: &[order::ORDER_GRID],
    },
    ParamGate {
        param: "grid_cell",
        when: "order",
        values: &[order::ORDER_GRID],
    },
    ParamGate {
        param: "grid_cell",
        when: "grid_from",
        values: &[order::FROM_CELL],
    },
    ParamGate {
        param: "axis",
        when: "order",
        values: &[order::ORDER_GRID, order::ORDER_FROM_POINT],
    },
    ParamGate {
        param: "origin_x",
        when: "order",
        values: &[order::ORDER_FROM_POINT],
    },
    ParamGate {
        param: "origin_y",
        when: "order",
        values: &[order::ORDER_FROM_POINT],
    },
    ParamGate {
        param: "reach",
        when: "order",
        values: &[order::ORDER_FROM_POINT],
    },
    // ⚠️ **A semente só é lida no `Random`** (ciclo 2, W2) — um controlo que o cook não abre
    // não é pintado.
    ParamGate {
        param: "seed",
        when: "order",
        values: &[order::ORDER_RANDOM],
    },
];

/// Param UI hints (M1.P1). `channel` / `ease_curve` / `ease_dir` are **named**
/// selectors (segmented buttons), `reverse` a checkbox — never number sliders.
/// Easing is a curve family × direction (the Penner set minus the transcendental
/// ones); the enum option index IS the param value.
pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "channel",
        label: "node.motion.stagger.param.channel",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.motion.stagger.param.channel.0",
                "node.motion.stagger.param.channel.1",
                "node.motion.stagger.param.channel.2",
                "node.motion.stagger.param.channel.3",
            ],
        },
    },
    ParamUiHint {
        param: "min",
        label: "node.motion.stagger.param.min",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "max",
        label: "node.motion.stagger.param.max",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "ease_curve",
        label: "node.motion.stagger.param.ease_curve",
        min: 0.0,
        // ⚠️ **Apendada**: a `Custom` é o índice 8, e as oito de sempre ficam onde
        // estavam — um documento autorado guarda o NÚMERO, não o nome.
        max: 14.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.motion.stagger.param.ease_curve.0",
                "node.motion.stagger.param.ease_curve.1",
                "node.motion.stagger.param.ease_curve.2",
                "node.motion.stagger.param.ease_curve.3",
                "node.motion.stagger.param.ease_curve.4",
                "node.motion.stagger.param.ease_curve.5",
                "node.motion.stagger.param.ease_curve.6",
                "node.motion.stagger.param.ease_curve.7",
                "node.motion.stagger.param.ease_curve.8",
                "node.motion.stagger.param.ease_curve.9",
                "node.motion.stagger.param.ease_curve.10",
                "node.motion.stagger.param.ease_curve.11",
                "node.motion.stagger.param.ease_curve.12",
                "node.motion.stagger.param.ease_curve.13",
                "node.motion.stagger.param.ease_curve.14",
            ],
        },
    },
    // A FORMA da ease `Custom` — um TEXT param (`CURVE_KEY`), não um `ParamSpec`.
    // Não-setada = `t`, ou seja o `Linear`. Ver `ease::EASE_CUSTOM`.
    ParamUiHint {
        param: CURVE_KEY,
        label: "node.motion.stagger.param.curve",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Curve,
    },
    ParamUiHint {
        param: "ease_dir",
        label: "node.motion.stagger.param.ease_dir",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.motion.stagger.param.ease_dir.0",
                "node.motion.stagger.param.ease_dir.1",
                "node.motion.stagger.param.ease_dir.2",
            ],
        },
    },
    // ⚠️ **A faixa é um ciclo INTEIRO** (`0..1`): a rampa fecha em si mesma, e um
    // deslize de `1` é o mesmo que nenhum. Um teto maior seria uma volta a mais
    // que o artista não distingue.
    ParamUiHint {
        param: "offset",
        label: "node.motion.stagger.param.offset",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "reverse",
        label: "node.motion.stagger.param.reverse",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    // ⭐⭐ **TRÊS ordens, e não sete** — ver o doc de [`order`]: `Reverse`/`From Edges` são o
    // toggle que já existe, e `By X`/`By Y` pedem uma ordenação (o `motion.sort` a montante).
    ParamUiHint {
        param: "order",
        label: "node.motion.stagger.param.order",
        min: 0.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.motion.stagger.param.order.0",
                "node.motion.stagger.param.order.1",
                "node.motion.stagger.param.order.2",
                "node.motion.stagger.param.order.3",
                "node.motion.stagger.param.order.4",
            ],
        },
    },
    ParamUiHint {
        param: "seed",
        label: "node.motion.stagger.param.seed",
        min: 0.0,
        max: 999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
    // ⭐ CICLO 17 — a forma das famílias e a ordem no espaço.
    ParamUiHint {
        param: "overshoot",
        label: "node.motion.stagger.param.overshoot",
        min: 0.0,
        max: 5.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "period",
        label: "node.motion.stagger.param.period",
        min: 0.05,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "steps",
        label: "node.motion.stagger.param.steps",
        min: 1.0,
        max: 64.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "damping",
        label: "node.motion.stagger.param.damping",
        min: 0.05,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "bez_x1",
        label: "node.motion.stagger.param.bez_x1",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "bez_y1",
        label: "node.motion.stagger.param.bez_y1",
        min: -1.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "bez_x2",
        label: "node.motion.stagger.param.bez_x2",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "bez_y2",
        label: "node.motion.stagger.param.bez_y2",
        min: -1.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "grid_cols",
        label: "node.motion.stagger.param.grid_cols",
        min: 1.0,
        max: 1000.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "grid_from",
        label: "node.motion.stagger.param.grid_from",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.motion.stagger.param.grid_from.0",
                "node.motion.stagger.param.grid_from.1",
                "node.motion.stagger.param.grid_from.2",
                "node.motion.stagger.param.grid_from.3",
            ],
        },
    },
    ParamUiHint {
        param: "grid_cell",
        label: "node.motion.stagger.param.grid_cell",
        min: 0.0,
        max: 10000.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "axis",
        label: "node.motion.stagger.param.axis",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.motion.stagger.param.axis.0",
                "node.motion.stagger.param.axis.1",
                "node.motion.stagger.param.axis.2",
            ],
        },
    },
    ParamUiHint {
        param: "origin_x",
        label: "node.motion.stagger.param.origin_x",
        min: -50.0,
        max: 50.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "origin_y",
        label: "node.motion.stagger.param.origin_y",
        min: -50.0,
        max: 50.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "reach",
        label: "node.motion.stagger.param.reach",
        min: 0.0,
        max: 50.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
];

/// **What each of this node's numbers IS** (doc 88, Wave A). This node's magnitude
/// is `FromChannel`: it means metres on Position, DEGREES on Rotation and a bare
/// scale factor on Size, so the panel resolves the unit per-channel. Declaring a
/// fixed `Length` here would scale degrees by `pixels_per_meter` — the failure
/// that turns a `±90` preset into a `±9000`.
pub(crate) static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "min",
        unit: ParamUnit::FromChannel,
    },
    ParamUnitDecl {
        param: "max",
        unit: ParamUnit::FromChannel,
    },
    // ⚠️ O ponto e o alcance do `From Point` são metros do mundo, mas NÃO se declaram `Length`:
    // num nó cujas magnitudes dependem do `channel`, o painel escalaria-os por `pixels_per_meter`
    // como se fossem a rampa (o censo `no_param_of_a_channel_driven_node_is_declared_a_fixed_length`).
];

/// **A faixa que estas magnitudes querem quando o canal é ANGULAR** — graus, não
/// unidades de mundo. Uma volta para cada lado, discada em graus inteiros.
///
/// ⚠️ Ela mora AQUI e não numa tabela do shell porque a tabela apodreceu: medida,
/// ela cobria três dos seis nós que precisavam dela, e cada um dos três ausentes
/// esperava o próprio report do artista.
pub(crate) const TURN: f32 = 360.0;
pub(crate) static PARAM_CHANNEL_RANGE: &[ParamChannelRange] = &[
    ParamChannelRange {
        param: "min",
        min: -TURN,
        max: TURN,
        step: 1.0,
    },
    ParamChannelRange {
        param: "max",
        min: -TURN,
        max: TURN,
        step: 1.0,
    },
];
