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

/// ⭐ A cache: o mesmo documento não volta ao roteador; mexer numa forma que não é ponta de nenhuma
/// seta também não; mexer numa das pontas refá-la a ela só.
#[test]
fn only_the_arrow_whose_shape_moved_is_rerouted() {
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
    // Na aresta esquerda, a 36 do (0, ½) — dentro de uma faixa de 40, que o deixaria colar.
    let got = anchor_for(el, [20.0, 80.0], 40.0);
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

/// Uma seta presa a PONTOS FIXOS no mesmo par não empurra a vizinha presa ao centro (foto da cena 3,
/// 06/10: o «não» por baixo tirava a seta da frente do vértice do losango).
#[test]
fn a_fixed_point_arrow_does_not_spread_its_centered_neighbour() {
    let mut doc = BoardDoc::default();
    let a = shape(&mut doc, ShapeType::Rectangle, [0.0, 0.0, 160.0, 100.0]);
    let b = shape(&mut doc, ShapeType::Diamond, [400.0, -20.0, 160.0, 140.0]);
    let front = link(&mut doc, center(a), center(b), Route::Elbow);
    let fixed = |t, uv| End::Bound {
        target: t,
        anchor: Anchor::Fixed(uv),
    };
    link(
        &mut doc,
        fixed(b, [0.5, 1.0]),
        fixed(a, [0.5, 1.0]),
        Route::Elbow,
    );
    let mut cache = RouteCache::default();
    cache.sync(&doc);
    assert_eq!(
        cache.get(front).unwrap().ends()[1],
        [400.0, 50.0],
        "no vértice esquerdo"
    );
}

/// ⭐ Uma forma largada no CAMINHO de uma seta não a mexe (ordem do dono, 06/10: «setas não se
/// reajustam sozinhas» — o idioma do Miro).
#[test]
fn a_shape_dropped_in_the_way_does_not_move_the_arrow() {
    let mut doc = BoardDoc::default();
    let a = shape(&mut doc, ShapeType::Rectangle, [0.0, 0.0, 160.0, 100.0]);
    let b = shape(&mut doc, ShapeType::Rectangle, [600.0, 0.0, 160.0, 100.0]);
    let wall = shape(&mut doc, ShapeType::Rectangle, [3000.0, 0.0, 100.0, 300.0]);
    let id = link(&mut doc, center(a), center(b), Route::Elbow);
    let mut cache = RouteCache::default();
    cache.sync(&doc);
    let before = cache.get(id).unwrap().clone();
    moved(&mut doc, wall, [-2670.0, -100.0]);
    cache.sync(&doc);
    assert_eq!(cache.rerouted(), 0, "a parede não é ponta da seta");
    assert_eq!(*cache.get(id).unwrap(), before);
}

/// ⭐ **A curva do Miro**: sem pontos de ajuste é UMA cúbica que sai e entra perpendicular às faces,
/// com o braço de 0,45 da distância entre as pontas (medido na captura do dono: 154 para 342,6).
#[test]
fn the_curve_is_one_cubic_leaving_and_entering_perpendicular_with_half_the_gap() {
    let mut doc = BoardDoc::default();
    let right = shape(&mut doc, ShapeType::Rectangle, [714.0, 106.0, 442.0, 188.0]);
    let left = shape(&mut doc, ShapeType::Rectangle, [67.0, 250.0, 341.0, 206.0]);
    let id = link(&mut doc, center(right), center(left), Route::Curved);
    let mut cache = RouteCache::default();
    cache.sync(&doc);
    let r = cache.get(id).unwrap();
    let v = &r.path.verts;
    assert_eq!(v.len(), 2, "uma cúbica só");
    let ([x0, y0], [x1, y1]) = (v[0].anchor, v[1].anchor);
    assert_eq!(
        ([x0, y0], [x1, y1]),
        ([714.0, 200.0], [408.0, 353.0]),
        "os meios das faces"
    );
    // O braço MEDIDO na captura do Miro: 154 (não a constante — senão o teste confirma-se a si mesmo).
    let k = v[0].anchor[0] - v[0].out_handle[0];
    assert!((k - 154.0).abs() < 1.0, "braço {k}, medido 154");
    assert_eq!(
        v[0].out_handle,
        [x0 - k, y0],
        "sai na horizontal, braço = ½ do afastamento"
    );
    assert_eq!(v[1].in_handle, [x1 + k, y1], "entra na horizontal");
}

/// Duas pontas que saem para o MESMO lado (o «não» por baixo): o piso do braço dá a volta.
#[test]
fn a_curve_whose_ends_leave_the_same_way_still_loops_out() {
    let mut doc = BoardDoc::default();
    let a = shape(&mut doc, ShapeType::Rectangle, [0.0, 0.0, 160.0, 100.0]);
    let b = shape(&mut doc, ShapeType::Rectangle, [400.0, 0.0, 160.0, 100.0]);
    let fixed = |t, uv| End::Bound {
        target: t,
        anchor: Anchor::Fixed(uv),
    };
    let id = link(
        &mut doc,
        fixed(b, [0.5, 1.0]),
        fixed(a, [0.5, 1.0]),
        Route::Curved,
    );
    let mut cache = RouteCache::default();
    cache.sync(&doc);
    let r = cache.get(id).unwrap();
    assert!(
        r.bbox[3] > 100.0 + 0.5 * POINT_ARM * 400.0,
        "a volta desce abaixo das caixas: {:?}",
        r.bbox
    );
}

/// ⭐ Os PONTOS DE AJUSTE: a seta passa por eles (curva, recta e cotovelo), e cada trecho tem a sua
/// pega do meio (onde se arrasta para criar um ponto novo).
#[test]
fn the_arrow_passes_through_its_waypoints_and_each_leg_has_a_middle_handle() {
    for route in [Route::Curved, Route::Straight, Route::Elbow] {
        let mut doc = BoardDoc::default();
        let a = shape(&mut doc, ShapeType::Rectangle, [0.0, 0.0, 160.0, 100.0]);
        let b = shape(&mut doc, ShapeType::Rectangle, [600.0, 0.0, 160.0, 100.0]);
        let mut c = Connector::new(
            center(a),
            center(b),
            route,
            Style::new(None, Some(ink()), ink()),
        );
        c.waypoints = vec![[300.0, 300.0], [450.0, -200.0]];
        let el = Element::new_connector(doc.mint_id(), doc.z_on_top(), c);
        let id = el.id;
        BoardOp::Put(el).apply(&mut doc);
        let mut cache = RouteCache::default();
        cache.sync(&doc);
        let r = cache.get(id).unwrap();
        let line = r.polyline();
        for w in [[300.0, 300.0], [450.0, -200.0]] {
            assert!(
                line.contains(&w),
                "{route:?}: a rota não passa em {w:?}: {line:?}"
            );
        }
        assert_eq!(r.waypoints(), &[[300.0, 300.0], [450.0, -200.0]]);
        assert_eq!(r.leg_mids.len(), 3, "{route:?}: um meio por trecho");
        // A 1.ª ponta sai rumo ao 1.º ponto (para baixo), não rumo à outra caixa.
        assert_eq!(r.sides[0], Dir::North, "{route:?}");
    }
}

/// A cache REVÊ só a seta tocada (não todas): mexer numa forma que não é ponta de ninguém revê zero;
/// mudar só os PONTOS de uma seta refá-la.
#[test]
fn the_cache_revisits_only_the_touched_arrow_and_sees_waypoint_changes() {
    let mut doc = BoardDoc::default();
    let a = shape(&mut doc, ShapeType::Rectangle, [0.0, 0.0, 160.0, 100.0]);
    let b = shape(&mut doc, ShapeType::Rectangle, [400.0, 0.0, 160.0, 100.0]);
    let c = shape(&mut doc, ShapeType::Rectangle, [0.0, 400.0, 160.0, 100.0]);
    let lone = shape(&mut doc, ShapeType::Rectangle, [3000.0, 0.0, 160.0, 100.0]);
    let ab = link(&mut doc, center(a), center(b), Route::Curved);
    link(&mut doc, center(a), center(c), Route::Curved);
    let mut cache = RouteCache::default();
    cache.sync(&doc);
    moved(&mut doc, lone, [5.0, 0.0]);
    cache.sync(&doc);
    assert_eq!(cache.revisited(), 0, "ninguém está preso à forma solta");
    let mut el = doc.get(ab).unwrap().clone();
    el.connector_mut().unwrap().waypoints = vec![[280.0, 300.0]];
    BoardOp::Put(el).apply(&mut doc);
    cache.sync(&doc);
    assert_eq!((cache.revisited(), cache.rerouted()), (1, 1));
    assert!(cache.get(ab).unwrap().polyline().contains(&[280.0, 300.0]));
}

/// ⭐ A linha acaba no CENTRO DA BASE da ponta de seta e é a MESMA curva da guia (3.º smoke do dono,
/// 06/10: com o triângulo, a guia da selecção e o traço separavam-se).
#[test]
fn on_a_curve_the_line_is_the_route_cut_at_the_middle_of_the_head_base() {
    use ph2d_vector::{ParamCurve, ParamCurveNearest, Point};
    let mut doc = BoardDoc::default();
    let a = shape(&mut doc, ShapeType::Rectangle, [0.0, 0.0, 300.0, 80.0]);
    let b = shape(&mut doc, ShapeType::Pill, [500.0, 0.0, 160.0, 80.0]);
    let bottom = |t| End::Bound {
        target: t,
        anchor: Anchor::Fixed([0.5, 1.0]),
    };
    let id = link(&mut doc, bottom(b), bottom(a), Route::Curved);
    let mut cache = RouteCache::default();
    cache.sync(&doc);
    let r = cache.get(id).unwrap();
    let w = 2.0;
    let inset = marker(Head::Triangle).inset(HEAD_SCALE) * w;
    let d = drawn(r, [Head::None, Head::Triangle], w);
    let full = ph2d_vec_render::build_bezpath(&r.path);
    let on_route = |p: Point| {
        full.segments()
            .map(|s| s.nearest(p, 1e-9).distance_sq)
            .fold(f64::INFINITY, f64::min)
            .sqrt()
    };
    // Cada ponto do traço está NA rota (a guia).
    for seg in d.line.segments() {
        for k in 0..=20 {
            let p = seg.eval(f64::from(k) / 20.0);
            assert!(on_route(p) < 1e-3, "o traço saiu da guia em {p:?}");
        }
    }
    // E acaba a `inset` da ponta, no eixo da cabeça: o centro da base.
    let end = d.line.segments().last().unwrap().end();
    let tip = Point::new(150.0, 80.0);
    assert!(((end - tip).hypot() - inset).abs() < 1e-6, "{end:?}");
    // O meio dos dois cantos de TRÁS do triângulo (os mais longe do bico) é onde o traço acaba.
    let mut corners: Vec<Point> = d.heads[0]
        .0
        .elements()
        .iter()
        .filter_map(|e| match e {
            ph2d_vector::PathEl::MoveTo(p)
            | ph2d_vector::PathEl::LineTo(p)
            | ph2d_vector::PathEl::CurveTo(_, _, p) => Some(*p),
            _ => None,
        })
        .collect();
    corners.sort_by(|a, b| (*b - tip).hypot().total_cmp(&(*a - tip).hypot()));
    let base = corners[0].midpoint(corners[1]);
    assert!(
        (base - end).hypot() < 1e-6,
        "base {base:?}, o traço acaba em {end:?}"
    );
}

/// Num ponto de ajuste o braço é 0,4 de cada trecho vizinho (MEDIDO na captura do Miro —
/// `POINT_ARM`), na tangente de trás para a frente.
#[test]
fn a_waypoint_arm_is_the_measured_fraction_of_each_leg() {
    let mut doc = BoardDoc::default();
    let mut c = Connector::new(
        End::Free([0.0, 0.0]),
        End::Free([400.0, 0.0]),
        Route::Curved,
        Style::new(None, Some(ink()), ink()),
    );
    c.waypoints = vec![[200.0, 150.0]];
    let el = Element::new_connector(doc.mint_id(), doc.z_on_top(), c);
    let id = el.id;
    BoardOp::Put(el).apply(&mut doc);
    let mut cache = RouteCache::default();
    cache.sync(&doc);
    let v = &cache.get(id).unwrap().path.verts[1];
    assert_eq!(v.anchor, [200.0, 150.0]);
    // Tangente horizontal (de (0,0) a (400,0)); cada trecho mede 250.
    assert!(
        (v.out_handle[0] - (200.0 + 100.0)).abs() < 1e-9 && (v.out_handle[1] - 150.0).abs() < 1e-9
    );
    assert!(
        (v.in_handle[0] - (200.0 - 100.0)).abs() < 1e-9 && (v.in_handle[1] - 150.0).abs() < 1e-9
    );
}
