use super::{BoardSet, Camera, FORMAT_VERSION};
use crate::{BoardOp, Element, ElementKind, Rgba};

#[test]
fn tabs_keep_their_order_through_create_duplicate_move_and_remove() {
    let mut set = BoardSet::default();
    let a = set.create("A".into());
    let b = set.create("B".into());
    let a2 = set.duplicate(a, "A cópia".into()).unwrap();
    let names = |s: &BoardSet| s.boards().iter().map(|b| b.name.clone()).collect::<Vec<_>>();
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
    board.camera = Camera { center_x: 120.5, center_y: -40.0, zoom: 2.5 };
    let id = board.doc.mint_id();
    let z = board.doc.z_on_top();
    let el = Element {
        id,
        kind: ElementKind::Rect { fill: Rgba([1, 2, 3, 4]) },
        x: 1.0,
        y: 2.0,
        w: 3.0,
        h: 4.0,
        z,
        version: 0,
        nonce: 7,
        deleted: false,
    };
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
