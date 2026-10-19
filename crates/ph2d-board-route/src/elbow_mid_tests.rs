//! W23 B5(1) — a lei do segmento do meio, com as caixas dos oráculos (doc 13 §1: R1 = 0,0 100×100,
//! R2 = 400,300 100×100; o tldraw 5.5.2 e o Excalidraw 0.18.1 correram sobre estas).

use ph2d_board_model::{
    Anchor, BoardDoc, BoardOp, Connector, Element, ElementId, End, Rgba, Route, Shape, ShapeType,
    Style,
};

use crate::{ElbowMid, RouteCache};

struct Scene {
    doc: BoardDoc,
    r2: ElementId,
    arrow: ElementId,
}

impl Scene {
    fn new() -> Self {
        let mut doc = BoardDoc::default();
        let ink = Rgba([0, 0, 0, 255]);
        let mut boxed = |doc: &mut BoardDoc, bx: [f64; 4]| {
            let s = Shape {
                kind: ShapeType::Rectangle,
                style: Style::new(None, Some(ink), ink),
                text: Default::default(),
            };
            let el = Element::new_shape(doc.mint_id(), doc.z_on_top(), s, bx);
            let id = el.id;
            BoardOp::Put(el).apply(doc);
            id
        };
        let r1 = boxed(&mut doc, [0.0, 0.0, 100.0, 100.0]);
        let r2 = boxed(&mut doc, [400.0, 300.0, 100.0, 100.0]);
        let at = |target| End::Bound {
            target,
            anchor: Anchor::Center,
        };
        let c = Connector::new(at(r1), at(r2), Route::Elbow, Style::new(None, Some(ink), ink));
        let el = Element::new_connector(doc.mint_id(), doc.z_on_top(), c);
        let arrow = el.id;
        BoardOp::Put(el).apply(&mut doc);
        Self { doc, r2, arrow }
    }

    fn elbow(&self) -> (Option<ElbowMid>, Vec<[f64; 2]>) {
        let mut cache = RouteCache::default();
        cache.sync(&self.doc);
        let r = cache.get(self.arrow).unwrap();
        (r.elbows.first().copied().flatten(), r.polyline())
    }

    fn pin(&mut self, frac: Option<f64>) {
        let mut el = self.doc.get(self.arrow).unwrap().clone();
        el.connector_mut().unwrap().set_mid(0, frac);
        BoardOp::Put(el).apply(&mut self.doc);
    }

    fn r2_to(&mut self, x: f64, y: f64) {
        let mut el = self.doc.get(self.r2).unwrap().clone();
        (el.x, el.y) = (x, y);
        BoardOp::Put(el).apply(&mut self.doc);
    }
}

/// ⭐ O segmento do meio arrastado até x = 310 (o +60 dos oráculos) segue a fracção quando a R2 vai
/// para (700, 300) — proporcional, como o tldraw (310 → 538,95 lá), e não absoluto como o
/// Excalidraw (que o deixava em 310).
#[test]
fn a_dragged_middle_follows_its_fraction_when_a_box_moves() {
    let mut s = Scene::new();
    let (born, line) = s.elbow();
    let born = born.expect("de nascença sai em Z");
    assert!(born.vertical);
    assert!((born.at[0] - 250.0).abs() < 1e-6, "nasce a meio: {line:?}");
    assert!((born.frac(born.at) - 0.5).abs() < 1e-9, "o meio é a fracção 0,5");
    let f = born.frac([310.0, 0.0]);
    s.pin(Some(f));
    let (m, line) = s.elbow();
    assert!((m.unwrap().at[0] - 310.0).abs() < 1e-6, "{line:?}");
    assert_eq!(line[1][0], line[2][0], "o segmento continua vertical");
    s.r2_to(700.0, 300.0);
    let (m, line) = s.elbow();
    let m = m.expect("continua Z");
    let x = m.at[0];
    assert!((x - 310.0).abs() > 50.0, "seguiu a caixa: {line:?}");
    let want = m.span[0] + f * (m.span[1] - m.span[0]);
    assert!((x - want).abs() < 1e-6, "{x} contra a fracção {want}");
}

/// ⭐ Na recta (a R2 alinhada com a R1) não há segmento do meio: a fracção fica guardada sem uso, e
/// volta a valer quando o Z volta (o tldraw; o Excalidraw perdia-a para sempre).
#[test]
fn a_fraction_unused_by_a_straight_line_comes_back_with_the_z() {
    let mut s = Scene::new();
    let f = s.elbow().0.unwrap().frac([310.0, 0.0]);
    s.pin(Some(f));
    s.r2_to(400.0, 0.0);
    let (m, line) = s.elbow();
    assert!(m.is_none(), "recta: {line:?}");
    assert_eq!(
        s.doc.get(s.arrow).unwrap().connector().unwrap().mid(0),
        Some(f)
    );
    s.r2_to(400.0, 300.0);
    assert!((s.elbow().0.unwrap().at[0] - 310.0).abs() < 1e-6);
}

/// ⭐ Nos dois extremos (0 e 1, e além: o arrasto prende) o segmento do meio fica FORA das duas
/// caixas — o Excalidraw atravessava a caixa (foto `excalidraw_C3_*`).
#[test]
fn the_middle_never_enters_a_box() {
    let mut s = Scene::new();
    for f in [-1.0, 0.0, 1.0, 2.0] {
        s.pin(Some(f));
        let (m, line) = s.elbow();
        let x = m.unwrap().at[0];
        assert!(x > 100.0 && x < 400.0, "f = {f}: x = {x} ({line:?})");
        assert!(line.windows(2).all(|w| w[0] != w[1]), "nenhum segmento nulo: {line:?}");
    }
}
