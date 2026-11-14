//! A curva Custom com pontos: igual à Bézier sem pontos, criar um ponto não muda a forma, apagar
//! volta ao que era, ao contrário é o espelho, e o `x` nunca anda para trás.

use super::*;
use crate::Interp;

const US: [f64; 9] = [0.0, 0.05, 0.2, 0.33, 0.5, 0.61, 0.8, 0.95, 1.0];

fn s_curve() -> EaseSpline {
    EaseSpline::from_bezier(0.42, 0.0, 0.58, 1.0)
}

#[test]
fn without_points_it_is_exactly_the_two_handle_bezier() {
    // A curva guarda `f32` (a `Interp::Bezier` guarda `f64`): compara-se com os mesmos números.
    let f = |v: f32| f64::from(v);
    let b = Interp::bezier(f(0.42), 0.0, f(0.58), 1.0);
    let sp = s_curve();
    for u in US {
        assert!((sp.eval(u) - b.remap(u)).abs() < 1e-6, "u={u}"); // dois solvers, a 1e-6
    }
    assert_eq!(Interp::Spline(sp).tangent_handles(), b.tangent_handles());
}

#[test]
fn a_new_point_lands_on_the_curve_and_keeps_its_shape() {
    let mut sp = EaseSpline::from_bezier(0.1, 0.9, 0.3, 1.4);
    let antes: Vec<f64> = (0..=40).map(|i| sp.eval(f64::from(i) / 40.0)).collect();
    let i = sp.insert(0.37).expect("cabe");
    assert_eq!((i, sp.points().len()), (1, 1));
    let p = sp.node(i).p;
    assert!((f64::from(p[0]) - 0.37).abs() < 1e-5);
    assert!((f64::from(p[1]) - sp.eval(0.37)).abs() < 1e-5);
    for (k, v) in antes.iter().enumerate() {
        let u = k as f64 / 40.0;
        assert!((sp.eval(u) - v).abs() < 1e-4, "a forma mudou em u={u}");
    }
    // Um segundo ponto à esquerda do primeiro entra na ordem certa.
    assert_eq!(sp.insert(0.2), Some(1));
    assert!(sp.node(1).p[0] < sp.node(2).p[0]);
    assert!(sp.remove(1) && sp.remove(1));
    assert_eq!(sp.points().len(), 0);
    assert!(!sp.remove(0), "o início não se apaga");
}

#[test]
fn backwards_is_the_mirror_through_the_middle() {
    let mut sp = s_curve();
    sp.insert(0.3);
    sp.move_point(1, 0.3, 0.7);
    let r = sp.reversed();
    for u in US {
        assert!((r.eval(u) - (1.0 - sp.eval(1.0 - u))).abs() < 1e-6, "u={u}");
    }
    assert_eq!(Interp::Spline(sp).reversed(), Interp::Spline(r));
}

#[test]
fn a_dragged_point_stays_between_its_neighbours_and_u_maps_to_one_value() {
    let mut sp = s_curve();
    sp.insert(0.3);
    sp.insert(0.7);
    sp.move_point(1, 0.95, 1.3);
    assert!(sp.node(1).p[0] < sp.node(2).p[0], "o x não cruza o vizinho");
    assert!(
        (sp.node(1).p[1] - 1.3).abs() < 1e-6,
        "o y é livre (passa do 1)"
    );
    sp.move_handle(2, SplineSide::Out, 2.0, 0.5);
    let n = sp.node_count();
    for i in 0..n - 1 {
        let (a, b) = (sp.node(i), sp.node(i + 1));
        assert!(a.h_out[0] >= a.p[0] && a.h_out[0] <= b.p[0]);
        assert!(b.h_in[0] >= a.p[0] && b.h_in[0] <= b.p[0]);
    }
    // Uma alça de um ponto do meio: a outra alinha (o ponto fica liso).
    let no = sp.node(2);
    let (a, b) = (
        [no.h_in[0] - no.p[0], no.h_in[1] - no.p[1]],
        [no.h_out[0] - no.p[0], no.h_out[1] - no.p[1]],
    );
    assert!((a[0] * b[1] - a[1] * b[0]).abs() < 1e-4 || a == [0.0, 0.0] || b == [0.0, 0.0]);
    let mut prev = -1.0;
    for i in 0..=200 {
        let u = f64::from(i) / 200.0;
        let (xs, _) = sp.controle(sp.trecho(u));
        assert!(xs[0] <= u + 1e-9 && u <= xs[3] + 1e-9);
        assert!(u >= prev);
        prev = u;
        assert!(sp.eval(u).is_finite());
    }
}

#[test]
fn the_slope_is_the_derivative_of_the_value() {
    let mut sp = s_curve();
    sp.insert(0.5);
    sp.move_point(1, 0.5, 0.8);
    for u in [0.1, 0.3, 0.45, 0.7, 0.9] {
        let h = 1e-6;
        let num = (sp.eval(u + h) - sp.eval(u - h)) / (2.0 * h);
        assert!(
            (sp.slope(u) - num).abs() < 1e-3,
            "u={u}: {} vs {num}",
            sp.slope(u)
        );
    }
}

#[test]
fn it_holds_eight_points_and_refuses_the_ninth() {
    let mut sp = s_curve();
    for k in 1..=SPLINE_MAX_POINTS {
        assert!(sp.insert(k as f64 / 9.0).is_some(), "ponto {k}");
    }
    assert!(!sp.has_room());
    assert_eq!(sp.insert(0.99), None);
    // A medida do tecto: 24 B por ponto em toda `Interp` (ver o doc do módulo).
    assert_eq!(core::mem::size_of::<SplinePoint>(), 24);
    assert_eq!(core::mem::size_of::<Interp>(), 216);
}
