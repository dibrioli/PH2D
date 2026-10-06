use super::*;
use ph2d_board_model::{
    BoardDoc, BoardOp, Connector, Element, ElementId, End, Rgba, Route, Shape, ShapeType, Style,
};

fn ink() -> Rgba {
    Rgba([0, 0, 0, 255])
}

fn shape(doc: &mut BoardDoc, kind: ShapeType, bx: [f64; 4]) -> ElementId {
    let s = Shape {
        kind,
        style: Style::new(None, Some(ink()), ink()),
        text: String::new(),
    };
    let el = Element::new_shape(doc.mint_id(), doc.z_on_top(), s, bx);
    let id = el.id;
    BoardOp::Put(el).apply(doc);
    id
}

fn center(t: ElementId) -> End {
    End::Bound {
        target: t,
        anchor: Anchor::Center,
    }
}

fn link(doc: &mut BoardDoc, a: End, b: End, route: Route) -> ElementId {
    let c = Connector::new(a, b, route, Style::new(None, Some(ink()), ink()));
    let el = Element::new_connector(doc.mint_id(), doc.z_on_top(), c);
    let id = el.id;
    BoardOp::Put(el).apply(doc);
    id
}

fn moved(doc: &mut BoardDoc, id: ElementId, d: [f64; 2]) {
    let mut el = doc.get(id).unwrap().clone();
    el.translate(d);
    BoardOp::Put(el).apply(doc);
}

/// ⭐ A cache: o mesmo documento não volta ao roteador; mexer numa forma LONGE da seta também não;
/// mexer numa das pontas refá-la a ela só.
#[test]
fn only_the_arrow_whose_end_or_nearby_obstacle_moved_is_rerouted() {
    let mut doc = BoardDoc::default();
    let a = shape(&mut doc, ShapeType::Rectangle, [0.0, 0.0, 160.0, 100.0]);
    let b = shape(&mut doc, ShapeType::Rectangle, [400.0, 0.0, 160.0, 100.0]);
    let c = shape(&mut doc, ShapeType::Rectangle, [0.0, 400.0, 160.0, 100.0]);
    let far = shape(
        &mut doc,
        ShapeType::Rectangle,
        [5000.0, 5000.0, 160.0, 100.0],
    );
    link(&mut doc, center(a), center(b), Route::Elbow);
    link(&mut doc, center(a), center(c), Route::Elbow);
    let mut cache = RouteCache::default();
    cache.sync(&doc);
    assert_eq!(cache.rerouted(), 2, "a primeira vez, todas");
    cache.sync(&doc);
    assert_eq!(
        cache.rerouted(),
        2,
        "nada mudou: o sync nem corre (a contagem é a do último)"
    );
    moved(&mut doc, far, [10.0, 0.0]);
    cache.sync(&doc);
    assert_eq!(
        cache.rerouted(),
        0,
        "uma forma longe não é obstáculo de ninguém"
    );
    moved(&mut doc, b, [0.0, 30.0]);
    cache.sync(&doc);
    assert_eq!(cache.rerouted(), 1, "só a seta presa a b");
}

/// Uma forma que entra no CAMINHO de uma seta é obstáculo: a seta refaz-se e desvia dela.
#[test]
fn a_shape_dropped_in_the_way_reroutes_the_arrow_around_it() {
    let mut doc = BoardDoc::default();
    let a = shape(&mut doc, ShapeType::Rectangle, [0.0, 0.0, 160.0, 100.0]);
    let b = shape(&mut doc, ShapeType::Rectangle, [600.0, 0.0, 160.0, 100.0]);
    let wall = shape(&mut doc, ShapeType::Rectangle, [3000.0, 0.0, 100.0, 300.0]);
    let id = link(&mut doc, center(a), center(b), Route::Elbow);
    let mut cache = RouteCache::default();
    cache.sync(&doc);
    assert_eq!(
        cache.get(id).unwrap().pts.len(),
        2,
        "sem nada no meio, uma recta"
    );
    moved(&mut doc, wall, [-2670.0, -100.0]);
    cache.sync(&doc);
    assert_eq!(cache.rerouted(), 1);
    let r = cache.get(id).unwrap();
    assert!(
        r.pts.len() > 2,
        "a parede no caminho obriga a dobrar: {:?}",
        r.pts
    );
    assert!(
        r.pts
            .iter()
            .all(|p| !(p[0] > 330.0 && p[0] < 430.0 && p[1] > -100.0 && p[1] < 200.0)),
        "nenhum ponto dentro da parede: {:?}",
        r.pts
    );
}

