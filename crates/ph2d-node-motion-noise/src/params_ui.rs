//! **A superfície de PARAMS deste nó** — hints, grupos, unidades e faixas, cortadas do
//! `lib.rs` no teto de LOC (HR-18) pela costura que o irmão `motion.oscillator` já tinha.
//!
//! O corte é por RESPONSABILIDADE e não por tamanho: o `lib.rs` responde *o que um ruído
//! É* (o manifesto, o campo, o laço no tempo) e este arquivo *como o painel o pinta*.
//! Nada aqui é lido pelo `eval` — se fosse, o corte estaria no sítio errado.

use super::MAX_OCTAVES;
use ph2d_node_registry::{
    ParamChannelRange, ParamGate, ParamGroup, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget,
};

/// **O que o `loop_len` É** (doc 88, Wave A): uma DURAÇÃO. É a única unidade deste nó — a
/// `amplitude` é `FromChannel` como a do oscilador seria, mas aqui ela não é declarada porque
/// este nó ainda não passou pela varredura de unidades; o `loop_len` entra declarado para não
/// nascer com a dívida.
pub(crate) static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "loop_len",
        unit: ParamUnit::Seconds,
    },
    // ⚠️ **`Angle` (graus), e não «o que o canal falar».** Esta rotação gira o ESPAÇO
    // do campo — ela não tem nada a ver com o canal em que o ruído escreve, e um
    // `FromChannel` aqui faria a faixa dela mudar quando o artista trocasse de canal.
    ParamUnitDecl {
        param: "rotation",
        unit: ParamUnit::Angle,
    },
];

/// A linha do eixo Y pertence ao modo que a lê, e a mais nenhum — o precedente do
/// `column` do `motion.drive` e do `curve` do `motion.time_remap`. Um slider de
/// `Scale Y` pintado sob `uniform` seria um controle que não move um quadro.
pub(crate) static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: "scale_y",
        when: "uniform",
        values: &[0],
    },
    // ⚠️ **A MÉTRICA só existe na base CELULAR** — nas outras não há distância nenhuma a
    // medir, e um knob pintado que não é lido é o defeito que a caça aos knobs mortos
    // (doc 90) desta linha existiu para apagar.
    ParamGate {
        param: "metric",
        when: "base",
        values: &[2],
    },
    // ⚠️ **A FAIXA e a AMPLITUDE são a MESMA saída em duas réguas**, então mostrar as
    // duas seria pior que um botão morto: três números na tela a discordar sobre a
    // mesma grandeza, sem nada a dizer qual manda. É verbatim a decisão que o
    // `time_mode`/`bpm` do irmão `motion.oscillator` já tomou.
    ParamGate {
        param: "amplitude",
        when: "range_mode",
        values: &[0],
    },
    ParamGate {
        param: "min",
        when: "range_mode",
        values: &[1],
    },
    ParamGate {
        param: "max",
        when: "range_mode",
        values: &[1],
    },
];

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
    // ⚠️ **A `base` e a `metric` moram aqui** e não soltas no topo: elas são a forma do
    // campo tanto quanto a escala e as oitavas. Soltas, elas cresciam a altura do corpo do
    // painel — ver a nota do `Space` abaixo.
    ParamGroup::new("base", "Field"),
    ParamGroup::new("metric", "Field"),
    ParamGroup::new("scale", "Field"),
    ParamGroup::new("octaves", "Field"),
    ParamGroup::new("roughness", "Field"),
    ParamGroup::new("seed", "Field"),
    // Em que relógio ele anda.
    ParamGroup::new("speed", "Timing"),
    ParamGroup::new("loop_len", "Timing"),
    // O ESPAÇO em que ele é amostrado — a seção nova (folha 06 linha 20).
    //
    // ⭐⭐ **NASCE FECHADA desde 2026-08-25, e a razão é uma REGRA e não conforto.** Ao
    // ganhar a `base` este nó passou a desenhar **673 px** num corpo de `664`, e o gate
    // `the_dock_overflow_is_named_not_discovered` reprovou. A regra escrita no §5 do
    // `CLAUDE.md` diz o que fazer: *um SEGUNDO nome na lista de excepções significa que a
    // resposta virou secções recolhíveis* — e a máquina do `.folded()` foi construída por
    // esta mesma linha, três blocos antes, exactamente para isto.
    //
    // ⚠️ **O `Space` é a candidata certa** entre as três: a anisotropia do campo é o
    // controlo mais avançado do nó (a `Field` e o `Timing` são o que se toca sempre), e ela
    // é a secção mais nova. Fechada, o corpo desce ~2 fileiras e o inspector volta a abrir
    // sem precisar da roda.
    ParamGroup::new("rotation", "Space").folded(),
    ParamGroup::new("uniform", "Space").folded(),
    ParamGroup::new("scale_y", "Space").folded(),
];

pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    // ⭐ **A BASE do ruído** (doc 89, folha 06 linha 21). ⚠️ Ela NÃO é o `type`: o `type`
    // escolhe a rectificação por oitava, a base escolhe o ruído em si.
    ParamUiHint {
        param: "base",
        label: "Base",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Gradient", "Value", "Cellular"],
        },
    },
    // ⚠️ **O vocabulário é o do `motion.voronoi`, literalmente** — um censo no
    // `registry-init` afirma que as duas listas são a mesma.
    ParamUiHint {
        param: "metric",
        // ⚠️ **`Distance` e nao `Metric`** — o `motion.voronoi` ja shipava esse rotulo para a
        // mesma pergunta, e o censo `metric_vocabulary` apanhou a divergencia no primeiro
        // dia. *O que ja shipou ganha:* mudar o outro mexeria num nome que o artista aprendeu.
        label: "Distance",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Euclidean", "Manhattan", "Chebyshev"],
        },
    },
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
        // ⚠️ **Apendado**: o `Position XY` é o índice 4, e os quatro de sempre ficam
        // onde estavam — um documento autorado guarda o NÚMERO, não o nome.
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["X", "Y", "Rotation", "Size", "Position XY"],
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
    // A RÉGUA da mesma saída — `Amplitude` é o nó que sempre shipou.
    ParamUiHint {
        param: "range_mode",
        label: "Range",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Amplitude", "Min / Max"],
        },
    },
    // ⚠️ Onde a saída de facto CAI — e ela cai lá seja qual for o `type`, que é a
    // razão de este par existir (ver `NoiseType::natural_range`).
    ParamUiHint {
        param: "min",
        label: "Minimum",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "max",
        label: "Maximum",
        min: -10.0,
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
    // ── O ESPAÇO ────────────────────────────────────────────────────────────────
    // ⚠️ Uma volta para cada lado, discada em graus inteiros — a mesma régua do
    // `angle` do `motion.orbit`, que é a outra rotação de ESPAÇO da casa.
    ParamUiHint {
        param: "rotation",
        label: "Rotation",
        min: -360.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "uniform",
        label: "Uniform",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    // A MESMA faixa do `scale`: os dois eixos são a mesma grandeza, e duas réguas
    // diferentes para o mesmo número é como um artista aprende que são coisas
    // diferentes.
    ParamUiHint {
        param: "scale_y",
        label: "Scale Y",
        min: 0.02,
        max: 2.0,
        step: 0.02,
        widget: ParamWidget::Slider,
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
