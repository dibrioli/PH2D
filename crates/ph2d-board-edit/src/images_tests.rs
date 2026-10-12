//! W14 — pôr imagens no quadro, redimensioná-las e levá-las numa cópia.

use super::*;
use crate::{Command, Metrics, Mods, Pointer};
use ph2d_board_model::{ImageFile, Rgba};
use ph2d_text::TextSystem;

const VIEW: [f64; 4] = [-10_000.0, -10_000.0, 10_000.0, 10_000.0];
const NONE: Mods = Mods {
    shift: false,
    ctrl: false,
    alt: false,
};

pub(crate) fn editor() -> Editor {
    let ink = Rgba([20, 20, 20, 255]);
    Editor::new(
        Style::new(None, Some(ink), ink),
        Metrics {
            handle: 8.0,
            rotate_offset: 24.0,
            snap: 8.0,
            drag: 3.0,
            click_size: [160.0, 100.0],
            paste_offset: 10.0,
            bind: 8.0,
            dot: 40.0,
            eraser: 10.0,
            frame_title: 20.0,
        },
    )
}

fn at(p: [f64; 2]) -> Pointer {
    Pointer {
        world: p,
        px: 1.0,
        mods: NONE,
    }
}

fn drag(ed: &mut Editor, doc: &mut BoardDoc, h: &mut History, from: [f64; 2], to: [f64; 2]) {
    let mut ts = TextSystem::without_system_fonts();
    ed.pointer_down(doc, h, &mut ts, at(from), VIEW);
    let mid = [(from[0] + to[0]) / 2.0, (from[1] + to[1]) / 2.0];
    ed.pointer_move(doc, &mut ts, at(mid));
    ed.pointer_move(doc, &mut ts, at(to));
    ed.pointer_up(doc, h, at(to));
}

fn key(n: u8) -> FileId {
    FileId([n; 32])
}

/// O tamanho de nascença: o natural, reduzido (nunca ampliado) até caber.
#[test]
fn an_image_is_born_natural_and_only_shrinks_to_fit() {
    assert_eq!(birth_size([4000, 3000], [500.0, 500.0]), [500.0, 375.0]);
    assert_eq!(birth_size([40, 30], [500.0, 500.0]), [40.0, 30.0]);
    assert_eq!(birth_size([100, 1000], [500.0, 400.0]), [40.0, 400.0]);
}

/// ⭐ Três imagens: uma grelha 2 × 2 (a 4.ª casa vazia) centrada no ponto, pela ordem em que vieram,
/// seleccionadas — e UM desfazer tira as três.
#[test]
fn several_images_land_in_a_grid_around_the_point_as_one_step() {
    let (mut ed, mut doc, mut h) = (editor(), BoardDoc::default(), History::default());
    let items = [
        (key(1), [100, 100]),
        (key(2), [100, 50]),
        (key(3), [100, 100]),
    ];
    let made = ed.insert_images(&mut doc, &mut h, &items, [0.0, 0.0], [1e6, 1e6]);
    assert_eq!(made.len(), 3);
    let gap = 100.0 * GAP_FRAC;
    let boxes: Vec<[f64; 4]> = made
        .iter()
        .map(|id| {
            let e = doc.get(*id).unwrap();
            [e.x, e.y, e.w, e.h]
        })
        .collect();
    let half = (200.0 + gap) / 2.0;
    assert_eq!(boxes[0], [-half, -half, 100.0, 100.0]);
    // A 2.ª na coluna da direita, centrada na linha de cima (100 de alto).
    assert_eq!(boxes[1], [-half + 100.0 + gap, -half + 25.0, 100.0, 50.0]);
    assert_eq!(boxes[2], [-half, -half + 100.0 + gap, 100.0, 100.0]);
    assert_eq!(doc.get(made[1]).unwrap().image().unwrap().file, key(2));
    assert_eq!(ed.selection().len(), 3);
    let mut ts = TextSystem::without_system_fonts();
    ed.command(&mut doc, &mut h, &mut ts, Command::Undo);
    assert_eq!(doc.live_len(), 0, "um passo tira as três");
}

/// ⭐ Redimensionar uma imagem pelo canto mantém a proporção (como uma nota); clicar dentro dela
/// selecciona-a (a geometria trata-a como a caixa dela).
#[test]
fn a_corner_drag_keeps_the_image_proportion_and_a_click_inside_selects_it() {
    let (mut ed, mut doc, mut h) = (editor(), BoardDoc::default(), History::default());
    let id = ed.insert_images(
        &mut doc,
        &mut h,
        &[(key(1), [200, 100])],
        [0.0, 0.0],
        [1e6, 1e6],
    )[0];
    drag(&mut ed, &mut doc, &mut h, [100.0, 50.0], [300.0, 60.0]);
    let e = doc.get(id).unwrap();
    assert!((e.w / e.h - 2.0).abs() < 1e-9, "{} × {}", e.w, e.h);
    assert!(e.w > 200.0, "cresceu: {}", e.w);
    drag(
        &mut ed,
        &mut doc,
        &mut h,
        [5000.0, 5000.0],
        [5000.0, 5000.0],
    );
    assert!(ed.selection().is_empty());
    let c = doc.get(id).unwrap().center();
    drag(&mut ed, &mut doc, &mut h, c, c);
    assert!(ed.selection().contains(&id));
}

