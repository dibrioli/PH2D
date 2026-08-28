//! `value.lfo` — the value-domain PRODUCER of a continuous oscillation (Motion
//! Nodes M2, the value domain — doc 12). It is the pure *producer* form of
//! `motion.oscillator`: where the oscillator bundles the wave AND writes a
//! transform channel, this emits the wave as a **value** on its own socket, to be
//! routed by `motion.drive` (or reshaped by `value.map_range`, gated by
//! `pulse.sample_hold`, …). This is the TouchDesigner **LFO CHOP** / the Cavalry
//! oscillator behaviour, made a first-class value source (doc 12 §5).
//!
//! **The value type** is the continuous per-instance scalar field
//! `(Instances, Scalar, Frame)` on the `v` column — the continuous dual of the
//! pulse (doc 12). Cardinality follows the geometry: the optional `in` port is
//! read for its **count only** (like the oscillator reads N from its stream) —
//! connected → a length-N field with per-instance `phase_stagger` (a travelling
//! wave across the grid); **unconnected → a length-1 field** (one global
//! oscillation, held across every instance by `motion.drive`'s broadcast rule).
//! Nothing from the input stream is passed through — this mints a fresh value.
//!
//! Reads the playhead, holds no state → `Effect::Temporal` (pull-side, like the
//! oscillator). The waveform math is transcendental-free (HR-5); see `wave.rs`.
//!
//! `value_i = waveform(wave, t / period + phase + i · phase_stagger) · amplitude
//! + offset`, with `period` clamped to `MIN_PERIOD` (never divides by zero).

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, CountLawCtx, GpuKernel, SourceWindow};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod wave;
use wave::waveform;

