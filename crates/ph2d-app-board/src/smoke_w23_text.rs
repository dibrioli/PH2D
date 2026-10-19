//! Cena 26 — W23 B5(2), os textos ao longo da linha de uma seta (doc 14), módulo filho do roteador
//! [`super`]: três caixas (Idea → Plan → Done); a seta curva traz «next step» deitado AO LONGO dela e
//! vem seleccionada, a recta traz «then» na HORIZONTAL — os dois modos à vista — e as quatro lições. O
//! gate `the_w23_text_scene_opens_with_the_words_in_reach` conduz o arrasto que a lição 1 pede.

use ph2d_board_model::{
    Anchor, BoardOp, BoardSet, Caption, ElementId, End, Route, ShapeType, TextTurn,
};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::board_view::{self, default_style};
use ph2d_editor_core::screens::hero::{board_nav, document_tabs};

/// As lições, pela ordem das notas.
const LESSONS: [&str; 4] = [
    "board.smoke.text.l1",
    "board.smoke.text.l2",
    "board.smoke.text.l3",
    "board.smoke.text.l4",
];

/// As caixas (Idea, Plan, Done) e as notas por baixo.
const BOX: [f64; 2] = [160.0, 100.0];
const BOXES: [[f64; 2]; 3] = [[0.0, 0.0], [520.0, 220.0], [520.0, 560.0]];
const NOTES_Y: f64 = 760.0;
const NOTE_W: f64 = 220.0;
const NOTE_GAP: f64 = 20.0;

/// Onde nasce o texto da seta curva (a 3/10 da linha, para se ver que não é o meio).
pub(super) const FIRST_AT: f64 = 0.3;

/// A cena; devolve a seta curva (a seleccionada).
pub(super) fn scene_text(hero: &mut HeroScreen) -> ElementId {
    let mut set = BoardSet::default();
    let id = set.create(ph2d_i18n::tr("board.smoke.text.board").to_owned());
    let doc = &mut set.get_mut(id).expect("acabou de nascer").doc;
    let style = default_style();
    let keys = [
        "board.smoke.text.idea",
        "board.smoke.text.plan",
        "board.smoke.text.done",
    ];
    let ends: Vec<End> = BOXES
        .into_iter()
        .zip(keys)
        .map(|([x, y], key)| End::Bound {
            target: super::put_shape(
                doc,
                ShapeType::Rectangle,
                style.clone(),
                key,
                [x, y, BOX[0], BOX[1]],
            ),
            anchor: Anchor::Center,
        })
        .collect();
    let curved = super::put_arrow(doc, ends[0], ends[1], &style, "", &[]);
    let straight = super::put_arrow(doc, ends[1], ends[2], &style, "", &[]);
    for (arrow, key, at, route, turn) in [
        (
            curved,
            "board.smoke.text.next",
            FIRST_AT,
            Route::Curved,
            TextTurn::Along,
        ),
        (
            straight,
            "board.smoke.text.then",
            Caption::MIDDLE,
            Route::Straight,
            TextTurn::Level,
        ),
    ] {
        let mut el = doc.get(arrow).expect("a seta").clone();
        let c = el.connector_mut().expect("é seta");
        c.route = route;
        c.turn = turn;
        c.captions = vec![Caption::new(ph2d_i18n::tr(key), at)];
        BoardOp::Put(el).apply(doc);
    }
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
    board_view::select(hero, [curved]);
    curved
}

#[cfg(test)]
#[path = "smoke_w23_text_tests.rs"]
mod tests;
