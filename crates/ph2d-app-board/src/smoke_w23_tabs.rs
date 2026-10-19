//! Cena 27 — W23 B5(4), o `Ctrl+Z` das operações de aba (doc 15), módulo filho do roteador
//! [`super`]: três quadros (Ideas · Plan · Archive), cada um com o que tem, e o do meio aberto com as
//! quatro lições. O gate `the_w23_tabs_scene_brings_the_archive_back_with_ctrl_z` conduz a lição 1.

use ph2d_board_model::{BoardId, BoardSet, ShapeType};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::board_view::default_style;
use ph2d_editor_core::screens::hero::{board_nav, document_tabs};

/// Os quadros, pela ordem das abas: o nome e o que traz escrito na caixa.
const BOARDS: [(&str, &str); 3] = [
    ("board.smoke.tabs.ideas", "board.smoke.tabs.ideas_box"),
    ("board.smoke.tabs.plan", "board.smoke.tabs.plan_box"),
    ("board.smoke.tabs.archive", "board.smoke.tabs.archive_box"),
];

/// As lições, pela ordem das notas (no quadro do meio).
const LESSONS: [&str; 4] = [
    "board.smoke.tabs.l1",
    "board.smoke.tabs.l2",
    "board.smoke.tabs.l3",
    "board.smoke.tabs.l4",
];

const BOX: [f64; 4] = [0.0, 0.0, 240.0, 120.0];
const NOTES_Y: f64 = 200.0;
const NOTE_W: f64 = 240.0;
const NOTE_GAP: f64 = 20.0;

/// A cena; devolve os três quadros pela ordem das abas.
pub(super) fn scene_tabs(hero: &mut HeroScreen) -> [BoardId; 3] {
    let mut set = BoardSet::default();
    let style = default_style();
    let ids = BOARDS.map(|(name, text)| {
        let id = set.create(ph2d_i18n::tr(name).to_owned());
        let doc = &mut set.get_mut(id).expect("acabou de nascer").doc;
        super::put_shape(doc, ShapeType::Rectangle, style.clone(), text, BOX);
        id
    });
    let plan = &mut set.get_mut(ids[1]).expect("o do meio").doc;
    for (i, key) in LESSONS.into_iter().enumerate() {
        let x = i as f64 * (NOTE_W + NOTE_GAP);
        super::put_note(
            plan,
            ShapeType::Sticky,
            1,
            super::note_text(key),
            [x, NOTES_Y, NOTE_W],
        );
    }
    document_tabs::load(hero, set);
    hero.documents.activate(Some(ids[1]));
    board_nav::show(hero, false, false);
    ids
}

#[cfg(test)]
#[path = "smoke_w23_tabs_tests.rs"]
mod tests;
