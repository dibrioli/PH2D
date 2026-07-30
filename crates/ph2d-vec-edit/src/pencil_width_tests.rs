//! Gates da **fonte de largura** — arquivo irmão de `pencil_width.rs`.
//!
//! Os oráculos são de COMPORTAMENTO do desenho, não da fórmula: *o trecho rápido sai mais fino
//! que o lento*, *um gesto de velocidade constante não pendura perfil nenhum*. Um gate que
//! recomputasse a normalização para conferir a normalização seria o espelho sempre-verde que
//! esta linha já pegou duas vezes.

use super::*;

/// Um gesto reto de `n` amostras cujo `dt` é dado por `dt_of` (o relógio de parede simulado).
fn gesture(n: usize, dt_of: impl Fn(usize) -> u128) -> (Vec<[f64; 2]>, Vec<PenDynamics>) {
    let mut t_ns = 0u128;
    let mut pts = Vec::with_capacity(n);
    let mut dyns = Vec::with_capacity(n);
    for i in 0..n {
        #[allow(clippy::cast_precision_loss)]
        let u = i as f64 / (n - 1) as f64;
        pts.push([u * 10.0, 0.0]);
        t_ns += dt_of(i);
        dyns.push(PenDynamics {
            pressure: 1.0,
            t_ns,
        });
    }
    (pts, dyns)
}

/// **Rápido AFINA.** É a convenção de todo DCC e a leitura que a mão espera; invertê-la desenharia
/// um borrão em todo gesto rápido, que é justamente quando o artista quer uma linha fina.
#[test]
fn the_fast_half_is_thinner_than_the_slow_half() {
    // Metade lenta (dt grande), metade rápida (dt pequeno) — o espaçamento é o MESMO, então a
    // única grandeza que muda é o relógio.
    let (pts, dyns) = gesture(120, |i| if i < 60 { 8_000_000 } else { 1_000_000 });
    let st = width_stops(WidthSource::Speed, &pts, &dyns);
    assert!(!st.is_empty(), "um gesto que muda de velocidade tem perfil");
    let slow = st.at(0.2);
    let fast = st.at(0.8);
    assert!(
        fast < slow,
        "o trecho RÁPIDO ({fast:.3}) não saiu mais fino que o lento ({slow:.3})"
    );
}

/// **Velocidade constante não pendura perfil nenhum.** O neutro é a AUSÊNCIA: um traço que não
/// tem o que dizer não deixa um componente com oito multiplicadores iguais no documento.
#[test]
fn a_constant_speed_gesture_has_no_profile() {
    let (pts, dyns) = gesture(120, |_| 4_000_000);
    assert!(width_stops(WidthSource::Speed, &pts, &dyns).is_empty());
}

/// **`Uniform` é a lista VAZIA** — o produto de antes desta wave, byte a byte.
#[test]
fn the_uniform_source_produces_nothing() {
    let (pts, dyns) = gesture(120, |i| {
        4_000_000 + u128::try_from(i).unwrap_or(0) * 100_000
    });
    assert!(width_stops(WidthSource::Uniform, &pts, &dyns).is_empty());
}

/// **A pressão engrossa** — a direção OPOSTA à da velocidade, e é ela que faz as duas fontes
/// serem duas leituras da mesma máquina em vez de duas máquinas.
#[test]
fn pressing_harder_makes_it_thicker() {
    let n = 120;
    let (pts, mut dyns) = gesture(n, |_| 4_000_000);
    for (i, d) in dyns.iter_mut().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let u = i as f32 / (n - 1) as f32;
        d.pressure = 0.1 + 0.9 * u;
    }
    let st = width_stops(WidthSource::Pressure, &pts, &dyns);
    assert!(
        st.at(0.9) > st.at(0.1),
        "apertar mais não engrossou: {:.3} no fim contra {:.3} no começo",
        st.at(0.9),
        st.at(0.1)
    );
}

