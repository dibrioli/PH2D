//! **O que o PAINEL mostra deste nó** — hints, seções, portões e o teto digitável.
//!
//! ⚠️ **O corte foi FORÇADO pelo HR-18 e a costura é por RESPONSABILIDADE**, a mesma do
//! `mask.rs` do `field.box`: o `lib.rs` responde *o que o nó calcula* (manifesto, `contour`,
//! `eval`, kernel, LUTs) e isto responde *o que o artista vê e pode escrever*. Uma cresce
//! quando a lei muda; a outra, quando a UI muda — e elas quase nunca mudam no mesmo dia.
//!
//! A única coisa que atravessa a costura é o [`CURVE_KEY`](super::CURVE_KEY): o nome do param
//! de TEXTO em que a curva viaja, que o `lib.rs` lê e esta tabela nomeia.

use ph2d_node_registry::{ParamGroup, ParamHardMax, ParamUiHint, ParamWidget};

use super::CURVE_KEY;

/// As SEÇÕES deste nó (doc 88 B3). Treze controles numa lista plana são uma parede; em dois
/// grupos, com os essenciais soltos na frente, eles viram três perguntas.
///
/// ⚠️ **A TRANSFERÊNCIA inteira fica de FORA de propósito** — `contour` (qual curva),
/// `CURVE_KEY` (a curva), `curvature` e `steps` (as formas das outras contours) e
/// `inner_offset`. Param sem grupo pinta antes de toda seção, e é ali que os essenciais devem
/// estar: a transferência é a razão de existir deste nó, e pô-la numa seção seria escondê-la
/// atrás de um clique.
///
/// ⚠️ E isto foi corrigido pelo gate `selected_field_remap_yields_an_interactive_curve_row`,
/// que já afirmava que a curva é a PRIMEIRA row: a primeira versão desta tabela punha o
/// `CURVE_KEY` num grupo "Shape" — contradizendo, duas linhas acima, o comentário que dizia
/// para não a sepultar.
pub(crate) static PARAM_GROUPS: &[ParamGroup] = &[
    // A FAIXA de saída: entre que números o resultado vive.
    ParamGroup::new("min", "Range"),
    ParamGroup::new("max", "Range"),
    ParamGroup::new("multiplier", "Range"),
    ParamGroup::new("clamp", "Range"),
    ParamGroup::new("invert", "Range"),
    // E quanto do resultado chega ao mundo.
    ParamGroup::new("strength", "Output"),
    ParamGroup::new("probability", "Output"),
    ParamGroup::new("seed", "Output"),
];

/// **UM NÚMERO POR CONTORNO, E SÓ O DO CONTORNO ESCOLHIDO APARECE.**
///
/// ⚠️ **Isto nasceu de um smoke** (Enio, 2026-08-21: *"Curve offset e outros parâmetros
/// não têm efeito"*). Ele estava com o contorno em `Curve` e tinha ao lado, vivos no
/// painel, o `curvature` (que é do Quadratic) e o `steps` (que é do Step/Quantize) —
/// dois knobs que ali não fazem nada **por desenho**. Um controle vivo que não muda
/// nada é indistinguível de um bug, e foi lido como um.
///
/// ⛔ **O `curve` (o editor de curva) NÃO é gateado, de propósito.** O gate
/// `selected_field_remap_yields_an_interactive_curve_row` afirma que ele é a PRIMEIRA
/// linha e é interactivo; gateá-lo por `contour == 4` esconderia-o no modo default
/// (Quadratic) e reprovaria aquele gate. Se um dia a decisão for escondê-lo, é aquele
/// gate que se reconcilia primeiro — não este.
pub(crate) static PARAM_GATES: &[ph2d_node_registry::ParamGate] = &[
    ph2d_node_registry::ParamGate {
        param: "curvature",
        when: "contour",
        values: &[1],
    },
    ph2d_node_registry::ParamGate {
        param: "steps",
        when: "contour",
        values: &[2, 3],
    },
    ph2d_node_registry::ParamGate {
        param: "curve_offset",
        when: "contour",
        values: &[4],
    },
];

