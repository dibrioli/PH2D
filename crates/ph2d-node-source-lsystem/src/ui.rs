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
    // ⭐⭐ **AS TRÊS LETRAS QUE PLANTAM UM OBJECTO** — `J`, `K`, `M` (ver [`LEAF_PARAMS`]).
    // ⚠️ `ParamWidget::Source`: o painel pinta chips dos nomes que a app publicou, que é o que
    // separa *exprimível* de *alcançável* — o artista escolhe uma forma desenhada por nome, sem
    // saber que `J` vale 74.
    ParamUiHint {
        param: LEAF_PARAMS[0],
        label: "Leaf (J)",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Source,
    },
    ParamUiHint {
        param: LEAF_PARAMS[1],
        label: "Leaf (K)",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Source,
    },
    ParamUiHint {
        param: LEAF_PARAMS[2],
        label: "Leaf (M)",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Source,
    },
    // ⭐⭐ **OS CINCO CONTROLOS** que o 2.º smoke de 2026-08-30 pediu.
    ParamUiHint {
        param: param::LEAF_FIRST_LEVEL,
        label: "First Level",
        min: 1.0,
        // ⚠️ **`12` é o tecto de PROFUNDIDADE que uma planta desta casa alcança** — o
        // `MAX_MODULES` corta a derivação muito antes de 12 níveis de encaixe numa gramática
        // que ramifica. Acima disto o knob não teria sujeito.
        max: 12.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::LEAF_ANGLE,
        label: "Leaf Angle",
        min: -180.0,
        max: 180.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: param::LEAF_SPREAD,
        label: "Leaf Spread",
        min: 0.0,
        max: 180.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: param::LEAF_FRONT,
        label: "Leaves In Front",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::LEAF_SIZE,
        label: "Leaf Size",
        min: 0.0,
        // ⚠️ **`8` é o tecto do SLIDER, não do modelo** — a caixa aceita mais, e o `step` de
        // `0,01` dá o tecto digitável derivado (doc 91). Uma folha `8×` já é maior que o ramo
        // que a segura.
        max: 8.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::LEAF_SIZE_JITTER,
        label: "Size Jitter",
        min: 0.0,
        // `1` = de metade ao dobro; acima disto a folha pode ficar em zero, e uma folha
        // invisível não é uma variação, é uma folha perdida.
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::LEAF_POS_JITTER,
        label: "Position Jitter",
        min: 0.0,
        // Em FRACÇÃO do tamanho da folha: `1` = ±meia folha, que é o quanto se pode empurrar
        // antes de ela se descolar do ramo que a plantou.
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::LEAF_EFFECTS,
        label: "Effects Reach Leaves",
        min: 0.0,
        max: (LEAF_EFFECTS_LABELS.len() - 1) as f32,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: LEAF_EFFECTS_LABELS,
        },
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
    // ⭐ As folhas são plantadas pela membrana das FITAS, que só corre em `Branches`. Em
    // `Segments` os elementos `J`/`K`/`M` saem no esqueleto com o `sym` — alcançáveis por
    // composição, mas sem objecto próprio.
    ph2d_node_registry::ParamGate {
        param: LEAF_PARAMS[0],
        when: param::GEOMETRY,
        values: &[GEOMETRY_BRANCHES],
    },
    ph2d_node_registry::ParamGate {
        param: LEAF_PARAMS[1],
        when: param::GEOMETRY,
        values: &[GEOMETRY_BRANCHES],
    },
    ph2d_node_registry::ParamGate {
        param: LEAF_PARAMS[2],
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
    // ⭐⭐ **OS TRÊS QUE A CAÇA DE 2026-08-31 ACHOU** — report do Enio: *"descubra para cada
    // Preset quais os parâmetros que não são usados e esconda do painel"*.
    //
    // ⚠️ **Nenhum dos três tem um SÍMBOLO a denunciá-lo**, e é isso que os separa dos dois
    // acima: o `Width Scale` morre onde não há `!` e o `Length Scale` onde não há `"`, que um
    // scanner de texto ([`Reads::of`]) vê. Estes três morrem por **como o `build` os lê**, e só
    // o produto responde — ver `tests/no_preset_shows_a_knob_its_grammar_cannot_read.rs`.
    ph2d_node_registry::ParamGate {
        param: param::STEP_SCALE,
        when: param::PRESET,
        values: PRESETS_READING_STEP_SCALE,
    },
    ph2d_node_registry::ParamGate {
        param: param::CONTINUOUS_LENGTH,
        when: param::PRESET,
        values: PRESETS_GROWING_BY_TIP,
    },
    ph2d_node_registry::ParamGate {
        param: param::CONTINUOUS_ANGLE,
        when: param::PRESET,
        values: PRESETS_GROWING_BY_REFINEMENT,
    },
    // ⛔⛔ **E OS MESMOS DOIS, TAMBÉM PELO MODO** — o defeito que a auditoria de seis lentes
    // achou na cura de ontem (doc 96 §1.2).
    //
    // Os gates por molde acima incluem o `PRESET_CUSTOM`, com o argumento *«no Custom a
    // gramática é a que o artista escreveu, então esconder ali é adivinhar»*. ⚠️⚠️ **Esse
    // argumento é verdade para uma gramática ESCRITA À MÃO e falso para a GUIADA** — e o
    // `Custom` é o preset de fábrica, o `Guided` é o modo de fábrica, logo os dois nasciam
    // pintados e mortos **no primeiro ecrã de um nó recém-largado**.
    //
    // ⭐ **No modo guiado o app DERIVA a gramática, e sabe exactamente qual é**: ela é sempre
    // paramétrica (`F(s)`, o comprimento viaja no módulo ⇒ o `Setup::step` nunca é lido) e
    // cresce sempre pela PONTA (⇒ o braço do `build` faz `ang_frac = frac` e nunca lê o
    // `Grow Angle`). ⚠️ **Medido em TODO o espaço que os sliders alcançam** — 750 células de
    // `branches × segments × variation × bend`, em geração inteira e fraccionária: **zero**
    // refinam, **zero** em que qualquer um dos dois mexa um bit.
    //
    // ⚠️ **Duas entradas para o mesmo param compõem por E** (`Visibility::shows` esconde se
    // QUALQUER gate reprovar), que é o que deixa a pergunta ser feita em duas metades
    // independentes: *o molde lê isto?* e *este modo lê isto?*
    ph2d_node_registry::ParamGate {
        param: param::STEP_SCALE,
        when: param::MODE,
        values: &[MODE_GRAMMAR],
    },
    ph2d_node_registry::ParamGate {
        param: param::CONTINUOUS_ANGLE,
        when: param::MODE,
        values: &[MODE_GRAMMAR],
    },
];

