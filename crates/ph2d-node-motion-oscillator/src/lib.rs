#![forbid(unsafe_code)]
//! `motion.oscillator` — a Motion **behaviour**: oscillates a chosen channel of
//! the stream over the playhead, **added** to the existing value and scaled
//! per-instance by the multiplicative `falloff` column (§1.2; absent → `1.0`).
//! Each instance samples the waveform at `phase = t·frequency + i·phase_stagger`,
//! so a non-zero `phase_stagger` sends a travelling wave across the grid. Reads
//! the playhead but holds no state → `Effect::Temporal` (pull-side). Every other
//! column passes through unchanged (count preserved).
//!
//! Waveforms are **transcendental-free** (HR-5): `phase` is measured in *cycles*
//! (unit period) and the shapes are piecewise polynomial. The "Sine" wave is a
//! parabolic approximation with a 2nd-order correction (Capens/devmaster) — ~0.09%
//! off a true sine using only multiply + abs — since a real `sin` is
//! non-deterministic (plan §1.7).
//!
//! Params (read via `ctx.param`):
//! - `channel` (1): target — `0` X, `1` Y, `2` Rotation, `3` Size.
//! - `wave` (0): shape — `0` Sine (parabolic), `1` Triangle, `2` Square, `3` Saw,
//!   `4` Spike (a narrow unipolar pulse).
//! - `amplitude` (1): peak of the oscillation (channel-native units).
//! - `frequency` (1): cycles per second of playhead.
//! - `phase_stagger` (0.1): per-instance phase offset (cycles) → the travelling wave.
//! - `offset` (0): a DC shift of the oscillation centre.
//! - `phase` (0): a global phase offset (cycles) — where in the cycle it starts.
//!
//! `delta_i = (wave(t·frequency + i·phase_stagger + phase)·amplitude + offset)·falloff_i`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod gpu;
use gpu::GPU_KERNEL;
mod params_ui;
use params_ui::{PARAM_GATES, PARAM_GROUPS, PARAM_HARD_MAX, PARAM_HINTS, PARAM_UNITS};
mod channel;
use channel::{apply_channel_delta, falloff_at};
use ph2d_nodegraph::attr::par_build;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.oscillator"),
    name: "motion.oscillator",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Reads the playhead → pull-side, HR-5-exempt for the clock (the waveform
    // math is nonetheless transcendental-free for cross-platform stability).
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "channel",
            default: 1.0,
        },
        ParamSpec {
            name: "wave",
            default: 0.0,
        },
        ParamSpec {
            name: "amplitude",
            default: 1.0,
        },
        ParamSpec {
            name: "frequency",
            default: 1.0,
        },
        ParamSpec {
            name: "phase_stagger",
            default: 0.1,
        },
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        ParamSpec {
            name: "phase",
            default: 0.0,
        },
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

/// The fractional part of `p` in `[0,1)` — IEEE `floor` is correctly-rounded and
/// deterministic (HR-5-safe, unlike `sin`).
fn frac(p: f32) -> f32 {
    p - p.floor()
}

