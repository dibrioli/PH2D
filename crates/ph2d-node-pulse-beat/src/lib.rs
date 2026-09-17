//! `pulse.beat` — the beat SOURCE: a metronome that emits a PULSE directly from
//! the playhead (Motion Nodes M2, pulse family; handoff doc `09_handoff_pulse_*`).
//!
//! This is the node the family was missing. `pulse.threshold` turns a *signal*
//! into a pulse — but the module had no signal *source*, so the demo faked a
//! clock by oscillating the invisible Rotation channel and thresholding it: two
//! nodes coupled through a transform channel nobody renders (the "clock hack",
//! doc 09 §1). Every mature tool keeps the clock in its utility family instead:
//! MiniCavalry `lfo` (a `pulse` out per cycle), Max `metro`, TouchDesigner Beat
//! CHOP. `pulse.beat` is that source: period in, pulse out — no channel, nothing
//! to mis-wire.
//!
//! **Semantics:** the beat grid is `t = offset + k·period`. Each tick computes
//! the cycle index `k = floor((t − offset)/period)` and fires when `k` differs
//! from the one carried on the `pre` self-loop — the producer-side edge
//! detection shared by the whole family (`pulse.threshold`'s `armed`,
//! `motion.step`'s `count_tick`). The very first primed tick fires too (Max's
//! `metro` bangs on start), so a scene beats the moment it starts playing.
//! `floor` on IEEE doubles is correctly rounded → deterministic (HR-5); no
//! transcendentals anywhere.
//!
//! **Effect::Temporal, deliberately** (a deviation from the doc 09 §4.1 sketch,
//! which copied the threshold's `Pure`): this node reads `ctx.playhead()`, and
//! only a `Temporal` manifest folds the playhead into the memo fingerprint
//! (`cook.rs`). Declaring it `Pure` would let a same-tick re-cook at a moved
//! playhead return a stale beat. The precedent is `motion.oscillator` — reads
//! the playhead → `Temporal`.
//!
//! Uniform across instances (a global beat, `phase_stagger = 0` by nature):
//! every row fires on the same tick. Per-row swing/stagger is a follow-up for
//! when a per-instance value domain exists (doc 09 §4.3).

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The instance stream this node paces: read only for its row count, passed
/// through nowhere — the beat is a pure source, the stream just tells it N.
const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The pulse type (mirror of `ph2d_node_pulse_threshold::PULSE`; kept local so
/// this crate stays a leaf drop-crate — the shared vocabulary is the port
/// `(Instances, Scalar, Event)`, not a shared symbol).
pub const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);

/// The canonical column of a pulse stream: `1.0` on the tick it fires.
pub const PULSE_COL: &str = "pulse";
/// The cycle index `k` carried on the `pre` self-loop (the edge memory).
const CYCLE_COL: &str = "beat_cycle";
/// `1.0` once the loop has carried a real cycle index. Distinguishes "no state
/// yet" (first tick → fire the start beat) from a legitimate `k = 0.0`, for any
/// `offset` — a sentinel value inside `beat_cycle` itself could collide.
const PRIMED_COL: &str = "beat_primed";

