#![forbid(unsafe_code)]
//! `value.noise` — the value-domain PRODUCER of a coherent **noise** field: a
//! smooth per-instance random value that varies across instances AND evolves over
//! time (Motion Nodes M2, the value domain — doc 12/69). It is the pure *producer*
//! form of `motion.wiggle` (which writes a transform channel), exactly as
//! `value.lfo` is the producer of `motion.oscillator` — it emits the noise as a
//! **value** on its own socket, to be routed by `motion.drive`, reshaped by
//! `value.curve` / `value.map_range`, or gated by a pulse.
//!
//! **Coherent, not white — the distinction from `value.instance_field`'s Random.**
//! Random is a hash per instance: neighbours are UNCORRELATED (a value that jumps,
//! white noise). Noise samples a continuous field: neighbouring instances read
//! nearby lattice points, so they flow TOGETHER (a smooth gradient across the row
//! that drifts over time) — the "give it life" driver of every motion tool:
//! AE's `wiggle()`, C4D MoGraph's Random(Noise), TouchDesigner's Noise CHOP,
//! Blender's Noise Texture, Houdini's `noise`/`turb`.
//!
//! **The two axes** (doc 69): `frequency` scales the INSTANCE axis (spatial detail
//! across the row — low is a smooth swell, high decorrelates neighbours) and
//! `speed` scales the TIME axis (temporal evolution — 0 freezes the field). `seed`
//! offsets the lattice (a different slice of the same field). `octaves` +
//! `roughness` are the fBm knobs (fractal detail, Blender's Detail/Roughness) —
//! octaves 1 is a single layer, the SAME field a `motion.wiggle` sample reads.
//! `amplitude` scales and `offset` shifts the result.
//!
//! `value_i = fbm(t·speed, i·frequency + seed, octaves, roughness) · amplitude +
//! offset`, with `fbm ∈ [-1, 1]` (normalized by the octave-weight sum, so adding
//! detail never grows the range). See [`noise`].
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12). Cardinality follows the geometry:
//! the optional `in` port is read for its **count only** (like `value.lfo`) —
//! unconnected → a length-1 field (one global wiggle, held across every instance
//! by `motion.drive`'s broadcast rule). Reads the playhead, holds no state →
//! `Effect::Temporal` (pull-side, like the LFO). Transcendental-free (HR-5); the
//! GPU kernel is the WGSL port of the same lattice + fade + fBm, so the node is
//! **device-resident** — it cooks on the GPU, no CPU fallback.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod kernel;
mod noise;
use kernel::GPU_KERNEL;
use noise::{CellFeature, Kernel, fbm_2d};

