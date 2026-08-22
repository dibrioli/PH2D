//! Gates do `value.lfo` — a onda, a régua do tempo (`time_mode`/`bpm`) e a rampa
//! de entrada (`fade_in`).
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o
//! corte é o que a casa já usa noutros nós (`value.pattern`): o `lib.rs` fica com a
//! LEI e o irmão com as PROVAS. Os caminhos não mudam — `#[path]` mantém o módulo
//! a chamar-se `tests` e `use super::*` resolve como sempre resolveu.

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
fn lfo_at(playhead: f64, connect_grid: bool, setup: impl FnOnce(&mut Graph, NodeId)) -> Vec<f32> {
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

/// **SEM RAMPA, O ENVELOPE E' `1` -- e NUNCA NaN.**
///
/// ⚠️ **A mutacao que este gate mata e' escrever so' o clamp** (`(t/fade).
/// clamp(0,1)`): com `fade = 0` e `t = 0` isso e' `0/0 = NaN`, e o clamp de
/// `f32` propaga NaN. O instante zero e' o primeiro quadro de TODO documento,
/// entao o param novo apagaria o no' exactamente para quem nunca lhe tocou.
#[test]
fn with_no_fade_the_envelope_is_one_and_never_nan() {
    for &fade in &[0.0, -1.0, -0.0] {
        for k in -10..40 {
            let t = k as f32 * 0.25;
            let e = fade_envelope(t, fade);
            assert_eq!(e, 1.0, "t={t} fade={fade}");
        }
    }
    // O caso exacto: `0 / 0`.
    assert_eq!(fade_envelope(0.0, 0.0), 1.0);
}

/// **A RAMPA SOBE DO ZERO ATE' 1 E FICA LA'** -- monotona, ancorada nos dois
/// extremos, e saturada depois do fim.
#[test]
fn the_ramp_rises_from_zero_to_one_and_stays() {
    let fade = 2.0;
    assert_eq!(fade_envelope(0.0, fade), 0.0, "no instante zero, nada");
    assert_eq!(fade_envelope(1.0, fade), 0.5, "a meio da rampa, metade");
    assert_eq!(fade_envelope(2.0, fade), 1.0, "no fim, cheia");
    assert_eq!(fade_envelope(90.0, fade), 1.0, "e fica cheia");
    assert_eq!(fade_envelope(-3.0, fade), 0.0, "antes do zero, nada");
    let mut prev = -1.0;
    for k in 0..=40 {
        let e = fade_envelope(k as f32 * 0.1, fade);
        assert!(e >= prev, "monotona em t={}", k as f32 * 0.1);
        prev = e;
    }
}

/// **A RAMPA MULTIPLICA A AMPLITUDE, NAO O VALOR** -- o `offset` (o centro da
/// oscilacao) fica onde esta'.
///
/// ⚠️ **E' o gate que separa as duas implementacoes que passariam num teste de
/// "o fade faz alguma coisa".** Multiplicar o valor INTEIRO faria o elemento
/// nascer na ORIGEM e viajar ate' ao centro — um movimento que ninguem pediu, e
/// que num documento com `offset` grande e' um voo atravessado da tela. A
/// fixture usa uma quadrada, cuja saida no instante zero e' `+amplitude`
/// exacta: o salto que a rampa existe para apagar, e um numero que nao depende
/// de nenhuma aproximacao de seno.
#[test]
fn the_ramp_scales_the_swing_not_the_centre() {
    const SQUARE: i32 = 2;
    let (amp, offset, fade) = (3.0f32, 100.0f32, 2.0f32);
    let at = |t: f32| waveform(SQUARE, t / 1.0) * amp * fade_envelope(t, fade) + offset;
    assert_eq!(at(0.0), offset, "no instante zero: o CENTRO, sem salto");
    // ⚠️ `t = 1.0` com periodo 1.0 e' o INICIO do 2º ciclo (`f = 0`), entao a
    // quadrada vale `+1` outra vez -- a meia excursao e' para CIMA.
    assert_eq!(at(1.0), offset + 1.5, "a meio da rampa: meia excursao");
    assert_eq!(at(2.0), offset + 3.0, "no fim: excursao cheia");
    // ⚠️ O CONTROLE: sem a rampa o no' SALTA para `offset + amp` no instante
    // zero -- o fenomeno que a rampa apaga tem de existir na fixture.
    assert_eq!(
        waveform(SQUARE, 0.0) * amp * fade_envelope(0.0, 0.0) + offset,
        offset + amp,
        "sem rampa, o salto acontece"
    );
}
