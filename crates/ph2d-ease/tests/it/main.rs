//! Os gates contra o ORÁCULO corrido (anime.js 3.2.2, MIT — `tests/fixtures/animejs.txt`,
//! gerado por `docs/Motion Nodes/ferramentas/animejs_stagger_oracle/corre.sh`).
//!
//! ⚠️ O anime.js arredonda a distância do `stagger` a centésimos (medido: `√13 → 3,61`), por isso a
//! régua da cascata é `0,006` sobre o normalizado; as curvas saem com 6 casas.

use ph2d_ease::order::{self, Order};
use ph2d_ease::Shape;

const FIXTURA: &str = include_str!("../fixtures/animejs.txt");

struct Caso<'a> {
    nome: &'a str,
    n: u32,
    cols: u32,
    from: &'a str,
    axis: &'a str,
    ease: &'a str,
    valores: Vec<f64>,
}

fn casos() -> Vec<Caso<'static>> {
    FIXTURA
        .lines()
        .filter_map(|l| l.strip_prefix("STAGGER "))
        .map(|l| {
            let (cab, vals) = l.split_once(" | ").expect("separador");
            let c: Vec<&str> = cab.split(' ').collect();
            Caso {
                nome: c[0],
                n: c[1].parse().expect("n"),
                cols: c[2].parse().expect("cols"),
                from: c[4],
                axis: c[5],
                ease: c[6],
                valores: vals.split(' ').map(|v| v.parse().expect("valor")).collect(),
            }
        })
        .collect()
}

fn curva(nome: &str) -> Vec<f64> {
    FIXTURA
        .lines()
        .filter_map(|l| l.strip_prefix("EASE "))
        .find_map(|l| {
            let (n, vals) = l.split_once(" | ")?;
            (n == nome).then(|| vals.split(' ').map(|v| v.parse().expect("valor")).collect())
        })
        .unwrap_or_else(|| panic!("curva {nome} ausente da fixtura"))
}

fn ordem(c: &Caso<'_>) -> Order {
    let (from, cell) = match c.from {
        "first" => (order::FROM_FIRST, 0),
        "center" => (order::FROM_CENTER, 0),
        "last" => (order::FROM_LAST, 0),
        k => (order::FROM_CELL, k.parse().expect("índice")),
    };
    let axis = match c.axis {
        "x" => order::AXIS_X,
        "y" => order::AXIS_Y,
        _ => order::AXIS_BOTH,
    };
    Order {
        kind: order::ORDER_GRID,
        cols: c.cols,
        from,
        cell,
        axis,
        ..Order::INDEX
    }
}

#[test]
fn the_grid_wave_matches_the_oracle_step_by_step() {
    let cs = casos();
    assert!(cs.len() >= 20, "a fixtura encolheu: {}", cs.len());
    for c in &cs {
        let o = ordem(c);
        let max = c.valores.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
        for (i, v) in (0_u32..).zip(&c.valores) {
            let raw = order::raw_at(&o, i, c.n, [0.0, 0.0]);
            let nosso = match c.ease {
                "-" => f64::from(raw),
                "easeInQuad" => f64::from(ph2d_ease::ease(ph2d_ease::QUAD, 0, raw, &Shape::DEFAULT)),
                "easeOutCubic" => {
                    f64::from(ph2d_ease::ease(ph2d_ease::CUBIC, 1, raw, &Shape::DEFAULT))
                }
                e => panic!("ease {e} sem par"),
            };
            // O anime.js devolve a distância (com SINAL num eixo, sem easing); nós a posição 0..1.
            let oraculo = v.abs() / max;
            assert!(
                (nosso - oraculo).abs() < 0.006,
                "{} i {i}: nosso {nosso} × oráculo {oraculo}",
                c.nome
            );
        }
    }
}

fn confere(nome: &str, f: impl Fn(f32) -> f32, tol: f64) {
    for (k, r) in (0_u16..).zip(curva(nome)) {
        let t = f32::from(k) / 64.0;
        let e = f64::from(f(t));
        assert!((e - r).abs() < tol, "{nome} t {t}: {e} × {r}");
    }
}

#[test]
fn sine_and_expo_match_the_oracle() {
    let s = Shape::DEFAULT;
    confere("easeInSine", |t| ph2d_ease::ease(ph2d_ease::SINE, 0, t, &s), 2e-6);
    confere("easeOutSine", |t| ph2d_ease::ease(ph2d_ease::SINE, 1, t, &s), 2e-6);
    confere("easeInOutSine", |t| ph2d_ease::ease(ph2d_ease::SINE, 2, t, &s), 2e-6);
    confere("easeInExpo", |t| ph2d_ease::ease(ph2d_ease::EXPO, 0, t, &s), 2e-6);
    confere("easeOutExpo", |t| ph2d_ease::ease(ph2d_ease::EXPO, 1, t, &s), 2e-6);
    confere("easeInOutExpo", |t| ph2d_ease::ease(ph2d_ease::EXPO, 2, t, &s), 2e-6);
}

#[test]
fn elastic_matches_the_oracle_at_amplitude_1_period_0_3() {
    let s = Shape::DEFAULT;
    confere("easeOutElastic(1,.3)", |t| ph2d_ease::ease(ph2d_ease::ELASTIC, 1, t, &s), 2e-6);
    confere("easeInElastic(1,.3)", |t| ph2d_ease::ease(ph2d_ease::ELASTIC, 0, t, &s), 2e-6);
    confere("easeInOutElastic(1,.3)", |t| ph2d_ease::ease(ph2d_ease::ELASTIC, 2, t, &s), 2e-6);
}

#[test]
fn bezier_matches_the_css_curves_of_the_oracle() {
    for (nome, b) in [
        ("cubicBezier(.25,.1,.25,1)", [0.25, 0.1, 0.25, 1.0]),
        ("cubicBezier(.68,-0.55,.27,1.55)", [0.68, -0.55, 0.27, 1.55]),
        ("cubicBezier(.42,0,.58,1)", [0.42, 0.0, 0.58, 1.0]),
    ] {
        confere(nome, |t| ph2d_ease::bezier(t, b), 1e-5);
    }
}
