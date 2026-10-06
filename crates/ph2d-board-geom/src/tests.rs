use super::*;
use ph2d_board_model::{ElementId, FracKey, Rgba, Style};

fn shape(kind: ShapeType, round: bool) -> Shape {
    let ink = Rgba([0, 0, 0, 255]);
    let mut style = Style::new(Some(ink), Some(ink), ink);
    style.round = round;
    Shape {
        kind,
        style,
        text: String::new(),
    }
}

fn el(kind: ShapeType, [x, y, w, h]: [f64; 4], angle: f64) -> Element {
    let mut e = Element::new_shape(
        ElementId(1),
        FracKey::between(None, None),
        shape(kind, false),
        [x, y, w, h],
    );
    e.angle = angle;
    e
}

#[test]
fn every_shape_type_has_a_closed_outline_inside_its_box() {
    for &t in ShapeType::ALL {
        for round in [false, true] {
            let o = outline(&shape(t, round), 200.0, 120.0);
            let b = o.fill.bounding_box();
            assert!(b.area() > 0.0, "{t:?} sem contorno");
            // Folga de 0,1 un.: a curva do Documento do catálogo vectorial sobe 0,04 acima do topo.
            let e = 0.1;
            assert!(
                b.x0 >= -e && b.y0 >= -e && b.x1 <= 200.0 + e && b.y1 <= 120.0 + e,
                "{t:?} (round={round}) sai da caixa: {b:?}"
            );
            // O centro da caixa de texto está DENTRO da forma.
            let [x, y, w, h] = text_rect(t, 200.0, 120.0);
            assert!(
                o.fill.contains(Point::new(x + w / 2.0, y + h / 2.0)),
                "{t:?}: o texto vive fora da forma"
            );
        }
    }
}

#[test]
fn the_corner_radius_is_a_quarter_of_the_short_side_up_to_the_cap() {
    assert_eq!(corner_radius(40.0), 10.0);
    assert_eq!(corner_radius(128.0), 32.0);
    assert_eq!(corner_radius(1000.0), 32.0);
}

#[test]
fn a_round_rectangle_leaves_its_corner_and_keeps_its_sides() {
    let o = outline(&shape(ShapeType::Rectangle, true), 100.0, 100.0);
    assert!(!o.fill.contains(Point::new(1.0, 1.0)), "o canto ainda lá");
    assert!(
        o.fill.contains(Point::new(50.0, 1.0)),
        "o lado de cima perdeu-se"
    );
}

#[test]
fn hit_follows_the_rotation_and_the_outline_not_the_box() {
    // Losango 100×100 em (0,0): o canto da CAIXA não é da forma.
    let d = el(ShapeType::Diamond, [0.0, 0.0, 100.0, 100.0], 0.0);
    assert!(hit(&d, [50.0, 50.0], 0.0));
    assert!(!hit(&d, [5.0, 5.0], 0.0));
    assert!(hit(&d, [5.0, 5.0], 3.0) == false && hit(&d, [26.0, 26.0], 2.0));
    // Rectângulo 200×20 rodado 90°: passa a ocupar a vertical à volta de (100, 10).
    let r = el(
        ShapeType::Rectangle,
        [0.0, 0.0, 200.0, 20.0],
        std::f64::consts::FRAC_PI_2,
    );
    assert!(hit(&r, [100.0, 90.0], 0.0));
    assert!(
        !hit(&r, [180.0, 10.0], 0.0),
        "a ponta de antes de rodar ainda acerta"
    );
}

#[test]
fn the_shape_grows_down_only_when_the_text_does_not_fit() {
    assert_eq!(height_for_text(ShapeType::Rectangle, 100.0, 20.0), 100.0);
    let grown = height_for_text(ShapeType::Rectangle, 100.0, 200.0);
    assert_eq!(grown, 200.0 + 2.0 * TEXT_PADDING);
    // No losango o texto só usa metade da altura: precisa do DOBRO.
    let d = height_for_text(ShapeType::Diamond, 10.0, 100.0);
    assert!((d - (100.0 + 2.0 * TEXT_PADDING) * 2.0).abs() < 1e-9);
}

#[test]
fn to_world_puts_the_local_box_on_the_element() {
    let e = el(ShapeType::Rectangle, [10.0, 20.0, 30.0, 40.0], 0.0);
    let p = to_world(&e) * Point::new(30.0, 40.0);
    assert!((p.x - 40.0).abs() < 1e-9 && (p.y - 60.0).abs() < 1e-9);
    let r = el(
        ShapeType::Rectangle,
        [0.0, 0.0, 20.0, 10.0],
        std::f64::consts::FRAC_PI_2,
    );
    let q = to_world(&r) * Point::new(0.0, 0.0);
    let w = r.rotate([0.0, 0.0]);
    assert!(
        (q.x - w[0]).abs() < 1e-9 && (q.y - w[1]).abs() < 1e-9,
        "dois caminhos de rodar discordam"
    );
}

/// ⛔ O catálogo vectorial é Y para cima; o quadro é Y para baixo. Os símbolos que têm um «em baixo»
/// têm-no em baixo: a onda do Documento, a ponta do «fora da página», o bico do balão, a tampa do
/// cilindro em cima, e a operação manual com o lado longo em cima.
#[test]
fn flowchart_symbols_are_the_right_way_up() {
    let (w, h) = (100.0, 100.0);
    let o = |t| outline(&shape(t, false), w, h);
    // Documento: o canto inferior direito é recto, o esquerdo desce mais (a folha) — o topo é recto.
    let doc = o(ShapeType::Document).fill;
    assert!(
        doc.contains(Point::new(50.0, 2.0)),
        "o topo do documento não é recto"
    );
    assert!(
        !doc.contains(Point::new(50.0, 99.0)),
        "a onda não está em baixo"
    );
    // Fora da página: a ponta em baixo, ao meio.
    let off = o(ShapeType::OffPage).fill;
    assert!(off.contains(Point::new(50.0, 95.0)) && !off.contains(Point::new(5.0, 95.0)));
    // Balão: o corpo em cima (o canto superior esquerdo é corpo), o bico em baixo.
    let sp = o(ShapeType::SpeechRect).fill;
    assert!(
        sp.contains(Point::new(10.0, 10.0)),
        "o corpo do balão não está em cima"
    );
    // Operação manual: larga em cima, estreita em baixo.
    let tr = o(ShapeType::Trapezoid).fill;
    assert!(tr.contains(Point::new(3.0, 3.0)) && !tr.contains(Point::new(3.0, 97.0)));
    // Cilindro: a tampa (linha de construção) na metade de cima.
    let cy = o(ShapeType::Cylinder).lines.bounding_box();
    assert!(cy.y1 < h / 2.0, "a tampa está em baixo: {cy:?}");
}
