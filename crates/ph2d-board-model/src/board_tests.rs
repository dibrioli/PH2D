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
