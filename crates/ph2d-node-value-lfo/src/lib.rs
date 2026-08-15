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
    ],
    lowerings: &[LoweringKind::Cpu],
};

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
        let lfo_v = lfo_wave(i32(lfo_round(params.wave)), lfo_phase)\n\
        \x20   * params.amplitude + params.offset;\n\
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
        // Cardinality follows the geometry: N from the (optional) input, else the
        // length-1 global oscillation (broadcast by `motion.drive`).
        let n = ctx.input(0).count().max(1);
        let value: Vec<f32> = (0..n)
            .map(|i| {
                let phase = t / period + phase0 + i as f32 * stagger;
                waveform(wave, phase) * amplitude + offset
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
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    Ok(())
}

use ph2d_node_registry::{ParamGate, ParamUiHint, ParamWidget};

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
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

    // A grid source: `n` instances at the origin, so the LFO can read a count.
    static GRID_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.lfo.test.grid"),
        name: "value.lfo.test.grid",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: INST_VEC2,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Grid;
    impl NodeOp for Grid {
        fn manifest(&self) -> &'static NodeManifest {
            &GRID_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0]; 3])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == GRID_MAN.id => Some(&Grid),
                t if t == MANIFEST.id => Some(&ValueLfo),
                _ => None,
            }
        }
    }

    fn vals(s: &Stream) -> Vec<f32> {
        match s.get(VALUE_COL).unwrap() {
            Column::Scalar(v) => v.clone(),
            _ => panic!("v"),
        }
    }

    /// Cook the LFO at `playhead`; `connect_grid` decides whether it reads a
    /// count from a source (length-N) or stands alone (length-1 global).
    fn lfo_at(
        playhead: f64,
        connect_grid: bool,
        setup: impl FnOnce(&mut Graph, NodeId),
    ) -> Vec<f32> {
        let mut g = Graph::new();
        let lfo = g.add_node("value.lfo");
        if connect_grid {
            let grid = g.add_node("value.lfo.test.grid");
            g.connect(Edge {
                from: (grid, 0),
                to: (lfo, 0),
                delayed: false,
            })
            .unwrap();
        }
        setup(&mut g, lfo);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, lfo, playhead).unwrap();
        vals(out[0].as_stream())
    }

    /// UNCONNECTED input → one global value (length-1). This is the field that
    /// `motion.drive` broadcasts across every instance (the doc-12 rule).
    #[test]
    fn an_unconnected_lfo_emits_a_single_global_value() {
        // Default Sine at t=0 is 0 (phase 0). amplitude 5 → still 0 at the zero.
        let v = lfo_at(0.0, false, |g, lfo| {
            g.set_param(lfo, "amplitude", 5.0);
        });
        assert_eq!(v, vec![0.0], "one global value");
        // A quarter period reaches the parabolic peak → +amplitude.
        let v = lfo_at(0.5, false, |g, lfo| {
            g.set_param(lfo, "period", 2.0); // t=0.5 → phase ¼
            g.set_param(lfo, "amplitude", 3.0);
        });
        assert_eq!(v, vec![3.0], "quarter period → peak amplitude");
    }

    /// CONNECTED input → the value field's length follows the geometry (N=3),
    /// and `phase_stagger` sends a travelling wave across it: at t=0 with
    /// stagger ¼, instance 0 sits at phase 0 (→0) and instance 1 at the peak.
    #[test]
    fn a_connected_lfo_emits_a_field_with_a_travelling_wave() {
        let v = lfo_at(0.0, true, |g, lfo| {
            g.set_param(lfo, "amplitude", 2.0);
            g.set_param(lfo, "phase_stagger", 0.25);
        });
        assert_eq!(v.len(), 3, "length follows the connected geometry");
        assert_eq!(v[0], 0.0, "instance 0 at phase 0");
        assert_eq!(v[1], 2.0, "instance 1 staggered to the peak");
    }

    /// FALSIFICATION of the clamp/DC path: `offset` shifts the centre and
    /// `phase` advances the cycle without touching the playhead.
    #[test]
    fn offset_shifts_the_centre_and_phase_advances_the_cycle() {
        let v = lfo_at(0.0, false, |g, lfo| g.set_param(lfo, "offset", 2.0));
        assert_eq!(v, vec![2.0], "DC offset with no oscillation");
        // phase ¼ at t=0 starts the cycle at the peak.
        let v = lfo_at(0.0, false, |g, lfo| {
            g.set_param(lfo, "amplitude", 3.0);
            g.set_param(lfo, "phase", 0.25);
        });
        assert_eq!(v, vec![3.0], "phase advances to the peak");
    }

    /// A tiny/zero `period` must not divide by zero — it clamps to `MIN_PERIOD`
    /// and stays finite.
    #[test]
    fn a_zero_period_never_divides_by_zero() {
        let v = lfo_at(1.0, false, |g, lfo| g.set_param(lfo, "period", 0.0));
        assert!(v[0].is_finite(), "clamped period keeps the value finite");
    }

    /// **A régua BPM é a mesma grandeza noutra unidade** — e o número que a prova é
    /// o que liga este nó ao irmão `motion.oscillator`.
    ///
    /// Ele fala Hz e converte `bpm/60`; este fala segundos-por-ciclo e converte
    /// `60/bpm`. **120 BPM ⇒ 2 ciclos/s lá ⇒ 0,5 s por ciclo aqui**, e os dois são
    /// recíprocos exactos — é isso que torna a palavra "BPM" a mesma palavra nos
    /// dois nós em vez de duas convenções que se parecem.
    ///
    /// ⚠️ O gate vive AQUI e não num teste cruzado: uma crate-nó não pode depender
    /// de outra crate-nó (drop-crate, ADR-0075), então o que se pina é o número, e
    /// o doc nomeia o irmão.
    #[test]
    fn bpm_is_the_same_ruler_the_oscillator_uses() {
        // A conversão, isolada da onda.
        assert_eq!(seconds_per_cycle(1.0, 999.0, 120.0), 0.5, "120 BPM = 0,5 s");
        assert_eq!(seconds_per_cycle(1.0, 999.0, 60.0), 1.0, "60 BPM = 1 s");
        // E o recíproco do irmão: 120 BPM = 2 ciclos por segundo.
        assert_eq!(1.0 / seconds_per_cycle(1.0, 999.0, 120.0), 120.0 / 60.0);
        // ⚠️ CONTROLE: em Seconds o `bpm` é INERTE — sem isto, um modo que
        // ignorasse o `time_mode` e lesse sempre o BPM passaria nas linhas acima.
        assert_eq!(
            seconds_per_cycle(0.0, 0.25, 999.0),
            0.25,
            "Seconds ignora o BPM"
        );
    }

    /// **O default é o mundo anterior, ao bit** — a régua nova não move um valor
    /// enquanto ninguém a escolhe.
    ///
    /// O oráculo é a expressão que SHIPAVA, escrita à mão: chamar
    /// `seconds_per_cycle` para computar o que se espera dela seria o gate
    /// sempre-verde que este repo já documentou três vezes.
    #[test]
    fn seconds_is_byte_identical_to_the_world_before_the_ruler() {
        for period in [0.05f32, 0.25, 1.0, 2.5, 8.0, 0.0, -3.0] {
            let want = period.max(MIN_PERIOD);
            let got = seconds_per_cycle(0.0, period, 120.0);
            assert_eq!(got.to_bits(), want.to_bits(), "period {period}");
        }
        // E pelo cook, no caminho real: o valor de sempre com os params novos nos
        // defaults do manifesto.
        let v = lfo_at(0.5, false, |g, lfo| {
            g.set_param(lfo, "period", 2.0);
            g.set_param(lfo, "amplitude", 3.0);
        });
        assert_eq!(v, vec![3.0], "quarto de período → pico, como antes");
    }

    /// **Um BPM degenerado nunca produz um valor não-finito.** O irmão deste gate
    /// é `a_zero_period_never_divides_by_zero`, e o mecanismo é OUTRO: ali o
    /// divisor é o param, aqui o param é o dividendo — `60/0` é `inf`, e o que
    /// tem de ser provado é que `t/inf` sai finito em vez de NaN.
    #[test]
    fn a_degenerate_bpm_never_produces_a_non_finite_value() {
        for bpm in [0.0f32, -1.0, -1e30, 1e30] {
            let v = lfo_at(3.25, false, |g, lfo| {
                g.set_param(lfo, "time_mode", 1.0);
                g.set_param(lfo, "bpm", bpm);
            });
            assert!(v[0].is_finite(), "bpm {bpm} → {v:?}");
        }
        // E o caso ZERO é a fase CONGELADA, não uma onda: o mesmo valor em dois
        // instantes distintos.
        let frozen = |t: f64| {
            lfo_at(t, false, |g, lfo| {
                g.set_param(lfo, "time_mode", 1.0);
                g.set_param(lfo, "bpm", 0.0);
            })
        };
        assert_eq!(frozen(0.0), frozen(9.75), "bpm 0 congela a fase");
    }

    /// **A régua BPM anda o relógio** — o modo não é só uma etiqueta.
    #[test]
    fn the_bpm_ruler_drives_the_wave() {
        // 120 BPM = 0,5 s por ciclo ⇒ em t = 0,125 s a fase é ¼ ⇒ o pico.
        let v = lfo_at(0.125, false, |g, lfo| {
            g.set_param(lfo, "time_mode", 1.0);
            g.set_param(lfo, "bpm", 120.0);
            g.set_param(lfo, "amplitude", 3.0);
        });
        assert_eq!(v, vec![3.0], "120 BPM põe o pico em t = 1/8 s");
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