/// Shortest honoured period, in seconds. A zero/negative `period` would make
/// the cycle index infinite (division by zero) — clamped here, not honoured.
const MIN_PERIOD: f32 = 1e-3;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("pulse.beat"),
    name: "pulse.beat",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // Feedback: last tick's pulse output carries the cycle index (the edge
        // memory). Named `state` so the editor plumbs its `pre` self-loop on drop.
        PortSpec {
            name: "state",
            ty: PULSE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: PULSE,
    }],
    // Temporal: reads the playhead, so the playhead must gate the memo (see the
    // module doc — the doc 09 sketch said Pure; that would serve stale beats).
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        // Seconds per beat. Clamped to `MIN_PERIOD` at eval.
        ParamSpec {
            name: "period",
            default: 1.0,
        },
        // Phase shift of the beat grid, in seconds: beats land at
        // `offset + k·period`.
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        // ⚠️ **Apendados**: a RÉGUA do mesmo número (`0` = Seconds, o nó que
        // sempre shipou). Ver [`seconds_per_beat`] — e é o mesmo par que os irmãos
        // `value.lfo` e `motion.oscillator` já têm, escrito na régua DESTE nó.
        ParamSpec {
            name: "time_mode",
            default: 0.0,
        },
        ParamSpec {
            name: "bpm",
            default: 120.0,
        },
        // A fase POR LINHA, em segundos. `0` = todas as linhas na mesma batida, o
        // metrónomo uniforme de sempre. ⚠️ O doc-header deste nó confessava isto
        // como *follow-up* enquanto os dois irmãos já o tinham.
        ParamSpec {
            name: "phase_stagger",
            default: 0.0,
        },
        // Quantas batidas o metrónomo dá antes de parar. `0` = **sem janela**, o
        // metrónomo eterno de sempre. Ver [`in_window`].
        ParamSpec {
            name: "count",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// **Segundos por batida, na régua que o artista escolheu** (`time_mode`: `0`
/// segundos, `1` BPM) — a porta única.
///
/// ⚠️ **NÃO é um segundo controlo de velocidade: é a UNIDADE do mesmo número.** O
/// irmão `value.lfo` fala `period` (segundos) e converte `60/bpm`; o
/// `motion.oscillator` fala `frequency` (Hz) e converte `bpm/60`. Este fala
/// `period`, logo é o mesmo `60/bpm` do primeiro — e a folha 12 nomeava o BPM como
/// *"aritmética mental do artista"*, que é precisamente o que uma régua apaga.
///
/// ⚠️ **O piso é o `MIN_PERIOD` que o nó já tinha**, e não um `MIN_BPM` novo: um
/// segundo guarda seria um segundo sítio onde a mesma degenerescência é decidida.
/// Um BPM zero dá `60/0 = inf`, que **não é NaN** — a grelha congela num compasso
/// infinito, que é a leitura honesta de *"zero batidas por minuto"*.
fn seconds_per_beat(mode: f32, period: f32, bpm: f32) -> f32 {
    if mode >= 0.5 { 60.0 / bpm } else { period }
}

/// ⭐⭐⭐ **O KERNEL** (ADR-0126) — a porta WGSL do [`step`], **inteiramente no dispositivo**.
///
/// ⚠️⚠️ **Este é o nó que a medição da W2 nomeou.** Antes dele, a cadeia
/// `grid → beat → sim.spawn(pulse) → output` tinha a fronteira em **`sim.spawn:0`**: o `sim.spawn`
/// TEM kernel e caía na mesma, porque a porta `pulse` dele vinha de um nó que o planeador não
/// podia reivindicar. *Um metrónomo sem kernel não custa o metrónomo: custa a simulação inteira*
/// — o caminho que a [auditoria 98] mede em `50,9×`.
///
/// ⚠️ **Nada é lido da porta 0.** O [`eval`] da CPU só lhe pergunta o COMPRIMENTO, e a saída dele
/// é um stream novo; ligar uma coluna aqui seria o kernel a ler o que a lei não lê.
///
/// ⚠️⚠️ **DIVERGÊNCIA DECLARADA — o índice do ciclo é um `floor`, e o `playhead` do dispositivo é
/// `f32`.** A [`cycle_index`] da CPU divide em `f64` de propósito (*«horas de execução ficam
/// exactas»*); o uniform do módulo gerado carrega o instante em `f32`. As duas rotas só podem
/// discordar quando `t` cai **dentro de um ULP** do instante de uma batida — e aí discordam por um
/// TIQUE inteiro, não por ε, porque a saída é uma comparação (`prev ≠ k`) e uma comparação não tem
/// ε. É a mesma família do *fio da navalha* que o `gpu_cpu_parity_pulse` mede no `pulse.compare`, e
/// a CPU continua a ser o caminho canónico (ADR-0126).
///
/// ⚠️ **Uma linha SEM história dispara**, e é preciso dizê-lo em WGSL: o `prev.get(i) != Some(k)`
/// da CPU é verdadeiro tanto quando o índice mudou como quando a coluna **não existe** — daí o
/// `!HAS_state_beat_cycle` no meio da condição. Sem ele, o primeiro tique de cada linha lia a
/// identidade `0`, e uma cena que arrancasse com `k = 0` nascia **muda**.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let bt_raw = select(params.period, 60.0 / params.bpm, params.time_mode >= 0.5);\n\
        // `f32::max(MIN)` do Rust devolve MIN para NaN; o `max` da WGSL nao promete isso.\n\
        let bt_per = select(BEAT_MIN_PERIOD, bt_raw, bt_raw >= BEAT_MIN_PERIOD);\n\
        let bt_shift = f32(i) * params.phase_stagger;\n\
        let bt_k = floor((params.playhead - params.offset - bt_shift) / bt_per);\n\
        // `primed` e' GLOBAL -- o elemento 0, como o `v.first()` da CPU.\n\
        let bt_primed = read_state_beat_primed(0u) > 0.5;\n\
        let bt_fire = !bt_primed || !HAS_state_beat_cycle\n\
        \x20   || read_state_beat_cycle(i) != bt_k;\n\
        // ⚠️ **`params.count_`, com o sublinhado:** o param deste nó chama-se `count`, que é o
\
        // nome do uniform de CONTAGEM que o módulo gerado já traz — o `wgsl_field` dá ao param
\
        // colidente um campo próprio. Sem o sublinhado o corpo compara um `u32` de elementos com
\
        // `0.5`, e a placa recusa o módulo inteiro (foi assim que este defeito apareceu).
\
        let bt_win = params.count_ < 0.5\n\
        \x20   || (bt_k >= 0.0 && bt_k < bt_round(params.count_));\n\
        write_pulse(i, select(0.0, 1.0, bt_fire && bt_win));\n\
        write_beat_cycle(i, bt_k);\n\
        write_beat_primed(i, 1.0);\n",
    wgsl_lib: "\
        // O gemeo de [`MIN_PERIOD`]. Os dois literais movem-se juntos ou a paridade acusa.\n\
        const BEAT_MIN_PERIOD: f32 = 1e-3;\n\
        // Rust `f32::round` = meio para LONGE do zero; o `round` da WGSL e' meio-par.\n\
        fn bt_round(x: f32) -> f32 {\n\
        \x20   return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: PULSE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: CYCLE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 1,
        },
        ColumnBinding {
            column: PRIMED_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 1,
        },
    ],
    params: &[
        "period",
        "offset",
        "time_mode",
        "bpm",
        "phase_stagger",
        "count",
    ],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **A JANELA DE ATIVIDADE** — a batida de índice `k` conta?