/// **A pressão de um RATO não inventa perfil.** Hoje a shell entrega `1.0` em toda amostra (o
/// fato medido no cabeçalho do módulo), e o resultado tem de ser um traço uniforme — não um
/// perfil de multiplicadores iguais pendurado em toda forma que o artista desenhar.
#[test]
fn a_mouse_reporting_full_pressure_produces_no_profile() {
    let (pts, dyns) = gesture(120, |_| 4_000_000);
    assert!(dyns.iter().all(|d| (d.pressure - 1.0).abs() < f32::EPSILON));
    assert!(width_stops(WidthSource::Pressure, &pts, &dyns).is_empty());
}

/// **O perfil é função do CAMINHO, não da contagem de amostras.** Amostrar o MESMO gesto duas
/// vezes mais denso (mesmas posições, mesmo relógio de parede) tem de dar essencialmente o mesmo
/// perfil — senão a espessura de um traço passaria a depender da taxa de eventos da máquina, que
/// é a doença que esta engine já curou quatro vezes no relevo.
#[test]
fn the_profile_is_a_fact_of_the_path_not_of_the_sample_rate() {
    let coarse = gesture(60, |i| if i < 30 { 16_000_000 } else { 2_000_000 });
    let dense = gesture(240, |i| if i < 120 { 4_000_000 } else { 500_000 });
    let a = width_stops(WidthSource::Speed, &coarse.0, &coarse.1);
    let b = width_stops(WidthSource::Speed, &dense.0, &dense.1);
    for k in 0..=10 {
        let t = f64::from(k) / 10.0;
        let (x, y) = (a.at(t), b.at(t));
        assert!(
            (x - y).abs() < 0.12,
            "t={t}: o perfil grosseiro deu {x:.3} e o denso {y:.3} — a taxa de amostragem vazou \
             para o desenho"
        );
    }
}

/// **Um relógio parado não vira um pico.** Dois eventos com o mesmo carimbo dariam `dt = 0`, e
/// tratar isso como "infinitamente rápido" poria o topo da faixa num artefato — achatando o
/// traço inteiro, porque a normalização é relativa ao pico.
#[test]
fn a_stalled_clock_does_not_become_a_spike() {
    let (pts, mut dyns) = gesture(120, |i| if i < 60 { 8_000_000 } else { 1_000_000 });
    // Três eventos empilhados no mesmo instante, no meio do trecho lento.
    let t = dyns[20].t_ns;
    for d in &mut dyns[20..23] {
        d.t_ns = t;
    }
    let st = width_stops(WidthSource::Speed, &pts, &dyns);
    // ⚠️ A faixa é conferida com folga de ponto flutuante, e não com `contains`: `MIN + 1.0 *
    // (MAX - MIN)` dá `1,4500000000000002` em `f64`, e clampar isso no produto seria uma segunda
    // defesa de uma propriedade que ninguém consegue ver (2e-16 de um multiplicador).
    const EPS: f64 = 1e-9;
    for s in st.as_slice() {
        assert!(
            s.mult.is_finite() && s.mult >= MIN_MULT - EPS && s.mult <= MAX_MULT + EPS,
            "parada fora da faixa: {s:?}"
        );
    }
    assert!(
        st.at(0.8) < st.at(0.2),
        "o trecho rápido deixou de ser o fino"
    );
}

/// **Um gesto curto demais não tem o que dizer** — e a resposta é a ausência, não um perfil de
/// duas paradas que o `power_stroke` moldaria numa fita torta.
#[test]
fn a_two_sample_gesture_has_no_profile() {
    let (pts, dyns) = gesture(2, |_| 4_000_000);
    assert!(width_stops(WidthSource::Speed, &pts, &dyns).is_empty());
}

/// **Um descompasso entre posições e dinâmicas NÃO adivinha.** É a forma que um chamador novo
/// erra, e devolver vazio é a única resposta que não desenha uma mentira.
#[test]
fn a_length_mismatch_produces_nothing() {
    let (pts, dyns) = gesture(120, |i| {
        4_000_000 + u128::try_from(i).unwrap_or(0) * 100_000
    });
    assert!(width_stops(WidthSource::Speed, &pts, &dyns[..60]).is_empty());
}