/// ⭐ Presa ao CENTRO, a seta sai pela face que olha para a outra forma — e troca de face quando a
/// forma passa para baixo (o «nunca atravessa a forma» do Miro).
#[test]
fn a_center_anchor_follows_the_nearest_face() {
    let mut doc = BoardDoc::default();
    let a = shape(&mut doc, ShapeType::Rectangle, [0.0, 0.0, 160.0, 100.0]);
    let b = shape(&mut doc, ShapeType::Rectangle, [400.0, 0.0, 160.0, 100.0]);
    let id = link(&mut doc, center(a), center(b), Route::Elbow);
    let mut cache = RouteCache::default();
    cache.sync(&doc);
    assert_eq!(
        cache.get(id).unwrap().ends()[0],
        [160.0, 50.0],
        "a face direita"
    );
    moved(&mut doc, b, [-400.0, 600.0]);
    cache.sync(&doc);
    // `North` é +y: «para baixo» no quadro (doc da crate).
    assert_eq!(cache.get(id).unwrap().sides[0], Dir::North);
    assert_eq!(
        cache.get(id).unwrap().ends()[0],
        [80.0, 100.0],
        "a face de baixo"
    );
}

/// A recta presa ao centro encosta no CONTORNO verdadeiro (numa elipse, não na caixa dela).
#[test]
fn a_straight_arrow_touches_the_real_outline() {
    let mut doc = BoardDoc::default();
    let a = shape(&mut doc, ShapeType::Ellipse, [0.0, 0.0, 200.0, 200.0]);
    let b = shape(&mut doc, ShapeType::Rectangle, [600.0, 600.0, 100.0, 100.0]);
    let id = link(&mut doc, center(a), center(b), Route::Straight);
    let mut cache = RouteCache::default();
    cache.sync(&doc);
    let p = cache.get(id).unwrap().ends()[0];
    let r = (p[0] - 100.0).hypot(p[1] - 100.0);
    assert!(
        (r - 100.0).abs() < 0.5,
        "no círculo (raio 100), não na quina da caixa: {p:?} r={r}"
    );
}

/// ⭐ A regra da ligação: miolo = centro; faixa junto ao contorno = ponto fixo (colado ao meio do
/// lado quando perto dele); fora = nada.
#[test]
fn the_binding_rule_center_inside_fixed_near_the_outline() {
    let mut doc = BoardDoc::default();
    let id = shape(&mut doc, ShapeType::Rectangle, [0.0, 0.0, 200.0, 100.0]);
    let el = doc.get(id).unwrap();
    let band = 10.0;
    assert_eq!(anchor_for(el, [100.0, 50.0], band), Some(Anchor::Center));
    assert_eq!(
        anchor_for(el, [204.0, 53.0], band),
        Some(Anchor::Fixed([1.0, 0.5])),
        "colado ao meio"
    );
    let Some(Anchor::Fixed([u, v])) = anchor_for(el, [197.0, 20.0], band) else {
        panic!("perto do contorno, por dentro: ponto fixo");
    };
    assert!((u - 1.0).abs() < 1e-9 && (v - 0.2).abs() < 1e-9, "{u} {v}");
    assert_eq!(anchor_for(el, [260.0, 50.0], band), None);
}

/// Um meio de lado que NÃO está no contorno (o lado esquerdo de um triângulo) não cola.
#[test]
fn a_side_middle_off_the_outline_does_not_snap() {
    let mut doc = BoardDoc::default();
    let id = shape(&mut doc, ShapeType::Triangle, [0.0, 0.0, 200.0, 100.0]);
    let el = doc.get(id).unwrap();
    let got = anchor_for(el, [48.0, 52.0], 10.0);
    assert!(
        matches!(got, Some(Anchor::Fixed(uv)) if uv != [0.0, 0.5]),
        "o (0, ½) do triângulo é vazio: {got:?}"
    );
}

