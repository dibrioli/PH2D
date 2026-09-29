//! Os gates da lei do lançamento. Cada um nomeia a mutação que o deve fazer sangrar.

use super::*;

const MS: u128 = 1_000_000;

/// Um dedo a andar a velocidade constante: `v` px/s durante `n` amostras espaçadas de `dt_ms`.
fn steady(v: f32, n: usize, dt_ms: u128) -> Vec<(u128, f32)> {
    (0..n)
        .map(|i| {
            let t = 1_000 * MS + i as u128 * dt_ms * MS;
            #[allow(clippy::cast_precision_loss)]
            let y = v * (i as f32 * dt_ms as f32 / 1000.0);
            (t, y)
        })
        .collect()
}

/// **Um dedo a velocidade constante mede-se a essa velocidade.**
///
/// Mutação: trocar o sinal do ajuste · medir `Δy/Δt` entre as pontas SEM o horizonte.
#[test]
fn a_steady_finger_measures_its_own_speed() {
    let s = steady(1200.0, 12, 8);
    let t_up = s.last().unwrap().0 + 5 * MS;
    let v = release_velocity(&s, t_up);
    assert!((v - 1200.0).abs() < 1.0, "mediu {v} para 1200 px/s");
}

/// **Só contam as amostras da janela** — um arrasto que começou devagar e acabou depressa lança
/// com a velocidade do FIM.
///
/// Mutação: apagar o filtro do [`HORIZON_NS`].
#[test]
fn only_the_last_hundred_milliseconds_count() {
    let mut s = steady(100.0, 40, 10); // 400 ms devagar
    let (t0, y0) = *s.last().unwrap();
    for i in 1..=8 {
        #[allow(clippy::cast_precision_loss)]
        s.push((t0 + i * 10 * MS, y0 + 3000.0 * (i as f32 * 0.010)));
    }
    let t_up = s.last().unwrap().0 + MS;
    let v = release_velocity(&s, t_up);
    assert!(v > 2000.0, "a velocidade do fim é 3000 px/s e mediu {v}");
}

/// **Um dedo que parou antes de largar não lança.**
///
/// Mutação: apagar a guarda do [`STOPPED_NS`] — a lista arrastada, segurada e largada saltava.
#[test]
fn a_finger_that_stopped_before_lifting_does_not_throw() {
    let s = steady(2000.0, 10, 8);
    let t_up = s.last().unwrap().0 + 60 * MS;
    assert_eq!(release_velocity(&s, t_up), 0.0);
}

/// **Largar devagar é pousar**, e o conteúdo anda ao CONTRÁRIO do dedo.
///
/// Mutação: tirar o `-` do [`launch`] · trocar `>=` por `<` no limiar.
#[test]
fn a_slow_release_rests_and_a_fast_one_scrolls_against_the_finger() {
    assert_eq!(launch(MIN_FLING * 0.5), None);
    assert_eq!(launch(-MIN_FLING * 0.5), None);
    // Dedo para BAIXO depressa ⇒ o deslocamento de rolagem DESCE (a lista volta para o topo).
    assert!(launch(3000.0).unwrap() < 0.0);
    assert!(launch(-3000.0).unwrap() > 0.0);
    assert_eq!(launch(-1e6).unwrap(), MAX_FLING);
}

/// **A distância total é `v₀/k` — a MESMA para qualquer taxa de quadros.**
///
/// Mutação: integrar por Euler (`pos + v·dt`, `v·e`) — a 30 Hz e a 240 Hz as distâncias passam a
/// diferir.
#[test]
fn the_distance_travelled_does_not_depend_on_the_frame_rate() {
    let travel = |hz: f64| {
        let (mut pos, mut v, mut done) = (0.0_f32, 2000.0_f32, false);
        while !done {
            (pos, v, done) = step(pos, v, 1.0 / hz, 1e9);
        }
        pos
    };
    let expected = (2000.0 / decay_rate()) as f32;
    for hz in [30.0, 60.0, 144.0, 240.0] {
        let d = travel(hz);
        assert!(
            (d - expected).abs() < 1.0,
            "a {hz} Hz andou {d}, a lei manda {expected}"
        );
    }
}

/// **Bater numa borda PÁRA** — e não se atravessa.
///
/// Mutação: apagar o `clamp` · devolver a velocidade em vez de zero na borda.
#[test]
fn hitting_an_edge_stops_the_flight() {
    let (pos, v, done) = step(95.0, 4000.0, 0.1, 100.0);
    assert_eq!((pos, v, done), (100.0, 0.0, true));
    let (pos, v, done) = step(5.0, -4000.0, 0.1, 100.0);
    assert_eq!((pos, v, done), (0.0, 0.0, true));
}

/// **A constante de tempo é a da taxa publicada** — `0,998` por ms dá `~0,5 s`.
///
/// Mutação: trocar o `DECAY_PER_MS`.
#[test]
fn the_decay_is_the_published_normal_rate() {
    let tau = 1.0 / decay_rate();
    assert!((tau - 0.4995).abs() < 0.001, "tau = {tau}");
}