/// **As pontas do perfil são as pontas do traço.** A 1ª e a última fatia sentam no MEIO delas, e
/// sem ancorar as paradas extremas em 0 e 1 o afinamento da ponta — que é o que o artista mais
/// vê — ficaria de fora do domínio.
#[test]
fn the_profile_spans_the_whole_stroke() {
    let (pts, dyns) = gesture(120, |i| if i < 60 { 8_000_000 } else { 1_000_000 });
    let st = width_stops(WidthSource::Speed, &pts, &dyns);
    let s = st.as_slice();
    assert!((s[0].pos - 0.0).abs() < 1e-12, "a 1ª parada não está em 0");
    assert!(
        (s[s.len() - 1].pos - 1.0).abs() < 1e-12,
        "a última parada não está em 1"
    );
}

/// **O identificador de fio vai e volta.** É o que o painel guarda e a shell traduz; um
/// desconhecido cai em `Uniform`, a fonte que não inventa geometria nenhuma.
#[test]
fn the_wire_identifier_round_trips() {
    for src in [
        WidthSource::Uniform,
        WidthSource::Speed,
        WidthSource::Pressure,
    ] {
        assert_eq!(WidthSource::from_wire(src.wire()), src);
    }
    assert_eq!(WidthSource::from_wire("acelerometro"), WidthSource::Uniform);
}

/// **Um gesto que acelera MONOTONICAMENTE não pode dar um perfil que sobe e desce.**
///
/// ⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE.** Tirar a média do filtro casado — voltar a
/// AMOSTRAR a série ponto a ponto, que foi a 1ª versão do módulo e o que a medição reprovou —
/// deixava os onze gates anteriores verdes: todos afirmam extremos (*o rápido é mais fino*, *o
/// constante não tem perfil*), e o aliasing não move extremos, ele põe degraus no MEIO.
///
/// A fixture tem de conter o fenômeno, e o fenômeno é o **jitter do relógio**: com `dt` perfeito
/// o ponto-a-ponto acerta. O factor multiplicativo aqui (`0,5×..1,5×`) é o que um laço de eventos
/// de facto entrega.
#[test]
fn a_monotonic_gesture_gives_a_monotonic_profile() {
    let n = 240;
    let mut t_ns = 0u128;
    let mut pts = Vec::with_capacity(n);
    let mut dyns = Vec::with_capacity(n);
    for i in 0..n {
        #[allow(clippy::cast_precision_loss)]
        let u = i as f64 / (n - 1) as f64;
        pts.push([u * 10.0, 0.0]);
        // A mão acelera do começo ao fim, sem voltar atrás uma única vez.
        let speed = 0.5 + 2.5 * u;
        #[allow(clippy::cast_precision_loss)]
        let jitter =
            1.0 + 0.5 * (((i as f64 * 12.9898).sin() * 43758.545).fract().abs() - 0.5) * 2.0;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let dt = (4_000_000.0 / speed * jitter).max(1.0) as u128;
        t_ns += dt;
        dyns.push(PenDynamics {
            pressure: 1.0,
            t_ns,
        });
    }
    let st = width_stops(WidthSource::Speed, &pts, &dyns);
    let m: Vec<f64> = st.as_slice().iter().map(|s| s.mult).collect();
    assert!(m.len() >= 4, "perfil curto demais para medir monotonia");
    // Tolerância pequena e NÃO zero: o filtro reduz o ruído, não o apaga. O que ela recusa é um
    // DEGRAU — o `0,886 → 0,350 → 0,528` que a medição viu com o reamostrador ponto-a-ponto.
    const SLACK: f64 = 0.05;
    let range = MAX_MULT - MIN_MULT;
    for (i, w) in m.windows(2).enumerate() {
        assert!(
            w[1] <= w[0] + SLACK * range,
            "a parada {} SUBIU ({:.3} → {:.3}) num gesto que só acelera — o perfil está a \
             desenhar o relógio, não a mão: {m:?}",
            i + 1,
            w[0],
            w[1]
        );
    }
}