/// O ponto fixo roda com a forma.
#[test]
fn a_fixed_point_turns_with_its_shape() {
    let mut doc = BoardDoc::default();
    let a = shape(&mut doc, ShapeType::Rectangle, [0.0, 0.0, 200.0, 100.0]);
    let mut el = doc.get(a).unwrap().clone();
    el.angle = std::f64::consts::FRAC_PI_2;
    BoardOp::Put(el).apply(&mut doc);
    let el = doc.get(a).unwrap();
    let p = fixed_world(el, [1.0, 0.5]);
    assert!(
        (p[0] - 100.0).abs() < 1e-9 && (p[1] - 150.0).abs() < 1e-9,
        "{p:?}"
    );
}

/// Apagar a forma deixa a ponta ONDE ESTAVA (a seta não voa para a origem).
#[test]
fn an_end_whose_shape_is_deleted_stays_where_it_was() {
    let mut doc = BoardDoc::default();
    let a = shape(&mut doc, ShapeType::Rectangle, [0.0, 0.0, 160.0, 100.0]);
    let b = shape(&mut doc, ShapeType::Rectangle, [400.0, 0.0, 160.0, 100.0]);
    let id = link(&mut doc, center(a), center(b), Route::Elbow);
    let mut cache = RouteCache::default();
    cache.sync(&doc);
    let before = cache.get(id).unwrap().ends()[1];
    BoardOp::Delete(b).apply(&mut doc);
    cache.sync(&doc);
    assert_eq!(cache.get(id).unwrap().ends()[1], before);
}

/// Duas setas no mesmo par de formas afastam-se (a segunda não nasce por baixo da primeira).
#[test]
fn parallel_arrows_do_not_overlap() {
    let mut doc = BoardDoc::default();
    let a = shape(&mut doc, ShapeType::Rectangle, [0.0, 0.0, 160.0, 100.0]);
    let b = shape(&mut doc, ShapeType::Rectangle, [400.0, 0.0, 160.0, 100.0]);
    let one = link(&mut doc, center(a), center(b), Route::Elbow);
    let two = link(&mut doc, center(a), center(b), Route::Elbow);
    let mut cache = RouteCache::default();
    cache.sync(&doc);
    let (p, q) = (
        cache.get(one).unwrap().ends()[0],
        cache.get(two).unwrap().ends()[0],
    );
    assert!((p[1] - q[1]).abs() >= SPREAD_STEP - 1e-9, "{p:?} {q:?}");
}

/// A curva é a rota do cotovelo SUAVIZADA: as mesmas pontas, sem quinas.
#[test]
fn the_curve_is_the_elbow_smoothed() {
    let mut doc = BoardDoc::default();
    let a = shape(&mut doc, ShapeType::Rectangle, [0.0, 0.0, 160.0, 100.0]);
    let b = shape(&mut doc, ShapeType::Rectangle, [400.0, 300.0, 160.0, 100.0]);
    let elbow = link(&mut doc, center(a), center(b), Route::Elbow);
    let curve = link(&mut doc, center(a), center(b), Route::Curved);
    let mut cache = RouteCache::default();
    cache.sync(&doc);
    let (e, c) = (cache.get(elbow).unwrap(), cache.get(curve).unwrap());
    assert!(e.pts.len() > 2);
    assert!(
        c.path.verts.iter().any(|v| v.out_handle != v.anchor),
        "a curva tem braços"
    );
}

/// A ponta de seta tem o bico NA ponta da rota; a cheia recua a linha o que tapa.
#[test]
fn the_head_tip_sits_on_the_end_and_a_filled_head_trims_the_line() {
    use ph2d_vector::Shape as _;
    let mut doc = BoardDoc::default();
    let p = End::Free([0.0, 0.0]);
    let q = End::Free([200.0, 0.0]);
    let id = link(&mut doc, p, q, Route::Straight);
    let mut cache = RouteCache::default();
    cache.sync(&doc);
    let r = cache.get(id).unwrap();
    let d = drawn(r, [Head::None, Head::Triangle], 2.0);
    let line = d.line.bounding_box();
    let head = d.heads[0].0.bounding_box();
    assert!((head.x1 - 200.0).abs() < 1e-6, "o bico na ponta: {head:?}");
    assert!(
        line.x1 < 200.0 - 1.0,
        "a linha recua sob o triângulo: {line:?}"
    );
    assert!(d.heads[0].1, "o triângulo é cheio");
    let open = drawn(r, [Head::None, Head::Arrow], 2.0);
    assert!(!open.heads[0].1, "o «V» traça-se");
}
