//! The node's **param UI metadata** — labels, ranges, widgets, units. Split from
//! `lib.rs` at the HR-18 LOC cap, on the seam the emitter already uses
//! (`ph2d-node-motion-emitter/src/params_ui.rs`): none of this is behaviour, so the
//! node computes exactly the same result whatever a slider looks like.

use ph2d_node_registry::{
    ParamGate, ParamGroup, ParamHardMax, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget,
};

/// O teto que a MÁQUINA (ou o bom senso) impõe, alcançável por DIGITAÇÃO — o slider fica
/// onde a MÃO trabalha (soft/hard do Blender; doc 88 §11). O curso de antes é este número:
/// nada ficou inalcançável, só deixou de ser o que o dedo percorre.
pub(crate) static PARAM_HARD_MAX: &[ParamHardMax] = &[ParamHardMax {
    param: "phase_stagger",
    max: 2.0,
}];

/// Param UI hints (M1.P1). `channel` / `wave` are **named** selectors (segmented
/// buttons) — never number sliders. The enum option index IS the param value
/// (channel 0..3; wave 0..4 = Sine/Tri/Square/Saw/Spike — "Sine" is the
/// user-facing name for the transcendental-free parabolic approximation,
/// "Spike" a narrow unipolar pulse at the cycle start).
pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "channel",
        label: "Channel",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["X", "Y", "Rotation", "Size"],
        },
    },
    ParamUiHint {
        param: "wave",
        label: "Wave",
        min: 0.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Sine", "Tri", "Square", "Saw", "Spike"],
        },
    },
    ParamUiHint {
        param: "amplitude",
        label: "Amplitude",
        min: 0.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "frequency",
        label: "Frequency",
        min: 0.0,
        max: 8.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "phase_stagger",
        label: "Stagger",
        min: 0.0,
        max: 1.0,
        step: 0.02,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset",
        label: "Offset",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "phase",
        label: "Phase",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "time_mode",
        label: "Time Mode",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Seconds", "BPM"],
        },
    },
    // A faixa de um BPM é a de uma música, não a de um Hz: 20 é um *largo* muito
    // lento e 300 passa o topo de qualquer gênero. Uma faixa 0..8 aqui (a do
    // `frequency`) faria o slider inteiro caber entre 0 e 8 batidas por minuto.
    ParamUiHint {
        param: "bpm",
        label: "BPM",
        min: 20.0,
        max: 300.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
];

/// **Só a régua escolhida aparece.**
///
/// `frequency` e `bpm` são o MESMO número em duas unidades, então mostrar os dois seria
/// mostrar um controle que o cook não lê — e pior que o botão morto: dois números na tela
/// que discordam entre si sobre a mesma grandeza, sem nada dizendo qual manda.
pub(crate) static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: "frequency",
        when: "time_mode",
        values: &[0],
    },
    ParamGate {
        param: "bpm",
        when: "time_mode",
        values: &[1],
    },
];

/// As SEÇÕES deste nó (doc 88 B3). Dez controles, e os quatro do TEMPO só falam entre si.
///
/// ⚠️ Ficam soltos `channel`, `wave`, `amplitude` e `offset` — o que a onda É e quanto ela
/// vale. Um oscilador que abre com a régua de tempo na cara e a amplitude escondida seria a
/// hierarquia ao contrário.
pub(crate) static PARAM_GROUPS: &[ParamGroup] = &[
    // Que relógio a onda anda.
    ParamGroup::new("time_mode", "Timing"),
    ParamGroup::new("frequency", "Timing"),
    ParamGroup::new("bpm", "Timing"),
    ParamGroup::new("phase", "Timing"),
    ParamGroup::new("phase_stagger", "Timing"),
];

/// **What each of this node's numbers IS** (doc 88, Wave A). This node's magnitude
/// is `FromChannel`: it means metres on Position, DEGREES on Rotation and a bare
/// scale factor on Size, so the panel resolves the unit per-channel. Declaring a
/// fixed `Length` here would scale degrees by `pixels_per_meter` — the failure
/// that turns a `±90` preset into a `±9000`.
pub(crate) static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "amplitude",
        unit: ParamUnit::FromChannel,
    },
    ParamUnitDecl {
        param: "offset",
        unit: ParamUnit::FromChannel,
    },
];
