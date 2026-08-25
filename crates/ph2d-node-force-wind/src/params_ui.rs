//! **As TABELAS DE PAINEL do `force.wind`** — hints, tectos digitáveis, unidades e o gate
//! do modo.
//!
//! Separado do [`super`] pelo tecto de LOC (HR-18, 700 para `crates/`), no corte que os
//! nós grandes desta casa já usam (`motion.emitter/params_ui.rs`): o pai responde *o que a
//! força calcula*, aqui fica *como o artista a disca*.

use super::{AIR_RESIST, MODE, MODE_LABELS};
use ph2d_node_registry::{ParamGateAbove, ParamUiHint, ParamWidget};

/// **A LEI DA SEGUNDA OITAVA** (doc 90 §1 — a caça aos knobs mortos, 2026-08-22).
///
/// ⚠️ `lacunarity` e `roughness` são os dois números que descrevem **a relação entre oitavas
/// consecutivas**, e o `ph2d-fbm` aplica-os (`px *= lacunarity`, `amp *= gain`) **depois** de
/// somar a oitava corrente. Com `octaves = 1` — o **default deste nó** — não existe oitava
/// seguinte, e os dois são provadamente inertes: mexê-los não muda um bit da saída.
///
/// O sintoma media-se no painel do nó recém-largado: **dois sliders de aspecto fractal, lado a
/// lado, ambos mudos**. E é a pior forma do defeito — *dois knobs inertes vizinhos ensinam que
/// o BLOCO de ruído não funciona, e não que falta subir um terceiro número primeiro.*
///
/// ⚠️ Um `ParamGate` não serviria: ele arredonda a inteiro e compara com uma lista de índices
/// de enum, e `octaves` é uma grandeza. A pergunta *"apareça quando isto passar de 1"* é
/// exactamente o [`ParamGateAbove`].
/// **O ângulo é CÍCLICO, e é por isso que o piso é negativo** — bloco Z, doc 91.
///
/// ⚠️ **A cena `=24` autora `angle = −90` e o campo digitava `[0, 360]`.** O nó honra o valor
/// perfeitamente (`frac(p) = p − p.floor()` leva `−0,25` a `0,75`, então `−90° ≡ 270°`), mas o
/// artista não conseguia escrevê-lo — e escrever `−90` para *"para baixo"* é o gesto natural de
/// quem vem de qualquer outra ferramenta. Acusação da sonda
/// `what_the_corpus_authors_and_no_one_can_type`.
///
/// **De que recurso é este teto: do SIGNIFICADO** (`CLAUDE.md` §0.0), não da precisão. Uma volta
/// inteira esgota as direções: `450°` desenha o mesmo vento que `90°`, então um campo que
/// aceitasse mais estaria a oferecer uma escolha que não existe — a mesma lei do
/// `sim.spawn::probability`, que para em `1` porque acima dali todo nascimento acontece.
pub(super) static PARAM_HARD_MAX: &[ph2d_node_registry::ParamHardMax] =
    &[ph2d_node_registry::ParamHardMax {
        param: "angle",
        max: 360.0,
    }];

/// A volta NEGATIVA — a metade que faltava.
pub(super) static PARAM_HARD_MIN: &[ph2d_node_registry::ParamHardMin] =
    &[ph2d_node_registry::ParamHardMin {
        param: "angle",
        min: -360.0,
    }];

pub(super) static PARAM_GATES_ABOVE: &[ParamGateAbove] = &[
    ParamGateAbove {
        param: "lacunarity",
        when: "octaves",
        above: 1.0,
    },
    ParamGateAbove {
        param: "roughness",
        when: "octaves",
        above: 1.0,
    },
];

/// Param UI hints (M1.P1).
pub(super) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "octaves",
        label: "Octaves",
        min: 1.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: "type",
        label: "Noise Type",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["fBm", "Turbulence", "Ridged"],
        },
    },
    ParamUiHint {
        param: "lacunarity",
        label: "Lacunarity",
        min: 1.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "roughness",
        label: "Roughness",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "loop_period",
        label: "Loop Period",
        min: 0.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "angle",
        label: "Angle",
        min: 0.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    // ⚠️ O curso do arrasto é **uma volta**, de propósito, e o teto digitável abre a volta
    // NEGATIVA — ver [`PARAM_HARD_MIN`].
    ParamUiHint {
        param: "strength",
        label: "Strength",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "gust",
        label: "Gust",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "gust_freq",
        label: "Gust Frequency",
        min: 0.1,
        max: 5.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 100.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
    ParamUiHint {
        param: MODE,
        label: "Mode",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: MODE_LABELS,
        },
    },
    ParamUiHint {
        param: AIR_RESIST,
        label: "Air Resistance",
        min: 0.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
];