/// ⭐ Copiar num quadro e colar noutro leva os bytes das imagens; os que o destino já tem ficam.
#[test]
fn a_copy_carries_its_image_bytes_to_another_board() {
    let (mut ed, mut doc, mut h) = (editor(), BoardDoc::default(), History::default());
    let f = ImageFile::new(vec![1, 2, 3], [3, 1]);
    let id = f.id();
    let mut files = Files::new();
    files.insert(id, f.clone());
    files.insert(key(9), ImageFile::new(vec![9], [1, 1]));
    ed.insert_images(&mut doc, &mut h, &[(id, [3, 1])], [0.0, 0.0], [1e6, 1e6]);
    let mut ts = TextSystem::without_system_fonts();
    ed.command(&mut doc, &mut h, &mut ts, Command::Copy);
    ed.keep_clip_files(&files);
    let mut other = Files::new();
    ed.give_clip_files(&mut other);
    assert_eq!(
        other.keys().copied().collect::<Vec<_>>(),
        [id],
        "só os da cópia"
    );
    assert_eq!(other[&id], f);
}

/// ⭐ Uma seta desenhada até DENTRO de uma imagem prende-se a ela (a geometria trata-a como a caixa
/// dela — o Excalidraw prende setas a imagens).
#[test]
fn an_arrow_dropped_on_an_image_binds_to_it() {
    let (mut ed, mut doc, mut h) = (editor(), BoardDoc::default(), History::default());
    let id = ed.insert_images(
        &mut doc,
        &mut h,
        &[(key(1), [200, 100])],
        [0.0, 0.0],
        [1e6, 1e6],
    )[0];
    ed.tool = crate::Tool::Connector;
    drag(&mut ed, &mut doc, &mut h, [-400.0, 0.0], [10.0, 5.0]);
    let arrow = doc
        .live()
        .find_map(|e| e.connector().cloned())
        .expect("a seta nasceu");
    assert!(
        matches!(arrow.end, ph2d_board_model::End::Bound { target, .. } if target == id),
        "{:?}",
        arrow.end
    );
}

/// Auditoria W14 — a forma seguinte (`Ctrl+seta`) não nasce por cima de uma imagem: salta-a.
#[test]
fn the_next_shape_does_not_land_on_an_image() {
    let (mut ed, mut doc, mut h) = (editor(), BoardDoc::default(), History::default());
    let shape = ph2d_board_model::Shape {
        kind: ph2d_board_model::ShapeType::Rectangle,
        style: Style::new(None, Some(Rgba([0, 0, 0, 255])), Rgba([0, 0, 0, 255])),
        text: ph2d_board_model::RichText::default(),
    };
    let rect = Element::new_shape(
        doc.mint_id(),
        doc.z_on_top(),
        shape,
        [0.0, 0.0, 100.0, 100.0],
    );
    let rid = rect.id;
    let _ = BoardOp::Put(rect).apply(&mut doc);
    // A imagem exactamente onde a seguinte nasceria (à direita, a um passo).
    let img = ed.insert_images(
        &mut doc,
        &mut h,
        &[(key(1), [100, 100])],
        [210.0, 50.0],
        [1e6, 1e6],
    )[0];
    ed.select(&doc, [rid]);
    let mut ts = TextSystem::without_system_fonts();
    ed.command(&mut doc, &mut h, &mut ts, Command::Grow([1.0, 0.0]));
    let b = doc.get(img).unwrap().aabb();
    let made: Vec<_> = doc
        .live()
        .filter(|e| e.shape().is_some() && e.id != rid)
        .collect();
    assert_eq!(made.len(), 1, "a seguinte nasceu");
    let a = made[0].aabb();
    let overlap = a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3];
    assert!(!overlap, "nasceu por cima da imagem: {a:?} × {b:?}");
}

/// ⭐ W16 — espelhar como o Excalidraw: à volta do centro da caixa de TODAS as seleccionadas — cada
/// uma vai para o lado espelhado, o ângulo troca de sinal, a imagem vira; duas vezes = como estava.
#[test]
fn flipping_mirrors_around_the_selection_and_twice_is_identity() {
    let (mut doc, mut h) = (BoardDoc::default(), History::default());
    let mut ed = editor();
    let ids = ed.insert_images(
        &mut doc,
        &mut h,
        &[(FileId([1; 32]), [100, 50]), (FileId([2; 32]), [40, 40])],
        [0.0, 0.0],
        [1e9, 1e9],
    );
    let mut a = doc.get(ids[0]).unwrap().clone();
    a.angle = 0.4;
    BoardOp::Put(a).apply(&mut doc);
    let before: Vec<Element> = ids.iter().map(|i| doc.get(*i).unwrap().clone()).collect();
    let mid = {
        let b0 = before[0].aabb();
        let b1 = before[1].aabb();
        (b0[0].min(b1[0]) + b0[2].max(b1[2])) / 2.0
    };
    assert!(ed.flip_images(&mut doc, &mut h, 0));
    for (id, was) in ids.iter().zip(&before) {
        let el = doc.get(*id).unwrap();
        let (cx, wc) = (el.x + el.w / 2.0, was.x + was.w / 2.0);
        assert!((cx - (2.0 * mid - wc)).abs() < 1e-9, "o centro espelhado");
        assert_eq!((el.y, el.w, el.h), (was.y, was.w, was.h));
        assert_eq!(el.angle, -was.angle);
        assert_eq!(el.image().unwrap().flip, [true, false]);
    }
    assert!(ed.flip_images(&mut doc, &mut h, 0));
    for (id, was) in ids.iter().zip(&before) {
        let el = doc.get(*id).unwrap();
        assert!((el.x - was.x).abs() < 1e-9 && el.angle == was.angle);
        assert_eq!(el.image().unwrap().flip, [false, false]);
    }
    ed.selection.clear();
    assert!(!ed.flip_images(&mut doc, &mut h, 1), "sem imagens, nada");
}