///
/// `count = 0` é **sem janela**: toda batida conta, que é o metrónomo eterno que
/// este nó sempre foi (e o default, logo byte-idêntico).
///
/// ⚠️ **A janela custa UM param e não dois, e é a `offset` que a paga.** A folha
/// pedia *"começa em T, N batidas, para"* — mas `offset` já É o T: o doc do param
/// diz literalmente *"beats land at `offset + k·period`"*, então o índice `k` já é
/// contado a partir dele. Um `start` novo seria uma segunda origem da mesma grelha,
/// a discordar da primeira no dia em que alguém mexesse numa só.
fn in_window(k: f32, count: f32) -> bool {
    if count < 0.5 {
        return true;
    }
    k >= 0.0 && k < count.round()
}

/// The beat-grid cycle index at playhead `t`: `floor((t − offset)/period)`.
/// `f64` division — the playhead is `f64`, and hours of runtime stay exact.
///
/// ⚠️ **`shift` é a fase DESTA LINHA** (`i · phase_stagger`), somada ao `offset`
/// antes da divisão. Com `phase_stagger = 0` ela vale `0` para toda linha e a
/// expressão é literalmente a que estava aqui — o `- 0.0` de um `f64` é exacto.
fn cycle_index(t: f64, period: f32, offset: f32, shift: f32) -> f32 {
    let period = period.max(MIN_PERIOD) as f64;
    ((t - offset as f64 - shift as f64) / period).floor() as f32
}

/// One tick: a row fires iff ITS cycle index moved since the carried one — or on
/// the very first primed tick (the start beat).
///
/// ⚠️ **A decisão passou de UMA para N, e a coluna de estado já era N-wide.** Ela
/// guardava `vec![k; n]` — o mesmo número repetido —, então a memória por-linha não
/// custou coluna nenhuma: custou parar de ler a linha `0` para toda a gente. Com
/// `phase_stagger = 0` os N índices são o mesmo número e o resultado é o de sempre.
fn step(n: usize, ks: &[f32], count: f32, state: &Stream) -> Stream {
    let prev = match state.get(CYCLE_COL) {
        Some(Column::Scalar(v)) => Some(v),
        _ => None,
    };
    let primed = matches!(state.get(PRIMED_COL), Some(Column::Scalar(v)) if v.first().copied().unwrap_or(0.0) > 0.5);
    let pulse: Vec<f32> = (0..n)
        .map(|i| {
            let k = ks[i];
            let fire = if primed {
                // ⚠️ Uma linha SEM história (o stream de estado mais curto que o de
                // geometria — uma cena que ganhou peças a meio) conta como nova, e
                // portanto dispara: é o mesmo *start beat* que a primeira volta dá.
                prev.map(|v| v.get(i).copied()) != Some(Some(k))
            } else {
                true
            };
            f32::from(fire && in_window(k, count))
        })
        .collect();
    Stream::new(n)
        .with(PULSE_COL, Column::Scalar(pulse))
        .with(CYCLE_COL, Column::Scalar(ks.to_vec()))
        .with(PRIMED_COL, Column::Scalar(vec![1.0; n]))
}