/// The instance stream type — the optional `in` port, read for its count only.
const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_value_lfo::VALUE`; kept local so this stays a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value output column (the canonical `value`-domain column).
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.noise"),
    name: "value.noise",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    // Reads the playhead → pull-side; the noise is nonetheless deterministic
    // (a pure function of `(i, t, params)`), so scrubbing reproduces it exactly.
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "frequency",
            default: 0.2,
        },
        ParamSpec {
            name: "speed",
            default: 0.5,
        },
        ParamSpec {
            name: "octaves",
            default: 1.0,
        },
        ParamSpec {
            name: "roughness",
            default: 0.5,
        },
        ParamSpec {
            name: "amplitude",
            default: 1.0,
        },
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        // ⚠️ **Apendado**: `0` = Index (o nó que sempre shipou), `1` = World.
        ParamSpec {
            name: "space",
            default: 0.0,
        },
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
        // ⚠️ **Apendado, e o nome NÃO é `type`**: o `motion.noise` já gastou essa
        // palavra na LEI fractal (fBm/Turbulence/Ridged) e aqui a pergunta é o
        // RUÍDO DE BASE. `0` = Value (o que sempre shipou) · `1` = Perlin ·
        // `2` = Cellular.
        ParamSpec {
            name: "kernel",
            default: 0.0,
        },
        // Só o Cellular os lê — o `ParamGate` esconde-os nos outros dois.
        ParamSpec {
            name: "feature",
            default: 0.0,
        },
        ParamSpec {
            name: "jitter",
            default: 1.0,
        },
        // ⚠️ **Apendado**: era a const `LACUNARITY = 2.0` do `noise.rs`. `2.0` ⇒
        // hoje, byte a byte (a folha já a recebia; o que muda é de onde vem).
        ParamSpec {
            name: "lacunarity",
            default: 2.0,
        },
        // ⚠️ **Apendado**: `0` = sem laço, o nó que sempre shipou. O nome é o dos
        // IRMÃOS que já fecham o laço (`force.curl`/`force.wind`); o `motion.noise`
        // chama-o `loop_len`, e a maioria manda.
        ParamSpec {
            name: "loop_period",
            default: 0.0,
        },
        // ⚠️ **Apendados**: o deslize CONTÍNUO do domínio — o `seed` feito
        // animável. `0` ⇒ hoje. Ver [`Sample::at`].
        ParamSpec {
            name: "pan_x",
            default: 0.0,
        },
        ParamSpec {
            name: "pan_y",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The node's knobs, bundled so [`Sample::at`] takes an instance + a time rather
/// than eight arguments. Read once per cook from the [`EvalCtx`].
struct Sample {
    frequency: f32,
    speed: f32,
    octaves: u32,
    roughness: f32,
    amplitude: f32,
    offset: f32,
    seed: f32,
    kernel: Kernel,
    feature: CellFeature,
    jitter: f32,
    lacunarity: f32,
    loop_period: f32,
    pan_x: f32,
    pan_y: f32,
}

impl Sample {
    fn from_ctx(ctx: &mut EvalCtx<'_>) -> Self {
        Self {
            frequency: ctx.param("frequency"),
            speed: ctx.param("speed"),
            // `round().max(1)` mirrors the WGSL `clamp(round(.), 1, 8)`; `fbm_2d`
            // caps at MAX_OCTAVES, so a wild param can never unbound the loop.
            octaves: ctx.param("octaves").round().max(1.0) as u32,
            roughness: ctx.param("roughness"),
            amplitude: ctx.param("amplitude"),
            offset: ctx.param("offset"),
            seed: ctx.param("seed"),
            kernel: Kernel::from_index(ctx.param("kernel")),
            feature: CellFeature::from_index(ctx.param("feature")),
            jitter: ctx.param("jitter"),
            lacunarity: ctx.param("lacunarity"),
            loop_period: ctx.param("loop_period"),
            pan_x: ctx.param("pan_x"),
            pan_y: ctx.param("pan_y"),
        }
    }

    /// **A porta única do campo** — as duas leituras (fila e espaço) diferem só
    /// no PONTO que amostram, nunca no ruído que somam. Duas cópias divergiriam
    /// no dia em que um kernel novo entrasse, e o modo World é exactamente onde
    /// ninguém olha.
    fn field(&self, x: f32, y: f32) -> f32 {
        let (k, feat, j) = (self.kernel, self.feature, self.jitter);
        fbm_2d(
            x,
            y,
            self.octaves,
            self.lacunarity,
            self.roughness,
            |px, py| noise::base(k, feat, j, px, py),
        ) * self.amplitude
            + self.offset
    }

    /// **A porta única do TEMPO** — o campo no instante `t`, com o laço FECHADO
    /// quando `loop_period > 0` (doc 89 folha 15, *"uma ferramenta de motion
    /// design cujo ruído não fecha o laço não faz um GIF"*).
    ///
    /// ⚠️ **A costura vem da folha [`ph2d_fbm::loop_times`] e já tinha três
    /// consumidores** (`motion.noise`, `force.curl`, `force.wind`) — este nó era
    /// o único da família que lia `t` linearmente. O raciocínio inteiro viaja com
    /// ela: o tempo **WRAPA primeiro** (misturar `campo(t)` com `campo(t−L)` não
    /// fecha, são campos diferentes nas duas pontas) e o peso é **smoothstep**
    /// (com peso linear o valor fecha e a DERIVADA salta, o que lê como um tranco
    /// a cada volta).
    ///
    /// ⚠️ **`w == 0` pula a segunda amostra**, e é o caminho de sempre: sem laço
    /// a folha devolve `(t, t, 0)` ⇒ uma amostra, no mesmo ponto de antes.
    ///
    /// ⚠️ **O `speed` não quebra o fecho**: as duas amostras atravessam o MESMO
    /// `x_of`, então em `τ = 0` o resultado é `campo(0)` e em `τ → L` o peso vai
    /// a 1 sobre `campo(0)` de novo — a costura fecha no mesmo número seja qual
    /// for a velocidade.
    fn over_time(&self, t: f32, x_of: impl Fn(f32) -> f32, y: f32) -> f32 {
        let (t_a, t_b, w) = ph2d_fbm::loop_times(t, self.loop_period);
        let a = self.field(x_of(t_a), y);
        if w == 0.0 {
            return a;
        }
        a + (self.field(x_of(t_b), y) - a) * w
    }

    /// Instance `i`'s value at playhead `t`. `x = t·speed + pan_x` (the time
    /// axis), `y = i·frequency + seed + pan_y` (the instance axis), then
    /// `fbm·amplitude + offset`.
    ///
    /// ⚠️ **O `pan` mede RETICULADO, não mundo — e é o `seed` feito contínuo.**
    /// A célula da folha 15 nomeia exactamente isso: o `seed` *desloca o
    /// reticulado* (outra fatia do campo) mas é inteiro e de passo 1, logo é um
    /// **re-sorteio** e não um **deslize animável**. Escolher a unidade do
    /// `seed` — e não a do mundo — é o que mantém o knob a significar UMA coisa
    /// nos dois modos de amostragem: em World a alternativa seria `(px + pan)·
    /// frequency`, que é unidade de mundo, e o mesmo controlo passaria a medir
    /// duas grandezas diferentes conforme o `Sample`. O gate que prova a régua é
    /// `a_pan_of_one_is_a_seed_of_one` — `pan_y = 1` tem de dar o campo de
    /// `seed = 1`, ao bit.
    fn at(&self, i: u32, t: f32) -> f32 {
        let y = i as f32 * self.frequency + self.seed + self.pan_y;
        self.over_time(t, |tt| tt * self.speed + self.pan_x, y)
    }

    /// **O valor no PONTO `(px, py)`** — o mesmo campo, amostrado no espaço em vez
    /// de na fila (doc 89 folha 10, o P0 do `field.noise`).
    ///
    /// ⚠️ **Sem isto o campo procedural era inexprimível, e o motivo era ESTE.** A
    /// célula nomeava dois bloqueios: (1) *"o `drive` não tem canal `falloff`"* —
    /// **morreu**, o canal existe e escreve a coluna — e (2) *"o `value.noise` é
    /// indexado, não tem `P`, logo não é um campo ESPACIAL"*. Com o eixo espacial,
    /// `value.noise(space = World) → motion.drive(channel = Falloff)` **é** o campo
    /// de ruído: composição, que é o desenho desta biblioteca, e não um nó novo.
    ///
    /// ⚠️ **O TEMPO continua no mesmo eixo `x`**, somado à posição em vez de a
    /// substituí-la: um campo espacial que congelasse ao ganhar `P` perderia o
    /// *"animável"* que a referência (MOPs Noise Falloff) pede no mesmo fôlego.
    fn at_world(&self, px: f32, py: f32, t: f32) -> f32 {
        let y = py * self.frequency + self.seed + self.pan_y;
        self.over_time(
            t,
            |tt| px * self.frequency + tt * self.speed + self.pan_x,
            y,
        )
    }
}

