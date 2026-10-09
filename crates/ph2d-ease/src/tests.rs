use super::*;

const PONTOS: u32 = 4096;

fn ts() -> impl Iterator<Item = f32> {
    #[expect(clippy::cast_precision_loss, reason = "k <= 4096")]
    (0..=PONTOS).map(|k| k as f32 / PONTOS as f32)
}

/// A tabela do `motion.stagger` ANTES desta crate (ciclo 2), congelada aqui como régua: as
/// famílias `0..7` têm de continuar iguais a ela AO BIT.
mod antes {
    fn ease_in(curve: i32, t: f32) -> f32 {
        match curve {
            1 => t * t,
            2 => t * t * t,
            3 => t * t * t * t,
            4 => t * t * t * t * t,
            5 => 1.0 - (1.0 - t * t).max(0.0).sqrt(),
            6 => {
                const C1: f32 = 1.70158;
                const C3: f32 = C1 + 1.0;
                C3 * t * t * t - C1 * t * t
            }
            7 => 1.0 - bounce_out(1.0 - t),
            _ => t,
        }
    }
    fn bounce_out(t: f32) -> f32 {
        const N1: f32 = 7.5625;
        const D1: f32 = 2.75;
        if t < 1.0 / D1 {
            N1 * t * t
        } else if t < 2.0 / D1 {
            let t = t - 1.5 / D1;
            N1 * t * t + 0.75
        } else if t < 2.5 / D1 {
            let t = t - 2.25 / D1;
            N1 * t * t + 0.9375
        } else {
            let t = t - 2.625 / D1;
            N1 * t * t + 0.984375
        }
    }
    pub fn ease(curve: i32, dir: i32, t: f32) -> f32 {
        if curve == 0 {
            return t;
        }
        match dir {
            1 => 1.0 - ease_in(curve, 1.0 - t),
            2 => {
                if t < 0.5 {
                    ease_in(curve, 2.0 * t) * 0.5
                } else {
                    1.0 - ease_in(curve, 2.0 - 2.0 * t) * 0.5
                }
            }
            _ => ease_in(curve, t),
        }
    }
}

#[test]
fn the_eight_families_of_cycle_2_are_bit_identical() {
    let s = Shape::DEFAULT;
    for curve in 0..=7 {
        for dir in 0..=2 {
            for t in ts() {
                assert_eq!(
                    ease(curve, dir, t, &s).to_bits(),
                    antes::ease(curve, dir, t).to_bits(),
                    "família {curve} dir {dir} t {t}"
                );
            }
        }
    }
}

#[test]
fn every_family_and_direction_is_endpoint_exact() {
    let s = Shape::DEFAULT;
    for curve in 0..=LAST {
        for dir in 0..=2 {
            assert_eq!(ease(curve, dir, 0.0, &s), 0.0, "ease({curve},{dir},0)");
            assert_eq!(ease(curve, dir, 1.0, &s), 1.0, "ease({curve},{dir},1)");
        }
    }
}

#[test]
fn in_out_crosses_the_midpoint_exactly() {
    let s = Shape::DEFAULT;
    for &curve in USES_DIRECTION {
        assert!(
            (ease(curve, 2, 0.5, &s) - 0.5).abs() < 1e-6,
            "meio da família {curve}: {}",
            ease(curve, 2, 0.5, &s)
        );
    }
}

#[test]
fn the_polynomial_sin_and_exp2_hold_their_measured_error() {
    let mut pior_sin = 0.0_f64;
    let mut pior_exp = 0.0_f64;
    for k in 0..=200_000_u32 {
        let x = -3.0 + 6.0 * f64::from(k) / 200_000.0;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a entrada é f32 por desenho"
        )]
        let xf = x as f32;
        let ref_sin = (std::f64::consts::TAU * f64::from(xf)).sin();
        pior_sin = pior_sin.max((f64::from(sin_turn(xf)) - ref_sin).abs());
        let xe = -10.0 + 10.0 * f64::from(k) / 200_000.0;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a entrada é f32 por desenho"
        )]
        let xef = xe as f32;
        let ref_exp = f64::from(xef).exp2();
        pior_exp = pior_exp.max((f64::from(exp2(xef)) - ref_exp).abs() / ref_exp);
    }
    assert!(pior_sin < 4e-7, "sin: {pior_sin:e}");
    assert!(pior_exp < 2.5e-7, "exp2: {pior_exp:e}");
}

/// Penner em `f64` — a referência das famílias transcendentais.
fn penner_in(curve: i32, t: f64, period: f64) -> f64 {
    use std::f64::consts::{FRAC_PI_2, TAU};
    match curve {
        SINE => 1.0 - (FRAC_PI_2 * t).cos(),
        EXPO => {
            if t == 0.0 {
                0.0
            } else {
                (10.0 * t - 10.0).exp2()
            }
        }
        ELASTIC => {
            if t == 0.0 || t == 1.0 {
                t
            } else {
                let q = t - 1.0;
                -((10.0 * q).exp2() * ((q - period / 4.0) * TAU / period).sin())
            }
        }
        _ => unreachable!(),
    }
}

