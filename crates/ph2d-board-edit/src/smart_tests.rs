//! ⭐ W23 B13 — o QuickShape do vetor na caneta do quadro pelo caminho de quem desenha (doc 17 §2.2):
//! segurar parado no fim do traço; o mesmo meio segundo do vetor; a forma que se vê é a reconhecida e
//! vira o objecto do quadro que a desenha igual (ou tinta limpa); `Ctrl+Z` devolve o traço.

use super::*;
use ph2d_board_model::{End, Head, Pen, ShapeType};
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
