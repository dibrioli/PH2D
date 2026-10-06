use super::{BoardSet, Camera, FORMAT_VERSION};
use crate::{BoardOp, Element, Rgba, Shape, ShapeType, Style};

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
    assert_eq!(back, set, "relido e regravado no formato 2, fica igual");
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
        text: String::new(),
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
    assert_eq!(FORMAT_VERSION, 2);
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
        text: String::new(),
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