#[test]
fn sine_expo_elastic_match_penner_in_f64() {
    let s = Shape::DEFAULT;
    for curve in [SINE, EXPO, ELASTIC] {
        let mut pior = 0.0_f64;
        for t in ts() {
            let r = penner_in(curve, f64::from(t), f64::from(s.period));
            pior = pior.max((f64::from(ease_in(curve, t, &s)) - r).abs());
        }
        assert!(pior < 1e-6, "família {curve}: {pior:e}");
    }
}

#[test]
fn elastic_at_the_default_period_is_penners_two_pi_over_three() {
    // `c4 = 2π/3` de Penner ⇔ período `0,3`: `(10t − 10,75)·c4 = (t − 1,075)·2π/0,3`.
    let s = Shape::DEFAULT;
    let t = 0.37_f64;
    let c4 = std::f64::consts::TAU / 3.0;
    let r = -((10.0 * t - 10.0).exp2() * ((10.0 * t - 10.75) * c4).sin());
    #[expect(clippy::cast_possible_truncation, reason = "t é f32 por desenho")]
    let e = ease_in(ELASTIC, t as f32, &s);
    assert!((f64::from(e) - r).abs() < 1e-6, "{e} vs {r}");
}

/// A mola integrada em `f64` (RK4, `200 000` passos) — a forma fechada tem de a seguir.
fn spring_rk4(z: f64, ate: f64) -> f64 {
    let lam = 1000.0_f64.ln();
    let w = if z < 0.999 {
        lam / z
    } else if z <= 1.001 {
        lam
    } else {
        lam / (z - (z * z - 1.0).sqrt())
    };
    let f = |x: f64, v: f64| (v, -w * w * (x - 1.0) - 2.0 * z * w * v);
    let passos = 200_000_u32;
    let h = ate / f64::from(passos);
    let (mut x, mut v) = (0.0_f64, 0.0_f64);
    for _ in 0..passos {
        let (a1, b1) = f(x, v);
        let (a2, b2) = f(x + 0.5 * h * a1, v + 0.5 * h * b1);
        let (a3, b3) = f(x + 0.5 * h * a2, v + 0.5 * h * b2);
        let (a4, b4) = f(x + h * a3, v + h * b3);
        x += h / 6.0 * (a1 + 2.0 * a2 + 2.0 * a3 + a4);
        v += h / 6.0 * (b1 + 2.0 * b2 + 2.0 * b3 + b4);
    }
    x
}

#[test]
fn the_closed_form_spring_follows_the_integrated_one() {
    for z in [0.1_f32, 0.3, 0.7, 1.0, 1.5, 3.0] {
        let fim = spring_rk4(f64::from(z), 1.0);
        for k in 1..20_u8 {
            let t = f64::from(k) / 20.0;
            let r = spring_rk4(f64::from(z), t) / fim;
            #[expect(clippy::cast_possible_truncation, reason = "t é f32 por desenho")]
            let e = spring(t as f32, z);
            assert!((f64::from(e) - r).abs() < 2e-5, "ζ {z} t {t}: {e} vs {r}");
        }
    }
}

#[test]
fn an_underdamped_spring_overshoots_and_an_overdamped_one_does_not() {
    let sobe = ts().map(|t| spring(t, 0.2)).fold(0.0_f32, f32::max);
    assert!(sobe > 1.1, "ζ 0,2 ultrapassa: {sobe}");
    let lenta = ts().map(|t| spring(t, 2.0)).fold(0.0_f32, f32::max);
    assert!(lenta <= 1.0, "ζ 2 não ultrapassa: {lenta}");
}

#[test]
fn steps_in_is_jump_end_and_out_is_jump_start() {
    let s = Shape {
        steps: 4.0,
        ..Shape::DEFAULT
    };
    assert_eq!(ease(STEPS, 0, 0.2, &s), 0.0);
    assert_eq!(ease(STEPS, 0, 0.3, &s), 0.25);
    assert_eq!(ease(STEPS, 0, 0.99, &s), 0.75);
    // Out = 1 − In(1 − t): o primeiro degrau sobe logo.
    assert_eq!(ease(STEPS, 1, 0.01, &s), 0.25);
}

#[test]
fn bezier_linear_handles_are_the_identity() {
    let b = [1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0];
    for t in ts() {
        assert!((bezier(t, b) - t).abs() < 1e-6, "t {t}: {}", bezier(t, b));
    }
}

#[test]
fn spring_and_bezier_ignore_the_direction() {
    let s = Shape::DEFAULT;
    for curve in [SPRING, BEZIER] {
        for t in ts() {
            let a = ease(curve, 0, t, &s).to_bits();
            assert_eq!(a, ease(curve, 1, t, &s).to_bits());
            assert_eq!(a, ease(curve, 2, t, &s).to_bits());
        }
    }
}