struct ValueNoise;

impl NodeOp for ValueNoise {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let s = Sample::from_ctx(ctx);
        let t = ctx.playhead() as f32;
        // Cardinality follows the geometry; unconnected → one degenerate value.
        let world = ctx.param("space").round() as i32 == 1;
        let input = ctx.input(0);
        let n = input.count().max(1);
        let pos = match input.get("P") {
            Some(Column::Vec2(v)) if world => Some(v.clone()),
            _ => None,
        };
        let v: Vec<f32> = (0..n)
            .map(|i| match pos.as_ref().and_then(|p| p.get(i)) {
                // ⚠️ Sem coluna `P` o modo World CAI no índice, e não em zero: um
                // stream sem posição não tem espaço a amostrar, e um campo que
                // colapsasse num valor só leria como "o ruído morreu".
                Some(p) => s.at_world(p[0], p[1], t),
                None => s.at(i as u32, t),
            })
            .collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(v)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueNoise))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Noise",
            // Utility grey: a value producer, plumbing (not a transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_gates_above(MANIFEST.id, PARAM_GATES_ABOVE);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

use ph2d_node_registry::{
    ParamGate, ParamGateAbove, ParamHardMax, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget,
};

/// **A LEI DA SEGUNDA OITAVA** (doc 90 §1): `roughness` (o `gain`) e `lacunarity` descrevem a
/// relação entre oitavas CONSECUTIVAS, e o `ph2d-fbm` aplica-os depois de somar a oitava
/// corrente — a `octaves = 1` (o default) nenhum dos dois toca a saída. Os irmãos exactos são
/// o `force.wind` e o `motion.wiggle`.
///
/// ⚠️ A tabela `PARAM_GATES` deste nó já escondia `feature`/`jitter` e **esquecia estes dois** —
/// o defeito não era a ausência de mecanismo, era a lista incompleta.
static PARAM_GATES_ABOVE: &[ParamGateAbove] = &[
    ParamGateAbove {
        param: "roughness",
        when: "octaves",
        above: 1.0,
    },
    ParamGateAbove {
        param: "lacunarity",
        when: "octaves",
        above: 1.0,
    },
];

