//! W12 — a nota escondida pelo que o desenho EMITE (a gravação da cena, a mesma do SVG/PDF).

use std::collections::BTreeSet;

use ph2d_board_model::{
    BoardOp, BoardSet, Element, ElementId, Rgba, RichText, STICKY_SIDE, Shape, ShapeType, Style,
};
use ph2d_board_route::RouteCache;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{DrawOp, PathEl, Shape as _, VectorScene};

use crate::RenderCache;

fn ops(unread: bool, mine: bool) -> Vec<DrawOp> {
    let mut set = BoardSet::default();
    let bid = set.create("p".into());
    let b = set.get_mut(bid).unwrap();
    let ink = Rgba([20, 20, 20, 255]);
    let shape = Shape {
        kind: ShapeType::Sticky,
        style: Style::new(Some(Rgba([255, 230, 120, 255])), None, ink),
        text: RichText::plain("segredo"),
    };
    let el = Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        shape,
        [20.0, 20.0, STICKY_SIDE, STICKY_SIDE],
    );
    let id: ElementId = el.id;
    let _ = BoardOp::Put(el).apply(&mut b.doc);
    let board = &set.boards()[0];
    let mut routes = RouteCache::default();
    routes.sync(&board.doc);
    let mut scene = VectorScene::new();
    scene.record();
    let mut ts = TextSystem::without_system_fonts();
    let mut cache = RenderCache::default();
    if unread {
        cache.unread = BTreeSet::from([id]);
    }
    if mine {
        cache.mine_private = BTreeSet::from([id]);
    }
    crate::paint(
        board,
        [0.0, 0.0, 640.0, 480.0],
        &mut scene,
        Theme::Forge,
        &mut ts,
        &mut cache,
        &routes,
        None,
    );
    scene.take_record()
}

fn count(ops: &[DrawOp]) -> [usize; 3] {
    let mut n = [0; 3];
    for o in ops {
        match o {
            DrawOp::Glyphs(_) => n[0] += 1,
            DrawOp::Stroke { .. } => n[1] += 1,
            DrawOp::Fill { .. } => n[2] += 1,
            _ => {}
        }
    }
    n
}

/// ⭐ A quem NÃO lê: nenhuma letra sai do desenho (nem as barras de longe) — só o olho (o selo e o
/// ícone) e os rabiscos. Ao autor: as mesmas letras de sempre, e o olho.
#[test]
fn an_unread_note_paints_no_letter_only_the_eye_and_the_squiggle() {
    let [glyphs, strokes, fills] = count(&ops(false, false));
    assert!(glyphs > 0, "controlo: a nota às claras desenha o texto");
    assert_eq!(count(&ops(true, false)), [0, strokes + 2, fills + 1]);
    assert_eq!(count(&ops(false, true)), [glyphs, strokes + 1, fills + 1]);
}

/// O olho é o ícone Lucide `eye-closed` INTEIRO, igual ao do repositório (`docs/design/icons`).
#[test]
fn the_eye_is_the_whole_lucide_eye_closed() {
    let svg = include_str!("../../../docs/design/icons/eye-closed.svg");
    let file: Vec<&str> = svg
        .split("d=\"")
        .skip(1)
        .filter_map(|s| s.split('"').next())
        .collect();
    assert_eq!(file, super::EYE_CLOSED_SVG);
    let p = super::eye_path();
    let moves = p
        .elements()
        .iter()
        .filter(|e| matches!(e, PathEl::MoveTo(_)))
        .count();
    assert_eq!(moves, file.len());
    let b = p.bounding_box();
    assert!(
        b.x0 >= 0.0 && b.y0 >= 0.0 && b.x1 <= 24.0 && b.y1 <= 24.0,
        "{b:?}"
    );
}