struct PulseBeat;

impl NodeOp for PulseBeat {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let period = seconds_per_beat(
            ctx.param("time_mode"),
            ctx.param("period"),
            ctx.param("bpm"),
        );
        let offset = ctx.param("offset");
        let stagger = ctx.param("phase_stagger");
        let count = ctx.param("count");
        let t = ctx.playhead();
        let n = ctx.input(0).count();
        let ks: Vec<f32> = (0..n)
            .map(|i| cycle_index(t, period, offset, i as f32 * stagger))
            .collect();
        let out = step(n, &ks, count, ctx.input(1));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(PulseBeat))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_key: "node.pulse.beat.name",
            // Utility grey: pulse plumbing, not a visible transform.
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    // Só a régua escolhida aparece — o gêmeo exacto do gate do `value.lfo`.
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    // ⚠️ E o teto DIGITÁVEL do período, que é outra grandeza que o curso da mão.
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    Ok(())
}

use ph2d_node_registry::{ParamGate, ParamHardMax, ParamUiHint, ParamWidget};

/// **Só a régua escolhida aparece.** `period` e `bpm` são o MESMO número em duas
/// unidades: mostrar os dois seria pior que um botão morto — dois números na tela
/// a discordar sobre a mesma grandeza, sem nada a dizer qual manda.
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

/// **O TETO DIGITÁVEL do período — e ele existe porque o slider NÃO é um limite.**
///
/// ⚠️ **Isto é um defeito, não uma feature** (folha 12 linha 36, §0 do `CLAUDE.md`):
/// o slider parava em `8 s` e **não havia `ParamHardMax`**, então o bridge caía no
/// `hint.max` e uma batida mais lenta que 8 segundos era **indigitável**. Nenhuma
/// referência tem teto de período, e o número `8` nunca teve medição atrás — ele era
/// o curso confortável da mão, promovido a limite por omissão.
///
/// O teto real é o do TIPO: um `f32` de segundos. `1e6 s` são onze dias e meio de
/// compasso, muito além de qualquer documento, e continua exacto na aritmética de
/// `f64` do playhead. O curso do slider **não muda** — quem arrasta continua a
/// trabalhar em `0,05‥8`.
static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "period",
        max: 1_000_000.0,
    },
    // A janela: o mesmo raciocínio. Contar batidas não tem recurso atrás.
    ParamHardMax {
        param: "count",
        max: 1_000_000.0,
    },
];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "period",
        label: "node.pulse.beat.param.period",
        min: 0.05,
        max: 8.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset",
        label: "node.pulse.beat.param.offset",
        min: -4.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    // A RÉGUA do mesmo número — `Seconds` é o nó que sempre shipou.
    ParamUiHint {
        param: "time_mode",
        label: "node.pulse.beat.param.time_mode",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &[
                "node.pulse.beat.param.time_mode.0",
                "node.pulse.beat.param.time_mode.1",
            ],
        },
    },
    // ⚠️ A faixa é a MESMA do irmão `value.lfo` (20‥300), de propósito: dois nós
    // que falam BPM e discordam sobre onde a mão trabalha ensinam que o número
    // significa coisas diferentes.
    ParamUiHint {
        param: "bpm",
        label: "node.pulse.beat.param.bpm",
        min: 20.0,
        max: 300.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    // A fase POR LINHA, em segundos. `0` = o metrónomo uniforme de sempre.
    ParamUiHint {
        param: "phase_stagger",
        label: "node.pulse.beat.param.phase_stagger",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // Quantas batidas antes de parar. `0` = sem janela (o metrónomo eterno).
    ParamUiHint {
        param: "count",
        label: "node.pulse.beat.param.count",
        min: 0.0,
        max: 64.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
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
static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: "period",
    unit: ParamUnit::Seconds,
}];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
