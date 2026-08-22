//! Gates do `motion.oscillator` — a onda, a régua do tempo, a largura do pulso
//! e a FAIXA (`Min`/`Max`), com a armadilha do `Spike` que ela cura.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o
//! corte é o que a crate já usa no irmão `skew_tests.rs`: a LEI no `lib.rs`, as
//! PROVAS num arquivo por assunto. `#[path]` mantém o módulo a chamar-se `tests`.

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
            let v = waveform(kind, p, 0.5);
            assert!((-1.0..=1.0).contains(&v), "wave {kind} at {p} = {v}");
            assert!(
                (waveform(kind, p, 0.5) - waveform(kind, p + 1.0, 0.5)).abs() < 1e-5,
                "wave {kind} periodic at {p}"
            );
        }
    }
    // Anchor points of the corrected sine approximation (preserved).
    assert_eq!(waveform(0, 0.0, 0.5), 0.0);
    assert_eq!(waveform(0, 0.25, 0.5), 1.0);
    assert_eq!(waveform(0, 0.75, 0.5), -1.0);
    // Spike: a narrow unipolar pulse — 1 at the cycle start, 0 through most.
    assert_eq!(waveform(4, 0.0, 0.5), 1.0);
    assert_eq!(waveform(4, 0.5, 0.5), 0.0);
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
        worst_corrected = worst_corrected.max((waveform(0, f, 0.5) - truth).abs());
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

/// Varre um ciclo inteiro e devolve `[piso, tecto]` do deslocamento em Y da
/// primeira peça — o que a onda de facto ENTREGA.
/// Quantos passos a varredura dá num ciclo — e o número que DERIVA a barra dos
/// gates de faixa abaixo.
const SWEEP: usize = 400;

/// A folga que uma varredura finita obriga, em unidades da faixa pedida.
///
/// ⚠️ **Ela é DERIVADA e não escolhida, e o `Saw` é a razão.** A dente de serra
/// tem o topo ABERTO — ela é `2f − 1` com `f ∈ [0, 1)`, então aproxima-se de
/// `+1` e salta sem lá chegar. Nenhuma varredura de passo finito a apanha no
/// máximo, e apertar a barra até ela passar seria pedir ao produto o que a
/// aritmética proíbe. Dois passos de folga cobrem o pior caso com margem.
fn sweep_tolerance(min: f32, max: f32) -> f32 {
    (max - min).abs() * 2.0 / SWEEP as f32
}

fn swing_over_a_cycle(setup: impl Fn(&mut Graph, NodeId) + Copy) -> [f32; 2] {
    let mut lo = f32::INFINITY;
    let mut hi = f32::NEG_INFINITY;
    for k in 0..=SWEEP {
        let t = k as f64 / SWEEP as f64;
        let p = osc_y_at(t, |g, n| {
            g.set_param(n, "phase_stagger", 0.0);
            setup(g, n);
        });
        lo = lo.min(p[0][1]);
        hi = hi.max(p[0][1]);
    }
    [lo, hi]
}

/// **`range_mode = 0` É O NÓ QUE SEMPRE SHIPOU — BIT-A-BIT, NAS CINCO ONDAS.**
///
/// ⚠️ O `min`/`max` é posto num valor NÃO-neutro de propósito: com os defaults
/// (`-1`/`1`) um vazamento seria invisível nas quatro ondas bipolares.
#[test]
fn the_amplitude_ruler_is_the_node_that_shipped_in_every_wave() {
    for wave in 0..=4 {
        let before = osc_y_at(0.3, |g, n| {
            g.set_param(n, "wave", wave as f32);
            g.set_param(n, "amplitude", 2.5);
            g.set_param(n, "offset", 0.4);
        });
        let after = osc_y_at(0.3, |g, n| {
            g.set_param(n, "wave", wave as f32);
            g.set_param(n, "amplitude", 2.5);
            g.set_param(n, "offset", 0.4);
            g.set_param(n, "range_mode", 0.0);
            g.set_param(n, "min", 7.0);
            g.set_param(n, "max", 11.0);
        });
        assert_eq!(
            before, after,
            "onda {wave}: a regua desligada mudou a saida"
        );
    }
}

