//! Gates do `motion.oscillator` — a onda, a régua do tempo, a largura do pulso
//! e a FAIXA (`Min`/`Max`), com a armadilha do `Spike` que ela cura.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o
//! corte é o que a crate já usa no irmão `skew_tests.rs`: a LEI no `lib.rs`, as
//! PROVAS num arquivo por assunto. `#[path]` mantém o módulo a chamar-se `tests`.

use super::*;
use crate::params_ui::PARAM_HINTS;
use ph2d_node_registry::ParamWidget;
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
            let v = waveform(kind, p, 0.5, None);
            assert!((-1.0..=1.0).contains(&v), "wave {kind} at {p} = {v}");
            assert!(
                (waveform(kind, p, 0.5, None) - waveform(kind, p + 1.0, 0.5, None)).abs() < 1e-5,
                "wave {kind} periodic at {p}"
            );
        }
    }
    // Anchor points of the corrected sine approximation (preserved).
    assert_eq!(waveform(0, 0.0, 0.5, None), 0.0);
    assert_eq!(waveform(0, 0.25, 0.5, None), 1.0);
    assert_eq!(waveform(0, 0.75, 0.5, None), -1.0);
    // Spike: a narrow unipolar pulse — 1 at the cycle start, 0 through most.
    assert_eq!(waveform(4, 0.0, 0.5, None), 1.0);
    assert_eq!(waveform(4, 0.5, 0.5, None), 0.0);
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
        worst_corrected = worst_corrected.max((waveform(0, f, 0.5, None) - truth).abs());
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

// ─── A SEXTA FORMA: a que o artista DESENHA (doc 89, folha 06) ───

/// A curva das provas — um V invertido: sobe até ao meio e desce. `V(f) = 2f` até
/// `½` e `2(1−f)` depois, então ela vale `0,5` em `f = 0,25` e `1,0` em `f = 0,5`.
const CURVE_V: &str = "c1 0:0:L 0.5:1:L 1:0:L";

/// **A ONDA CUSTOM É A CURVA AUTORADA, E SEM CURVA É A SERRA.**
///
/// ⚠️ As duas metades são precisas, e uma só passaria com o ramo meio-feito: sem a
/// primeira, um `Custom` que ignorasse o text param leria a identidade e ninguém
/// notava; sem a segunda, um `Custom` que devolvesse `0` numa curva ausente seria um
/// **controle morto** — a forma escolhida no dropdown e nada a mexer-se —, que é
/// exactamente o que o `fade` deste nó custou uma vez.
#[test]
fn the_custom_wave_is_the_authored_shape_and_an_unset_curve_is_the_saw() {
    // t = 0,25 com frequency 1 e pulse_width neutro ⇒ a fase é `f = 0,25`.
    let bare = osc_y_at(0.25, |g, osc| {
        g.set_param(osc, "wave", WAVE_CUSTOM as f32);
        g.set_param(osc, "amplitude", 4.0);
        g.set_param(osc, "phase_stagger", 0.0);
    });
    // A identidade: `y = f = 0,25` ⇒ 0,25 × 4 = 1,0.
    assert_eq!(
        bare[0][1], 1.0,
        "curva ausente = a serra, nunca um zero morto"
    );

    let drawn = osc_y_at(0.25, |g, osc| {
        g.set_param(osc, "wave", WAVE_CUSTOM as f32);
        g.set_param(osc, "amplitude", 4.0);
        g.set_param(osc, "phase_stagger", 0.0);
        g.set_text_param(osc, CURVE_KEY, CURVE_V);
    });
    // `V(0,25) = 0,5` ⇒ 0,5 × 4 = 2,0.
    assert_eq!(drawn[0][1], 2.0, "a onda desenhada É a curva autorada");

    // ⚠️ E o CONTROLE: a curva só morde a forma `Custom`. Autorada sobre a `Sine`,
    // ela não pode mover um número — senão o text param seria um segundo canal
    // secreto a agir sobre as cinco formas de sempre.
    let sine_plain = osc_y_at(0.25, |g, osc| {
        g.set_param(osc, "amplitude", 4.0);
        g.set_param(osc, "phase_stagger", 0.0);
    });
    let sine_with_curve = osc_y_at(0.25, |g, osc| {
        g.set_param(osc, "amplitude", 4.0);
        g.set_param(osc, "phase_stagger", 0.0);
        g.set_text_param(osc, CURVE_KEY, CURVE_V);
    });
    assert_eq!(
        sine_plain, sine_with_curve,
        "uma curva autorada nao toca nas formas enumeradas"
    );
}

