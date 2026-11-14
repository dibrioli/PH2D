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
    // A medida do tecto: 24 B por ponto em toda `Interp` (ver o doc do módulo); os tipos de alça
    // moram na folga ao lado do `n` e não sobem nada.
    assert_eq!(core::mem::size_of::<SplinePoint>(), 24);
    assert_eq!(core::mem::size_of::<Interp>(), 216);
}

/// A alça oposta em relação ao ponto `i`: `(comprimento da de chegada, da de saída, seno do ângulo
/// entre a de chegada e o oposto da de saída)` — seno 0 = na mesma reta.
fn bracos(sp: &EaseSpline, i: usize) -> (f32, f32, f32) {
    let no = sp.node(i);
    let a = [no.h_in[0] - no.p[0], no.h_in[1] - no.p[1]];
    let b = [no.p[0] - no.h_out[0], no.p[1] - no.h_out[1]];
    let (la, lb) = (a[0].hypot(a[1]), b[0].hypot(b[1]));
    (
        la,
        lb,
        (a[0] * b[1] - a[1] * b[0]) / (la * lb).max(f32::EPSILON),
    )
}

fn com_um_ponto() -> EaseSpline {
    let mut sp = s_curve();
    sp.insert(0.5);
    sp
}

#[test]
fn each_handle_kind_moves_the_other_handle_like_the_vector_module() {
    // Um ponto novo nasce Smooth (o corte não muda a forma: as alças já estão na mesma reta).
    let base = com_um_ponto();
    assert_eq!(base.kind(1), Some(HandleKind::Smooth));
    assert_eq!(
        (base.kind(0), base.kind(2)),
        (None, None),
        "as pontas não têm tipo"
    );
    let antes = base.node(1);

    // Corner: a outra alça não se mexe.
    let mut c = base;
    assert!(c.set_kind(1, HandleKind::Corner));
    assert_eq!(c.node(1), antes, "trocar para Corner não mexe nas alças");
    c.move_handle(1, SplineSide::Out, 0.6, 0.2);
    assert_eq!(c.node(1).h_in, antes.h_in);
    assert_eq!(c.node(1).h_out, [0.6, 0.2]);

    // Smooth: na mesma reta, cada uma com o seu comprimento.
    let mut s = base;
    let (li, ..) = bracos(&s, 1);
    s.move_handle(1, SplineSide::Out, 0.6, 0.2);
    let (li2, lo2, sen) = bracos(&s, 1);
    assert!(sen.abs() < 1e-4 && (li2 - li).abs() < 1e-5 && (lo2 - li2).abs() > 1e-2);

    // Symmetric: o espelho exato.
    let mut y = base;
    assert!(y.set_kind(1, HandleKind::Symmetric));
    let (a, b, sen) = bracos(&y, 1);
    assert!(
        sen.abs() < 1e-4 && (a - b).abs() < 1e-5,
        "ao trocar: média dos dois"
    );
    y.move_handle(1, SplineSide::Out, 0.6, 0.2);
    let (a, b, sen) = bracos(&y, 1);
    assert!(
        sen.abs() < 1e-4 && (a - b).abs() < 1e-5,
        "ao arrastar: espelho"
    );
    assert_eq!(y.node(1).h_out, [0.6, 0.2]);

    // Symmetric cujo espelho sairia do trecho: as duas encurtam juntas e ficam iguais.
    // (um ponto em 0,3: a alça de saída pode ir a 0,6 de distância, a de chegada só a 0,3)
    let mut z = s_curve();
    z.insert(0.3);
    z.set_kind(1, HandleKind::Symmetric);
    z.move_handle(1, SplineSide::Out, 0.9, 0.5);
    let (a, b, sen) = bracos(&z, 1);
    assert!(sen.abs() < 1e-4 && (a - b).abs() < 1e-5, "{a} {b} {sen}");
    assert!(
        z.node(1).h_in[0] >= 0.0 && z.node(1).h_out[0] < 0.9 - 1e-3,
        "encurtou"
    );

    // Trocar para o tipo que já é não muda nada.
    assert!(!y.set_kind(1, HandleKind::Symmetric));
    assert!(!y.set_kind(0, HandleKind::Corner), "o início não tem tipo");
}

#[test]
fn a_corner_turned_smooth_lines_its_handles_up_and_collapsed_ones_get_a_direction() {
    let mut sp = com_um_ponto();
    sp.set_kind(1, HandleKind::Corner);
    sp.move_handle(1, SplineSide::Out, 0.6, 0.2);
    assert!(bracos(&sp, 1).2.abs() > 1e-2, "o canto quebrou a reta");
    assert!(sp.set_kind(1, HandleKind::Smooth));
    assert!(bracos(&sp, 1).2.abs() < 1e-4, "Smooth volta a pôr na reta");
    // As duas recolhidas no ponto: Symmetric dá-lhes a direção dos vizinhos.
    let p = sp.node(1).p;
    sp.set_kind(1, HandleKind::Corner);
    sp.move_handle(1, SplineSide::In, p[0], p[1]);
    sp.move_handle(1, SplineSide::Out, p[0], p[1]);
    assert!(sp.set_kind(1, HandleKind::Symmetric));
    let (a, b, sen) = bracos(&sp, 1);
    assert!(a > 1e-2 && (a - b).abs() < 1e-5 && sen.abs() < 1e-4);
}

#[test]
fn the_kind_travels_with_its_point_when_points_come_and_go() {
    let mut sp = s_curve();
    sp.insert(0.3);
    sp.insert(0.7);
    sp.set_kind(1, HandleKind::Corner);
    sp.set_kind(2, HandleKind::Symmetric);
    // Um ponto novo entre os dois: os de depois andam um, com o tipo.
    assert_eq!(sp.insert(0.5), Some(2));
    let tipos = |sp: &EaseSpline| {
        (1..sp.node_count() - 1)
            .map(|i| sp.kind(i).unwrap())
            .collect::<Vec<_>>()
    };
    use HandleKind::{Corner, Smooth, Symmetric};
    assert_eq!(tipos(&sp), [Corner, Smooth, Symmetric]);
    // Ao contrário, a ordem inverte.
    assert_eq!(tipos(&sp.reversed()), [Symmetric, Smooth, Corner]);
    // Apagar o do meio: os outros ficam com o seu.
    assert!(sp.remove(2));
    assert_eq!(tipos(&sp), [Corner, Symmetric]);
    assert!(sp.remove(1));
    assert_eq!(tipos(&sp), [Symmetric]);
    // Um ponto novo onde morava um apagado nasce Smooth, não herda o tipo antigo.
    sp.insert(0.2);
    assert_eq!(tipos(&sp), [Smooth, Symmetric]);
}
