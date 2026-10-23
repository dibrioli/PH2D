//! ⭐ W23 B13 — o QuickShape do vetor na caneta do quadro pelo caminho de quem desenha (doc 17 §2.2):
//! segurar parado no fim do traço; o mesmo meio segundo do vetor; a forma que se vê é a reconhecida e
//! vira o objecto do quadro que a desenha igual (ou tinta limpa); `Ctrl+Z` devolve o traço.

use super::*;
use ph2d_board_model::{End, Head, Pen, ShapeType, Style};
use ph2d_vec_edit::quickshape_hold::HOLD_NS;

const T0: u128 = 1_000_000_000_000;
/// O passo do relógio entre dois pontos do traço (um rato a 250 Hz).
const DT: u128 = 4_000_000;

/// Os pontos de `corners` (fechado se o último é o 1.º), `per` amostras por lado.
fn sides(corners: &[[f64; 2]], per: usize) -> Vec<[f64; 2]> {
    let mut v = vec![corners[0]];
    for w in corners.windows(2) {
        for k in 1..=per {
            let u = k as f64 / per as f64;
            v.push([
                w[0][0] + (w[1][0] - w[0][0]) * u,
                w[0][1] + (w[1][1] - w[0][1]) * u,
            ]);
        }
    }
    v
}

fn rotated(corners: &[[f64; 2]], deg: f64) -> Vec<[f64; 2]> {
    let (s, c) = deg.to_radians().sin_cos();
    corners
        .iter()
        .map(|p| [p[0] * c - p[1] * s, p[0] * s + p[1] * c])
        .collect()
}

impl World {
    /// Desenha `pts` com a caneta; `hold` = fica PARADO `HOLD_NS` no fim antes de largar.
    fn ink(&mut self, pts: &[[f64; 2]], hold: Option<u128>, m: Mods) -> Option<ElementId> {
        self.ed.tool = Tool::Pen(Pen::Pen);
        let mut t = T0;
        self.ed.at_time(t);
        self.ed.pointer_down(
            &mut self.doc,
            &mut self.h,
            &mut self.ts,
            at(pts[0][0], pts[0][1], m),
            VIEW,
        );
        for p in &pts[1..] {
            t += DT;
            self.ed.at_time(t);
            self.ed
                .pointer_move(&mut self.doc, &mut self.ts, at(p[0], p[1], m));
        }
        let end = pts[pts.len() - 1];
        if let Some(wait) = hold {
            t += wait;
            self.ed.at_time(t);
            self.ed
                .pointer_move(&mut self.doc, &mut self.ts, at(end[0], end[1], m));
        }
        self.ed.at_time(t);
        self.ed
            .pointer_up(&mut self.doc, &mut self.h, at(end[0], end[1], m));
        self.doc.live_in_z_order().last().map(|e| e.id)
    }

    fn top(&self) -> Element {
        self.doc
            .live_in_z_order()
            .last()
            .map(|e| (*e).clone())
            .expect("um elemento")
    }
}

fn kind(el: &Element) -> Option<ShapeType> {
    el.shape().map(|s| s.kind)
}

const RECT: [[f64; 2]; 5] = [
    [0.0, 0.0],
    [400.0, 0.0],
    [400.0, 250.0],
    [0.0, 250.0],
    [0.0, 0.0],
];

#[test]
fn holding_a_box_makes_a_rectangle_and_ctrl_z_gives_the_stroke_back() {
    let mut w = world();
    w.ink(&sides(&RECT, 50), Some(HOLD_NS), NONE);
    let el = w.top();
    assert_eq!(kind(&el), Some(ShapeType::Rectangle), "{:?}", el.kind);
    assert!(
        (el.w - 400.0).abs() < 1.0 && (el.h - 250.0).abs() < 1.0,
        "{} × {}",
        el.w,
        el.h
    );
    let st = &el.shape().unwrap().style;
    assert!(
        st.fill.is_none() && st.stroke.is_some() && !st.round,
        "o traço da caneta, sem fundo"
    );
    assert_eq!(w.doc.live_len(), 1);
    assert!(w.cmd(Command::Undo));
    assert!(
        w.top().ink().is_some(),
        "o 1.º Ctrl+Z devolve o traço como foi desenhado"
    );
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.doc.live_len(), 0);
}

