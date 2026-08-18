//! **A superfície de PARAMS deste nó** — hints, grupos, unidades e faixas, cortadas do
//! `lib.rs` no teto de LOC (HR-18) pela costura que o irmão `motion.oscillator` já tinha.
//!
//! O corte é por RESPONSABILIDADE e não por tamanho: o `lib.rs` responde *o que um ruído
//! É* (o manifesto, o campo, o laço no tempo) e este arquivo *como o painel o pinta*.
//! Nada aqui é lido pelo `eval` — se fosse, o corte estaria no sítio errado.

use super::MAX_OCTAVES;
use ph2d_node_registry::{
    ParamChannelRange, ParamGroup, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget,
};

/// **O que o `loop_len` É** (doc 88, Wave A): uma DURAÇÃO. É a única unidade deste nó — a
/// `amplitude` é `FromChannel` como a do oscilador seria, mas aqui ela não é declarada porque
/// este nó ainda não passou pela varredura de unidades; o `loop_len` entra declarado para não
/// nascer com a dívida.
pub(crate) static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: "loop_len",
    unit: ParamUnit::Seconds,
}];

/// As SEÇÕES deste nó (doc 88 B3). Nove controles respondem a três perguntas.
///
/// ⚠️ **"Timing" é o MESMO título do `motion.oscillator`, de propósito** — os dois respondem
/// *em que relógio isto anda*, e dois nomes para a mesma pergunta ensinariam que são coisas
/// diferentes (o precedente dos dois nós de curva, que partilham "Curve").
///
/// Ficam SOLTOS `channel`, `amplitude` e `type`: onde o ruído escreve, quanto ele vale, e que
/// ruído ele é.
pub(crate) static PARAM_GROUPS: &[ParamGroup] = &[
    // A FORMA do campo.
    ParamGroup::new("scale", "Field"),
    ParamGroup::new("octaves", "Field"),
    ParamGroup::new("roughness", "Field"),
    ParamGroup::new("seed", "Field"),
    // Em que relógio ele anda.
    ParamGroup::new("speed", "Timing"),
    ParamGroup::new("loop_len", "Timing"),
];

pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    // ⚠️ A faixa começa em 1: lacunarity < 1 faz as oitavas ficarem mais GRANDES
    // que a base — o campo perde a leitura fractal e vira um borrão de baixa
    // frequência. `2` é o universal, e `1,5..3` é onde a mão trabalha.
    ParamUiHint {
        param: "lacunarity",
        label: "Lacunarity",
        min: 1.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
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
        param: "amplitude",
        label: "Amplitude",
        min: 0.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "scale",
        label: "Scale",
        min: 0.02,
        max: 2.0,
        step: 0.02,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "octaves",
        label: "Octaves",
        min: 1.0,
        max: MAX_OCTAVES as f32,
        step: 1.0,
        widget: ParamWidget::IntSlider,
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
        param: "type",
        label: "Type",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["fBm", "Turbulence", "Ridged"],
        },
    },
    ParamUiHint {
        param: "speed",
        label: "Speed",
        min: 0.0,
        max: 3.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    // A faixa de um loop é a de um take de motion graphics: 0 (nunca fecha) até 30 s.
    // ⚠️ **CORRIGIDO (doc 89, grupo B):** este comentário dizia que *"a caixa aceita
    // além dele pelo `ParamHardMax`"* e isso é FALSO — este nó nunca chamou
    // `register_param_hard_max`, e o shell resolve `param_hard_max(..).unwrap_or(max)`,
    // logo a caixa PARA nos 30 s. Um ciclo mais longo é hoje inalcançável neste nó.
    // O irmão `value.noise` declara o dele (2²⁴, precisão de representação); subir
    // este muda o que a caixa aceita e é decisão do dono deste nó.
    ParamUiHint {
        param: "loop_len",
        label: "Loop Length",
        min: 0.0,
        max: 30.0,
        step: 0.1,
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
];

/// **A faixa que estas magnitudes querem quando o canal é ANGULAR** — graus, não
/// unidades de mundo. Uma volta para cada lado, discada em graus inteiros.
///
/// ⚠️ Ela mora AQUI e não numa tabela do shell porque a tabela apodreceu: medida,
/// ela cobria três dos seis nós que precisavam dela, e cada um dos três ausentes
/// esperava o próprio report do artista.
const TURN: f32 = 360.0;
pub(crate) static PARAM_CHANNEL_RANGE: &[ParamChannelRange] = &[ParamChannelRange {
    param: "amplitude",
    min: 0.0,
    max: TURN,
    step: 1.0,
}];