/// A periodic waveform at `phase` (in cycles, period 1) — bipolar `[-1,1]` except
/// **Spike** (a unipolar `[0,1]` pulse). All shapes are piecewise polynomial →
/// transcendental-free (HR-5). Unknown / `0` is the parabolic sine-approximation.
fn waveform(kind: i32, phase: f32) -> f32 {
    let f = frac(phase);
    match kind {
        1 => {
            // Triangle: 0 at 0, +1 at ¼, 0 at ½, −1 at ¾.
            if f < 0.25 {
                4.0 * f
            } else if f < 0.75 {
                2.0 - 4.0 * f
            } else {
                4.0 * f - 4.0
            }
        }
        2 => {
            // Square: +1 first half, −1 second.
            if f < 0.5 { 1.0 } else { -1.0 }
        }
        3 => 2.0 * f - 1.0, // Saw: −1 → +1 rising.
        4 => {
            // Spike: a narrow unipolar pulse at the cycle start (a periodic kick).
            const SPIKE_WIDTH: f32 = 0.08;
            if f < SPIKE_WIDTH { 1.0 } else { 0.0 }
        }
        _ => {
            // Parabolic sine-approximation: a +hump over [0,½), a −hump over
            // [½,1), each `±4u(1−u)` — continuous, 0 at 0/½, ±1 at ¼/¾.
            let p = if f < 0.5 {
                let u = f * 2.0;
                4.0 * u * (1.0 - u)
            } else {
                let u = (f - 0.5) * 2.0;
                -4.0 * u * (1.0 - u)
            };
            // 2nd-order correction (Capens/devmaster): the bare parabola is ~5.6%
            // off a true sine (visibly rounder at the crest); `0.225·(p·|p|−p)+p`
            // drops that to ~0.09% using only multiply + abs (transcendental-free,
            // HR-5). Endpoint/range-preserving: 0→0, ±1→±1, stays in [-1,1].
            const Q: f32 = 0.225;
            Q * (p * p.abs() - p) + p
        }
    }
}

/// **Ciclos por segundo, na régua que o artista escolheu** (`time_mode`: `0` segundos,
/// `1` BPM).
///
/// ⚠️ Isto NÃO é um segundo multiplicador de frequência — é a UNIDADE do mesmo número, a
/// mesma família do px/m da Wave A. A distinção importa porque o Cavalry também traz um
/// *Time Scale*, e esse **não foi construído de propósito**: sem uma porta de tempo externa,
/// `sin(2π·(s·t)·f) ≡ sin(2π·t·(s·f))`, ou seja Time Scale É Frequency por identidade
/// algébrica — um knob que não pode mudar nada que o outro não mude.
fn cycles_per_second(mode: f32, frequency: f32, bpm: f32) -> f32 {
    if mode >= 0.5 { bpm / 60.0 } else { frequency }
}

// ⛔ **O `fade` foi construído e REMOVIDO no mesmo dia** (smoke do Enio, doc 88 B3) — não
// reconstrua sem ler isto. Era o *Strength Fade to Zero* do Cavalry portado como uma rampa em
// segundos a partir do zero ABSOLUTO do playhead, e tinha três defeitos, o primeiro MEDIDO:
//
// 1. **Expirava.** O slider ia até 10 s, então a partir de ~10 s de relógio TODO valor da
//    faixa entregava amplitude zero — o estado permanente do controle era *expirado*, e na
//    tela isso lê como *"o oscilador travou tudo"*, que foi o report.
// 2. **A régua era invisível.** No Cavalry o fade desvanece ao longo da DURAÇÃO DA
//    COMPOSIÇÃO — uma janela com começo e fim na régua. Aqui a janela começava num zero que
//    o artista não vê e terminava num instante que ele não vê; a duração da composição **não
//    existe neste nível** (`EvalCtx` oferece `playhead`/`dt`/`started`, e nada mais).
// 3. **Era uma SEGUNDA PORTA.** `ctx.param` resolve **wire > override > default**, então
//    `value.time → value.map_range → amplitude` já É um fade — com a régua VISÍVEL no grafo,
//    o começo e o fim escolhidos pelo artista, e o painel marcando o param como *driven*.
//
// ⚠️ O `time_mode`/`bpm` FICA, e a distinção é o que separa os dois: uma UNIDADE não expira.
// BPM é a mesma frequência noutra régua e vale igual no segundo 0 e no segundo 600.
//
// O gate `no_control_of_this_oscillator_expires_with_the_clock` guarda a CLASSE, não este
// campo: qualquer knob futuro cuja unidade seja *"segundos desde um zero que ninguém vê"*
// nasce vermelho nele.

struct MotionOscillator;