/// **O que o `loop_period` É** (doc 88, Wave A): uma DURAÇÃO, em segundos.
///
/// ⚠️ É a única unidade declarada aqui, e a ausência das outras é deliberada: a
/// magnitude do domínio de VALOR **não tem unidade própria** (o `amplitude` deste
/// nó vale metros em `P`, graus em `rot` e nada em `tint` — o `ParamUnit::None` do
/// registry escreve exactamente isso). O `pan` mede RETICULADO, que não é nenhuma
/// das unidades do vocabulário; uma unidade errada é pior que uma ausente.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "loop_period",
        unit: ParamUnit::Seconds,
    },
    // ⚠️ **A MAGNITUDE, quando o fio cai num PARAM** (doc 58 + doc 88, 2026-08-28) — a mesma
    // lei da irmã `value.lfo`, e a nota acima é a razão dela, não a objecção a ela: a unidade
    // é propriedade do FLUXO, e um param **DIRIGIDO** é um fluxo que termina em UM param
    // declarado, não numa coluna que pode ser qualquer coisa.
    //
    // ⚠️ **O PAR, e a completude é gateada**: a saída é `ruído·amplitude + offset`, homogénea
    // de grau 1 nos dois juntos. ⛔ `frequency`/`speed`/`pan_*` medem o RETICULADO e o
    // `loop_period` mede tempo — nenhum vive na unidade do que o nó emite.
    ParamUnitDecl {
        param: "amplitude",
        unit: ParamUnit::FromWire,
    },
    ParamUnitDecl {
        param: "offset",
        unit: ParamUnit::FromWire,
    },
];

/// **Os dois knobs do celular só existem no celular** (`kernel == 2`). Um
/// controlo que não faz nada é pior que um controlo que falta — e aqui a
/// alternativa é literal: `feature` e `jitter` não são lidos por nenhum dos
/// outros dois kernels (`noise::base` ramifica antes deles).
static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: "feature",
        when: "kernel",
        values: &[2],
    },
    ParamGate {
        param: "jitter",
        when: "kernel",
        values: &[2],
    },
];

/// **Onde uma coordenada de RETICULADO deixa de ser uma coordenada** — `2²³`.
///
/// ⚠️ **MEDIDO, e a medição derrubou a aritmética que eu tinha escrito** (sonda
/// `measure_where_lacunarity_stops_resolving`). O número óbvio seria `2²⁴`, o
/// último inteiro que um `f32` representa; o que de facto morre primeiro é a
/// **parte FRACIONÁRIA**: a partir de `2²³` o ULP de um `f32` é `1.0`, então
/// `x − floor(x)` é zero em TODO ponto, o `fade` devolve 0 e a interpolação
/// colapsa na hash da quina — o ruído deixa de ser coerente uma oitava antes de
/// deixar de ser um número. É este o recurso (**precisão de representação**) e é
/// ele que decide o tecto digitável do `pan`.
const LATTICE_LAST_FRACTIONAL: f32 = 8_388_608.0;

/// **Onde um RELÓGIO deixa de resolver um segundo** — `2²⁴`.
///
/// Um `loop_period` não é uma coordenada de reticulado: ele entra em `t/L` e
/// `u·L`, e nenhum dos dois degenera com `L` grande (com `L` enorme e `t`
/// pequeno o wrap devolve `t` de volta). O que degenera é a régua: acima de
/// `2²⁴` um `f32` não separa dois segundos vizinhos, e um laço cujo comprimento
/// não distingue segundos não é um laço. Recurso diferente, número diferente —
/// e é por isso que são duas constantes e não uma.
const CLOCK_LAST_EXACT_SECOND: f32 = 16_777_216.0;

/// O teto que a MÁQUINA (ou o bom senso) impõe, alcançável por DIGITAÇÃO — o slider fica
/// onde a MÃO trabalha (soft/hard do Blender; doc 88 §11). O curso de antes é este número:
/// nada ficou inalcançável, só deixou de ser o que o dedo percorre.
static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "amplitude",
        max: 100.0,
    },
    ParamHardMax {
        param: "frequency",
        max: 4.0,
    },
    // ⚠️ Um laço mais longo que o slider é legítimo (um take de 90 s fecha em 90 s)
    // e nada no modelo o impede — o que impede é a RÉGUA, e só muito acima.
    ParamHardMax {
        param: "loop_period",
        max: CLOCK_LAST_EXACT_SECOND,
    },
    // O pan mede RETICULADO, e a régua dele é a outra.
    ParamHardMax {
        param: "pan_x",
        max: LATTICE_LAST_FRACTIONAL,
    },
    ParamHardMax {
        param: "pan_y",
        max: LATTICE_LAST_FRACTIONAL,
    },
    // ⚠️ **A `lacunarity` NÃO tem tecto digitável, e a medição é o motivo** — ver
    // o doc-comment do hint. Uma entrada aqui igual ao tecto do slider seria uma
    // linha que não faz nada (`param_hard_max(..).unwrap_or(max)`), que é o
    // controlo morto que este repo não shipa.
];

