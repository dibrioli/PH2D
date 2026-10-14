//! Cena 12 — O QUADRO AO VIVO (W11, `docs/MiroClone/08_ao_vivo.md`), módulo filho do roteador
//! [`super`]: a retrospectiva «Retro» (um frame com três notas e um fluxo de três caixas com setas) JÁ
//! ABERTA à rede, com o painel Live aberto e o código à vista, e por baixo uma nota por lição na ORDEM
//! REAL (o código · a firewall · a 2.ª janela entra · desenhar junto · seguir · trazer todos e fechar).
//! A 2.ª janela é OUTRO PH2D (a cena 1 basta: o painel acha a «Retro» na lista). O gate
//! `the_live_scene_hosts_and_a_second_window_joins_by_its_code` conduz o que a cena arma.

use super::team::{INK, arrow, boxed, note, tr};
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
    expect(dead_code, reason = "só o gate da cena lê as lições")
)]
pub(crate) struct Lessons {
    pub board: BoardId,
    pub lessons: [ElementId; 6],
    pub view: [f64; 4],
}

/// A margem do que os painéis tapam (a da cena 10, medida na foto dela: em unidades do mundo).
const MARGIN_X: f64 = 450.0;
const MARGIN_Y: f64 = 90.0;
const GAP: f64 = 40.0;
const SMALL: f64 = 150.0;

/// Monta a retrospectiva em `b`.
pub(crate) fn build(b: &mut Board) -> Lessons {
    build_with(b, "board.smoke.live")
}

/// A retrospectiva com as lições e o título de `prefix` (`<prefix>.l1..l6`, `<prefix>.title`).
fn build_with(b: &mut Board, prefix: &str) -> Lessons {
    let side = STICKY_SIDE;
    let width = 6.0 * side + 5.0 * GAP;
    let frame = super::organize::frame(
        &mut b.doc,
        &tr("board.smoke.live.frame"),
        [0.0, 0.0, width, 320.0],
        None,
    );
    let y0 = 90.0;
    for (i, (key, color)) in [
        ("board.smoke.live.n_well", 12),
        ("board.smoke.live.n_better", 5),
        ("board.smoke.live.n_idea", 1),
    ]
    .into_iter()
    .enumerate()
    {
        let x = GAP + i as f64 * (SMALL + GAP);
        note(b, key, color, [x, y0, SMALL, SMALL], false, Some(frame));
    }
    let fx = 3.0 * (SMALL + GAP) + 2.0 * GAP;
    let (w, step) = (130.0, 190.0);
    let flow: Vec<ElementId> = [
        "board.smoke.live.plan",
        "board.smoke.live.draw",
        "board.smoke.live.share",
    ]
    .iter()
    .enumerate()
    .map(|(i, k)| boxed(b, k, [fx + i as f64 * step, 130.0, w, 70.0], frame))
    .collect();
    arrow(b, flow[0], flow[1]);
    arrow(b, flow[1], flow[2]);
    let y = 320.0 + GAP;
    let lessons: [ElementId; 6] = std::array::from_fn(|i| {
        let key = format!("{prefix}.l{}", i + 1);
        note(
            b,
            &key,
            1,
            [i as f64 * (side + GAP), y, side, side],
            false,
            None,
        )
    });
    let title = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(None, None, INK),
        text: RichText::plain(&tr(&format!("{prefix}.title"))),
    };
    let t = Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        title,
        [0.0, -80.0, width, 40.0],
    );
    super::team::put(b, t);
    Lessons {
        board: b.id,
        lessons,
        view: [
            -MARGIN_X,
            -80.0 - MARGIN_Y,
            width + MARGIN_X,
            y + side + MARGIN_Y,
        ],
    }
}

/// A cena 12 no hero: o quadro aberto à rede (a família serve o pedido na 1.ª imagem) e o painel
/// Live aberto. Devolve os ids (os testes conduzem-na pelo MESMO caminho do smoke).
pub(super) fn scene_live(hero: &mut HeroScreen) -> Lessons {
    let mut set = BoardSet::default();
    let id = set.create(tr("board.smoke.live.board"));
    let l = build(set.get_mut(id).expect("acabou de nascer"));
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show_box(hero, false, false, l.view);
    live_board::toggle(hero);
    hero.documents.ask_live(LiveRequest::Host);
    l
}

/// A cena 22 (ADR-0187): a mesma «Retro» aberta à rede E à internet — o link já copiado para mandar —
/// com as seis lições do convite pela internet. A 2.ª janela cola o link e clica «Join by link».
/// `PH2D_BOARD_INTERNET=local` ensaia sem sair para a rede pública.
pub(super) fn scene_internet(hero: &mut HeroScreen) -> Lessons {
    let mut set = BoardSet::default();
    let id = set.create(tr("board.smoke.net.board"));
    let l = build_with(
        set.get_mut(id).expect("acabou de nascer"),
        "board.smoke.net",
    );
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_nav::show_box(hero, false, false, l.view);
    live_board::toggle(hero);
    hero.documents.ask_live(LiveRequest::Host);
    hero.documents.ask_live(LiveRequest::InviteInternet);
    l
}

#[cfg(test)]
#[path = "smoke_live_tests.rs"]
mod tests;