#[test]
fn a_rotated_box_keeps_its_angle() {
    let mut w = world();
    w.ink(&sides(&rotated(&RECT, 30.0), 50), Some(HOLD_NS), NONE);
    let el = w.top();
    assert_eq!(kind(&el), Some(ShapeType::Rectangle));
    let a = el.angle.to_degrees().rem_euclid(90.0);
    assert!((a - 30.0).abs() < 0.5, "{a}");
}

#[test]
fn a_circle_and_a_rhombus() {
    let mut w = world();
    let circle: Vec<[f64; 2]> = (0..=120)
        .map(|i| {
            let a = std::f64::consts::TAU * f64::from(i) / 120.0;
            [500.0 + 200.0 * a.cos(), 500.0 + 200.0 * a.sin()]
        })
        .collect();
    w.ink(&circle, Some(HOLD_NS), NONE);
    let el = w.top();
    assert_eq!(kind(&el), Some(ShapeType::Ellipse));
    assert!(
        (el.w - el.h).abs() < 0.05 * el.w,
        "um círculo: {} × {}",
        el.w,
        el.h
    );
    let rhombus = [
        [0.0, -150.0],
        [100.0, 0.0],
        [0.0, 150.0],
        [-100.0, 0.0],
        [0.0, -150.0],
    ];
    w.ink(&sides(&rhombus, 40), Some(HOLD_NS), NONE);
    assert_eq!(kind(&w.top()), Some(ShapeType::Diamond));
}

/// O triângulo REGULAR (Shift) é o do quadro; um triângulo qualquer não — fica a tinta LIMPA, com os
/// cantos onde a lei os pôs (o dono: «exatamente como no Vector»).
#[test]
fn a_triangle_the_board_cannot_draw_stays_a_clean_stroke() {
    let mut w = world();
    let tri = [[0.0, 0.0], [300.0, 0.0], [150.0, -260.0], [0.0, 0.0]];
    w.ink(&sides(&tri, 50), Some(HOLD_NS), SHIFT);
    assert_eq!(
        kind(&w.top()),
        Some(ShapeType::Triangle),
        "o regular é o do quadro"
    );
    let scalene = [[0.0, 0.0], [400.0, 0.0], [60.0, -220.0], [0.0, 0.0]];
    w.ink(&sides(&scalene, 50), Some(HOLD_NS), NONE);
    let el = w.top();
    assert!(
        el.ink().is_some(),
        "o quadro não tem este triângulo: {:?}",
        el.kind
    );
    // A geometria LIMPA, um ponto por px de ecrã (o traço tinha um a cada 8 px), sobre o triângulo.
    let pts = crate::world_points(&el);
    for w in pts.windows(2) {
        let gap = (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]);
        assert!(gap <= PX + 1e-3, "a geometria limpa, não o traço: {gap:.2}");
    }
    for p in &pts {
        let d = (0..3)
            .map(|i| crate::point_segment([p[0], p[1]], scalene[i], scalene[i + 1]))
            .fold(f64::INFINITY, f64::min);
        assert!(d < 0.01, "sobre o triângulo: {d}");
    }
}

/// A recta de uma forma a outra vira uma ligação presa às duas (o Excalidraw).
#[test]
fn a_line_from_a_box_to_a_box_is_a_bound_arrow() {
    let mut w = world();
    let a = w.rect([0.0, 0.0, 100.0, 100.0]);
    let b = w.rect([600.0, 0.0, 100.0, 100.0]);
    w.cmd(Command::Escape);
    let line: Vec<[f64; 2]> = (0..=60)
        .map(|i| [50.0 + 10.0 * f64::from(i), 50.0])
        .collect();
    w.ink(&line, Some(HOLD_NS), NONE);
    let el = w.top();
    let c = el.connector().expect("uma ligação");
    assert!(
        matches!(c.start, End::Bound { target, .. } if target == a),
        "{:?}",
        c.start
    );
    assert!(
        matches!(c.end, End::Bound { target, .. } if target == b),
        "{:?}",
        c.end
    );
    assert_eq!(c.heads, [Head::None; 2]);
}

