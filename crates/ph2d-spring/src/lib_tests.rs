//! Os gates da MOLA — *ela chega, ela assenta, e ela CARREGA a velocidade*.

use super::*;

/// Quanto tempo (em segundos) ela leva a assentar, e onde parou.
fn run(mut st: SpringState, s: Spring) -> (f64, f64) {
    let mut t = 0.0;
    for _ in 0..6000 {
        if st.advance(1.0 / 60.0, s) {
            return (t, st.x);
        }
        t += 1.0 / 60.0;
    }
    (f64::INFINITY, st.x)
}

/// ⭐ **Ela CHEGA, e chega em tempo de UI.**
///
/// ⚠️ Sem este gate uma mola mal parametrizada animaria para sempre e a máquina nunca chamaria
/// `arrive` — a pose exata nunca pousaria, e a cena derivaria a cada hover.
#[test]
fn the_default_spring_settles_and_it_settles_fast() {
    let (t, x) = run(SpringState::at_rest(), Spring::default());
    assert!(
        t.is_finite() && t < 1.5,
        "a mola default nao assentou em tempo de UI (levou {t:.3}s)"
    );
    assert!(
        (x - 1.0).abs() < 1.0e-2,
        "ela assentou fora do alvo (x = {x:.4})"
    );
}

/// **O amortecimento faz o que o nome diz** — crítico não passa do alvo, sub-amortecido passa.
///
/// ⚠️ É o gate que impede o `ζ` de ser um número que não muda nada: sem ele, um slider inerte
/// passaria despercebido (o `x` final é 1,0 nos dois casos).
#[test]
fn damping_decides_whether_it_overshoots() {
    let peak = |z: f64| {
        let mut st = SpringState::at_rest();
        let s = Spring {
            stiffness: DEFAULT_STIFFNESS,
            damping: z,
        };
        let mut hi = 0.0_f64;
        for _ in 0..600 {
            st.advance(1.0 / 60.0, s);
            hi = hi.max(st.x);
        }
        hi
    };
    let critical = peak(1.0);
    let bouncy = peak(0.35);
    assert!(
        critical <= 1.001,
        "o critico passou do alvo (pico {critical:.4})"
    );
    assert!(
        bouncy > 1.05,
        "o sub-amortecido NAO passou do alvo (pico {bouncy:.4}) — o knob seria inerte"
    );
}

/// **A rigidez faz o que o nome diz** — mais rígida chega antes.
#[test]
fn stiffness_decides_how_fast_it_gets_there() {
    let soft = run(
        SpringState::at_rest(),
        Spring {
            stiffness: 4.0,
            damping: 1.0,
        },
    )
    .0;
    let hard = run(
        SpringState::at_rest(),
        Spring {
            stiffness: 40.0,
            damping: 1.0,
        },
    )
    .0;
    assert!(
        hard < soft * 0.5,
        "a rigidez nao encurtou o caminho ({hard:.3}s contra {soft:.3}s)"
    );
}

/// ⭐⭐ **O QUE A MOLA COMPRA: ela retoma COM velocidade, e uma curva não.**
///
/// ⚠️ É a wave inteira num gate. `SpringState::resuming(v)` parte com a velocidade que a cena
/// tinha; `at_rest()` parte parada. Sem a diferença, a mola seria uma segunda forma de escrever
/// uma curva — e o `Cubic InOut` a **0,00×** que a medição nomeia continuaria sem resposta.
#[test]
fn a_resumed_spring_carries_the_velocity_a_curve_would_have_dropped() {
    let s = Spring::default();
    let after = |mut st: SpringState| {
        st.advance(1.0 / 60.0, s);
        st.x
    };
    let cold = after(SpringState::at_rest());
    let warm = after(SpringState::resuming(3.0));
    assert!(
        warm > cold * 1.5,
        "a mola retomada arrancou como se estivesse parada ({warm:.5} contra {cold:.5}) — \
         a continuidade de velocidade e' a unica coisa que ela compra sobre uma curva"
    );
}

/// **Os tetos dos sliders são honrados**, e por uma porta só.
#[test]
fn the_slider_range_is_honoured_by_one_door() {
    let wild = Spring {
        stiffness: 1.0e6,
        damping: -4.0,
    }
    .clamped();
    assert!((wild.stiffness - MAX_STIFFNESS).abs() < f64::EPSILON);
    assert!((wild.damping - MIN_DAMPING).abs() < f64::EPSILON);
}

/// **O passo do integrador NÃO é a taxa de quadros.**
///
/// ⚠️ Um integrador cujo passo fosse o `dt` do quadro daria trajetórias diferentes em máquinas
/// diferentes — a cena andaria de um jeito a 60 fps e de outro a 144. O mesmo tempo total,
/// entregue em fatias diferentes, tem de dar (praticamente) o mesmo `x`.
#[test]
fn the_trajectory_does_not_depend_on_the_frame_rate() {
    let s = Spring::default();
    let walk = |dt: f64, n: usize| {
        let mut st = SpringState::at_rest();
        for _ in 0..n {
            st.advance(dt, s);
        }
        st.x
    };
    let at60 = walk(1.0 / 60.0, 30);
    let at144 = walk(1.0 / 144.0, 72);
    assert!(
        (at60 - at144).abs() < 1.0e-3,
        "meio segundo a 60 fps deu {at60:.5} e a 144 deu {at144:.5} — o passo esta' a seguir \
         a taxa de quadros"
    );
}
