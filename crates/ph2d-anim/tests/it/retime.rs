//! ⭐ **A lei do Retime por trecho** (plano 18 §3 e §5): `valor(u) = lerp(v0, v1, E(W(u)))`. O ritmo
//! muda QUANDO cada parte do easing acontece; as chaves, os instantes e o desenho do easing (os
//! quiques do Bounce) ficam.

use ph2d_anim::{
    AnimValue, AttributeEvaluator, Easing, EasingFamily, EasingMode, Extrap, Interp, Key,
    LinearInterp, RationalTime, Retime, Track,
};

fn f(v: AnimValue) -> f64 {
    match v {
        AnimValue::Float(x) => f64::from(x),
        other => panic!("not a scalar: {other:?}"),
    }
}

fn eased(family: EasingFamily) -> Interp {
    Interp::Eased(Easing {
        family,
        mode: EasingMode::Out,
    })
}

const W: Retime = Retime {
    x1: 0.6,
    y1: 0.1,
    x2: 0.85,
    y2: 0.5,
};

fn track(interp: Interp, retime: Option<Retime>) -> Track {
    let mut k0 = Key::new(RationalTime::ZERO, AnimValue::Float(0.0), interp);
    k0.retime = retime;
    let k1 = Key::new(
        RationalTime::from_seconds(2.0),
        AnimValue::Float(100.0),
        Interp::Linear,
    );
    Track::new(vec![k0, k1])
}

fn familias() -> Vec<Interp> {
    let mut sp = ph2d_anim::EaseSpline::from_bezier(0.3, 0.0, 0.7, 1.0);
    sp.insert(0.5);
    vec![
        Interp::Linear,
        Interp::Hold,
        Interp::Nearest,
        eased(EasingFamily::Bounce),
        eased(EasingFamily::Elastic),
        eased(EasingFamily::Back),
        eased(EasingFamily::Quad),
        Interp::bezier(0.4, 0.0, 0.2, 1.0),
        Interp::bezier_w(0.3, 40.0, 0.7, -10.0),
        Interp::Spline(sp),
    ]
}

/// Sem ritmo, cada família amostra EXACTAMENTE a fórmula de antes do plano 18 (`interpolate` com o
/// `u` cru) — ao bit, nunca «perto».
#[test]
fn sem_ritmo_cada_familia_amostra_ao_bit_como_antes() {
    for interp in familias() {
        let tr = track(interp, None);
        for i in 0..=400 {
            let t = 2.0 * f64::from(i) / 400.0;
            let u = t / 2.0;
            let (v0, v1) = (AnimValue::Float(0.0), AnimValue::Float(100.0));
            let antes = match interp {
                Interp::BezierW { .. } => interp.value(0.0, 100.0, u) as f32,
                _ => f(AnimValue::lerp(v0, v1, interp.remap(u))) as f32,
            };
            let agora = match tr.sample(t) {
                AnimValue::Float(v) => v,
                other => panic!("{other:?}"),
            };
            // As pontas são as chaves (o fecho plano de sempre), não o easing em `u = 0|1`.
            let antes = if t >= 2.0 {
                100.0
            } else if t <= 0.0 {
                0.0
            } else {
                antes
            };
            assert_eq!(agora.to_bits(), antes.to_bits(), "{interp:?} t={t}");
        }
    }
}

/// Retime sobre o Bounce: as duas chaves ficam no lugar, o número de quiques (extremos locais) é o
/// mesmo, e o ritmo mudou mesmo (o meio do trecho lê outro valor).
#[test]
fn retime_sobre_o_bounce_mantem_as_chaves_e_os_quiques() {
    let extremos = |tr: &Track| {
        let v: Vec<f64> = (0..=4000)
            .map(|i| f(tr.sample(2.0 * f64::from(i) / 4000.0)))
            .collect();
        v.windows(3)
            .filter(|w| (w[1] - w[0]) * (w[2] - w[1]) < 0.0)
            .count()
    };
    let (sem, com) = (
        track(eased(EasingFamily::Bounce), None),
        track(eased(EasingFamily::Bounce), Some(W)),
    );
    assert_eq!(f(com.sample(0.0)), 0.0);
    assert_eq!(f(com.sample(2.0)), 100.0);
    assert!(extremos(&sem) >= 3, "o Bounce quica: {}", extremos(&sem));
    assert_eq!(extremos(&sem), extremos(&com), "os quiques ficam");
    assert!(
        (f(sem.sample(1.0)) - f(com.sample(1.0))).abs() > 1.0,
        "o ritmo mudou"
    );
}

/// O ritmo é monótono e vai de `(0,0)` a `(1,1)` para QUALQUER par de alças no quadrado — o tempo
/// nunca anda para trás dentro de um trecho.
#[test]
fn o_ritmo_e_monotono_em_todo_o_quadrado() {
    let passos = [0.0f32, 0.1, 0.35, 0.5, 0.8, 1.0];
    for &x1 in &passos {
        for &y1 in &passos {
            for &x2 in &passos {
                for &y2 in &passos {
                    let w = Retime::new(x1, y1, x2, y2);
                    assert!(w.eval(0.0).abs() < 1.0e-6 && (w.eval(1.0) - 1.0).abs() < 1.0e-6);
                    let mut antes = 0.0;
                    for i in 0..=200 {
                        let v = w.eval(f64::from(i) / 200.0);
                        assert!(v >= antes - 1.0e-6, "{w:?} desceu em {i}");
                        antes = v;
                    }
                }
            }
        }
    }
}

/// Inverter a faixa leva o ritmo com o easing: tocada ao contrário, é o espelho exacto.
#[test]
fn inverter_espelha_o_ritmo_com_o_easing() {
    let original = track(eased(EasingFamily::Bounce), Some(W));
    let mut invertida = original.clone();
    invertida.reverse_about(2.0);
    for i in 0..=200 {
        let t = 2.0 * f64::from(i) / 200.0;
        let (a, b) = (f(original.sample(t)), f(invertida.sample(2.0 - t)));
        assert!((a - b).abs() < 1.0e-3, "t={t}: {a} × {b}");
    }
}

/// O «Continue» depois da última chave segue a velocidade que TOCA na ponta (`E'·W'`).
#[test]
fn o_continue_segue_a_velocidade_com_o_ritmo() {
    let mut tr = track(Interp::Linear, Some(W));
    tr.set_post(Extrap::Continue);
    let rampa = f(tr.sample(2.5)) - 100.0;
    let esperado = 100.0 / 2.0 * W.slope(1.0) * 0.5;
    assert!((rampa - esperado).abs() < 1.0e-2, "{rampa} × {esperado}");
}