#[test]
fn an_arrow_on_the_empty_board_keeps_its_head() {
    let mut w = world();
    let mut pts: Vec<[f64; 2]> = (0..=60).map(|i| [10.0 * f64::from(i), 0.0]).collect();
    pts.extend(sides(&[[600.0, 0.0], [540.0, -40.0]], 10));
    pts.extend(sides(&[[540.0, -40.0], [600.0, 0.0], [540.0, 40.0]], 10));
    w.ink(&pts, Some(HOLD_NS), NONE);
    let c = w.top();
    let c = c.connector().expect("uma seta");
    assert!(matches!(c.start, End::Free(_)) && matches!(c.end, End::Free(_)));
    assert_eq!(c.heads, [Head::None, Head::Arrow]);
}

/// ⭐ O MESMO gatilho do vetor: 1 ns antes do meio segundo nada; no meio segundo a forma. E o
/// interruptor desligado manda.
#[test]
fn the_board_hold_is_the_vector_hold() {
    let mut w = world();
    // PARADO conta desde o 1.º ponto a menos da folga do fim (a mesma porta do vetor).
    let pts = sides(&RECT, 50);
    let j = ph2d_vec_edit::quickshape_hold::rest_index(
        &pts,
        ph2d_vec_edit::quickshape_hold::HOLD_SLOP_PX,
    );
    let still = (pts.len() - 1 - j) as u128 * DT;
    w.ink(&pts, Some(HOLD_NS - still - 1), NONE);
    assert!(w.top().ink().is_some(), "antes do meio segundo é traço");
    w.ink(&pts, Some(HOLD_NS - still), NONE);
    assert_eq!(
        kind(&w.top()),
        Some(ShapeType::Rectangle),
        "no meio segundo é a forma"
    );
    w.ink(&sides(&RECT, 50), None, NONE);
    assert!(w.top().ink().is_some(), "sem parar é traço");
    w.ed.quickshape = false;
    w.ink(&sides(&RECT, 50), Some(HOLD_NS), NONE);
    assert!(w.top().ink().is_some(), "desligado é traço");
    w.ed.quickshape = true;
    w.ink(&sides(&RECT, 50), Some(HOLD_NS), NONE);
    let top = w.top();
    assert_eq!(
        kind(&top),
        Some(ShapeType::Rectangle),
        "{:?} · {}",
        top.kind,
        w.doc.live_len()
    );
}

/// Com a forma viva, o 1.º Esc devolve o traço e o gesto continua; largar deixa o traço.
#[test]
fn escape_gives_the_stroke_back_and_the_gesture_goes_on() {
    let mut w = world();
    w.ed.tool = Tool::Pen(Pen::Pen);
    let pts = sides(&RECT, 50);
    let mut t = T0;
    w.ed.at_time(t);
    w.ed.pointer_down(&mut w.doc, &mut w.h, &mut w.ts, at(0.0, 0.0, NONE), VIEW);
    for p in &pts[1..] {
        t += DT;
        w.ed.at_time(t);
        w.ed.pointer_move(&mut w.doc, &mut w.ts, at(p[0], p[1], NONE));
    }
    w.ed.at_time(t + HOLD_NS);
    assert!(
        w.ed.ink_hold(&mut w.doc, false),
        "o quadro do gatilho faz nascer a forma"
    );
    assert!(w.ed.is_quick());
    assert!(w.cmd(Command::Escape));
    assert!(
        !w.ed.is_quick() && w.ed.is_busy(),
        "devolveu o traço e continua a desenhar"
    );
    w.ed.pointer_up(&mut w.doc, &mut w.h, at(0.0, 0.0, NONE));
    assert!(w.top().ink().is_some());
    assert!(crate::world_points(&w.top()).len() > 100, "o traço cru");
}

