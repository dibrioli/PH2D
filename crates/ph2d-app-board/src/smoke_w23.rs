//! Cena 24 — O BLOCO 3 DO W23 («quadro»), módulo filho do roteador [`super`]: o mapa mental da cena 8
//! em RASCUNHO (os ramos saem à mão, como as pílulas), com as seis notas trocadas pelas lições do que
//! mudou: pairar mostra «+» e «−», a letra pela raiz, todas as pontas, o botão direito, o trackpad e
//! as abas a mais. O gate `the_w23_scene_draws_the_map_by_hand_with_its_lessons` conduz o que ela arma.

use ph2d_board_model::{Anchor, BoardOp, BoardSet, ElementId, End, RichText, ShapeType};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::board_view::{self, default_style};
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

/// As duas caixas da lição 4, por baixo do frame do mapa: a largura, a altura e o vão entre elas.
const BOX: [f64; 2] = [200.0, 100.0];
const BOX_GAP: f64 = 240.0;

pub(super) fn scene_w23(hero: &mut HeroScreen) -> (super::mind::Lessons, ElementId) {
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
    // A lição 4: duas caixas com uma seta presa às duas (como o dono a desenha), a abrir seleccionada
    // — a barra dela, com o «…» de cada grupo de pontas, à vista desde o início.
    let doc = &mut board.doc;
    let (left, bottom) = doc
        .live()
        .map(ph2d_board_model::Element::aabb)
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(x, y), b| {
            (x.min(b[0]), y.max(b[3]))
        });
    let y = bottom + BOX_GAP / 2.0;
    let mut style = default_style();
    style.sketch = true;
    let mut boxed = |x: f64, key: &str| {
        let bx = [x, y, BOX[0], BOX[1]];
        let id = super::put_shape(doc, ShapeType::Rectangle, style.clone(), key, bx);
        End::Bound {
            target: id,
            anchor: Anchor::Center,
        }
    };
    let a = boxed(left, "board.smoke.w23.from");
    let b = boxed(left + BOX[0] + BOX_GAP, "board.smoke.w23.to");
    let arrow = super::put_arrow(doc, a, b, &style, "", &[]);
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show(hero, false, false);
    board_view::select(hero, [arrow]);
    (l, arrow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_a11y::NodeId;

    /// A cena 24 abre o mapa em rascunho (tudo à mão) com as seis lições do bloco 3.
    #[test]
    fn the_w23_scene_draws_the_map_by_hand_with_its_lessons() {
        let mut h = HeroScreen::new(NodeId(1));
        let (l, arrow) = scene_w23(&mut h);
        let b = h.documents.active_board().expect("o quadro aberto");
        assert!(b.sketch);
        assert!(b.doc.live().all(|el| el.style().sketch));
        let c = b
            .doc
            .get(arrow)
            .and_then(|el| el.connector())
            .expect("a seta");
        assert!(
            c.ends().iter().all(|e| matches!(e, End::Bound { .. })),
            "presa às duas caixas, como o dono a desenha"
        );
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
