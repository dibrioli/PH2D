use super::{BoardSet, Camera, FORMAT_VERSION};
use crate::{BoardOp, Element, Rgba, RichText, Shape, ShapeType, Style};

/// ⭐ Um ficheiro do formato 2 (W1–W2: o texto era uma `String`) continua a abrir — os bytes abaixo
/// foram gravados por esse build (`d8a331ec3`): um quadro «Ideias» com uma pílula rodada, cantos
/// redondos, com «olá» dentro, e uma seta em cotovelo presa a ela, com rótulo e um ponto de ajuste.
#[test]
fn a_format_2_file_still_opens_with_its_text_and_arrow() {
    let set = BoardSet::from_bytes(V2_BYTES).expect("o formato 2 lê-se");
    let b = &set.boards()[0];
    assert_eq!(b.name, "Ideias");
    let els = b.doc.live_in_z_order();
    assert_eq!(els.len(), 2);
    let s = els[0].shape().expect("a 1.ª é a forma");
    assert_eq!(s.kind, ShapeType::Pill);
    assert_eq!(s.text.as_str(), "olá");
    assert!(s.text.spans().is_empty());
    assert!(s.style.round);
    assert_eq!(els[0].angle, 0.5);
    let c = els[1].connector().expect("a 2.ª é a seta");
    assert_eq!(c.label, "sim");
    assert_eq!(c.waypoints, vec![[5.0, 6.0]]);
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(
        back, set,
        "relido e regravado no formato actual, fica igual"
    );
}

/// Gravados pelo build do formato 2 (as fontes de `d8a331ec3` compiladas à parte, `BoardSet::to_bytes`
/// escrito para o ficheiro) — nunca copiados à mão (uma cópia à mão de 251 números saiu com 253).
const V2_BYTES: &[u8] = include_bytes!("../fixtures/format_v2.bin");

/// O texto com trechos (W3) e uma nota grava-se e lê-se.
#[test]
fn rich_text_and_a_sticky_round_trip_through_the_bytes() {
    let mut set = BoardSet::default();
    let a = set.create("Notas".into());
    let doc = &mut set.get_mut(a).unwrap().doc;
    let ink = Rgba([1, 2, 3, 255]);
    let mut text = RichText::plain("uma boa ideia");
    text.toggle(4..7, crate::Mark::Bold);
    text.restyle(8..13, |m| m.color = Some(Rgba([200, 0, 0, 255])));
    let shape = Shape {
        kind: ShapeType::Sticky,
        style: Style::new(Some(Rgba(crate::STICKY_COLORS[2])), None, ink),
        text,
    };
    let el = Element::new_shape(
        doc.mint_id(),
        doc.z_on_top(),
        shape,
        [0.0, 0.0, 199.0, 199.0],
    );
    BoardOp::Put(el).apply(doc);
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(back, set);
}