/// Arrastar sem largar, com a forma viva, escala-a à volta do centro (o vetor).
#[test]
fn dragging_the_live_shape_scales_it() {
    let mut w = world();
    w.ed.tool = Tool::Pen(Pen::Pen);
    let pts = sides(&RECT, 50);
    let mut t = T0;
    w.ed.at_time(t);
    w.ed.pointer_down(&mut w.doc, &mut w.h, &mut w.ts, at(0.0, 0.0, NONE), VIEW);
    for p in &pts[1..] {
        t += DT;
        w.ed.at_time(t);
        w.ed.pointer_move(&mut w.doc, &mut w.ts, at(p[0], p[1], NONE));
    }
    w.ed.at_time(t + HOLD_NS);
    assert!(w.ed.ink_hold(&mut w.doc, false));
    // O fim do traço é (0, 0); o centro (200, 125): levar a mão ao dobro da distância dobra a forma.
    w.ed.pointer_move(&mut w.doc, &mut w.ts, at(-200.0, -125.0, NONE));
    w.ed.pointer_up(&mut w.doc, &mut w.h, at(-200.0, -125.0, NONE));
    let el = w.top();
    assert_eq!(kind(&el), Some(ShapeType::Rectangle));
    assert!(
        (el.w - 800.0).abs() < 2.0 && (el.h - 500.0).abs() < 2.0,
        "{} × {}",
        el.w,
        el.h
    );
}

/// O LARGAR que se perdeu (a janela perdeu o foco com a forma à vista; o gesto fecha-se no carregar
/// seguinte, `finish_gesture`) deixa os MESMOS dois passos que o largar: o 1.º `Ctrl+Z` devolve o
/// traço como foi desenhado (auditoria 21/10: era UM passo, e o `Ctrl+Z` apagava a forma).
#[test]
fn a_lost_release_with_the_shape_shown_still_gives_the_stroke_back() {
    let mut w = world();
    let pts = sides(&RECT, 50);
    w.ed.tool = Tool::Pen(Pen::Pen);
    let mut t = T0;
    w.ed.at_time(t);
    w.ed.pointer_down(&mut w.doc, &mut w.h, &mut w.ts, at(0.0, 0.0, NONE), VIEW);
    for p in &pts[1..] {
        t += DT;
        w.ed.at_time(t);
        w.ed.pointer_move(&mut w.doc, &mut w.ts, at(p[0], p[1], NONE));
    }
    t += HOLD_NS;
    w.ed.at_time(t);
    w.ed.pointer_move(&mut w.doc, &mut w.ts, at(0.0, 0.0, NONE));
    assert!(w.ed.is_quick(), "a forma à vista");
    assert!(w.ed.finish_gesture(&mut w.doc, &mut w.h));
    assert_eq!(kind(&w.top()), Some(ShapeType::Rectangle), "a forma fica");
    assert!(w.cmd(Command::Undo));
    assert!(
        w.top().ink().is_some(),
        "o 1.º Ctrl+Z devolve o traço como foi desenhado"
    );
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.doc.live_len(), 0);
}

/// A distância de `p` à linha `poly` (fechada se `closed`).
fn to_line(p: [f64; 2], poly: &[[f64; 2]], closed: bool) -> f64 {
    let n = if closed { poly.len() } else { poly.len() - 1 };
    (0..n)
        .map(|i| crate::point_segment(p, poly[i], poly[(i + 1) % poly.len()]))
        .fold(f64::INFINITY, f64::min)
}

/// Hausdorff entre duas linhas, cada uma amostrada a cada 0,25 ao longo dos lados.
fn hausdorff(a: &[[f64; 2]], b: &[[f64; 2]], closed: bool) -> f64 {
    let dense = |l: &[[f64; 2]]| {
        let n = if closed { l.len() } else { l.len() - 1 };
        let mut v = Vec::new();
        for i in 0..n {
            let (p, q) = (l[i], l[(i + 1) % l.len()]);
            let k = ((q[0] - p[0]).hypot(q[1] - p[1]) / 0.25).ceil().max(1.0) as usize;
            v.extend((0..=k).map(|j| {
                let t = j as f64 / k as f64;
                [p[0] + t * (q[0] - p[0]), p[1] + t * (q[1] - p[1])]
            }));
        }
        v
    };
    let one = |x: &[[f64; 2]], y: &[[f64; 2]]| {
        dense(x)
            .into_iter()
            .map(|p| to_line(p, y, closed))
            .fold(0.0, f64::max)
    };
    one(a, b).max(one(b, a))
}

