//! O que a sessão ao vivo mostra (cortado do `live.rs`, tecto de LOC): os avisos de quem entra e
//! sai, «trazer todos», e as pessoas, os cursores e a câmara de quem sigo.

use ph2d_board_model::BoardId;
use ph2d_board_sync::{self as sync, Event};
use ph2d_editor_core::documents::live::{LiveOther, LivePeer};
use ph2d_editor_core::screens::HeroScreen;
use ph2d_i18n::{tr, tr_with};

use super::fac::{self, refusal_key};

/// Um aviso da sessão, e «trazer todos».
pub(super) fn event(hero: &mut HeroScreen, board: BoardId, e: Event, names: &[(u64, String)]) {
    let name = |p: u64| {
        names
            .iter()
            .find(|(q, _)| *q == p)
            .map(|(_, n)| n.clone())
            .unwrap_or_default()
    };
    match e {
        Event::Joined(p) => {
            let n = fac::someone(hero, p.name);
            hero.documents
                .tell(tr_with("board.live.joined", &[("name", &n)]));
        }
        Event::Left(p) => {
            let n = fac::someone(hero, name(p));
            hero.documents
                .tell(tr_with("board.live.left", &[("name", &n)]));
        }
        Event::Summoned { from, camera } => {
            if let Some(b) = hero.documents.board_mut(board) {
                b.camera = camera;
            }
            hero.documents.activate(Some(board));
            hero.documents.live_view_mut().following = None;
            let n = fac::someone(hero, name(from));
            hero.documents
                .tell(tr_with("board.live.summoned", &[("name", &n)]));
        }
        Event::Denied(r) => hero.documents.tell(tr(refusal_key(r)).to_owned()),
        Event::Welcomed | Event::Refused { .. } | Event::Arrived(_) | Event::Ended => {}
    }
}

/// O que o ecrã desenha: as pessoas, os outros (cursor interpolado e selecção), e a câmara de quem
/// sigo.
pub(super) fn view<'a>(
    hero: &mut HeroScreen,
    board: BoardId,
    peers: &[LivePeer],
    seen: impl Fn(u64) -> Option<&'a sync::Seen>,
    now: u64,
) {
    // W12 — «Make names anonymous»: o nome nem chega ao ecrã, e ninguém se segue.
    let anon = hero
        .documents
        .live_view()
        .fac
        .private
        .as_ref()
        .is_some_and(|p| p.anonymous);
    let others: Vec<LiveOther> = peers
        .iter()
        .filter(|p| !p.me)
        .map(|p| {
            let s = seen(p.peer);
            LiveOther {
                peer: p.peer,
                name: if anon { String::new() } else { p.name.clone() },
                anonymous: anon,
                color: p.color,
                cursor: s.and_then(|s| s.cursor_at(now)),
                selection: s
                    .and_then(|s| s.last.as_ref())
                    .map(|l| l.selection.clone())
                    .unwrap_or_default(),
            }
        })
        .collect();
    let follow = hero.documents.live_view().following.filter(|_| !anon);
    let camera = follow.and_then(&seen).and_then(|s| s.camera_at(now));
    let v = hero.documents.live_view_mut();
    v.board = Some(board);
    v.peers = peers.to_vec();
    v.others = others;
    if anon || follow.is_some_and(|p| !peers.iter().any(|q| q.peer == p)) {
        v.following = None;
    }
    if let Some(cam) = camera
        && let Some(b) = hero.documents.board_mut(board)
    {
        b.camera = cam;
    }
}