/// The instance stream type — read for its count only (the optional `in` port).
const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_pulse_counter::VALUE`; kept local so this stays a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value output column (the canonical `value`-domain column).
const VALUE_COL: &str = "v";
/// The smallest cycle length, so `t / period` never divides by zero (mirror of
/// `pulse.beat`'s guard).
const MIN_PERIOD: f32 = 1e-3;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.lfo"),
    name: "value.lfo",
    // Optional: connected → count N + per-instance stagger; unconnected → one
    // global value. Read for its count only; never passed through.
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    // Reads the playhead → pull-side; HR-5-exempt for the clock (the waveform
    // math is nonetheless transcendental-free for cross-platform stability).
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        // Shape — 0 Sine (parabolic) · 1 Tri · 2 Square · 3 Saw · 4 Spike.
        ParamSpec {
            name: "wave",
            default: 0.0,
        },
        // Seconds per cycle (clamped ≥ MIN_PERIOD). The pulse-family vocabulary
        // (matches `pulse.beat`'s `period`) rather than the oscillator's frequency.
        ParamSpec {
            name: "period",
            default: 1.0,
        },
        // Peak of the oscillation (value-native units).
        ParamSpec {
            name: "amplitude",
            default: 1.0,
        },
        // A DC shift of the oscillation centre.
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        // A global phase offset (cycles) — where in the cycle it starts.
        ParamSpec {
            name: "phase",
            default: 0.0,
        },
        // Per-instance phase offset (cycles) → a travelling wave across the field
        // (needs a connected `in` for N > 1; 0 → every instance in lock-step).
        ParamSpec {
            name: "phase_stagger",
            default: 0.0,
        },
        // ⚠️ **Apendados**: a RÉGUA do mesmo número. `0` = Seconds, o nó que
        // sempre shipou. Ver [`seconds_per_cycle`].
        ParamSpec {
            name: "time_mode",
            default: 0.0,
        },
        ParamSpec {
            name: "bpm",
            default: 120.0,
        },
        // ⚠️ **Apendado**: a rampa de ENTRADA, em segundos. `0` = sem rampa, o nó
        // que sempre shipou (`x · 1.0` é `x` em IEEE-754). Ver [`fade_envelope`].
        ParamSpec {
            name: "fade_in",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// **A rampa de entrada da oscilação** — `clamp(t / fade_in, 0, 1)`, e `1` fixo
/// quando não há rampa.
///
/// Cavalry chama-lhe *Strength Fade to Zero*, e o que ele resolve é concreto: uma
/// LFO **começa no meio do movimento**. Uma senoide a `phase = 0` vale zero, mas a
/// sua DERIVADA é máxima — o elemento arranca a toda a velocidade no primeiro
/// quadro. Uma quadrada é pior: ela vale `+amplitude` no instante zero, e o objeto
/// **salta** antes de qualquer coisa se mexer. A rampa multiplica a amplitude, não
/// o valor: o `offset` (o centro da oscilação) fica onde está, e o que cresce do
/// nada é o **desvio** em torno dele.
///
/// ⚠️ **A origem é a do PLAYHEAD, e é deliberadamente a mesma que a fase usa.**
/// O nó não guarda estado (`Effect::Temporal`, pull-side) e portanto não tem uma
/// noção de *"desde que EU comecei"* — dizer que tem seria inventar um relógio que
/// o undo teria de ordenar. Ao partilhar a origem com a fase, a rampa e a onda
/// começam juntas, que é o que o artista vê e o que ele quer.
///
/// ⚠️ **`fade_in ≤ 0` devolve `1.0` por um RAMO, não pela álgebra:** `t / 0` é
/// `inf` (que o clamp salvaria) mas `0 / 0` é **NaN**, e um NaN no instante zero
/// envenenaria o primeiro quadro de todo documento já autorado — precisamente o
/// caso em que o param novo deveria ser invisível.
fn fade_envelope(t: f32, fade_in: f32) -> f32 {
    if fade_in <= 0.0 {
        1.0
    } else {
        (t / fade_in).clamp(0.0, 1.0)
    }
}

/// **Segundos por ciclo, na régua que o artista escolheu** (`time_mode`: `0`
/// segundos, `1` BPM) — a porta única, e a razão de ela existir.
///
/// ⚠️ **NÃO é um segundo controlo de velocidade: é a UNIDADE do mesmo número**, a
/// família do px/m da Wave A. O irmão `motion.oscillator` já tem exactamente este
/// par (`time_mode` + `bpm`, doc 88) e escreve-o na régua DELE — ele fala
/// `frequency` (Hz), então converte `bpm/60`; este fala `period` (segundos), então
/// converte `60/bpm`. **São recíprocos, e é isso que os torna a mesma grandeza:**
/// 120 BPM é 2 ciclos por segundo lá e meio segundo por ciclo aqui, e o gate
/// `bpm_is_the_same_ruler_the_oscillator_uses` pina o número que os liga.
///
/// ⚠️ **O piso é o `MIN_PERIOD` que o nó já tinha**, e não um `MIN_BPM` novo: um
/// segundo guarda seria um segundo lugar onde a mesma degenerescência é decidida.
/// Um BPM zero dá `60/0 = inf`, que **não é NaN** — `t/inf` é `0`, a fase congela
/// e a saída fica finita; um BPM negativo cai no piso e é a onda mais rápida que
/// o nó representa. Nenhum dos dois produz um valor não-finito, que é o que o
/// gate afirma.
fn seconds_per_cycle(mode: f32, period: f32, bpm: f32) -> f32 {
    let s = if mode >= 0.5 { 60.0 / bpm } else { period };
    s.max(MIN_PERIOD)
}

/// GPU compute kernel (ADR-0126) — the WGSL port of [`wave::waveform`], element
/// for element.
///
/// **This node is the first that does NOT ride its input through.** The engine's
/// default is `out = base + written columns`, which is what every instance
/// deformer wants; this one takes instances and emits a **VALUE** stream — one
/// `v` column and nothing else. The sequencer derives that from the manifest
/// (port 0 in-type vs out-type differ), so the kernel declares nothing extra —
/// and riding the base would have handed downstream a VALUE stream still
/// carrying `P`/`Index`, which the CPU's does not have.
///
/// `lfo_round` is round-half-away-from-zero to match Rust's `f32::round`: `wave`
/// picks a BRANCH, so the two conventions would choose different waveforms at a
/// half-integer ([[feedback_cpu_gpu_rounding_conventions_diverge]]). The period
/// takes the same `MIN_PERIOD` floor as the CPU — a zero period is a divide.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        // A REGUA do mesmo numero (`time_mode`: 0 segundos, 1 BPM) -- o gemeo de\n\
        // `seconds_per_cycle`. O piso e' o MIN_PERIOD que ja' existia: um BPM zero\n\
        // da' `inf`, e `t/inf` e' 0 (fase congelada, valor finito), nao NaN.\n\
        var lfo_sec = params.period;\n\
        if (params.time_mode >= 0.5) { lfo_sec = 60.0 / params.bpm; }\n\
        let lfo_period = max(lfo_sec, 1e-3);\n\
        let lfo_phase = params.playhead / lfo_period + params.phase\n\
        \x20   + f32(i) * params.phase_stagger;\n\
        // A rampa de entrada -- o gemeo de `fade_envelope`. O ramo (e nao a\n\
        // algebra) porque `0.0 / 0.0` e' NaN, e o instante zero e' o caso comum.\n\
        var lfo_env = 1.0;\n\
        if (params.fade_in > 0.0) {\n\
        \x20   lfo_env = clamp(params.playhead / params.fade_in, 0.0, 1.0);\n\
        }\n\
        let lfo_v = lfo_wave(i32(lfo_round(params.wave)), lfo_phase)\n\
        \x20   * params.amplitude * lfo_env + params.offset;\n\
        write_v(i, lfo_v);\n",
    wgsl_lib: "\
        fn lfo_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn lfo_wave(kind: i32, phase: f32) -> f32 {\n\
            let f = phase - floor(phase);\n\
            if (kind == 1) {\n\
                if (f < 0.25) { return 4.0 * f; }\n\
                if (f < 0.75) { return 2.0 - 4.0 * f; }\n\
                return 4.0 * f - 4.0;\n\
            }\n\
            if (kind == 2) {\n\
                if (f < 0.5) { return 1.0; }\n\
                return -1.0;\n\
            }\n\
            if (kind == 3) { return 2.0 * f - 1.0; }\n\
            if (kind == 4) {\n\
                if (f < 0.08) { return 1.0; }\n\
                return 0.0;\n\
            }\n\
            // Parabolic sine + Capens 2nd-order correction (HR-5, no sin).\n\
            var p: f32;\n\
            if (f < 0.5) {\n\
                let u = f * 2.0;\n\
                p = 4.0 * u * (1.0 - u);\n\
            } else {\n\
                let u = (f - 0.5) * 2.0;\n\
                p = -4.0 * u * (1.0 - u);\n\
            }\n\
            return 0.225 * (p * abs(p) - p) + p;\n\
        }\n",
    bindings: &[ColumnBinding {
        column: VALUE_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::Write,
        identity: [0.0; 4],
        port: 0,
    }],
    params: &[
        "wave",
        "period",
        "amplitude",
        "offset",
        "phase",
        "phase_stagger",
        // ⚠️ Esta lista não é derivada do manifesto: um param novo compila, coza na
        // CPU, e o device recusa o shader (`invalid field accessor`).
        "time_mode",
        "bpm",
        "fade_in",
    ],
    count_law: Some(lfo_count),
    variant_by_param: None,
    applicable: None,
};

/// **How wide is the field?** — the same expression `eval` uses, and the reason
/// the count law exists at all.
///
/// Connected, this is one oscillation per instance (a travelling wave across the
/// grid); **unconnected it is ONE global value**, held across every instance by
/// `motion.drive`'s broadcast rule. The engine's default law — "as wide as port
/// 0" — gets the connected case right and the unconnected one silently wrong:
/// an empty port is `0`, a zero-count stage is SKIPPED, and the whole `value.*`
/// family would be unreachable on the device the moment something consumed it.
fn lfo_count(c: &CountLawCtx<'_>) -> SourceWindow {
    SourceWindow::of_count(c.inputs.first().copied().unwrap_or(0).max(1) as usize)
}

struct ValueLfo;

impl NodeOp for ValueLfo {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let wave = ctx.param("wave").round() as i32;
        let period = seconds_per_cycle(
            ctx.param("time_mode"),
            ctx.param("period"),
            ctx.param("bpm"),
        );
        let amplitude = ctx.param("amplitude");
        let offset = ctx.param("offset");
        let phase0 = ctx.param("phase");
        let stagger = ctx.param("phase_stagger");
        let t = ctx.playhead() as f32;
        let env = fade_envelope(t, ctx.param("fade_in"));
        // Cardinality follows the geometry: N from the (optional) input, else the
        // length-1 global oscillation (broadcast by `motion.drive`).
        let n = ctx.input(0).count().max(1);
        let value: Vec<f32> = (0..n)
            .map(|i| {
                let phase = t / period + phase0 + i as f32 * stagger;
                waveform(wave, phase) * amplitude * env + offset
            })
            .collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(value)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueLfo))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "LFO",
            // Utility grey: a value SOURCE, plumbing (not a visible transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_hard_min(MANIFEST.id, PARAM_HARD_MIN);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    Ok(())
}

use ph2d_node_registry::{ParamGate, ParamHardMax, ParamHardMin, ParamUiHint, ParamWidget};

/// **Os tetos DIGITÁVEIS deste oscilador, MEDIDOS** — bloco Z, doc 91.
///
/// ⚠️ **Este nó é o mais acusado do repo: QUATRO valores em três cenas.** A `=12` autora
/// `period = 9` e `14`, a `=15` autora `amplitude = 180` e `offset = 180`, a `=9` e a `=15`
/// autoram `period = 12` — sobre arrastos que param em `8` e `10`. Sem entrada aqui o digitado
/// para no fim do ARRASTO (`ui.rs:206`), então o app publicava ondas que o artista não conseguia
/// escrever. Acusação da sonda `what_the_corpus_authors_and_no_one_can_type`.
///
/// **O recurso é a PRECISÃO** (`CLAUDE.md` §0.0): nenhum dos três satura — um período longo é
/// uma onda lenta, uma amplitude grande é uma onda alta, e as duas são respostas —, então o que
/// acaba é o `f32`: acima daqui somar o `step` do slider (0,05) **não move o número**. Derivado
/// a cada corrida pelo gate `every_precision_bound_param_types_to_the_measured_ceiling`.
static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "period",
        max: 1_048_576.0 - 0.0625,
    },
    ParamHardMax {
        param: "amplitude",
        max: 1_048_576.0 - 0.0625,
    },
    ParamHardMax {
        param: "offset",
        max: 1_048_576.0 - 0.0625,
    },
];

/// O piso do `offset`, e ele existe porque o `offset` **tem sinal**.
///
/// ⚠️ Um teto generoso com o piso de ontem deixaria metade do gesto inalcançável, e uma onda que
/// só se consegue levantar lê-se como bug do nó. O `period` e o `amplitude` ficam de fora: o
/// piso deles é do DESENHO (um período `≤ 0` não é uma onda), não da representação.
static PARAM_HARD_MIN: &[ParamHardMin] = &[ParamHardMin {
    param: "offset",
    min: -(1_048_576.0 - 0.0625),
}];

/// **Só a régua escolhida aparece.**
///
/// `period` e `bpm` são o MESMO número em duas unidades, então mostrar os dois
/// seria pior que um botão morto: dois números na tela que **discordam entre si**
/// sobre a mesma grandeza, sem nada dizendo qual manda. É verbatim a decisão que o
/// irmão `motion.oscillator` tomou para o par `frequency`/`bpm`.
static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: "period",
        when: "time_mode",
        values: &[0],
    },
    ParamGate {
        param: "bpm",
        when: "time_mode",
        values: &[1],
    },
];

static PARAM_HINTS: &[ParamUiHint] = &[
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
        param: "period",
        label: "Period",
        min: 0.05,
        max: 8.0,
        step: 0.05,
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
        param: "phase_stagger",
        label: "Stagger",
        min: 0.0,
        max: 2.0,
        step: 0.02,
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
    // A faixa de um BPM é a de uma música, não a de um período: 20 é um *largo*
    // muito lento e 300 passa o topo de qualquer género. É a mesma faixa do irmão
    // `motion.oscillator`, e é a mesma pelo mesmo motivo — uma faixa 0,05..8 aqui
    // (a do `period`) faria o slider inteiro caber entre 0 e 8 batidas por minuto.
    ParamUiHint {
        param: "bpm",
        label: "BPM",
        min: 20.0,
        max: 300.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    // A rampa de ENTRADA, em segundos. `0` = sem rampa (o nó de sempre); acima
    // disso a oscilação cresce do nada até à amplitude cheia.
    ParamUiHint {
        param: "fade_in",
        label: "Fade In",
        min: 0.0,
        max: 5.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
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
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "period",
        unit: ParamUnit::Seconds,
    },
    // A rampa é um TEMPO, e por isso é declarada — o irmão `bpm` não é (é uma
    // taxa) e continua nu de propósito.
    ParamUnitDecl {
        param: "fade_in",
        unit: ParamUnit::Seconds,
    },
    // ⚠️ **A MAGNITUDE, quando o fio cai num PARAM** (doc 58 + doc 88, 2026-08-28).
    //
    // A nota acima continua certa e é a razão desta: *"o `amplitude` deste nó vale metros em
    // `P`, graus em `rot` e nada em `tint`"* — a unidade é propriedade do FLUXO, não do nó. É
    // exactamente por isso que um param **DIRIGIDO** pode responder: ali o fluxo não termina
    // numa coluna (que pode ser qualquer coisa), termina em **UM param declarado com UMA
    // unidade declarada**, e o grafo sabe qual. A lacuna que a nota preferia a um número errado
    // só é honesta enquanto o destino é desconhecido.
    //
    // ⚠️ **Os DOIS juntos, e a completude é gateada** (`the_from_wire_set_is_the_output_scale`):
    // a saída é `w(fase)·amplitude·env + offset`, homogénea de grau 1 no PAR. Declarar só um
    // deles seria meia unidade — e o gate reprova, porque escalar o conjunto declarado deixaria
    // de escalar a saída.
    //
    // ⛔ O `period`/`fade_in` (segundos), o `bpm` (taxa), a `phase` (fracção do ciclo) e o
    // `phase_stagger` ficam de fora: nenhum deles vive na unidade do que o nó emite.
    ParamUnitDecl {
        param: "amplitude",
        unit: ParamUnit::FromWire,
    },
    ParamUnitDecl {
        param: "offset",
        unit: ParamUnit::FromWire,
    },
];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
