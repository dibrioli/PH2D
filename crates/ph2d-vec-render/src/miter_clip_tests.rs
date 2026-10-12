//! O contorno com o bico cortado (o gabarito `resvg` corre no `ph2d-tool-vector`).

use super::*;

/// Um «V» de ângulo interno `a`, vértice em `(0, 0)`, pernas de 10 para BAIXO (y+): o bico sobe.
fn vee(a: f64) -> BezPath {
    let h = (a / 2.0).to_radians();
    let mut p = BezPath::new();
    p.move_to((-10.0 * h.sin(), 10.0 * h.cos()));
    p.line_to((0.0, 0.0));
    p.line_to((10.0 * h.sin(), 10.0 * h.cos()));
    p
}

fn pen(w: f64) -> Stroke {
    Stroke::new(w).with_join(Join::Miter).with_miter_limit(4.0)
}

/// ⭐ Sem salto: o topo do traço desce CONTÍNUO pelo limite (29° → 28° antes saltava 43 px no
/// gabarito do `resvg`; aqui fica a `limite × ½w` = 2 acima do vértice).
#[test]
fn the_tip_never_drops_to_the_bevel_across_the_limit() {
    let mut last = f64::NAN;
    for i in 0..=152 {
        let a = 40.0 - f64::from(i) * 0.25;
        let top = stroke_outline(&vee(a), &pen(1.0), 1e-4).bounding_box().y0;
        assert!(
            top >= -2.0 - 1e-6,
            "ângulo {a}: o bico passou do tecto ({top})"
        );
        if last.is_finite() {
            assert!(
                (top - last).abs() < 0.05,
                "ângulo {a}: saltou de {last} para {top}"
            );
        }
        last = top;
    }
    assert!(
        (last + 2.0).abs() < 1e-6,
        "abaixo do limite o corte fica no tecto: {last}"
    );
}

/// O quadrilátero entra no MESMO sentido do corpo: dentro dele e no chanfro o winding não zera.
#[test]
fn the_cut_winds_like_the_body_so_the_fill_has_no_seam() {
    for flip in [false, true] {
        let mut v = vee(10.0);
        if flip {
            v = v.reverse_subpaths();
        }
        let out = stroke_outline(&v, &pen(1.0), 1e-4);
        for y in [-1.9, -1.0, -0.2] {
            assert_ne!(
                out.winding(Point::new(0.0, y)),
                0,
                "furo a y = {y} (sentido {flip})"
            );
        }
    }
}

/// Com tracejado, só as juntas que SE DESENHAM levam corte: um vão sobre o vértice não leva.
#[test]
fn a_dash_gap_over_the_corner_takes_no_cut() {
    let len = 10.0;
    let on = pen(1.0).with_dashes(0.0, [len - 1.0, 2.0]);
    assert!(patches(&vee(10.0), &on).is_empty(), "o vértice cai no vão");
    let through = pen(1.0).with_dashes(0.0, [len + 1.0, 2.0]);
    assert_eq!(patches(&vee(10.0), &through).len(), 1);
}

#[test]
fn round_and_bevel_joins_take_no_cut() {
    for j in [Join::Round, Join::Bevel] {
        assert!(patches(&vee(10.0), &pen(1.0).with_join(j)).is_empty());
    }
}

/// A convenção de sinal que a lei assume (área anti-horária com y para cima = winding `+1`) é a
/// da `kurbo`: um corte orientado pela lei soma-se ao corpo em vez de o anular.
#[test]
fn the_law_and_the_kurbo_agree_on_the_winding_sign() {
    let mut sq = BezPath::new();
    sq.move_to((0.0, 0.0));
    sq.line_to((1.0, 0.0));
    sq.line_to((1.0, 1.0));
    sq.line_to((0.0, 1.0));
    sq.close_path();
    assert_eq!(
        sq.winding(Point::new(0.5, 0.5)),
        1,
        "anti-horário (y para cima) = +1"
    );
    assert!(sq.area() > 0.0);
}

/// ⭐ O rectângulo de raio MINÚSCULO com traço grosso (a foto do dono de 11/10: um triângulo escuro
/// dentro do traço, nos cantos): o canto é uma curva mais apertada que meia caneta, e o contorno
/// tem tinta onde o chanfro a tirava — a `0,9 × ½w` do canto pela diagonal de FORA (o chanfro pára
/// a `0,707`).
#[test]
fn a_tiny_corner_radius_under_a_thick_pen_keeps_the_corner_inked() {
    let r = 1e-4;
    let mut bp = BezPath::new();
    bp.move_to((r, 0.0));
    bp.line_to((10.0 - r, 0.0));
    bp.curve_to((10.0, 0.0), (10.0, 0.0), (10.0, r));
    bp.line_to((10.0, 10.0));
    bp.line_to((0.0, 10.0));
    bp.line_to((0.0, r));
    bp.curve_to((0.0, 0.0), (0.0, 0.0), (r, 0.0));
    bp.close_path();
    let pen = pen(2.0);
    assert!(
        tighter_than(&bp, 1.0),
        "o canto de raio 1e-4 sob ½w = 1 é apertado"
    );
    let out = stroke_outline(&bp, &pen, 1e-4);
    let d = 0.9 / std::f64::consts::SQRT_2;
    for q in [Point::new(-d, -d), Point::new(10.0 + d, -d)] {
        assert_ne!(out.winding(q), 0, "{q:?}: o canto ficou sem tinta");
    }
    // O rectângulo de cantos vivos e o de raio largo não pagam o contorno exacto.
    let sharp = BezPath::from_vec(vec![
        PathEl::MoveTo(Point::new(0.0, 0.0)),
        PathEl::LineTo(Point::new(10.0, 0.0)),
        PathEl::LineTo(Point::new(10.0, 10.0)),
        PathEl::ClosePath,
    ]);
    assert!(!tighter_than(&sharp, 1.0));
}