/// O que um caso TEM de dar (onde o desenho decide; `Any` = forma do quadro ou tinta, o que a desenhar).
#[derive(Debug, PartialEq)]
enum Lands {
    Shape(ShapeType),
    Wire([Head; 2]),
    Ink,
    Any,
}

/// ⭐ Toda forma que a lei do vetor reconhece pousa num elemento do quadro que a DESENHA (a menos de um
/// px de ecrã) — objecto, ligação com as pontas certas, ou tinta limpa sobre ela — e nunca em nada. A
/// régua é independente da do produto (Hausdorff pelas amostras contra o caminho do vetor).
#[test]
fn every_recognised_shape_lands_on_an_element_that_draws_it() {
    use ph2d_vec_edit::quickshape::{ArcShape, ArrowShape, QuickShape};
    let poly = |c: &[[f64; 2]], closed| QuickShape::Polygon {
        corners: c.to_vec(),
        closed,
    };
    let regular = |n: usize, r: f64| -> Vec<[f64; 2]> {
        (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * i as f64 / n as f64 - std::f64::consts::FRAC_PI_2;
                [300.0 + r * a.cos(), 300.0 + r * a.sin()]
            })
            .collect()
    };
    let line = QuickShape::Line {
        a: [10.0, 20.0],
        b: [410.0, 220.0],
    };
    let arc = ArcShape {
        centre: [200.0, 200.0],
        rx: 150.0,
        ry: 90.0,
        rot: 0.4,
        start: 0.3,
        sweep: 2.2,
    };
    let arrow = |shaft: QuickShape, heads| {
        QuickShape::Arrow(ArrowShape {
            shaft: Box::new(shaft),
            heads,
        })
    };
    let rect = rotated(&RECT[..4], 30.0);
    let rhombus = [[0.0, -150.0], [100.0, 0.0], [0.0, 150.0], [-100.0, 0.0]];
    let cases: Vec<(&str, QuickShape, Lands)> = vec![
        (
            "rect rodado",
            poly(&rect, true),
            Lands::Shape(ShapeType::Rectangle),
        ),
        (
            "losango",
            poly(&rhombus, true),
            Lands::Shape(ShapeType::Diamond),
        ),
        (
            "triângulo regular",
            poly(&regular(3, 150.0), true),
            Lands::Any,
        ),
        (
            "triângulo qualquer",
            poly(&[[0.0, 0.0], [400.0, 0.0], [60.0, -220.0]], true),
            Lands::Ink,
        ),
        ("pentágono", poly(&regular(5, 150.0), true), Lands::Any),
        ("hexágono", poly(&regular(6, 150.0), true), Lands::Any),
        ("octógono", poly(&regular(8, 150.0), true), Lands::Any),
        (
            "paralelogramo",
            poly(
                &[[0.0, 0.0], [300.0, 0.0], [380.0, 150.0], [80.0, 150.0]],
                true,
            ),
            Lands::Any,
        ),
        (
            "polilinha aberta",
            poly(&[[0.0, 0.0], [200.0, 0.0], [200.0, 150.0]], false),
            Lands::Ink,
        ),
        (
            "elipse rodada",
            QuickShape::Ellipse {
                centre: [300.0, 200.0],
                rx: 180.0,
                ry: 70.0,
                rot: 0.5,
            },
            Lands::Shape(ShapeType::Ellipse),
        ),
        ("arco", QuickShape::Arc(arc), Lands::Ink),
        (
            "biarco",
            QuickShape::Biarc([
                arc,
                ArcShape {
                    centre: [200.0 + 2.0 * 150.0 * (0.3f64 + 2.2).cos(), 200.0],
                    sweep: -1.5,
                    ..arc
                },
            ]),
            Lands::Ink,
        ),
        ("recta", line.clone(), Lands::Wire([Head::None; 2])),
        (
            "seta no fim",
            arrow(line.clone(), [None, Some(30.0)]),
            Lands::Wire([Head::None, Head::Arrow]),
        ),
        (
            "seta no início",
            arrow(line.clone(), [Some(30.0), None]),
            Lands::Wire([Head::Arrow, Head::None]),
        ),
        (
            "seta nas duas",
            arrow(line.clone(), [Some(30.0); 2]),
            Lands::Wire([Head::Arrow; 2]),
        ),
        (
            "seta de cabo curvo",
            arrow(QuickShape::Arc(arc), [None, Some(30.0)]),
            Lands::Ink,
        ),
    ];
    let ink = ph2d_board_model::Rgba([20, 40, 60, 255]);
    let pen_style = Style::new(None, Some(ink), ink);
    for (name, shape, want) in cases {
        let mut w = world();
        let id = w.doc.mint_id();
        let z = ph2d_board_model::FracKey::between(None, None);
        let el =
            w.ed.smart_landing(&w.doc, &shape, Pen::Pen, &pen_style, PX, (id, z));
        assert_eq!(el.id, id, "{name}");
        let path = shape.to_path();
        let truth: Vec<[f64; 2]> = ph2d_board_geom::cover::vec_polyline(&path, 0.05).concat();
        let got = if let Some(s) = el.shape() {
            if let Lands::Shape(k) = want {
                assert_eq!(s.kind, k, "{name}");
            }
            assert!(
                matches!(want, Lands::Shape(_) | Lands::Any),
                "{name}: {:?} não devia ser forma",
                s.kind
            );
            assert!(path.closed, "{name}: só um contorno fechado vira forma");
            let o = ph2d_board_geom::world_outline(&el).expect("contorno");
            let polys = ph2d_board_geom::cover::polygons(&o.fill, 0.05);
            assert_eq!(polys.len(), 1, "{name}");
            polys[0].clone()
        } else if let Some(c) = el.connector() {
            let Lands::Wire(heads) = want else {
                panic!("{name}: uma ligação, devia ser {want:?}");
            };
            assert_eq!(c.heads, heads, "{name}: as pontas");
            let ends = [c.start, c.end].map(|e| match e {
                End::Free(p) => p,
                End::Bound { .. } => panic!("{name}: preso ao quadro vazio"),
            });
            let QuickShape::Line { a, b } = (match &shape {
                QuickShape::Arrow(a) => a.shaft.as_ref().clone(),
                s => s.clone(),
            }) else {
                panic!("{name}: ligação sem recta");
            };
            assert_eq!(ends, [a, b], "{name}: as pontas da recta");
            continue;
        } else if let Some(i) = el.ink() {
            assert!(
                matches!(want, Lands::Ink | Lands::Any),
                "{name}: tinta, devia ser {want:?}"
            );
            assert!(i.clean, "{name}: a tinta LIMPA");
            if matches!(shape, QuickShape::Arrow(_)) {
                // As farpas desenham-se por cima do cabo: o cabo inteiro está na tinta.
                let pts: Vec<[f64; 2]> = crate::world_points(&el)
                    .iter()
                    .map(|p| [p[0], p[1]])
                    .collect();
                for p in &truth {
                    assert!(to_line(*p, &pts, false) <= PX, "{name}: o cabo na tinta");
                }
                continue;
            }
            crate::world_points(&el)
                .iter()
                .map(|p| [p[0], p[1]])
                .collect()
        } else {
            panic!("{name}: {:?}", el.kind);
        };
        let d = hausdorff(&got, &truth, path.closed);
        assert!(d <= PX, "{name}: desenha a {d:.3} px da forma reconhecida");
    }
}