#[test]
fn tabs_keep_their_order_through_create_duplicate_move_and_remove() {
    let mut set = BoardSet::default();
    let a = set.create("A".into());
    let b = set.create("B".into());
    let a2 = set.duplicate(a, "A cópia".into()).unwrap();
    let names = |s: &BoardSet| {
        s.boards()
            .iter()
            .map(|b| b.name.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(names(&set), ["A", "A cópia", "B"]);
    assert!(set.move_tab(b, 0));
    assert_eq!(names(&set), ["B", "A", "A cópia"]);
    assert!(set.move_tab(b, 99), "posição além do fim vai para o fim");
    assert_eq!(names(&set), ["A", "A cópia", "B"]);
    assert_eq!(set.remove(a2).map(|b| b.name), Some("A cópia".into()));
    assert!(set.remove(a2).is_none());
    assert!(!set.rename(a2, "x".into()));
}

#[test]
fn ids_are_never_reused_after_a_remove() {
    let mut set = BoardSet::default();
    let a = set.create("A".into());
    set.remove(a);
    let b = set.create("B".into());
    assert_ne!(a, b);
}

#[test]
fn bytes_round_trip_with_content_camera_and_order() {
    let mut set = BoardSet::default();
    let a = set.create("Retro".into());
    set.create("Ideias".into());
    let board = set.get_mut(a).unwrap();
    board.camera = Camera {
        center_x: 120.5,
        center_y: -40.0,
        zoom: 2.5,
    };
    let id = board.doc.mint_id();
    let z = board.doc.z_on_top();
    let shape = Shape {
        kind: ShapeType::Diamond,
        style: Style::new(
            Some(Rgba([1, 2, 3, 4])),
            Some(Rgba([5, 6, 7, 8])),
            Rgba([9, 9, 9, 9]),
        ),
        text: "linha 1\nlinha 2".into(),
    };
    let mut el = Element::new_shape(id, z, shape, [1.0, 2.0, 3.0, 4.0]);
    el.nonce = 7;
    el.angle = 0.25;
    BoardOp::Put(el).apply(&mut board.doc);
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(back, set);
}

#[test]
fn empty_bytes_are_a_project_without_boards() {
    assert!(BoardSet::from_bytes(&[]).unwrap().is_empty());
}

#[test]
fn another_format_version_is_refused_not_misread() {
    let mut set = BoardSet::default();
    set.create("A".into());
    set.version = FORMAT_VERSION + 1;
    let err = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap_err();
    assert!(err.contains("format version"), "{err}");
}

#[test]
fn garbage_is_an_error_not_a_panic() {
    assert!(BoardSet::from_bytes(&[0xFF, 0xFF, 0xFF]).is_err());
}

/// ⭐ Um ficheiro do formato 1 (W0: rectângulos cheios) continua a abrir — os bytes abaixo foram
/// gravados por esse build: um quadro «Retro» com um rectângulo `(1,2,3,4)` de cor `[1,2,3,4]`.
#[test]
fn a_format_1_file_still_opens_as_rectangles() {
    let set = BoardSet::from_bytes(V1_BYTES).expect("o formato 1 lê-se");
    let b = &set.boards()[0];
    assert_eq!(b.name, "Retro");
    let els = b.doc.live_in_z_order();
    assert_eq!(els.len(), 1);
    let el = els[0];
    assert_eq!(
        [el.x, el.y, el.w, el.h, el.angle],
        [1.0, 2.0, 3.0, 4.0, 0.0]
    );
    let s = el.shape().unwrap();
    assert_eq!(s.kind, ShapeType::Rectangle);
    assert_eq!(s.style.fill, Some(Rgba([1, 2, 3, 4])));
    assert!(s.text.is_empty());
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(
        back, set,
        "relido e regravado no formato actual, fica igual"
    );
}

/// Gravados pelo build do formato 1 (`e74c81f32`, `BoardSet::to_bytes`), não reescritos à mão.
const V1_BYTES: &[u8] = &[
    1, 1, 1, 5, 82, 101, 116, 114, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 240, 63, 1, 1, 1, 0, 1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 0, 64, 0,
    0, 0, 0, 0, 0, 8, 64, 0, 0, 0, 0, 0, 0, 16, 64, 2, 97, 48, 1, 0, 0, 1, 1, 2, 97, 48, 1,
];

/// Uma seta (W2) grava-se e lê-se: pontas presas e soltas, rota, pontas de seta, rótulo. A variante
/// entrou no FIM do `ElementKind` — o formato 2 não sobe, e um ficheiro só de formas lê-se igual.
#[test]
fn an_arrow_round_trips_through_the_bytes() {
    use crate::{Anchor, Connector, End, Head, Route};
    let mut set = BoardSet::default();
    let a = set.create("Fluxo".into());
    let doc = &mut set.get_mut(a).unwrap().doc;
    let ink = Rgba([1, 2, 3, 255]);
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(None, Some(ink), ink),
        text: RichText::default(),
    };
    let box_el = Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, [0.0, 0.0, 10.0, 10.0]);
    let target = box_el.id;
    BoardOp::Put(box_el).apply(doc);
    let mut c = Connector::new(
        End::Bound {
            target,
            anchor: Anchor::Fixed([1.0, 0.5]),
        },
        End::Free([40.0, -3.5]),
        Route::Curved,
        Style::new(None, Some(ink), ink),
    );
    c.heads = [Head::Circle, Head::Triangle];
    c.label = "sim".into();
    let arrow = Element::new_connector(doc.mint_id(), doc.z_on_top(), c);
    BoardOp::Put(arrow).apply(doc);
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(back, set);
    assert_eq!(FORMAT_VERSION, 4);
}

