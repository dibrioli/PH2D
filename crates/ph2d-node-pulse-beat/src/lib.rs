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
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Beat",
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
        label: "Period",
        min: 0.05,
        max: 8.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset",
        label: "Offset",
        min: -4.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    // A RÉGUA do mesmo número — `Seconds` é o nó que sempre shipou.
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
    // ⚠️ A faixa é a MESMA do irmão `value.lfo` (20‥300), de propósito: dois nós
    // que falam BPM e discordam sobre onde a mão trabalha ensinam que o número
    // significa coisas diferentes.
    ParamUiHint {
        param: "bpm",
        label: "BPM",
        min: 20.0,
        max: 300.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    // A fase POR LINHA, em segundos. `0` = o metrónomo uniforme de sempre.
    ParamUiHint {
        param: "phase_stagger",
        label: "Phase Stagger",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // Quantas batidas antes de parar. `0` = sem janela (o metrónomo eterno).
    ParamUiHint {
        param: "count",
        label: "Beat Count",
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
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::OpResolver;

    /// Run the metronome over `ticks` frames at 60 Hz, feeding the output back
    /// as `state` (what the `pre` self-loop does). Returns the fired playheads.
    fn beats(ticks: u64, period: f32, offset: f32) -> Vec<f64> {
        let mut state = Stream::new(1);
        let mut fired = Vec::new();
        for i in 0..ticks {
            let t = i as f64 / 60.0;
            state = step(1, &[cycle_index(t, period, offset, 0.0)], 0.0, &state);
            if let Some(Column::Scalar(v)) = state.get(PULSE_COL)
                && v[0] > 0.5
            {
                fired.push(t);
            }
        }
        fired
    }

    /// FALSIFICATION of the edge detection: over one second at 60 Hz with a
    /// 0.5 s period, the metronome fires exactly twice — the start beat and the
    /// one boundary inside the window (t = 0.5). Firing "while inside a cycle"
    /// — the bug the carried cycle index exists to prevent — would fire on all
    /// 60 ticks.
    #[test]
    fn it_beats_on_the_cycle_boundary_not_once_per_tick() {
        let fired = beats(60, 0.5, 0.0);
        assert_eq!(fired, vec![0.0, 0.5], "start beat + one boundary, not 60");
    }

    /// FALSIFICATION of "period is wired": halving the period doubles the beat
    /// count over the same window. A metronome that ignored its period (fixed
    /// rate, or the un-clamped division) could not show this ratio.
    #[test]
    fn halving_the_period_doubles_the_beats() {
        let slow = beats(240, 1.0, 0.0).len(); // 4 s → beats at 0,1,2,3
        let fast = beats(240, 0.5, 0.0).len(); // 4 s → beats at 0,.5,…,3.5
        assert_eq!((slow, fast), (4, 8));
    }

    /// The offset shifts the beat GRID: with `offset 0.3` the boundaries land
    /// at 0.3, 1.3, … The start beat still fires at t = 0 (a metronome bangs
    /// when it starts, wherever it is in the cycle — Max `metro`).
    #[test]
    fn the_offset_shifts_the_beat_grid() {
        let fired = beats(120, 1.0, 0.3);
        assert_eq!(fired.len(), 3);
        assert_eq!(fired[0], 0.0, "the start beat");
        // Boundaries land on the first tick at/after 0.3 and 1.3.
        assert!((fired[1] - 0.3).abs() < 1.0 / 60.0 + 1e-9, "{}", fired[1]);
        assert!((fired[2] - 1.3).abs() < 1.0 / 60.0 + 1e-9, "{}", fired[2]);
    }

    /// A NEGATIVE offset must not fake the start beat away: `beat_primed` is a
    /// separate column, so a first-tick cycle index that happens to be non-zero
    /// (offset −2.5 → k = 2 at t = 0) still reads as "no state yet" → fire.
    /// (A sentinel value inside `beat_cycle` would collide exactly here.)
    #[test]
    fn a_negative_offset_still_fires_the_start_beat() {
        let fired = beats(31, 1.0, -2.5);
        assert_eq!(fired[0], 0.0, "primed flag, not a magic cycle value");
        assert_eq!(fired.len(), 2, "then the 3.0-boundary at t = 0.5");
    }

    /// A degenerate `period ≤ 0` is clamped to `MIN_PERIOD`, never divided by:
    /// the cycle index stays finite and the node keeps ticking (at most one
    /// pulse per tick — a pulse column cannot express more).
    #[test]
    fn a_degenerate_period_is_clamped_not_divided_by_zero() {
        let fired = beats(10, 0.0, 0.0);
        assert!(fired.len() == 10, "every tick crosses ≥1 clamped boundary");
        for t in [0.0, 1.0, -3.0] {
            assert!(cycle_index(t, 0.0, 0.0, 0.0).is_finite());
            assert!(cycle_index(t, -1.0, 0.0, 0.0).is_finite());
        }
    }

    /// The beat is UNIFORM: every instance fires on the same tick (a global
    /// beat — the whole point of replacing the per-channel clock hack).
    #[test]
    fn every_instance_beats_together() {
        let state = step(3, &[0.0; 3], 0.0, &Stream::new(3));
        match state.get(PULSE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![1.0, 1.0, 1.0]),
            _ => panic!(),
        }
    }

    /// The playhead standing still (paused transport re-cooking) does NOT
    /// retrigger: same cycle index, primed state → silence until t moves on.
    #[test]
    fn a_paused_playhead_does_not_retrigger() {
        let mut state = Stream::new(1);
        let mut total = 0.0;
        for _ in 0..5 {
            state = step(1, &[cycle_index(1.25, 1.0, 0.0, 0.0)], 0.0, &state);
            if let Some(Column::Scalar(v)) = state.get(PULSE_COL) {
                total += v[0];
            }
        }
        assert_eq!(total, 1.0, "the start beat only; a held k never refires");
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }

    /// Corre o metrónomo com N linhas e devolve, por linha, os instantes em que ela
    /// disparou — a versão por-fileira do [`beats`].
    fn beats_rows(ticks: u64, n: usize, period: f32, stagger: f32, count: f32) -> Vec<Vec<f64>> {
        let mut state = Stream::new(n);
        let mut fired = vec![Vec::new(); n];
        for i in 0..ticks {
            let t = i as f64 / 60.0;
            let ks: Vec<f32> = (0..n)
                .map(|r| cycle_index(t, period, 0.0, r as f32 * stagger))
                .collect();
            state = step(n, &ks, count, &state);
            if let Some(Column::Scalar(v)) = state.get(PULSE_COL) {
                for (r, p) in v.iter().enumerate() {
                    if *p > 0.5 {
                        fired[r].push(t);
                    }
                }
            }
        }
        fired
    }

    /// **OS DEFAULTS SÃO O METRÓNOMO QUE SEMPRE SHIPOU** — `phase_stagger = 0` e
    /// `count = 0` dão exactamente a mesma lista de batidas, em toda linha.
    #[test]
    fn the_appended_params_default_to_the_metronome_that_shipped() {
        let one = beats(180, 0.5, 0.0);
        for row in beats_rows(180, 4, 0.5, 0.0, 0.0) {
            assert_eq!(row, one, "uma linha divergiu do metronomo uniforme");
        }
    }

    /// **A FASE POR LINHA ESCALONA AS BATIDAS — cada linha no seu tempo.**
    ///
    /// ⚠️ **O oráculo é a DIFERENÇA entre linhas vizinhas, não a lista de uma
    /// linha.** Um `stagger` que fosse lido uma vez e aplicado a todas (o bug de
    /// broadcast que a coluna de estado N-wide convidava) deslocaria as quatro
    /// juntas — a lista de cada uma mudaria, e só a diferença entre elas o apanha.
    #[test]
    fn the_per_row_phase_staggers_the_beats() {
        let stagger = 0.1_f32;
        let rows = beats_rows(240, 4, 1.0, stagger, 0.0);
        assert_eq!(rows.len(), 4, "quatro linhas pedidas, quatro devolvidas");
        for (r, row) in rows.iter().enumerate() {
            assert!(!row.is_empty(), "a linha {r} nao bateu");
        }
        // ⚠️ **O oráculo é a GRELHA de cada linha, não a n-ésima batida dela.** A
        // primeira versão deste gate comparou `rows[r][1]` com `rows[r−1][1]` e
        // reprovou sobre produto correcto: a *batida de partida* (o tique não
        // primado, comum a todas as linhas) desloca a indexação, então a «segunda
        // batida» da linha 1 e a da linha 0 não são a mesma batida da grelha.
        //
        // O que é exacto e não depende de quantas batidas houve: as batidas da
        // linha `r` caem em `r·stagger + k·period`, logo `(batida − r·stagger)` é
        // múltiplo do período — a menos de um quadro de 60 Hz, que é a resolução
        // com que a sonda observa.
        let period = 1.0_f64;
        for (r, row) in rows.iter().enumerate() {
            for &t in row.iter().skip(1) {
                let phase = (t - r as f64 * f64::from(stagger)).rem_euclid(period);
                let off = phase.min(period - phase);
                assert!(
                    off < 1.0 / 60.0 + 1e-9,
                    "linha {r}: a batida em {t} nao esta' na grelha dela ({off} fora)"
                );
            }
        }
        // ⚠️ E o CONTROLE: as quatro linhas TÊM de diferir. Um `stagger` lido uma
        // vez e aplicado a todas (o bug de broadcast que a coluna de estado N-wide
        // convidava) deslocaria as quatro juntas e passaria no teste acima.
        assert!(
            rows.iter().any(|r| *r != rows[0]),
            "as quatro linhas bateram igual — o stagger nao e' por-linha"
        );
    }

    /// **A JANELA PARA O METRÓNOMO DEPOIS DE N BATIDAS — e `0` é sem janela.**
    #[test]
    fn the_window_stops_the_metronome_after_n_beats() {
        // 240 tiques = 4 s; com período 0,5 s são 8 batidas sem janela.
        let free = beats_rows(240, 1, 0.5, 0.0, 0.0);
        assert_eq!(free[0].len(), 8, "sem janela: {:?}", free[0]);
        let limited = beats_rows(240, 1, 0.5, 0.0, 3.0);
        assert_eq!(limited[0].len(), 3, "com janela de 3: {:?}", limited[0]);
        // E as três são as TRÊS PRIMEIRAS — a janela corta o fim, não o meio.
        assert_eq!(limited[0], free[0][..3].to_vec());
    }

    /// **A RÉGUA DE BPM É O MESMO NÚMERO QUE O PERÍODO** — `120 BPM` é meio segundo
    /// por batida, e o gate pina o número que os liga.
    ///
    /// ⚠️ E o piso é o `MIN_PERIOD` que o nó já tinha: um BPM zero dá `inf`, que
    /// **não é NaN** — a grelha congela num compasso infinito, que é a leitura
    /// honesta de *"zero batidas por minuto"*.
    #[test]
    fn bpm_is_the_same_ruler_the_period_uses() {
        assert!((seconds_per_beat(1.0, 999.0, 120.0) - 0.5).abs() < 1e-6);
        assert!((seconds_per_beat(1.0, 999.0, 60.0) - 1.0).abs() < 1e-6);
        // Desligada, a régua devolve o `period` verbatim — o `bpm` nem é olhado.
        assert_eq!(seconds_per_beat(0.0, 0.37, 999.0), 0.37);
        // O degenerado: `inf`, não NaN, e o índice do ciclo continua finito.
        assert!(seconds_per_beat(1.0, 1.0, 0.0).is_infinite());
        assert!(cycle_index(9.0, seconds_per_beat(1.0, 1.0, 0.0), 0.0, 0.0).is_finite());
    }

    /// **O TETO DIGITÁVEL DO PERÍODO ESTÁ MUITO ACIMA DO SLIDER** — o defeito da
    /// folha 12 linha 36, fechado.
    ///
    /// ⚠️ **O que o gate mede não é o número, é a RELAÇÃO:** sem um `ParamHardMax`
    /// o bridge cai no `hint.max`, e o teto do curso da mão vira o teto do produto.
    /// Um período mais lento que 8 s era **indigitável**, e nenhuma referência tem
    /// teto de período nenhum.
    #[test]
    fn the_typed_ceiling_of_the_period_is_far_above_the_slider() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        let hints = reg.param_ui(MANIFEST.id).expect("hints");
        let slider = hints.iter().find(|h| h.param == "period").expect("period");
        let hard = reg
            .param_hard_max(MANIFEST.id, "period")
            .expect("o no' declara um teto digitavel para o period");
        assert!(
            hard > slider.max * 1000.0,
            "o teto digitavel ({hard}) tem de estar MUITO acima do curso da mao ({})",
            slider.max
        );
        // E o curso da mão NÃO mudou: quem arrasta continua onde estava.
        assert!(
            (slider.max - 8.0).abs() < 1e-6,
            "o slider mudou: {}",
            slider.max
        );
    }
}
