//! Cena 24 — O BLOCO 3 DO W23 («quadro»), módulo filho do roteador [`super`]: o mapa mental da cena 8
//! em RASCUNHO (os ramos saem à mão, como as pílulas), com as seis notas trocadas pelas lições do que
//! mudou: pairar mostra «+» e «−», a letra pela raiz, todas as pontas, o botão direito, o trackpad e
//! as abas a mais. O gate `the_w23_scene_draws_the_map_by_hand_with_its_lessons` conduz o que ela arma.

use ph2d_board_model::{BoardOp, BoardSet, RichText};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::{board_nav, document_tabs};

/// As lições, pela ordem das notas.
const LESSONS: [&str; 6] = [
    "board.smoke.w23.l1",
    "board.smoke.w23.l2",
    "board.smoke.w23.l3",
    "board.smoke.w23.l4",
    "board.smoke.w23.l5",
    "board.smoke.w23.l6",
];

pub(super) fn scene_w23(hero: &mut HeroScreen) -> super::mind::Lessons {
    let mut set = BoardSet::default();
    let id = set.create(ph2d_i18n::tr("board.smoke.w23.board").to_owned());
    let board = set.get_mut(id).expect("acabou de nascer");
    let l = super::mind::build(&mut board.doc);
    board.sketch = true;
    let all: Vec<_> = board.doc.live().cloned().collect();
    for mut el in all {
        el.style_mut().sketch = true;
        if let Some(i) = l.notes.iter().position(|n| *n == el.id)
            && let Some(s) = el.shape_mut()
        {
            s.text = RichText::plain(ph2d_i18n::tr(LESSONS[i]));
        }
        BoardOp::Put(el).apply(&mut board.doc);
    }
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show(hero, false, false);
    l
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_a11y::NodeId;

    /// A cena 24 abre o mapa em rascunho (tudo à mão) com as seis lições do bloco 3.
    #[test]
    fn the_w23_scene_draws_the_map_by_hand_with_its_lessons() {
        let mut h = HeroScreen::new(NodeId(1));
        let l = scene_w23(&mut h);
        let b = h.documents.active_board().expect("o quadro aberto");
        assert!(b.sketch);
        assert!(b.doc.live().all(|el| el.style().sketch));
        for (n, key) in l.notes.iter().zip(LESSONS) {
            let text = b
                .doc
                .get(*n)
                .unwrap()
                .shape()
                .unwrap()
                .text
                .as_str()
                .to_owned();
            assert_eq!(text, ph2d_i18n::tr(key));
        }
    }
}