/// ⭐ Uma ponta a cair numa forma prende-se a ela e a outra fica solta — cada ponta pela porta da seta.
#[test]
fn a_line_with_one_end_on_a_box_binds_only_that_end() {
    let mut w = world();
    let a = w.rect([0.0, 0.0, 100.0, 100.0]);
    w.cmd(Command::Escape);
    let out: Vec<[f64; 2]> = (0..=60)
        .map(|i| [50.0 + 10.0 * f64::from(i), 50.0])
        .collect();
    w.ink(&out, Some(HOLD_NS), NONE);
    let el = w.top();
    let c = el.connector().expect("uma ligação");
    assert!(
        matches!(c.start, End::Bound { target, .. } if target == a),
        "{:?}",
        c.start
    );
    assert!(matches!(c.end, End::Free(_)), "{:?}", c.end);
    let back: Vec<[f64; 2]> = out.iter().rev().map(|p| [p[0], p[1] + 300.0]).collect();
    let mut back = back;
    back.extend((1..=25).map(|i| [50.0, 350.0 - 10.0 * f64::from(i)]));
    w.ink(&back[back.len() - 31..], Some(HOLD_NS), NONE);
    let el = w.top();
    let c = el.connector().expect("uma ligação");
    assert!(matches!(c.start, End::Free(_)), "{:?}", c.start);
    assert!(
        matches!(c.end, End::Bound { target, .. } if target == a),
        "{:?}",
        c.end
    );
}