/// **A ponta só afina onde há puxão** — `Tropism Direction` sem `Tropism` não tem o que virar.
///
/// ⚠️ **Ele estava morto no instante em que o painel abre**, e não numa esquina: o `tropism`
/// nasce em `0`, e a lei sai por `if set.tropism == 0.0 { return 0.0 }` — medido inerte nos
/// **nove** moldes. *Um knob que não faz nada com os defaults de fábrica é o pior dos mortos:
/// ele é o primeiro que o artista experimenta.*
///
/// ⛔ **E a cura NÃO é escondê-lo por molde** — a medição diz que ele acorda em TODOS assim que
/// o vizinho sai de zero. Uma lista de moldes ali apagaria um controlo vivo em nove sítios; o
/// que a pergunta pede é um limiar, que é precisamente o que este irmão do [`ParamGate`] é.
pub(crate) static PARAM_GATES_ABOVE: &[ph2d_node_registry::ParamGateAbove] =
    &[ph2d_node_registry::ParamGateAbove {
        param: param::TROPISM_ANGLE,
        when: param::TROPISM,
        above: 0.0,
    }];

/// Os índices de molde cuja gramática contém `!` — mais o [`PRESET_CUSTOM`].
///
/// ⚠️ **Escrito à mão e GATEADO contra a derivação** (`Reads::of`), como os `PRESET_LABELS`:
/// uma `const` não pode iterar uma tabela, então a defesa contra as duas respostas divergirem
/// é o gate `the_read_gates_agree_with_what_each_grammar_contains`, não a boa vontade.
pub(crate) static PRESETS_READING_WIDTH_SCALE: &[i32] = &[0, 1, 4, 7, PRESET_CUSTOM as i32];

/// Os índices cuja gramática contém `"`. **Nenhum molde o tem** — só o `Custom`, que é onde o
/// modo guiado e o texto assado vivem.
pub(crate) static PRESETS_READING_LENGTH_SCALE: &[i32] = &[PRESET_CUSTOM as i32];