/// **A FAIXA PEDIDA É A FAIXA ENTREGUE — EXACTAMENTE, NAS CINCO ONDAS.**
///
/// ⚠️ Aqui o oráculo pode ser EXACTO (ao contrário do irmão `motion.noise`):
/// uma onda periódica visita os extremos dela dentro de um ciclo, então varrer
/// um ciclo mede a excursão inteira. É o gate que a folha 06 pedia — *"trocar
/// de onda muda a faixa em silêncio"* deixa de ser verdade.
#[test]
fn the_range_you_ask_for_is_the_range_you_get_in_every_wave() {
    let (min, max) = (2.0f32, 6.0f32);
    for wave in 0..=4 {
        let [lo, hi] = swing_over_a_cycle(move |g, n| {
            g.set_param(n, "wave", wave as f32);
            g.set_param(n, "range_mode", 1.0);
            g.set_param(n, "min", min);
            g.set_param(n, "max", max);
        });
        let tol = sweep_tolerance(min, max);
        assert!(
            (lo - min).abs() < tol && (hi - max).abs() < tol,
            "onda {wave}: pedi [{min}, {max}], recebi [{lo}, {hi}] (tol {tol})"
        );
    }
}

/// **A ARMADILHA DO SPIKE, MEDIDA — e o knob é o que a apaga.**
///
/// ⚠️ **É o controle que separa o produto correto de um que só compila.** A
/// conta de cabeça (`amplitude = (max−min)/2`, `offset = (min+max)/2`) acerta
/// nas quatro ondas bipolares e, no `Spike`, entrega **metade da excursão com o
/// piso levantado ao centro** — `[4, 6]` onde o painel diz `[2, 6]`. Sem esta
/// metade, o gate acima passaria com uma implementação que ignorasse a
/// polaridade, e o `Spike` seria o único a mentir — em silêncio.
#[test]
fn the_head_arithmetic_halves_the_spike_and_the_knob_restores_it() {
    let (min, max) = (2.0f32, 6.0f32);
    let tol = sweep_tolerance(min, max);
    let by_head = |wave: i32| {
        swing_over_a_cycle(move |g, n| {
            g.set_param(n, "wave", wave as f32);
            g.set_param(n, "amplitude", (max - min) * 0.5);
            g.set_param(n, "offset", (min + max) * 0.5);
        })
    };
    // Nas bipolares a conta de cabeça está CERTA — e o gate diz isso, senão
    // estaria a acusar o artista de um erro que ele não comete.
    for wave in 0..=3 {
        let [lo, hi] = by_head(wave);
        assert!(
            (lo - min).abs() < tol && (hi - max).abs() < tol,
            "onda {wave}: a conta de cabeca devia acertar, deu [{lo}, {hi}]"
        );
    }
    // No Spike ela erra por METADE, e o erro é no PISO.
    let [lo, hi] = by_head(4);
    assert!((hi - max).abs() < tol, "o TOPO acerta: {hi}");
    assert!(
        (lo - (min + max) * 0.5).abs() < tol,
        "o piso sobe ao CENTRO da faixa: {lo}"
    );
    // E o knob devolve o piso ao sítio.
    let [lo_fixed, hi_fixed] = swing_over_a_cycle(move |g, n| {
        g.set_param(n, "wave", 4.0);
        g.set_param(n, "range_mode", 1.0);
        g.set_param(n, "min", min);
        g.set_param(n, "max", max);
    });
    assert!(
        (lo_fixed - min).abs() < tol && (hi_fixed - max).abs() < tol,
        "com o knob: [{lo_fixed}, {hi_fixed}]"
    );
}