/// A revisão da sessão muda a cada operação aplicada e nunca se repete entre documentos (a cache de
/// rotas confia nisso para não confundir dois quadros).
#[test]
fn the_session_revision_moves_with_each_op_and_is_unique() {
    let mut set = BoardSet::default();
    let (a, b) = (set.create("A".into()), set.create("B".into()));
    let (ra, rb) = (set.get(a).unwrap().doc.rev(), set.get(b).unwrap().doc.rev());
    assert_ne!(ra, rb);
    let doc = &mut set.get_mut(a).unwrap().doc;
    let ink = Rgba([0, 0, 0, 255]);
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(None, Some(ink), ink),
        text: RichText::default(),
    };
    let el = Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, [0.0, 0.0, 1.0, 1.0]);
    let id = el.id;
    BoardOp::Put(el).apply(doc);
    assert_ne!(doc.rev(), ra);
    let r = doc.rev();
    assert!(BoardOp::Delete(id).apply(doc).is_some());
    assert_ne!(doc.rev(), r);
    let r = doc.rev();
    assert!(
        BoardOp::Delete(id).apply(doc).is_none(),
        "já apagado: nada mudou"
    );
    assert_eq!(doc.rev(), r, "uma op que não muda nada não mexe na revisão");
}

/// ⭐ Um ficheiro do formato 3 (W3) continua a abrir — gravado pelo build de `ea38b5e61` (o último
/// do formato 3): um rectângulo rodado com cantos redondos e texto com trechos, uma nota laranja e
/// uma seta curva presa ao rectângulo, com rótulo e um ponto de ajuste. Tudo nasce FINAL.
#[test]
fn a_format_3_file_still_opens_and_everything_in_it_is_final() {
    let set = BoardSet::from_bytes(V3_BYTES).expect("o formato 3 lê-se");
    let b = &set.boards()[0];
    assert_eq!(b.name, "Esboço");
    assert_eq!(b.camera.zoom, 1.5);
    assert!(!b.sketch);
    let els = b.doc.live_in_z_order();
    assert_eq!(els.len(), 3);
    let r = els[0].shape().expect("forma");
    assert_eq!(r.kind, ShapeType::Rectangle);
    assert_eq!(r.text.as_str(), "fazer o quê");
    assert_eq!(r.text.spans().len(), 2, "os trechos sobrevivem");
    assert!(r.style.round);
    assert_eq!(els[0].angle, 0.25);
    assert_eq!(els[1].shape().expect("nota").kind, ShapeType::Sticky);
    let c = els[2].connector().expect("seta");
    assert_eq!(c.label, "sim");
    assert_eq!(c.waypoints, vec![[300.0, 250.0]]);
    assert!(els.iter().all(|e| !e.style().sketch));
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(
        back, set,
        "relido e regravado no formato actual, fica igual"
    );
}

/// Gravados pelo build do formato 3 (`ea38b5e61`, um teste temporário que escreveu `to_bytes`).
const V3_BYTES: &[u8] = include_bytes!("../fixtures/format_v3.bin");

/// O rascunho (do quadro e de um elemento) e um traço da caneta gravam-se e lêem-se.
#[test]
fn sketch_and_a_pen_stroke_round_trip_through_the_bytes() {
    let mut set = BoardSet::default();
    let a = set.create("Caneta".into());
    let board = set.get_mut(a).unwrap();
    board.sketch = true;
    let doc = &mut board.doc;
    let ink = Rgba([1, 2, 3, 255]);
    let mut style = Style::new(None, Some(ink), ink);
    style.sketch = true;
    let shape = Shape {
        kind: ShapeType::Ellipse,
        style: style.clone(),
        text: RichText::default(),
    };
    let id = doc.mint_id();
    BoardOp::Put(Element::new_shape(
        id,
        doc.z_on_top(),
        shape,
        [0.0, 0.0, 50.0, 30.0],
    ))
    .apply(doc);
    let (stroke, bx) = crate::Ink::from_world(
        &[[10.0, 10.0, 0.5], [30.0, 12.0, 0.7], [50.0, 40.0, 0.4]],
        Style::new(None, Some(ink), ink),
        crate::Pen::Highlighter,
        true,
    );
    let id = doc.mint_id();
    BoardOp::Put(Element::new_ink(id, doc.z_on_top(), stroke, bx)).apply(doc);
    let back = BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(back, set);
}