/// ⭐ Em TODAS as famílias o 1.º `Ctrl+Z` devolve o traço tal como foi desenhado (cada ponto) e o 2.º
/// apaga-o: forma do quadro, ligação e tinta limpa.
#[test]
fn ctrl_z_gives_every_family_its_stroke_back() {
    let circle: Vec<[f64; 2]> = (0..=120)
        .map(|i| {
            let a = std::f64::consts::TAU * f64::from(i) / 120.0;
            [500.0 + 200.0 * a.cos(), 500.0 + 120.0 * a.sin()]
        })
        .collect();
    let rhombus = sides(
        &[
            [0.0, -150.0],
            [100.0, 0.0],
            [0.0, 150.0],
            [-100.0, 0.0],
            [0.0, -150.0],
        ],
        40,
    );
    let scalene = sides(&[[0.0, 0.0], [400.0, 0.0], [60.0, -220.0], [0.0, 0.0]], 50);
    let line: Vec<[f64; 2]> = (0..=60).map(|i| [10.0 * f64::from(i), 0.0]).collect();
    let mut arrow = line.clone();
    arrow.extend(sides(&[[600.0, 0.0], [540.0, -40.0]], 10));
    arrow.extend(sides(&[[540.0, -40.0], [600.0, 0.0], [540.0, 40.0]], 10));
    for (name, pts, family) in [
        ("elipse", circle, "ellipse"),
        ("losango", rhombus, "diamond"),
        ("triângulo qualquer", scalene, "ink"),
        ("recta", line, "line"),
        ("seta", arrow, "arrow"),
    ] {
        let mut w = world();
        w.ink(&pts, Some(HOLD_NS), NONE);
        let el = w.top();
        let got = match (el.shape(), el.connector(), el.ink()) {
            (Some(s), ..) => format!("{:?}", s.kind).to_lowercase(),
            (_, Some(c), _) if c.heads == [Head::None; 2] => "line".into(),
            (_, Some(_), _) => "arrow".into(),
            (.., Some(i)) if i.clean => "ink".into(),
            _ => format!("{:?}", el.kind),
        };
        assert_eq!(got, family, "{name}");
        assert!(w.cmd(Command::Undo), "{name}");
        let raw = w.top();
        assert!(raw.ink().is_some_and(|i| !i.clean), "{name}: o traço cru");
        let drawn: Vec<[f64; 2]> = crate::world_points(&raw)
            .iter()
            .map(|p| [p[0], p[1]])
            .collect();
        let mut pts = pts;
        pts.dedup();
        assert_eq!(drawn.len(), pts.len(), "{name}: cada ponto");
        for (p, q) in drawn.iter().zip(&pts) {
            assert!(
                (p[0] - q[0]).hypot(p[1] - q[1]) < 1e-3,
                "{name}: {p:?} ≠ {q:?}"
            );
        }
        assert!(w.cmd(Command::Undo), "{name}");
        assert_eq!(w.doc.live_len(), 0, "{name}");
    }
}

