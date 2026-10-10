//! O rascunho e a caneta no desenho.

use super::*;
use ph2d_board_model::{ElementId, FracKey, Shape};
use ph2d_vector::Shape as _;

fn stroke(width: f64, pressure: bool) -> (Element, Ink) {
    let ink_color = Rgba([10, 10, 10, 255]);
    let mut style = Style::new(None, Some(ink_color), ink_color);
    style.stroke_width = width;
    // A pressão VARIA (0,1 → 0,9): sem ela medida, a largura não pode segui-la.
    let pts: Vec<[f64; 3]> = (0..=40)
        .map(|i| [f64::from(i) * 5.0, 100.0, 0.1 + 0.8 * f64::from(i) / 40.0])
        .collect();
    let (ink, bx) = Ink::from_world(&pts, style, Pen::Pen, pressure);
    let el = Element::new_ink(ElementId(1), FracKey::between(None, None), ink.clone(), bx);
    (el, ink)
}

/// ⭐ A lei do Miro: sem pressão medida, a caneta tem a MESMA largura de ponta a ponta (fora das
/// tampas redondas).
#[test]
fn a_pen_stroke_without_pressure_keeps_its_width_from_end_to_end() {
    let (el, ink) = stroke(8.0, false);
    let path = ink_outline(&ink, el.w, el.h);
    let mid_y = el.h / 2.0;
    let mut checked = 0;
    for seg in path.segments() {
        let p = seg.eval(0.5);
        if p.x > 30.0 && p.x < el.w - 30.0 {
            assert!(
                ((p.y - mid_y).abs() - 4.0).abs() < 0.05,
                "meia largura 4 em x = {}: {}",
                p.x,
                p.y
            );
            checked += 1;
        }
    }
    assert!(
        checked > 20,
        "o miolo do traço foi medido ({checked} pontos)"
    );
    let b = path.bounding_box();
    assert!(
        (b.height() - 8.0).abs() < 0.1,
        "a altura é a espessura: {}",
        b.height()
    );
}

fn rect(id: u64) -> Element {
    let ink = Rgba([10, 10, 10, 255]);
    let mut style = Style::new(Some(Rgba([250, 200, 100, 255])), Some(ink), ink);
    style.sketch = true;
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style,
        text: Default::default(),
    };
    Element::new_shape(
        ElementId(id),
        FracKey::between(None, None),
        shape,
        [0.0, 0.0, 160.0, 100.0],
    )
}

fn drawn(el: &Element) -> (Vec<PathEl>, Vec<PathEl>) {
    let sh = el.shape().unwrap();
    let o = ph2d_board_geom::outline(sh, el.w, el.h);
    let r = rough_shape(el, sh.kind, &sh.style, &o);
    assert!(
        !r.line.is_empty() && !r.fill.is_empty(),
        "contorno e preenchimento à mão"
    );
    assert!(
        r.hachure.is_empty(),
        "o preenchimento é cheio, nunca às riscas (a letra lê-se nele)"
    );
    (r.line.elements().to_vec(), r.fill.elements().to_vec())
}

/// ⭐ A SEMENTE do elemento: o mesmo elemento treme sempre igual (quadro a quadro, sessão a
/// sessão); outro elemento treme de outra maneira.
#[test]
fn a_sketched_shape_draws_the_same_for_its_seed_and_differently_for_another() {
    assert_eq!(drawn(&rect(7)), drawn(&rect(7)));
    assert_ne!(drawn(&rect(7)).0, drawn(&rect(8)).0);
}

/// O rascunho fica PERTO da forma: o tremor é de mão, não de outra forma.
#[test]
fn the_sketch_stays_close_to_the_true_outline() {
    let el = rect(3);
    let (line, _) = drawn(&el);
    let mut p = BezPath::new();
    p.extend(line);
    let b = p.bounding_box();
    for (got, want) in [(b.x0, 0.0), (b.y0, 0.0), (b.x1, 160.0), (b.y1, 100.0)] {
        assert!((got - want).abs() < 8.0, "{got} longe de {want}");
    }
}

/// ⭐ A TINTA DO TEMA (smoke do dono, 07/10): a tinta de nascença desenha-se `#1e1e1e` num quadro
/// claro e no texto do tema num escuro — em TODOS os temas; uma cor escolhida pelo artista nunca
/// muda, nem o texto dentro de uma forma preenchida (lê-se contra o preenchimento).
#[test]
fn the_default_ink_is_near_black_on_light_boards_and_the_themes_text_on_dark_ones() {
    use ph2d_tokens::{ColorToken, Theme};
    let ink = Rgba(ph2d_board_model::DEFAULT_INK);
    let red = Rgba([200, 30, 30, 255]);
    let mut lights = 0;
    for theme in Theme::ALL {
        let bg = ColorToken::Bg1.resolve(theme);
        let light = Rgba([bg.r, bg.g, bg.b, 255]).luminance() >= 0.5;
        let t = crate::ThemeInk::of(theme);
        let drawn = t.color(ink).to_rgba8();
        let want = if light {
            lights += 1;
            [30, 30, 30, 255]
        } else {
            let c = ColorToken::Text1.resolve(theme);
            [c.r, c.g, c.b, c.a]
        };
        assert_eq!([drawn.r, drawn.g, drawn.b, drawn.a], want, "{theme:?}");
        let r = t.color(red).to_rgba8();
        assert_eq!(
            [r.r, r.g, r.b],
            [200, 30, 30],
            "{theme:?}: a cor do artista mudou"
        );
        let mut filled = Style::new(Some(Rgba([250, 230, 150, 255])), Some(ink), ink);
        let on_fill = t.text(&filled).to_rgba8();
        assert_eq!(
            [on_fill.r, on_fill.g, on_fill.b],
            [30, 30, 30],
            "{theme:?}: texto sobre preenchimento"
        );
        filled.fill = None;
        assert_eq!(t.text(&filled), t.color(ink));
    }
    assert!(
        lights > 0 && lights < Theme::ALL.len(),
        "há temas claros e escuros na régua"
    );
}

