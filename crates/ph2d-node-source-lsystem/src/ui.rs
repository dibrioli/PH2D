//! **A FACE DO NÓ — hints, gates de visibilidade, seções e unidades**
//!
//! Separado do [`super`] pelo teto de LOC (HR-18, 700 na workspace), no corte que a
//! pergunta desenha: lá fica *a lei*, aqui *o que o painel mostra dela*.

use super::*;

/// O tecto digitável de `Generations`.
///
/// ⚠️ **É o tecto da CAIXA, não o da cadeia** — o que de facto pára a derivação é o
/// [`MAX_MODULES`], porque a taxa de expansão é propriedade da REGRA: `F -> FF` duplica e
/// `F -> F[+F]F[-F]F` quintuplica, então 20 gerações de uma são triviais e da outra são
/// impossíveis. Este número existe só para a caixa não aceitar um `1e9` que faria o laço
/// externo girar mil milhões de vezes a não fazer nada depois de saturar.
pub(crate) const MAX_GENERATIONS: f32 = 32.0;

/// O tecto DIGITÁVEL, acima do que o slider arrasta — a mesma escada que o `sim.spawn` e o
/// `motion.emitter` usam: o arrasto fica na faixa útil, e quem sabe o que quer digita.
pub(crate) static PARAM_HARD_MAX: &[ph2d_node_registry::ParamHardMax] =
    &[ph2d_node_registry::ParamHardMax {
        param: param::GENERATIONS,
        max: MAX_GENERATIONS,
    }];

pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    // ⭐ **A GEOMETRIA vem primeiro de todas** — ela decide o que a planta É na tela: uma
    // pilha de ossos (`Segments`, o que este nó sempre emitiu, e o que os cinco `rig.*`
    // consomem) ou uma fita contínua por ramo (`Branches`, o que as quatro referências fazem).
    // ⚠️ Acima do `Mode` de propósito: aquele escolhe de onde vem a GRAMÁTICA, este escolhe o
    // que se vê — e a segunda pergunta é a que o artista faz primeiro.
    ParamUiHint {
        param: param::GEOMETRY,
        label: "Geometry",
        min: 0.0,
        max: (GEOMETRY_LABELS.len() - 1) as f32,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: GEOMETRY_LABELS,
        },
    },
    // ⭐ **O AFINAMENTO DA PONTA** — report do Enio (2026-08-30): *"as pontas não têm opção de
    // afinar"*. ⚠️ Só existe no modo `Branches` (ver [`PARAM_GATES`]): em `Segments` cada osso
    // é um retângulo próprio e não há ponta que afinar — um knob inerte ali ensinaria a
    // desconfiar dos vivos.
    ParamUiHint {
        param: param::TIP_TAPER,
        label: "Tip Taper",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **O MODO vem antes de tudo** — ele decide qual metade do painel existe.
    ParamUiHint {
        param: param::MODE,
        label: "Mode",
        min: 0.0,
        max: (MODE_LABELS.len() - 1) as f32,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: MODE_LABELS,
        },
    },
    // Os quatro números de FORMA — o modo guiado inteiro. Ver [`shape`].
    ParamUiHint {
        param: param::BRANCHES,
        label: "Branches",
        min: 1.0,
        max: shape::MAX_BRANCHES,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: param::SEGMENTS,
        label: "Trunk Segments",
        min: 1.0,
        max: shape::MAX_SEGMENTS,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: param::VARIATION,
        label: "Variation",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::BEND,
        label: "Bend",
        min: -30.0,
        max: 30.0,
        step: 0.5,
        widget: ParamWidget::Angle,
    },
    // ⚠️ **Depois os dois textos, e é o que o nó É por dentro**: um L-System é a gramática.
    // Os números são a interpretação dela.
    ParamUiHint {
        param: AXIOM_PARAM,
        label: "Axiom",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Text,
    },
    ParamUiHint {
        param: RULES_PARAM,
        label: "Rules",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Text,
    },
    // ⚠️ **`Slider`, não `IntSlider`** — a fracção é a feature: com o número a subir
    // continuamente a planta CRESCE, e com ele em degraus ela salta.
    ParamUiHint {
        param: param::GENERATIONS,
        label: "Generations",
        min: 0.0,
        max: 12.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::ANGLE,
        label: "Angle",
        min: 0.0,
        max: 180.0,
        step: 0.5,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: param::STEP,
        label: "Step",
        min: 0.01,
        max: 4.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::WIDTH,
        label: "Width",
        min: 0.01,
        max: 8.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::WIDTH_SCALE,
        label: "Width Scale",
        min: 0.1,
        max: 1.5,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::LENGTH_SCALE,
        label: "Length Scale",
        min: 0.1,
        max: 1.5,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::ROOT_ANGLE,
        label: "Root Angle",
        min: -180.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    // ⚠️ **POSITIVO puxa PARA a direcção; negativo empurra para longe dela.** A direcção já
    // tem um param próprio, então o SINAL aqui é a força e não um segundo eixo — e uma cena
    // desta linha nasceu com ele trocado, a fazer a planta com «gravidade» sair mais direita
    // do que a sem.
    ParamUiHint {
        param: param::TROPISM,
        label: "Tropism",
        min: -45.0,
        max: 45.0,
        step: 0.5,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: param::TROPISM_ANGLE,
        label: "Tropism Direction",
        min: -180.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    // ⚠️ **O molde vem PRIMEIRO de todos** — antes até do axioma. É a resposta ao *"não são
    // nada intuitivos"*: o artista escolhe um sítio por onde começar, vê a planta, e só depois
    // edita o texto. Um selector abaixo das caixas seria a ajuda escondida atrás do problema.
    ParamUiHint {
        param: param::PRESET,
        label: "Preset",
        min: 0.0,
        // ⚠️ **`PRESET_LABELS`, e não `PRESETS`** — a lista tem uma entrada a mais, o
        // [`PRESET_CUSTOM`], que não é um molde e sim *"nenhum destes"*.
        max: (PRESET_LABELS.len() - 1) as f32,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: PRESET_LABELS,
        },
    },
    ParamUiHint {
        param: param::ORIENT,
        label: "Shape Faces",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: ORIENT_LABELS,
        },
    },
    // As tres do CRESCIMENTO SUAVE (2026-08-29). Ver `turtle::walk` para a medicao.
    ParamUiHint {
        param: param::CONTINUOUS_LENGTH,
        label: "Grow Length",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    ParamUiHint {
        param: param::CONTINUOUS_ANGLE,
        label: "Grow Angle",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    ParamUiHint {
        param: param::GROWTH,
        label: "Growth",
        min: 0.0,
        max: 1.0,
        step: 0.005,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::STEP_SCALE,
        label: "Step Scale",
        min: 0.1,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::SEED,
        label: "Seed",
        min: 0.0,
        max: 9999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
];

/// **AS DUAS METADES NÃO SE VÊEM UMA À OUTRA** — o gate de visibilidade que faz o `Mode` ser
/// um modo em vez de um rótulo.
///
/// ⚠️ *Um controle que não faz nada não é pintado.* No guiado a gramática é derivada, então
/// as caixas de texto mostrariam o que o nó **não lê** — a pior forma de mentir num painel,
/// porque o artista edita e nada acontece. No modo gramática os quatro números de forma
/// deixam de alimentar seja o que for, pela mesma razão do outro lado.
///
/// ⚠️ **O `Preset` fica com a GRAMÁTICA**, e não com os sliders: um molde É uma gramática, e
/// escolher um no guiado escreveria num texto que ninguém está a ler.
pub(crate) static PARAM_GATES: &[ph2d_node_registry::ParamGate] = &[
    // ⭐ A ponta só afina onde há fita.
    ph2d_node_registry::ParamGate {
        param: param::TIP_TAPER,
        when: param::GEOMETRY,
        values: &[GEOMETRY_BRANCHES],
    },
    ph2d_node_registry::ParamGate {
        param: AXIOM_PARAM,
        when: param::MODE,
        values: &[MODE_GRAMMAR],
    },
    ph2d_node_registry::ParamGate {
        param: RULES_PARAM,
        when: param::MODE,
        values: &[MODE_GRAMMAR],
    },
    ph2d_node_registry::ParamGate {
        param: param::PRESET,
        when: param::MODE,
        values: &[MODE_GRAMMAR],
    },
    ph2d_node_registry::ParamGate {
        param: param::BRANCHES,
        when: param::MODE,
        values: &[MODE_GUIDED],
    },
    ph2d_node_registry::ParamGate {
        param: param::SEGMENTS,
        when: param::MODE,
        values: &[MODE_GUIDED],
    },
    ph2d_node_registry::ParamGate {
        param: param::VARIATION,
        when: param::MODE,
        values: &[MODE_GUIDED],
    },
    ph2d_node_registry::ParamGate {
        param: param::BEND,
        when: param::MODE,
        values: &[MODE_GUIDED],
    },
    // ⭐⭐ **O knob que a GRAMÁTICA ESCOLHIDA não lê não é pintado** — a outra metade da cura
    // dos moldes (auditoria 2026-08-29). Uma gramática sem `!` ignora o *Width Scale*; uma sem
    // `"` ignora o *Length Scale*. Medido: o `Length Scale` está **inerte nos 8/8 moldes**
    // (bbox bit-idêntica a `0,10` e a `1,50`) e **vivo** no `Custom` — que é onde o modo
    // guiado e a gramática assada aterram, e onde ele mexe a peça de `0,05` para `10,60`.
    // ⇒ *o knob não está morto: ele MORRE quando um molde é escolhido*, e é o molde que é o
    // sujeito do gate, nunca o modo.
    ph2d_node_registry::ParamGate {
        param: param::WIDTH_SCALE,
        when: param::PRESET,
        values: PRESETS_READING_WIDTH_SCALE,
    },
    ph2d_node_registry::ParamGate {
        param: param::LENGTH_SCALE,
        when: param::PRESET,
        values: PRESETS_READING_LENGTH_SCALE,
    },
];

/// Os índices de molde cuja gramática contém `!` — mais o [`PRESET_CUSTOM`].
///
/// ⚠️ **Escrito à mão e GATEADO contra a derivação** (`Reads::of`), como os `PRESET_LABELS`:
/// uma `const` não pode iterar uma tabela, então a defesa contra as duas respostas divergirem
/// é o gate `the_read_gates_agree_with_what_each_grammar_contains`, não a boa vontade.
pub(crate) static PRESETS_READING_WIDTH_SCALE: &[i32] = &[0, 1, 4, 7, PRESET_CUSTOM as i32];

/// Os índices cuja gramática contém `"`. **Nenhum molde o tem** — só o `Custom`, que é onde o
/// modo guiado e o texto assado vivem.
pub(crate) static PRESETS_READING_LENGTH_SCALE: &[i32] = &[PRESET_CUSTOM as i32];

/// **AS SEÇÕES** — quatro perguntas, e cada uma responde-se sem ler as outras.
///
/// ⚠️ **O `Mode` fica FORA de todas**, de propósito: as soltas são pintadas primeiro
/// (`split_into_sections`), e o controle que decide o que as seções contêm não pode viver
/// dentro de uma delas — muito menos dentro de uma que nasça fechada.
pub(crate) static PARAM_GROUPS: &[ph2d_node_registry::ParamGroup] = &[
    // ⚠️ Na secção da FORMA, e no topo dela: *o que a planta é na tela* é a mesma família de
    // pergunta que *quantos ramos* e *que ângulo* — e não a de *de onde vem a gramática*.
    ph2d_node_registry::ParamGroup::new(param::GEOMETRY, "Shape"),
    ph2d_node_registry::ParamGroup::new(param::TIP_TAPER, "Shape"),
    ph2d_node_registry::ParamGroup::new(param::BRANCHES, "Shape"),
    ph2d_node_registry::ParamGroup::new(param::SEGMENTS, "Shape"),
    ph2d_node_registry::ParamGroup::new(param::ANGLE, "Shape"),
    ph2d_node_registry::ParamGroup::new(param::BEND, "Shape"),
    ph2d_node_registry::ParamGroup::new(param::VARIATION, "Shape"),
    ph2d_node_registry::ParamGroup::new(param::PRESET, "Grammar"),
    ph2d_node_registry::ParamGroup::new(AXIOM_PARAM, "Grammar"),
    ph2d_node_registry::ParamGroup::new(RULES_PARAM, "Grammar"),
    ph2d_node_registry::ParamGroup::new(param::GENERATIONS, "Growth"),
    ph2d_node_registry::ParamGroup::new(param::GROWTH, "Growth"),
    ph2d_node_registry::ParamGroup::new(param::STEP, "Growth"),
    ph2d_node_registry::ParamGroup::new(param::LENGTH_SCALE, "Growth"),
    ph2d_node_registry::ParamGroup::new(param::WIDTH, "Growth"),
    ph2d_node_registry::ParamGroup::new(param::WIDTH_SCALE, "Growth"),
    ph2d_node_registry::ParamGroup::new(param::STEP_SCALE, "Growth"),
    ph2d_node_registry::ParamGroup::new(param::CONTINUOUS_LENGTH, "Growth"),
    ph2d_node_registry::ParamGroup::new(param::CONTINUOUS_ANGLE, "Growth"),
    // ⚠️ Esta nasce FECHADA: é a única cujos cinco defaults já dão uma planta de pé, e o
    // artista que nunca a abrir não perde nada.
    ph2d_node_registry::ParamGroup::new(param::ROOT_ANGLE, "Lean & Look").folded(),
    ph2d_node_registry::ParamGroup::new(param::TROPISM, "Lean & Look").folded(),
    ph2d_node_registry::ParamGroup::new(param::TROPISM_ANGLE, "Lean & Look").folded(),
    ph2d_node_registry::ParamGroup::new(param::ORIENT, "Lean & Look").folded(),
    ph2d_node_registry::ParamGroup::new(param::SEED, "Lean & Look").folded(),
];

/// **O que cada número É** (doc 88) — só as grandezas que são uma DISTÂNCIA de mundo.
///
/// O `step` é a única: um ângulo já tem a face dele pelo widget, e `width` é uma ESCALA
/// (vai para a coluna `size`, que é adimensional), não uma distância — declará-la como
/// `Length` faria a caixa mostrar pixels para um multiplicador.
pub(crate) static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: param::STEP,
    unit: ParamUnit::Length,
}];
