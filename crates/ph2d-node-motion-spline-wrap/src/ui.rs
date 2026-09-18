//! **A SUPERFÍCIE DE PAINEL do `motion.spline_wrap`** — os hints, as unidades, as
//! seções, e o gate de TEXTO que esconde as oito coordenadas quando há forma
//! desenhada.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o corte
//! é por RESPONSABILIDADE: o `lib.rs` responde *como o enrolamento funciona* e este
//! responde *como ele se apresenta*. É o mesmo corte do irmão `motion.trail`.

use ph2d_node_registry::{
    ParamGateText, ParamGroup, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget,
    RequiredTextParam,
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
/// ⭐⭐⭐ **A SECÇÃO `Curve` NASCE FECHADA** (ordem do dono, 2026-09-08).
///
/// *«Em vez de nascer com uma curva default com pontos no painel, melhor nascer inerte com um
/// botão para selecionar um path»* — e é a segunda metade do pedido de 12/08 (*«pontos e alças em
/// sliders num painel. Absurdo!»*), de que a row `Shape` foi a primeira.
///
/// ⚠️ **Dobrar e não APAGAR, e a diferença é medida.** Apagar os oito custaria as **quatro** cenas
/// da conferência que os escrevem à mão (`demos`, `demos_deform`, `demos_slice`, `demos_campo`) e
/// tiraria uma capacidade que ninguém pediu para tirar; dobrar tira-os do estado de NASCIMENTO,
/// que é exactamente o que a ordem diz. Eles ficam **a um clique**, com o cabeçalho a dizer
/// quantas rows esconde — o oposto de inalcançável, e é por isso que o censo
/// `every_param_the_card_hides_has_a_declared_reason` aceita esta explicação e recusaria um
/// esconder mudo.
///
/// ⚠️ **E o `folded` é o NASCIMENTO, nunca a memória:** o store lembra o que o artista escolheu, e
/// o painel semeia isto uma vez. Quem abrir a secção não a vê fechar-se no quadro seguinte.
pub(super) static PARAM_GROUPS: &[ParamGroup] = &[
    ParamGroup::new("p0x", "node.group.curve").folded(),
    ParamGroup::new("p0y", "node.group.curve").folded(),
    ParamGroup::new("p1x", "node.group.curve").folded(),
    ParamGroup::new("p1y", "node.group.curve").folded(),
    ParamGroup::new("p2x", "node.group.curve").folded(),
    ParamGroup::new("p2y", "node.group.curve").folded(),
    ParamGroup::new("p3x", "node.group.curve").folded(),
    ParamGroup::new("p3y", "node.group.curve").folded(),
];

/// **O caminho só é EXIGIDO quando não há curva autorada** — ver
/// [`RequiredTextParam`].
///
/// ⚠️ **A primeira redacção não tinha o predicado, e o portão de fecho apanhou-a:** as quatro
/// cenas da conferência escrevem os oito números à mão, e o ⚠ acusava-as de estarem inertes
/// enquanto elas embrulhavam. *Um aviso sobre um nó que funciona ensina o artista a ignorar o
/// aviso.*
///
/// ⚠️ **A pergunta é a MESMA que o `eval` faz** — *há curva?* — e não um proxy: os oito no zero
/// dão uma cúbica de comprimento zero, que é exactamente a condição do ramo inerte.
pub(super) static REQUIRED_TEXT: &[RequiredTextParam] = &[RequiredTextParam {
    param: PATH_PARAM,
    only_when: Some(|p| {
        ["p0x", "p0y", "p1x", "p1y", "p2x", "p2y", "p3x", "p3y"]
            .iter()
            .all(|k| p(k) == 0.0)
    }),
}];

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
        label: "node.motion.spline_wrap.param.path.source",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Source,
    },
    // ⭐⭐⭐ **O BOTÃO** (ordem do dono, 2026-09-08: *«um botão para selecionar um path no canvas
    // ou na hierarchy»*) — e ele escreve NO MESMO param que a row acima.
    //
    // ⚠️ **Duas rows, um param, e isso não são duas portas.** Uma porta é onde o valor entra, e é
    // uma só (`Graph::set_text_param`); estas são dois GESTOS para o mesmo param, como arrastar
    // um slider e digitar o número. Cada uma serve um momento: a lista serve quem sabe o nome, o
    // botão serve quem está a OLHAR para a forma e não sabe como ela se chama — que é
    // exactamente o caso de quem acabou de a desenhar.
    //
    // ⚠️ **Ela vem LOGO A SEGUIR à `Shape`, de propósito:** as duas respondem à mesma pergunta, e
    // separá-las faria o artista procurar a segunda depois de a primeira o ter desiludido.
    ParamUiHint {
        param: PATH_PARAM,
        label: "node.motion.spline_wrap.param.path.pick_selection",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::PickSelection,
    },
    ParamUiHint {
        param: "follow_rotation",
        label: "node.motion.spline_wrap.param.follow_rotation",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    ParamUiHint {
        param: "height_scale",
        label: "node.motion.spline_wrap.param.height_scale",
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
        label: "node.motion.spline_wrap.param.direction",
        min: -180.0,
        max: 180.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    // ⚠️ Um `Enum` e não um Toggle: os dois nomes são o vocabulário da referência (C4D), e
    // *"Keep Length"* diz o que faz enquanto *"não esticar"* pedia para se adivinhar o resto.
    ParamUiHint {
        param: "mode",
        label: "node.motion.spline_wrap.param.mode",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.motion.spline_wrap.param.mode.0",
                "node.motion.spline_wrap.param.mode.1",
            ],
        },
    },
    ParamUiHint {
        param: "offset",
        label: "node.motion.spline_wrap.param.offset",
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
        label: "node.motion.spline_wrap.param.from",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "to",
        label: "node.motion.spline_wrap.param.to",
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
        label: "node.motion.spline_wrap.param.size_start",
        min: 0.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: super::taper::SIZE_TAPER.1,
        label: "node.motion.spline_wrap.param.size_end",
        min: 0.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: super::taper::SIZE_TAPER.2,
        label: "node.motion.spline_wrap.param.size_profile",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.motion.spline_wrap.param.size_profile.0",
                "node.motion.spline_wrap.param.size_profile.1",
                "node.motion.spline_wrap.param.size_profile.2",
                "node.motion.spline_wrap.param.size_profile.3",
            ],
        },
    },
    pt("p0x", "node.motion.spline_wrap.param.p0x"),
    pt("p0y", "node.motion.spline_wrap.param.p0y"),
    pt("p1x", "node.motion.spline_wrap.param.p1x"),
    pt("p1y", "node.motion.spline_wrap.param.p1y"),
    pt("p2x", "node.motion.spline_wrap.param.p2x"),
    pt("p2y", "node.motion.spline_wrap.param.p2y"),
    pt("p3x", "node.motion.spline_wrap.param.p3x"),
    pt("p3y", "node.motion.spline_wrap.param.p3y"),
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