/// **O teto DIGITÁVEL dos degraus, MEDIDO.**
///
/// ⚠️ **A referência pede quatro ordens de grandeza mais do que o arrasto entrega** — o C4D
/// documenta *"Steps | [1..2³¹]"* e aqui o digitado parava em **32**, porque sem entrada nesta
/// tabela o `ui.rs:206` faz o campo digitar até ao fim do ARRASTO.
///
/// **De que recurso é o teto: da PRECISÃO** (`CLAUDE.md` §0.0). A lei é
/// `k = floor(t·n).min(n−1)` sobre `k/(n−1)`, e ela não satura: mais degraus é sempre mais
/// escada. O que acaba é o `f32` — acima de `2²⁴` somar o `step` do slider (1) **não move o
/// número**, e ali dois `steps` autoráveis vizinhos já são a mesma escada.
///
/// ⚠️ **A entrada `t` bate no mesmo muro pelo outro lado, e é isso que confirma a leitura:**
/// `t` é um `f32` em `[0,1]`, com ~`2²⁴` valores distintos, então nem o degrau nem quem o pisa
/// resolvem mais que isso. *Dois recursos independentes a dar o mesmo número é o sinal de que
/// o número é do problema, e não do instrumento.*
///
/// O valor é `2²⁴ − 1`, re-derivado a cada corrida pelo gate
/// `every_precision_bound_param_types_to_the_measured_ceiling` (`ph2d-node-registry-init`).
pub(crate) static PARAM_HARD_MAX: &[ParamHardMax] = &[ParamHardMax {
    param: "steps",
    max: 16_777_215.0,
}];

/// Param UI hints (M1.P1): the C4D Remapping controls. `contour` is the transfer
/// selector; `curve` is the A1 curve editor (a text param, live when contour = Curve);
/// `curvature` bends the Quadratic; `steps` counts the Step/Quantize levels; `strength`
/// is the input→remapped blend (0 = passthrough).
pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "inner_offset",
        label: "Inner Offset",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "contour",
        label: "Contour",
        min: 0.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["None", "Quadratic", "Step", "Quantize", "Curve"],
        },
    },
    ParamUiHint {
        // The Curve contour's shape — a TEXT param (`CURVE_KEY`), not a `ParamSpec`
        // (the `motion.expression` Text precedent). The panel draws the curve editor;
        // inert unless `contour = Curve`. `min/max/step` are inert for a curve widget.
        param: CURVE_KEY,
        label: "Curve",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Curve,
    },
    ParamUiHint {
        param: "curvature",
        label: "Curvature",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "steps",
        label: "Steps",
        min: 2.0,
        max: 32.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "min",
        label: "Min",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "max",
        label: "Max",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "multiplier",
        label: "Multiplier",
        min: 0.0,
        max: 4.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "clamp",
        label: "Clamp",
        min: 0.0,
        #[expect(clippy::cast_precision_loss, reason = "quatro rotulos")]
        max: (super::CLAMP_LABELS.len() - 1) as f32,
        step: 1.0,
        // Era um Toggle; os dois estados dele são os índices `0` e `1`, que a
        // escada preserva — ver `CLAMP_LABELS`.
        widget: ParamWidget::Enum {
            labels: super::CLAMP_LABELS,
        },
    },
    ParamUiHint {
        param: "invert",
        label: "Invert",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    // ⚠️ **A faixa é UMA VOLTA, e ela é a unidade natural do knob**: o deslocamento
    // dá a volta em 1, então `−1..1` cobre o percurso inteiro nos dois sentidos e
    // nada além disso é alcançável (`1.5` desenha o mesmo que `0.5`). Um slider mais
    // largo seria curso morto.
    ParamUiHint {
        param: "curve_offset",
        label: "Curve Offset",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "strength",
        label: "Strength",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "probability",
        label: "Probability",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 9999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
];