impl NodeOp for MotionOscillator {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let channel = ctx.param("channel").round() as i32;
        let wave = ctx.param("wave").round() as i32;
        let phase_stagger = ctx.param("phase_stagger");
        let offset = ctx.param("offset");
        let phase0 = ctx.param("phase");
        let t = ctx.playhead() as f32;
        let cps = cycles_per_second(
            ctx.param("time_mode"),
            ctx.param("frequency"),
            ctx.param("bpm"),
        );
        let amplitude = ctx.param("amplitude");
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            // Pure per-instance map → parallel above the threshold (bit-identical,
            // no reduction). GPU/M5 Fase 0.
            let deltas: Vec<f32> = par_build(n, |i| {
                let phase = t * cps + i as f32 * phase_stagger + phase0;
                // DC `offset` shifts the oscillation centre; the whole
                // contribution is falloff-masked (like every behaviour).
                (waveform(wave, phase) * amplitude + offset) * falloff_at(input, i)
            });
            apply_channel_delta(input, channel, &deltas)
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionOscillator))?;
    // M1.R1 — UI metadata. Behaviours modify transform channels → Transform
    // (blue) for now; a dedicated Behaviour category (cyan) is a follow-up.
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Oscillator",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    // GPU/M5 Fase 1 (ADR-0126): the WGSL lowering, registered on the side.
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params_ui::PARAM_HINTS;
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

    // Source: 2 instances at the origin (so the oscillation IS the output).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.oscillator.test.src"),
        name: "motion.oscillator.test.src",
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
    struct Src;
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionOscillator),
                _ => None,
            }
        }
    }

    fn osc_y_at(playhead: f64, setup: impl FnOnce(&mut Graph, NodeId)) -> Vec<[f32; 2]> {
        let mut g = Graph::new();
        let src = g.add_node("motion.oscillator.test.src");
        let osc = g.add_node("motion.oscillator");
        g.connect(Edge {
            from: (src, 0),
            to: (osc, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(osc, "channel", 1.0); // Y
        setup(&mut g, osc);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, osc, playhead).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }

    #[test]
    fn at_playhead_zero_the_oscillation_is_neutral() {
        // The default parabolic wave is 0 at phase 0; with phase_stagger 0 every
        // instance is at phase 0 → no displacement (deterministic origin).
        let p = osc_y_at(0.0, |g, osc| {
            g.set_param(osc, "amplitude", 5.0);
            g.set_param(osc, "phase_stagger", 0.0);
        });
        assert_eq!(p, vec![[0.0, 0.0], [0.0, 0.0]]);
    }

    #[test]
    fn quarter_cycle_parabolic_reaches_peak_amplitude() {
        // frequency 1, t = 0.25 → phase ¼ → parabolic peak +1 → Δy = +amplitude.
        let p = osc_y_at(0.25, |g, osc| {
            g.set_param(osc, "amplitude", 3.0);
            g.set_param(osc, "phase_stagger", 0.0);
        });
        assert_eq!(p, vec![[0.0, 3.0], [0.0, 3.0]]);
    }

    #[test]
    fn phase_stagger_offsets_later_instances() {
        // t=0, phase_stagger 0.25: instance 0 at phase 0 (→0), instance 1 at
        // phase ¼ (parabolic peak +1) → only the second instance displaces.
        let p = osc_y_at(0.0, |g, osc| {
            g.set_param(osc, "amplitude", 2.0);
            g.set_param(osc, "phase_stagger", 0.25);
        });
        assert_eq!(p, vec![[0.0, 0.0], [0.0, 2.0]]);
    }

    /// **BPM é a MESMA frequência noutra régua** — `120 BPM ≡ 2 Hz`, ao bit.
    ///
    /// O gate é uma IGUALDADE entre as duas rotas, não um número escolhido: é isso que torna
    /// `time_mode` uma unidade em vez de um segundo multiplicador. A mutação que troca o
    /// divisor sangra aqui e em lugar nenhum mais.
    #[test]
    fn bpm_is_the_same_frequency_in_another_ruler() {
        let hz = osc_y_at(0.3, |g, osc| {
            g.set_param(osc, "amplitude", 4.0);
            g.set_param(osc, "phase_stagger", 0.0);
            g.set_param(osc, "frequency", 2.0);
        });
        let bpm = osc_y_at(0.3, |g, osc| {
            g.set_param(osc, "amplitude", 4.0);
            g.set_param(osc, "phase_stagger", 0.0);
            g.set_param(osc, "time_mode", 1.0);
            g.set_param(osc, "bpm", 120.0);
        });
        assert_eq!(hz, bpm, "120 BPM tem de ser 2 Hz, ao bit");
        // E o controle: a régua escolhida MANDA — em BPM o `frequency` não é lido.
        let ignored = osc_y_at(0.3, |g, osc| {
            g.set_param(osc, "amplitude", 4.0);
            g.set_param(osc, "phase_stagger", 0.0);
            g.set_param(osc, "time_mode", 1.0);
            g.set_param(osc, "bpm", 120.0);
            g.set_param(osc, "frequency", 7.0);
        });
        assert_eq!(bpm, ignored, "em BPM o slider de Hz nao pode ter voto");
    }

    /// **A régua de tempo é o que o WGSL porta** — pinada aqui para os dois lados serem lidos
    /// lado a lado quando alguém mexer num deles.
    #[test]
    fn the_time_ruler_the_shader_ports() {
        assert_eq!(cycles_per_second(0.0, 3.0, 999.0), 3.0);
        assert_eq!(cycles_per_second(1.0, 999.0, 120.0), 2.0);
    }

    /// SONDA: a EXCURSÃO do oscilador ao longo do relógio — a grandeza que o gate vigia.
    /// `cargo test -p ph2d-node-motion-oscillator measure_the_excursion -- --ignored --nocapture`
    #[test]
    #[ignore = "sonda de medição"]
    fn measure_the_excursion_over_the_playhead() {
        println!("\n=== EXCURSAO pico-a-pico (amplitude 4.0, janela de 6 s) ===");
        for t0 in [0.0f64, 10.0, 30.0, 60.0, 600.0] {
            let (mut lo, mut hi) = (f32::MAX, f32::MIN);
            for k in 0..240 {
                let v = osc_y_at(t0 + f64::from(k) * 0.025, |g, osc| {
                    g.set_param(osc, "amplitude", 4.0);
                    g.set_param(osc, "phase_stagger", 0.0);
                })[0][1];
                lo = lo.min(v);
                hi = hi.max(v);
            }
            println!("  t0={t0:>6}: {:>6.3}", hi - lo);
        }
        println!();
    }

    /// **Um oscilador é PERIÓDICO: a excursão dele não pode depender de QUANDO você olha.**
    ///
    /// Nasceu VERMELHO sobre o `fade` (smoke do Enio, *"Fade out > 0 trava as shapes"*): ele era
    /// uma rampa a partir do zero ABSOLUTO do playhead, com o slider indo até 10 — medido, a
    /// partir de ~10 s de relógio TODO valor da faixa entregava amplitude zero, e o estado
    /// permanente do controle era *expirado*.
    ///
    /// O gate varre a FAIXA DECLARADA de cada param e compara a excursão pico-a-pico de uma
    /// janela cedo contra uma tarde. É de propósito que ele não nomeia `fade`: o que ele
    /// proíbe é a CLASSE — qualquer knob futuro cuja unidade seja *"segundos desde um zero que
    /// o artista não vê"* nasce vermelho aqui.
    #[test]
    fn no_control_of_this_oscillator_expires_with_the_clock() {
        // Excursão pico-a-pico numa janela de 6 s (contém um ciclo até no BPM mínimo, 20 =
        // 0,33 Hz), amostrada fino o bastante para pegar a crista.
        let excursion = |t0: f64, setup: &dyn Fn(&mut Graph, NodeId)| -> f32 {
            let (mut lo, mut hi) = (f32::MAX, f32::MIN);
            for k in 0..240 {
                let t = t0 + f64::from(k) * 0.025;
                let v = osc_y_at(t, |g, osc| setup(g, osc))[0][1];
                lo = lo.min(v);
                hi = hi.max(v);
            }
            hi - lo
        };
        for h in PARAM_HINTS {
            // Três pontos da faixa declarada: as duas pontas e o meio.
            for frac in [0.0f32, 0.5, 1.0] {
                let v = h.min + (h.max - h.min) * frac;
                let setup = move |g: &mut Graph, osc: NodeId| {
                    g.set_param(osc, "amplitude", 4.0);
                    g.set_param(osc, "phase_stagger", 0.0);
                    g.set_param(osc, h.param, v);
                };
                let early = excursion(0.0, &setup);
                let late = excursion(60.0, &setup);
                assert!(
                    (early - late).abs() <= 1e-3,
                    "{} = {v}: a excursao caiu de {early} para {late} entre t=0 e t=60 -- este \
                     controle EXPIRA com o relogio, e o artista nao ve a regua dele",
                    h.param
                );
            }
        }
    }

    #[test]
    fn waveforms_stay_in_range_and_are_periodic() {
        // Every shape is bounded to [-1,1] (Spike to [0,1]) and repeats per cycle.
        for kind in 0..=4 {
            for step in 0..40 {
                let p = step as f32 * 0.1;
                let v = waveform(kind, p);
                assert!((-1.0..=1.0).contains(&v), "wave {kind} at {p} = {v}");
                assert!(
                    (waveform(kind, p) - waveform(kind, p + 1.0)).abs() < 1e-5,
                    "wave {kind} periodic at {p}"
                );
            }
        }
        // Anchor points of the corrected sine approximation (preserved).
        assert_eq!(waveform(0, 0.0), 0.0);
        assert_eq!(waveform(0, 0.25), 1.0);
        assert_eq!(waveform(0, 0.75), -1.0);
        // Spike: a narrow unipolar pulse — 1 at the cycle start, 0 through most.
        assert_eq!(waveform(4, 0.0), 1.0);
        assert_eq!(waveform(4, 0.5), 0.0);
    }

    #[test]
    fn offset_shifts_the_centre_and_phase_advances_the_cycle() {
        // A DC `offset` moves the oscillation centre: default Sine at t=0 is 0, so
        // with offset 2 the instances sit at +2.
        let p = osc_y_at(0.0, |g, osc| {
            g.set_param(osc, "phase_stagger", 0.0);
            g.set_param(osc, "offset", 2.0);
        });
        assert_eq!(p, vec![[0.0, 2.0], [0.0, 2.0]]);
        // A global `phase` of ¼ starts the cycle at the peak (like advancing t):
        // amplitude 3, phase ¼ → Δy = +3 at t=0.
        let q = osc_y_at(0.0, |g, osc| {
            g.set_param(osc, "phase_stagger", 0.0);
            g.set_param(osc, "amplitude", 3.0);
            g.set_param(osc, "phase", 0.25);
        });
        assert_eq!(q, vec![[0.0, 3.0], [0.0, 3.0]]);
    }

    #[test]
    fn corrected_sine_beats_the_bare_parabola() {
        use std::f32::consts::TAU;
        // Compare against a true sine (std::sin — test-only, not in the cook) at
        // 64 phases. The corrected wave is well under 0.5% error everywhere and
        // strictly closer than the bare parabola (which peaks ~5.6% off).
        let bare = |f: f32| {
            if f < 0.5 {
                let u = f * 2.0;
                4.0 * u * (1.0 - u)
            } else {
                let u = (f - 0.5) * 2.0;
                -4.0 * u * (1.0 - u)
            }
        };
        let mut worst_corrected = 0.0f32;
        let mut worst_bare = 0.0f32;
        for k in 0..64 {
            let f = k as f32 / 64.0;
            let truth = (f * TAU).sin();
            worst_corrected = worst_corrected.max((waveform(0, f) - truth).abs());
            worst_bare = worst_bare.max((bare(f) - truth).abs());
        }
        assert!(
            worst_corrected < 0.005,
            "corrected worst = {worst_corrected}"
        );
        assert!(worst_bare > 0.04, "bare parabola should be visibly worse");
        assert!(worst_corrected < worst_bare);
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