static PARAM_HINTS: &[ParamUiHint] = &[
    // ⚠️ **Primeiro de todos**, porque é a maior decisão visual do nó: que ruído
    // de base a lei fractal soma. `Pattern` e não `Type` — o `motion.noise` já
    // gasta `Type` na LEI (fBm/Turbulence/Ridged), e a mesma palavra a
    // selecionar coisas diferentes em dois nós irmãos é o que faz um menu mentir.
    ParamUiHint {
        param: "kernel",
        label: "Pattern",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Value", "Perlin", "Cellular"],
        },
    },
    // Os dois seguintes são gateados no Cellular (ver `PARAM_GATES`).
    ParamUiHint {
        param: "feature",
        label: "Cell",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Cells", "Cracks"],
        },
    },
    ParamUiHint {
        param: "jitter",
        label: "Jitter",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **Depois**, e é o que decide o que o nó SIGNIFICA: o mesmo ruído lido
    // ao longo da FILA (o de sempre) ou no ESPAÇO. É o modo World que faz
    // `value.noise → motion.drive(Falloff)` ser um campo procedural (doc 89
    // folha 10, o P0 do `field.noise`).
    ParamUiHint {
        param: "space",
        label: "Sample",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Index", "World"],
        },
    },
    ParamUiHint {
        param: "frequency",
        label: "Frequency",
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "speed",
        label: "Speed",
        min: 0.0,
        max: 8.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "octaves",
        label: "Octaves",
        min: 1.0,
        max: 8.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **A faixa começa em 1** (a mesma do irmão `motion.noise`, pela mesma
    // razão): lacunarity < 1 faz as oitavas ficarem MAIORES que a base, o campo
    // perde a leitura fractal e vira um borrão de baixa frequência.
    //
    // ⚠️ **E o tecto de 4 É o tecto digitável, MEDIDO** (sonda
    // `measure_where_lacunarity_stops_resolving`). A oitava `k` amostra em
    // `x · lacunarityᵏ`, então no topo de uma pilha de 8 ela lê em `x · lac⁷`, e
    // a partir de [`LATTICE_LAST_FRACTIONAL`] aquela oitava **congela**:
    //
    // | lacunarity | topo em `x = 1` | topo em `x = 1000` |
    // |---|---|---|
    // | 2 | 29 de 64 fracções distintas | 25 de 64 |
    // | 4 | 29 de 64 | **1 de 64** (congelado) |
    // | 8 | 4 de 64 | 1 de 64 |
    // | 10 | **1 de 64** | 1 de 64 |
    //
    // ⚠️ **O tecto é função da COORDENADA, e é por isso que não há `ParamHardMax`:**
    // num campo perto da origem ele está em ~9,8 (`2^(23/7)`) e numa grade grande
    // — `i · frequency` com a frequency no tecto — já está em 4. Um único número
    // digitável seria certo numa cena e errado na seguinte; 4 é o valor que se
    // sustenta na PIOR cena que este nó consegue produzir.
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
    // A faixa de um laço é a de um take de motion graphics: 0 (nunca fecha) até
    // 30 s. Aqui o tecto É do painel, e a caixa aceita além dele — o
    // `ParamHardMax` está registrado, ao contrário do que o irmão `motion.noise`
    // afirmava sobre si mesmo.
    ParamUiHint {
        param: "loop_period",
        label: "Loop",
        min: 0.0,
        max: 30.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    // O pan mede RETICULADO — a mesma régua do `seed`, que fica logo abaixo. Uma
    // célula inteira é `1.0`, então ±8 atravessa dezasseis feições do campo, que
    // é mais do que qualquer deslize que se assista.
    ParamUiHint {
        param: "pan_x",
        label: "Pan X",
        min: -8.0,
        max: 8.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "pan_y",
        label: "Pan Y",
        min: -8.0,
        max: 8.0,
        step: 0.01,
        widget: ParamWidget::Slider,
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
        param: "offset",
        label: "Offset",
        min: -100.0,
        max: 100.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 1000.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "space_tests.rs"]
mod space_tests;

#[cfg(test)]
#[path = "time_tests.rs"]
mod time_tests;
