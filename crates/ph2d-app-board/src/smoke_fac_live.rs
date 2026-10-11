//! Cena 13 — O QUE É DE UMA PESSOA (W12, ADR-0182), módulo filho do roteador [`super`]: a «Private
//! retro» JÁ ABERTA à rede (o painel Live com o código) e o painel do modo privado aberto, com quatro
//! ideias num frame «Ideas» (para votar), um frame vazio «Write privately» (para escrever em privado)
//! e por baixo uma nota por lição na ORDEM REAL (entrar · ligar o modo · escrever escondido · revelar ·
//! votar · acabar). A 2.ª janela é OUTRO PH2D (a cena 1 basta). O gate
//! `the_private_scene_hides_until_the_reveal_and_votes_without_peeking` conduz o que a cena arma.

use super::team::{INK, note, tr};
use ph2d_board_model::{
    Board, BoardId, BoardSet, Element, ElementId, RichText, STICKY_SIDE, Shape, ShapeType, Style,
};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::documents::live::LiveRequest;
use ph2d_editor_core::screens::hero::board_nav::team::live_board;
use ph2d_editor_core::screens::hero::{board_nav, document_tabs};

/// O que o gate conduz.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "só o gate da cena lê os ids")
)]
pub(crate) struct Lessons {
    pub board: BoardId,
    pub ideas: [ElementId; 4],
    /// O frame das ideias (seleccioná-lo vota só nelas).
    pub ideas_frame: ElementId,
    /// O frame vazio onde se escreve em privado.
    pub private: ElementId,
    pub lessons: [ElementId; 6],
}

/// A margem do que os painéis tapam (a da cena 12, em unidades do mundo).
const MARGIN_X: f64 = 450.0;
const MARGIN_Y: f64 = 90.0;
const GAP: f64 = 40.0;
const SMALL: f64 = 150.0;

/// Monta a retrospectiva em `b`; devolve os ids e a vista que enquadra tudo.
pub(crate) fn build(b: &mut Board) -> (Lessons, [f64; 4]) {
    let side = STICKY_SIDE;
    let ideas_w = 4.0 * SMALL + 5.0 * GAP;
    let h = 320.0;
    let ideas_frame = super::organize::frame(
        &mut b.doc,
        &tr("board.smoke.faclive.ideas"),
        [0.0, 0.0, ideas_w, h],
        None,
    );
    let ideas: [ElementId; 4] = std::array::from_fn(|i| {
        let key = format!("board.smoke.faclive.n{}", i + 1);
        let x = GAP + i as f64 * (SMALL + GAP);
        note(b, &key, [12, 5, 1, 8][i], [x, 90.0, SMALL, SMALL], false, Some(ideas_frame))
    });
    let px = ideas_w + 2.0 * GAP;
    let private_w = 3.0 * SMALL + 4.0 * GAP;
    let private = super::organize::frame(
        &mut b.doc,
        &tr("board.smoke.faclive.private"),
        [px, 0.0, private_w, h],
        None,
    );
    let width = px + private_w;
    let y = h + GAP;
    let lessons: [ElementId; 6] = std::array::from_fn(|i| {
        let key = format!("board.smoke.faclive.l{}", i + 1);
        note(b, &key, 1, [i as f64 * (side + GAP), y, side, side], false, None)
    });
    let title = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(None, None, INK),
        text: RichText::plain(&tr("board.smoke.faclive.title")),
    };
    let t = Element::new_shape(b.doc.mint_id(), b.doc.z_on_top(), title, [0.0, -80.0, width, 40.0]);
    super::team::put(b, t);
    let lessons_w = 6.0 * side + 5.0 * GAP;
    let view = [
        -MARGIN_X,
        -80.0 - MARGIN_Y,
        width.max(lessons_w) + MARGIN_X,
        y + side + MARGIN_Y,
    ];
    (
        Lessons {
            board: b.id,
            ideas,
            ideas_frame,
            private,
            lessons,
        },
        view,
    )
}

/// A cena 13 no hero: o quadro aberto à rede (a família serve o pedido na 1.ª imagem), o painel Live
/// e o do modo privado abertos. Devolve os ids (os testes conduzem-na pelo MESMO caminho do smoke).
pub(super) fn scene_fac_live(hero: &mut HeroScreen) -> Lessons {
    let mut set = BoardSet::default();
    let id = set.create(tr("board.smoke.faclive.board"));
    let (l, view) = build(set.get_mut(id).expect("acabou de nascer"));
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show_box(hero, false, false, view);
    live_board::toggle(hero);
    board_nav::facilitate::toggle_private(hero);
    hero.documents.ask_live(LiveRequest::Host);
    l
}

#[cfg(test)]
#[path = "smoke_fac_live_tests.rs"]
mod tests;
