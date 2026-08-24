//! **A SUPERFÍCIE DE PAINEL do `motion.trail`** — os hints, as unidades e as seções.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o corte
//! é por RESPONSABILIDADE e não por tamanho: o `lib.rs` responde *o que o rastro FAZ*
//! e este responde *como ele se APRESENTA*. É o mesmo corte que os irmãos
//! `motion.boids` (`ui.rs`) e `motion.emitter` (`params_ui.rs`) já fizeram.

use super::{ECHO_BLEND, ECHO_BLEND_LABELS, FORWARD, MAX_LENGTH, MAX_SPACING, SOURCE};

use ph2d_node_registry::{ParamGroup, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget};

pub(super) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "length",
        label: "Length",
        min: 1.0,
        max: MAX_LENGTH as f32,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    // ⚠️ Os cinco rótulos abaixo dizem **"Tail"** porque o número é o estado da PONTA da
    // cauda, não uma taxa: é a diferença entre um slider linear no que se vê e o que o
    // smoke de 2026-08-08 reprovou. Um rótulo que dissesse só "Fade" deixaria o artista
    // adivinhar se 0.9 é por tick, por eco ou no fim — e as três respostas dão desenhos
    // que diferem por ordens de grandeza.
    // ⚠️ **O TETO vem ANTES do `Tail Alpha`**, e a ordem é a leitura: a rampa da cauda vai
    // *deste* número até àquele. Fechado em `1` pelo mesmo SIGNIFICADO — um fantasma mais
    // opaco que a fonte dele não é uma cauda, é outra coisa.
    ParamUiHint {
        param: super::ALPHA_MAX,
        label: "Tail Alpha Max",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "fade",
        label: "Tail Alpha",
        // Fechado pelo SIGNIFICADO, não por orçamento: a alfa é uma fração da cabeça viva,
        // e acima de 1 o fantasma ficaria mais opaco que a fonte dele.
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "shrink",
        label: "Tail Size",
        // ⚠️ Passa de 1 de propósito: abaixo é o cometa (a cauda afina), acima é a baforada
        // (a cauda ABRE). A lei antiga também permitia, mas exponencialmente — `1.1` por
        // tick virava 6,7× em 20 ticks; agora `2.0` é exatamente o dobro na ponta.
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "hue_shift",
        label: "Tail Hue Shift",
        // Uma volta INTEIRA para cada lado — o total que a cauda percorre. Além de 360°
        // ela repete matizes que já tem, então é onde a grandeza se fecha.
        min: -360.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "saturation",
        label: "Tail Saturation",
        // Abaixo de 1 a cauda desbota a cinza; acima ela satura — as duas direções são
        // usadas (fumaça × brasa), então a faixa não pode parar na identidade.
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "spin",
        label: "Tail Spin",
        // O mesmo fecho do matiz: a 360° a cauda completou uma revolução.
        min: -360.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    ParamUiHint {
        param: "spacing",
        label: "Spacing",
        min: 1.0,
        // O teto do slider É o teto do recurso: acima dele a janela de idade cresceria
        // sem o eco aparecer, entao nao ha faixa confortavel a separar da legal.
        max: MAX_SPACING as f32,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    // ⚠️ `Enum`, nunca slider: uma tag é um NOME, e não há meio-caminho entre `Add` e
    // `Multiply` — a mesma razão que o `Blend` do `motion.output` já escreve.
    ParamUiHint {
        param: ECHO_BLEND,
        label: "Echo Operator",
        min: 0.0,
        #[expect(clippy::cast_precision_loss, reason = "sete rotulos")]
        max: (ECHO_BLEND_LABELS.len() - 1) as f32,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &ECHO_BLEND_LABELS,
        },
    },
    ParamUiHint {
        param: SOURCE,
        label: "Source",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            // *Remembered* = o ring (o que sempre houve). *Resampled* = a entrada
            // RE-COZIDA nos instantes de cada eco.
            labels: &["Remembered", "Resampled"],
        },
    },
    ParamUiHint {
        param: FORWARD,
        label: "Forward Steps",
        min: 0.0,
        // O curso é o dos ecos que podem existir: `length` conta a cabeça, então
        // um a menos. O clamp real vive no `forward_of`, contra o `length` DESTE
        // nó — o slider não pode saber qual é.
        #[expect(clippy::cast_precision_loss, reason = "um teto de contagem")]
        max: (MAX_LENGTH - 1) as f32,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
];

/// O espaçamento é uma contagem de TICKS, os dois ângulos são GRAUS, e os três alvos
/// multiplicativos são FRAÇÕES da cabeça viva — a unidade que o painel declara é a que o
/// número É.
pub(super) static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "spacing",
        unit: ParamUnit::Count,
    },
    ParamUnitDecl {
        param: "fade",
        unit: ParamUnit::Ratio,
    },
    ParamUnitDecl {
        param: "shrink",
        unit: ParamUnit::Ratio,
    },
    ParamUnitDecl {
        param: "saturation",
        unit: ParamUnit::Ratio,
    },
    ParamUnitDecl {
        param: "length",
        unit: ParamUnit::Count,
    },
    ParamUnitDecl {
        param: "hue_shift",
        unit: ParamUnit::Angle,
    },
    ParamUnitDecl {
        param: "spin",
        unit: ParamUnit::Angle,
    },
];

/// **As seções** (doc 88 §B3, a metade visual). Sete knobs numa coluna são uma parede;
/// agrupados eles viram três perguntas — *que forma tem a cauda* (solto, no topo), *como
/// ela morre* e *que cor ela toma*. ⚠️ `length`/`spacing` ficam FORA de seção de
/// propósito: um param sem entrada é pintado ANTES de tudo, que é onde os essenciais
/// devem estar (a lei do `ParamGroup`, e o padrão do Blender).
pub(super) static PARAM_GROUPS: &[ParamGroup] = &[
    ParamGroup::new("fade", "Decay"),
    ParamGroup::new("shrink", "Decay"),
    ParamGroup::new("spin", "Decay"),
    ParamGroup::new("hue_shift", "Colour"),
    ParamGroup::new("saturation", "Colour"),
    // O operador é sobre a COR na tela, tanto quanto o matiz e a saturação.
    ParamGroup::new(ECHO_BLEND, "Colour"),
    // **De onde o eco vem**, e o que só a re-cozedura permite. Ficam juntos
    // porque o `Forward Steps` é INERTE no modo do ring — pô-lo em `Decay` faria
    // dele um knob que às vezes não faz nada, num sítio onde nada mais é assim.
    ParamGroup::new(SOURCE, "Source"),
    ParamGroup::new(FORWARD, "Source"),
];