/// ⭐ Censo: o quadro NÃO tem limiares de reconhecimento seus (doc 17 §2.2). Só o `smart.rs` fala com a
/// lei e o gatilho do vetor; ele usa as folgas do vetor pelo nome; e os seus números são só os do
/// desenho (o passo da tinta, a farpa), cada um nomeado aqui.
#[test]
fn the_board_has_no_recognition_thresholds_of_its_own() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let mut files = Vec::new();
    for dir in std::fs::read_dir(root).unwrap() {
        let dir = dir.unwrap().path();
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        if name.starts_with("ph2d-board-") || name == "ph2d-app-board" {
            files.extend(rs_files(&dir.join("src")));
        }
    }
    let hero = format!("{root}/ph2d-editor-core/src/screens/hero");
    files.extend(
        rs_files(std::path::Path::new(&hero))
            .into_iter()
            .filter(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("board")
            }),
    );
    files.retain(|p| {
        let n = p.file_name().unwrap().to_string_lossy();
        !n.ends_with("_tests.rs") && !n.starts_with("smart_test")
    });
    assert!(files.len() > 100, "o censo varre {} ficheiros", files.len());
    // O que cada ficheiro pede ao vetor (o caminho até ao `;`).
    let mut asks = std::collections::BTreeMap::new();
    for f in &files {
        let code = strip_comments(&std::fs::read_to_string(f).unwrap());
        let uses: Vec<String> = code
            .match_indices("ph2d_vec_edit::")
            .map(|(i, _)| {
                code[i..]
                    .split(';')
                    .next()
                    .unwrap()
                    .split_whitespace()
                    .collect()
            })
            .collect();
        if !uses.is_empty() {
            let rel = f.strip_prefix(root).unwrap().to_string_lossy().into_owned();
            asks.insert(rel, uses);
        }
    }
    let want: std::collections::BTreeMap<String, Vec<String>> = [
        (
            "ph2d-board-edit/src/ink.rs",
            &["ph2d_vec_edit::quickshape_hold::HoldTick"][..],
        ),
        (
            "ph2d-board-edit/src/smart.rs",
            &[
                "ph2d_vec_edit::pencil::MIN_SAMPLE_PX",
                "ph2d_vec_edit::quickshape::QuickShape",
                "ph2d_vec_edit::quickshape_hold::{HOLD_SLOP_PX,Hold,HoldTick,rest_index}",
            ][..],
        ),
    ]
    .into_iter()
    .map(|(f, u)| (f.to_owned(), u.iter().map(|s| (*s).to_owned()).collect()))
    .collect();
    assert_eq!(asks, want, "quem pede o quê à lei do vetor");
    let code = strip_comments(include_str!("smart.rs"));
    for name in [
        "HOLD_SLOP_PX * self.px",
        "MIN_SAMPLE_PX * self.px",
        ".tick(&self.samples",
    ] {
        assert!(
            code.contains(name),
            "o gatilho usa a folga do vetor: {name}"
        );
    }
    let consts: Vec<&str> = code
        .match_indices("const ")
        .map(|(i, _)| code[i + 6..].split(':').next().unwrap().trim())
        .collect();
    assert_eq!(
        consts,
        ["POLYGON_KINDS", "BARB_DEG"],
        "as constantes do smart.rs"
    );
    // Os números do smart.rs: índices e tamanhos (0–4, os 7 tipos), metades, o quarto de px do achatar,
    // a pressão 0,5 da tinta limpa, o ε, e a farpa de 30°.
    let allowed = [
        "0", "1", "2", "3", "4", "7", "1.0", "2.0", "4.0", "0.25", "0.5", "1e-9", "30.0",
    ];
    let mut odd = Vec::new();
    let b = code.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let starts = b[i].is_ascii_digit()
            && (i == 0
                || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_' || b[i - 1] == b'.'));
        if !starts {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < b.len()
            && (b[j].is_ascii_alphanumeric()
                || b[j] == b'_'
                || (b[j] == b'.' && b.get(j + 1).is_some_and(u8::is_ascii_digit))
                || (b[j] == b'-' && b[j - 1] == b'e'))
        {
            j += 1;
        }
        let lit = code[i..j].trim_end_matches('.');
        if !allowed.contains(&lit) {
            odd.push(lit.to_owned());
        }
        i = j;
    }
    assert!(odd.is_empty(), "números novos no smart.rs: {odd:?}");
}

fn rs_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return out;
    };
    for e in rd {
        let p = e.unwrap().path();
        if p.is_dir() {
            out.extend(rs_files(&p));
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
    out.sort();
    out
}

fn strip_comments(s: &str) -> String {
    s.lines()
        .map(|l| l.find("//").map_or(l, |i| &l[..i]))
        .collect::<Vec<_>>()
        .join("\n")
}
