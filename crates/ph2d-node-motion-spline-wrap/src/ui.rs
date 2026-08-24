//! **A SUPERFÍCIE DE PAINEL do `motion.spline_wrap`** — os hints, as unidades, as
//! seções, e o gate de TEXTO que esconde as oito coordenadas quando há forma
//! desenhada.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o corte
//! é por RESPONSABILIDADE: o `lib.rs` responde *como o enrolamento funciona* e este
//! responde *como ele se apresenta*. É o mesmo corte do irmão `motion.trail`.

use ph2d_node_registry::{
    ParamGateText, ParamGroup, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget,
};

use super::PATH_PARAM;

/// As oito coordenadas do polígono de controle só aparecem **sem** forma escolhida.
pub(super) static PARAM_GATES_TEXT: &[ParamGateText] = &[
    ParamGateText {
        param: "p0x",
        when_text: PATH_PARAM,
        when_present: false,
    },
    ParamGateText {
        param: "p0y",
        when_text: PATH_PARAM,
        when_present: false,
    },
    ParamGateText {
        param: "p1x",
        when_text: PATH_PARAM,
        when_present: false,
    },
    ParamGateText {
        param: "p1y",
        when_text: PATH_PARAM,
        when_present: false,
    },
    ParamGateText {
        param: "p2x",
        when_text: PATH_PARAM,
        when_present: false,
    },
    ParamGateText {
        param: "p2y",
        when_text: PATH_PARAM,
        when_present: false,
    },
    ParamGateText {
        param: "p3x",
        when_text: PATH_PARAM,
        when_present: false,
    },
    ParamGateText {
        param: "p3y",
        when_text: PATH_PARAM,
        when_present: false,
    },
];

/// As SEÇÕES deste nó (doc 88 B3). As oito coordenadas são UMA coisa — o polígono de controle
/// de uma cúbica —, e listá-las ao lado dos dois controles reais faz um nó de dois botões
/// parecer um nó de dez.
pub(super) static PARAM_GROUPS: &[ParamGroup] = &[
    ParamGroup::new("p0x", "Curve"),
    ParamGroup::new("p0y", "Curve"),
    ParamGroup::new("p1x", "Curve"),
    ParamGroup::new("p1y", "Curve"),
    ParamGroup::new("p2x", "Curve"),
    ParamGroup::new("p2y", "Curve"),
    ParamGroup::new("p3x", "Curve"),
    ParamGroup::new("p3y", "Curve"),
];

pub(super) static PARAM_HINTS: &[ParamUiHint] = &[
    // ⚠️ **A PRIMEIRA row, e é uma decisão de produto.** O Enio, no smoke de
    // 2026-08-12: *"esse é o tipo de nó que simplesmente não faz sentido num app
    // de última geração. Pontos e alças em sliders num painel. Absurdo! … um
    // botão no painel do nó para o usuário desenhar sua curva no canvas. Já
    // temos ferramentas maravilhosas para desenhos como no módulo vector."*
    //
    // O app já tinha escolhido esta resposta — para o IRMÃO. O `motion.path` diz,
    // no próprio doc-comment: *"a curva é uma forma desenhada de verdade em vez
    // de quatro params de ponto de controle"*. O que faltava era o deformador,
    // deixado para trás nos oito números; a rota, o widget e o gesto são os
    // mesmos, e um artista que aprendeu a escolher a forma num nó não a
    // re-aprende no outro.
    ParamUiHint {
        param: PATH_PARAM,
        label: "Shape",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Source,
    },
    ParamUiHint {
        param: "follow_rotation",
        label: "Follow Curve",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    ParamUiHint {
        param: "height_scale",
        label: "Height",
        min: 0.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **O MESMO rótulo, widget e curso que o `direction` do `motion.bend`** — os dois
    // respondem *"em que eixo isto corre?"*, e a volta INTEIRA é distinta (a `−90` o layout
    // deita-se para o outro lado, que não é o mesmo que `+90`).
    ParamUiHint {
        param: super::taper::DIRECTION,
        label: "Axis",
        min: -180.0,
        max: 180.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    // ⚠️ Um `Enum` e não um Toggle: os dois nomes são o vocabulário da referência (C4D), e
    // *"Keep Length"* diz o que faz enquanto *"não esticar"* pedia para se adivinhar o resto.
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Fit Spline", "Keep Length"],
        },
    },
    ParamUiHint {
        param: "offset",
        label: "Offset",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // ⚠️ A faixa `0..1` do hint É a faixa que o motor honra (fora dela o `s_at`
    // satura), então a caixa de texto não precisa de `ParamHardMin`/`Max` — os
    // dois só sabem ALARGAR a caixa para fora do slider, e alargá-la aqui seria
    // aceitar um número que o `clamp` desmente em silêncio.
    ParamUiHint {
        param: "from",
        label: "From",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "to",
        label: "To",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **O curso vai a `4` e começa em `0`** — o mesmo do `height_scale`, e pela mesma razão:
    // um afunilamento é um MULTIPLICADOR, `0` é *"some na ponta"* (o uso canónico de uma cauda)
    // e acima de `1` ele engrossa, que é a outra metade do que a referência desenha.
    ParamUiHint {
        param: super::taper::SIZE_TAPER.0,
        label: "Size Start",
        min: 0.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: super::taper::SIZE_TAPER.1,
        label: "Size End",
        min: 0.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: super::taper::SIZE_TAPER.2,
        label: "Size Profile",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Linear", "Quad", "Smooth", "Smoother"],
        },
    },
    pt("p0x", "P0 X"),
    pt("p0y", "P0 Y"),
    pt("p1x", "P1 X"),
    pt("p1y", "P1 Y"),
    pt("p2x", "P2 X"),
    pt("p2y", "P2 Y"),
    pt("p3x", "P3 X"),
    pt("p3y", "P3 Y"),
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
        param: "p0x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "p0y",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "p1x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "p1y",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "p2x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "p2y",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "p3x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "p3y",
        unit: ParamUnit::Length,
    },
];

const fn pt(param: &'static str, label: &'static str) -> ParamUiHint {
    ParamUiHint {
        param,
        label,
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    }
}