/// ⭐ O traço à mão de uma forma cabe na folga da camada de opacidade ([`hand_reach`]) — também no
/// «cartoonist» importado (W9: vértices soltos), em muitas sementes (a auditoria W9 viu os cantos
/// cortados a direito com a folga de só o traço).
#[test]
fn a_hand_drawn_shape_stays_inside_its_opacity_layer() {
    let ink = ph2d_board_model::Rgba([30, 30, 30, 255]);
    for (cartoonist, w, h) in [
        (true, 160.0, 100.0),
        (false, 160.0, 100.0),
        (true, 600.0, 40.0),
    ] {
        let mut st = Style::new(Some(ink), Some(ink), ink);
        st.sketch = true;
        st.look.cartoonist = cartoonist;
        st.stroke_width = 1.0;
        for id in 1..200 {
            let el = Element::new_shape(
                ElementId(id),
                FracKey::between(None, None),
                Shape {
                    kind: ShapeType::Rectangle,
                    style: st.clone(),
                    text: Default::default(),
                },
                [0.0, 0.0, w, h],
            );
            let o = ph2d_board_geom::outline(el.shape().unwrap(), w, h);
            let r = rough_shape(&el, ShapeType::Rectangle, &st, &o);
            let b = r.line.bounding_box();
            let pad = hand_reach(&st, w, h);
            assert!(
                b.x0 >= -pad && b.y0 >= -pad && b.x1 <= w + pad && b.y1 <= h + pad,
                "semente {id}, cartoonist {cartoonist}: {b:?} sai de ±{pad}"
            );
        }
    }
}

/// O maior afastamento (mundo) do traço `hand` à rota `route`, amostrado ao longo do traço.
fn worst_gap(hand: &BezPath, route: &BezPath) -> f64 {
    use ph2d_vector::{ParamCurve, ParamCurveNearest};
    let segs: Vec<_> = route.segments().collect();
    let mut worst: f64 = 0.0;
    for s in hand.segments() {
        for k in 0..=32 {
            let p = s.eval(f64::from(k) / 32.0);
            let d = segs
                .iter()
                .map(|r| r.nearest(p, 1e-9).distance_sq)
                .fold(f64::INFINITY, f64::min)
                .sqrt();
            worst = worst.max(d);
        }
    }
    worst
}

/// ⭐ O traço À MÃO de uma seta CURVA segue a curva dela — a guia da selecção (smoke do dono, 10/10:
/// «se aplicar a opção hand-draw a seta fica deformada»: com três pontos por cúbica o Catmull-Rom do
/// rough.js cortava o canto à saída e chegava torto à ponta). Sem tremor (rugosidade 0) o traço fica a
/// menos de [`HAND_CURVE_TOL`] da rota, na curva do Miro em «U» e na curva do Excalidraw com pontos.
#[test]
fn a_hand_drawn_curved_arrow_follows_its_route() {
    use ph2d_board_model::{Anchor, BoardDoc, BoardOp, Connector, End, Head, Route};
    let ink = ph2d_board_model::Rgba([30, 30, 30, 255]);
    let mut doc = BoardDoc::default();
    let shape = |doc: &mut BoardDoc, bx: [f64; 4]| {
        let el = Element::new_shape(
            doc.mint_id(),
            doc.z_on_top(),
            Shape {
                kind: ShapeType::Ellipse,
                style: Style::new(None, Some(ink), ink),
                text: Default::default(),
            },
            bx,
        );
        let id = el.id;
        BoardOp::Put(el).apply(doc);
        id
    };
    let a = shape(&mut doc, [0.0, 0.0, 400.0, 215.0]);
    let b = shape(&mut doc, [700.0, -200.0, 120.0, 155.0]);
    let bottom = |t| End::Bound {
        target: t,
        anchor: Anchor::Orbit([0.5, 1.0]),
    };
    let mut ids = Vec::new();
    for (route, waypoints) in [
        (Route::Curved, vec![]),
        (Route::Spline, vec![[300.0, 420.0], [650.0, 300.0]]),
    ] {
        let mut c = Connector::new(
            bottom(a),
            bottom(b),
            route,
            Style::new(None, Some(ink), ink),
        );
        c.heads = [Head::None, Head::Arrow];
        c.waypoints = waypoints;
        let el = Element::new_connector(doc.mint_id(), doc.z_on_top(), c);
        ids.push(el.id);
        BoardOp::Put(el).apply(&mut doc);
    }
    let mut routes = ph2d_board_route::RouteCache::default();
    routes.sync(&doc);
    let opts = rough::Options {
        roughness: 0.0,
        ..rough::Options::default()
    };
    for id in ids {
        let r = routes.get(id).unwrap();
        let d = ph2d_board_route::drawn(r, [Head::None, Head::Arrow], 2.0);
        let mut hand = BezPath::new();
        for s in hand_line(&d.line, &opts) {
            bez(&s.ops, &mut hand);
        }
        let gap = worst_gap(&hand, &d.line);
        assert!(
            gap <= HAND_CURVE_TOL + 1e-6,
            "seta {}: o traço à mão afasta-se {gap:.2} da rota",
            id.0
        );
    }
}