/// **Onde o `Step Scale` tem sujeito** — Bush · Weed · Koch · Dragon, mais o `Custom`.
///
/// # O mecanismo (⛔ não é a mesma pergunta que as duas de cima, apesar de a resposta coincidir)
///
/// O param entra por `step: p.step * step_scale^gerações`, que alimenta o `Setup::step` — e o
/// `Setup::step` é lido por um módulo de desenho **sem parâmetro** (`F`). Numa gramática
/// PARAMÉTRICA (`A(s) -> F(s)…`) o comprimento viaja dentro do módulo e sai da expressão, que
/// lê o `step` CRU pela ponte `Params::by_name` — o expoente nunca é aplicado.
///
/// ⇒ *o `Step Scale` é o controlo de comprimento das gramáticas clássicas, e nas paramétricas
/// quem manda é o `s` que a regra escreve.*
///
/// ⚠️⚠️ **A coincidência com [`PRESETS_GROWING_BY_REFINEMENT`] é ACIDENTE, e por isso são duas
/// constantes e não um alias.** As perguntas são outras — *«há um `F` sem parâmetro?»* contra
/// *«a figura refina ou cresce pela ponta?»* — e hoje as duas partem o corpus no mesmo sítio
/// porque as quatro clássicas são exactamente as quatro que refinam. Um molde paramétrico que
/// refinasse (ou o contrário) separá-las-ia, e um alias faria a segunda resposta seguir a
/// primeira **em silêncio**. *Duas leis com a mesma tabela hoje continuam a ser duas leis.*
pub(crate) static PRESETS_READING_STEP_SCALE: &[i32] = &[2, 3, 5, 6, PRESET_CUSTOM as i32];

/// **Onde o `Grow Length` tem sujeito** — os que crescem pela PONTA, mais o `Custom`.
///
/// # O mecanismo, e por que estes dois são COMPLEMENTARES
///
/// O `build` pergunta `grows_by_refining()` e **cada braço lê um interruptor só**:
/// - cresce pela ponta ⇒ `(if want_len { frac } else { 1.0 }, frac)` — a viragem é `frac`
///   sempre, e o `Grow Angle` **nunca é lido**;
/// - refina ⇒ `if want_ang { … } else { (1.0, 1.0) }` — o `Grow Length` **nunca é lido**.
///
/// ⇒ os dois nunca estão vivos ao mesmo tempo, e o painel mostrava sempre os dois. *Metade de
/// um par exclusivo é um knob morto em todo molde, o tempo todo.*
///
/// ⚠️ **A prova de que não são um só param com dois nomes** está no braço da ponta: ali a
/// viragem contínua não é *desligada*, é **obrigatória** (`ang_frac = frac`). Fundi-los daria ao
/// artista de um refinador o poder de desligar o que nas outras é lei.
pub(crate) static PRESETS_GROWING_BY_TIP: &[i32] = &[0, 1, 4, 7, PRESET_CUSTOM as i32];

/// **Onde o `Grow Angle` tem sujeito** — os que REFINAM, mais o `Custom`.
/// Ver [`PRESETS_GROWING_BY_TIP`] para o mecanismo, que é o mesmo lido do outro lado.
pub(crate) static PRESETS_GROWING_BY_REFINEMENT: &[i32] = &[2, 3, 5, 6, PRESET_CUSTOM as i32];

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
    // ⚠️ **Secção própria e DOBRADA**: a maioria das gramáticas não emite `J`/`K`/`M` nenhum, e
    // três campos abertos empurrariam o resto do painel para fora do corpo por uma feature que
    // aquela planta não usa. Quem escreve um `J` na gramática vai procurá-la.
    ph2d_node_registry::ParamGroup::new(LEAF_PARAMS[0], "Leaves").folded(),
    ph2d_node_registry::ParamGroup::new(LEAF_PARAMS[1], "Leaves").folded(),
    ph2d_node_registry::ParamGroup::new(LEAF_PARAMS[2], "Leaves").folded(),
    ph2d_node_registry::ParamGroup::new(param::LEAF_FIRST_LEVEL, "Leaves").folded(),
    ph2d_node_registry::ParamGroup::new(param::LEAF_ANGLE, "Leaves").folded(),
    ph2d_node_registry::ParamGroup::new(param::LEAF_SPREAD, "Leaves").folded(),
    ph2d_node_registry::ParamGroup::new(param::LEAF_FRONT, "Leaves").folded(),
    ph2d_node_registry::ParamGroup::new(param::LEAF_EFFECTS, "Leaves").folded(),
    ph2d_node_registry::ParamGroup::new(param::LEAF_SIZE, "Leaves").folded(),
    ph2d_node_registry::ParamGroup::new(param::LEAF_SIZE_JITTER, "Leaves").folded(),
    ph2d_node_registry::ParamGroup::new(param::LEAF_POS_JITTER, "Leaves").folded(),
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
