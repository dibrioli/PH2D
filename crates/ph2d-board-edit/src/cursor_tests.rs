//! W23 B5 — o cursor sobre as pegas da moldura.

use super::*;
use crate::{Metrics, Mods, Style};
use ph2d_board_model::{BoardOp, Element, Rgba, Shape, ShapeType};

fn editor_with_box(angle: f64) -> (Editor, BoardDoc) {
    let ink = Rgba([20, 20, 20, 255]);
    let metrics = Metrics {
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
    };
    let mut ed = Editor::new(Style::new(None, Some(ink), ink), metrics);
    let mut doc = BoardDoc::default();
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(None, Some(ink), ink),
        text: Default::default(),
    };
    let mut el = Element::new_shape(
        doc.mint_id(),
        doc.z_on_top(),
        shape,
        [0.0, 0.0, 200.0, 100.0],
    );
    el.angle = angle;
    let id = el.id;
    BoardOp::Put(el).apply(&mut doc);
    ed.select(&doc, [id]);
    (ed, doc)
}

/// O cursor na pega `d` da moldura.
fn on(ed: &Editor, doc: &BoardDoc, h: Handle) -> Option<BoardCursor> {
    let f = ed.frame(doc).unwrap();
    let p = f.handle(h, 1.0, ed.metrics());
    ed.cursor_at(
        doc,
        Pointer {
            world: p,
            px: 1.0,
            mods: Mods::default(),
        },
    )
}

/// ⭐ Cada pega pede a seta dupla do EIXO dela — rodada com a moldura —, a de rodar pede agarrar, e
/// fora das pegas não se pede nada.
#[test]
fn each_handle_asks_for_the_cursor_of_its_axis() {
    use crate::Dir::{E, N, NE, NW, S, SE, SW, W};
    use BoardCursor::{ResizeEw, ResizeNesw, ResizeNs, ResizeNwse, Rotate};
    let (ed, doc) = editor_with_box(0.0);
    for (d, want) in [
        (N, ResizeNs),
        (S, ResizeNs),
        (E, ResizeEw),
        (W, ResizeEw),
        (NE, ResizeNesw),
        (SW, ResizeNesw),
        (NW, ResizeNwse),
        (SE, ResizeNwse),
    ] {
        assert_eq!(on(&ed, &doc, Handle::Resize(d)), Some(want), "{d:?}");
    }
    assert_eq!(on(&ed, &doc, Handle::Rotate), Some(Rotate));
    let away = Pointer {
        world: [100.0, 50.0],
        px: 1.0,
        mods: Mods::default(),
    };
    assert_eq!(ed.cursor_at(&doc, away), None);
    // Rodada um quarto de volta, a pega do lado direito fica em cima/baixo.
    let (ed, doc) = editor_with_box(std::f64::consts::FRAC_PI_2);
    assert_eq!(on(&ed, &doc, Handle::Resize(E)), Some(ResizeNs));
    assert_eq!(on(&ed, &doc, Handle::Resize(NE)), Some(ResizeNwse));
}