/// **A CUSTOM DECLARA A PRÓPRIA POLARIDADE, SENÃO O `Min`/`Max` ENTREGA METADE.**
///
/// ⚠️ Este gate é o herdeiro directo da armadilha do `Spike` (folha 06, 22/08): a
/// aritmética que toda a gente faz de cabeça — `amp = (max−min)/2`, `off = (min+max)/2` —
/// assume a onda BIPOLAR. A `Custom` é unipolar `[0,1]` (o quadrado do editor), e com a
/// conta bipolar a faixa pedida `[−2, 3]` sairia como **`[0,5, 3,0]`**: metade da
/// excursão, com o piso levantado ao CENTRO, sem que um número do painel mudasse.
///
/// Ele varre um ciclo inteiro e mede os extremos que o nó de facto entrega.
#[test]
fn the_custom_wave_declares_its_own_polarity_so_min_max_delivers_the_asked_range() {
    const ASKED_MIN: f32 = -2.0;
    const ASKED_MAX: f32 = 3.0;
    let sample = |t: f64| {
        osc_y_at(t, |g, osc| {
            g.set_param(osc, "wave", WAVE_CUSTOM as f32);
            g.set_param(osc, "phase_stagger", 0.0);
            g.set_param(osc, "range_mode", 1.0);
            g.set_param(osc, "min", ASKED_MIN);
            g.set_param(osc, "max", ASKED_MAX);
        })[0][1]
    };
    // A serra percorre `[0,1]` ao longo do ciclo, então os extremos da faixa
    // aparecem dentro de uma varredura de um período (frequency = 1).
    const STEPS: u32 = 1000;
    let mut lo = f32::INFINITY;
    let mut hi = f32::NEG_INFINITY;
    for k in 0..=STEPS {
        let v = sample(f64::from(k) / f64::from(STEPS));
        lo = lo.min(v);
        hi = hi.max(v);
    }
    // ⚠️ **A tolerância sai da RESOLUÇÃO DA VARREDURA, não do gosto** — e é uma
    // propriedade da serra, não um defeito: `frac(1,0) = 0,0`, então `f = 1` NUNCA é
    // amostrado dentro de um período e o tecto é aproximado por baixo, no máximo um
    // passo. Um passo de fase vale `(max − min)/STEPS` na saída.
    let step = (ASKED_MAX - ASKED_MIN) / STEPS as f32;
    let tol = step * 1.5; // um passo + folga de ULP
    assert!(
        (lo - ASKED_MIN).abs() < tol,
        "o piso tem de ser o pedido ({ASKED_MIN}), deu {lo} — a conta bipolar daria 0,5"
    );
    assert!(
        (hi - ASKED_MAX).abs() < tol,
        "e o tecto o pedido ({ASKED_MAX}), deu {hi} (tolerancia {tol}, um passo da varredura)"
    );
    // ⚠️ E o que este gate de facto REJEITA: a conta bipolar poria o piso em `+0,5`.
    // Sem esta linha, uma tolerância generosa deixaria de distinguir as duas contas.
    // A conta bipolar: `amp = (max−min)/2`, `off = (min+max)/2`. Sobre uma onda que
    // só produz `[0,1]`, o piso dela é o próprio `off` — o CENTRO da faixa pedida.
    let bipolar_floor = (ASKED_MIN + ASKED_MAX) * 0.5;
    assert!(
        (lo - bipolar_floor).abs() > 1.0,
        "o piso medido ({lo}) tem de estar LONGE do que a conta bipolar daria ({bipolar_floor})"
    );
    // O controle POSITIVO: a `natural_range` de facto responde diferente para esta
    // forma. Sem ele, o gate passaria numa implementação que devolvesse `[-1,1]`
    // para tudo e por acaso acertasse a excursão desta fixture.
    assert_eq!(natural_range(WAVE_CUSTOM), (0.0, 1.0));
    assert_eq!(natural_range(0), (-1.0, 1.0));
}

/// **O DROPDOWN OFERECE A FORMA NOVA** — a costura entre a lei e o painel.
///
/// ⚠️ Um `WAVE_CUSTOM` que a `eval` entende e o painel não oferece é um gesto
/// inalcançável; um rótulo a mais é um botão que escolhe uma forma que não existe.
/// O gate mede os DOIS lados contra a mesma fonte.
#[test]
fn the_wave_dropdown_offers_exactly_the_shapes_the_law_knows() {
    let row = PARAM_HINTS
        .iter()
        .find(|h| h.param == "wave")
        .expect("a linha da onda");
    let ParamWidget::Enum { labels } = row.widget else {
        panic!("a onda é um seletor nomeado, nunca um slider")
    };
    assert_eq!(
        labels.len() as i32 - 1,
        WAVE_CUSTOM,
        "a ultima etiqueta é a Custom"
    );
    assert_eq!(labels[WAVE_CUSTOM as usize], "Custom");
    assert!(
        (row.max - WAVE_CUSTOM as f32).abs() < f32::EPSILON,
        "o teto do seletor é o indice da ultima forma"
    );
    // E a linha da CURVA existe, senão a forma nova nasce sem como ser desenhada.
    let curve_row = PARAM_HINTS
        .iter()
        .find(|h| h.param == CURVE_KEY)
        .expect("a linha da curva");
    assert!(matches!(curve_row.widget, ParamWidget::Curve));
}
