//! Cena 25 — W23 B5(1), o segmento do MEIO de uma seta cotovelo (doc 13), módulo filho do roteador
//! [`super`]: duas caixas desencontradas (as dos oráculos: 0,0 e 400,300, 100×100) com uma seta
//! cotovelo seleccionada — a pílula no meio da linha vertical à vista — e as quatro lições. O gate
//! `the_w23_elbow_scene_opens_with_the_middle_bar_in_reach` conduz o arrasto que a lição 1 pede.

use ph2d_board_model::{Anchor, BoardOp, BoardSet, ElementId, End, Route, ShapeType};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::board_view::{self, default_style};
use ph2d_editor_core::screens::hero::{board_nav, document_tabs};

/// As lições, pela ordem das notas.
const LESSONS: [&str; 4] = [
    "board.smoke.elbow.l1",
    "board.smoke.elbow.l2",
    "board.smoke.elbow.l3",
    "board.smoke.elbow.l4",
];

/// O lado das caixas e onde ficam (as do oráculo), e as notas por baixo.
const SIDE: f64 = 100.0;
const BOXES: [[f64; 2]; 2] = [[0.0, 0.0], [400.0, 300.0]];
const NOTES_Y: f64 = 480.0;
const NOTE_W: f64 = 220.0;
const NOTE_GAP: f64 = 20.0;

pub(super) fn scene_elbow(hero: &mut HeroScreen) -> ElementId {
    let mut set = BoardSet::default();
    let id = set.create(ph2d_i18n::tr("board.smoke.elbow.board").to_owned());
    let doc = &mut set.get_mut(id).expect("acabou de nascer").doc;
    let style = default_style();
    let keys = ["board.smoke.elbow.start", "board.smoke.elbow.finish"];
    let mut bound = Vec::new();
    for ([x, y], key) in BOXES.into_iter().zip(keys) {
        let b = super::put_shape(
            doc,
            ShapeType::Rectangle,
            style.clone(),
            key,
            [x, y, SIDE, SIDE],
        );
        bound.push(End::Bound {
            target: b,
            anchor: Anchor::Center,
        });
    }
    let arrow = super::put_arrow(doc, bound[0], bound[1], &style, "", &[]);
    let mut el = doc.get(arrow).expect("a seta").clone();
    el.connector_mut().expect("é seta").route = Route::Elbow;
    BoardOp::Put(el).apply(doc);
    for (i, key) in LESSONS.into_iter().enumerate() {
        let x = i as f64 * (NOTE_W + NOTE_GAP);
        super::put_note(
            doc,
            ShapeType::Sticky,
            1,
            super::note_text(key),
            [x, NOTES_Y, NOTE_W],
        );
    }
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show(hero, false, false);
    board_view::select(hero, [arrow]);
    arrow
}

#[cfg(test)]
#[path = "smoke_w23_elbow_tests.rs"]
mod tests;
