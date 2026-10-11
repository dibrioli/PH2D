//! Cena 14 — O HISTÓRICO DAS VOTAÇÕES (W13, ADR-0183), módulo filho do roteador [`super`]: a «Vote
//! history» JÁ ABERTA à rede (o painel Live com o código) e o painel da votação no separador
//! «Results», com cinco ideias num frame «Ideas», DUAS votações anteriores (a mais nova mostrada; a
//! mais velha cita uma nota apagada depois do fim) e por baixo uma nota por lição na ORDEM REAL (a
//! lista · abrir uma antiga · renomear · o toggle · votar de novo · apagar · a 2.ª janela). O gate
//! `the_history_scene_does_what_its_lessons_say` conduz o que a cena arma.

use std::collections::BTreeMap;

use super::team::{INK, note, put, tr};
use ph2d_board_model::{
    Board, BoardId, BoardOp, BoardSet, Element, ElementId, PollId, RichText, STICKY_SIDE, Shape,
    ShapeType, Style,
};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::documents::live::LiveRequest;
use ph2d_editor_core::screens::hero::board_nav::team::{live_board, now_unix};
use ph2d_editor_core::screens::hero::{board_nav, document_tabs};

/// O que o gate conduz.
#[cfg_attr(not(test), expect(dead_code, reason = "só o gate da cena lê os ids"))]
pub(crate) struct Lessons {
    pub board: BoardId,
    pub ideas: [ElementId; 5],
    /// A nota que a «Voting 1» votou e que se apagou depois do fim.
    pub gone: ElementId,
    /// «Voting 1» (há dois dias, a Ana) e «Voting 2» (há uma hora, o Rui; a mostrada).
    pub polls: [PollId; 2],
    pub lessons: [ElementId; 7],
}

/// A margem do que os painéis tapam (a da cena 13, em unidades do mundo).
const MARGIN_X: f64 = 450.0;
const MARGIN_Y: f64 = 90.0;
const GAP: f64 = 40.0;
const SMALL: f64 = 150.0;

/// Monta a retrospectiva em `b` com o histórico; devolve os ids e a vista que enquadra tudo.
pub(crate) fn build(b: &mut Board, now: i64) -> (Lessons, [f64; 4]) {
    let side = STICKY_SIDE;
    let ideas_w = 6.0 * SMALL + 7.0 * GAP;
    let h = 320.0;
    let frame = super::organize::frame(
        &mut b.doc,
        &tr("board.smoke.history.ideas"),
        [0.0, 0.0, ideas_w, h],
        None,
    );
    let at = |i: usize| [GAP + i as f64 * (SMALL + GAP), 90.0, SMALL, SMALL];
    let ideas: [ElementId; 5] = std::array::from_fn(|i| {
        let key = format!("board.smoke.history.n{}", i + 1);
        note(b, &key, [12, 5, 1, 8, 3][i], at(i), false, Some(frame))
    });
    // A nota que a votação antiga votou e que depois se apagou: a linha dela diz «(deleted)».
    let gone = note(b, "board.smoke.history.gone", 6, at(5), false, Some(frame));
    let _ = BoardOp::Delete(gone).apply(&mut b.doc);
    let t = |v: &[(ElementId, u32)]| v.iter().copied().collect::<BTreeMap<_, _>>();
    let old = b.team.add_poll(
        t(&[(ideas[0], 3), (gone, 2), (ideas[1], 1)]),
        "Ana",
        3,
        now - 2 * 86_400,
    );
    let new = b.team.add_poll(
        t(&[(ideas[2], 2), (ideas[3], 2), (ideas[4], 1)]),
        "Rui",
        2,
        now - 3_600,
    );
    let y = h + GAP;
    let lessons: [ElementId; 7] = std::array::from_fn(|i| {
        let key = format!("board.smoke.history.l{}", i + 1);
        note(b, &key, 1, [i as f64 * (side + GAP), y, side, side], false, None)
    });
    let title = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(None, None, INK),
        text: RichText::plain(&tr("board.smoke.history.title")),
    };
    let el = Element::new_shape(b.doc.mint_id(), b.doc.z_on_top(), title, [0.0, -80.0, ideas_w, 40.0]);
    put(b, el);
    let lessons_w = 7.0 * side + 6.0 * GAP;
    let view = [
        -MARGIN_X,
        -80.0 - MARGIN_Y,
        ideas_w.max(lessons_w) + MARGIN_X,
        y + side + MARGIN_Y,
    ];
    let l = Lessons {
        board: b.id,
        ideas,
        gone,
        polls: [old, new],
        lessons,
    };
    (l, view)
}

/// A cena 14 no hero: o quadro aberto à rede (a família serve o pedido na 1.ª imagem), o painel Live
/// e o da votação no separador «Results». Devolve os ids (os testes conduzem-na pelo MESMO caminho
/// do smoke).
pub(super) fn scene_history(hero: &mut HeroScreen) -> Lessons {
    let mut set = BoardSet::default();
    let id = set.create(tr("board.smoke.history.board"));
    let (l, view) = build(set.get_mut(id).expect("acabou de nascer"), now_unix());
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show_box(hero, false, false, view);
    live_board::toggle(hero);
    board_nav::facilitate::toggle_vote(hero);
    hero.documents.ask_live(LiveRequest::Host);
    l
}

#[cfg(test)]
#[path = "smoke_history_tests.rs"]
mod tests;
