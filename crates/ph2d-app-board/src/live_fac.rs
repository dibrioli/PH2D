//! **A facilitação ao vivo na família** ([ADR-0182]), módulo filho de [`super`]: os pedidos do ecrã
//! (o modo privado, a votação, o meu boletim) à sessão, as recusas em frase, e a vista que o ecrã
//! desenha — nomes no lugar dos pares, o que é escondido de mim, o que é meu — com os avisos das
//! viradas.
//!
//! [ADR-0182]: ../../../docs/architecture/decisions/0182-o-que-e-de-uma-pessoa-mora-no-cofre-do-anfitriao.md

use std::collections::BTreeSet;

use ph2d_board_model::ElementId;
use ph2d_board_sync::{self as sync, Ask, Ballot, Event, Refusal};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::documents::live::{LiveFac, LivePeer, LivePrivate, LiveRequest, LiveVote};
use ph2d_i18n::{tr, tr_with};

use super::{Live, State, now_ms};

/// Os pedidos da facilitação (os outros `LiveRequest` são do [`super::request`]).
pub(super) fn request(st: &mut State, hero: &mut HeroScreen, req: LiveRequest) {
    let a = match req {
        LiveRequest::PrivateOn {
            anonymous,
            anyone_can_end,
        } => Ask::PrivateOn {
            anonymous,
            anyone_can_end,
        },
        LiveRequest::PrivateOff => Ask::PrivateOff,
        LiveRequest::AnyoneCanReveal(on) => Ask::AnyoneCanReveal(on),
        LiveRequest::RevealMine => Ask::RevealMine,
        LiveRequest::VoteStart { setup, targets } => Ask::VoteStart { setup, targets },
        LiveRequest::VoteEnd => Ask::VoteEnd,
        LiveRequest::Ballot { round, votes, done } => {
            let b = Ballot { round, votes, done };
            match &mut st.live {
                Some(Live::Host { s, .. }) => {
                    if !s.ballot(b) {
                        hero.documents
                            .tell(tr(refusal_key(Refusal::BallotRefused)).to_owned());
                    }
                }
                Some(Live::Guest {
                    net: Some(n), s, ..
                }) => _ = n.send(s.ballot(b).encode()),
                _ => {}
            }
            return;
        }
        _ => return,
    };
    ask(st, hero, a);
}

/// Um pedido ao anfitrião (o meu, se sou eu): o que ele fizer sai no próximo tique.
fn ask(st: &mut State, hero: &mut HeroScreen, a: Ask) {
    match &mut st.live {
        Some(Live::Host { s, board, .. }) => {
            let Some(b) = hero.documents.board_mut(*board) else {
                return;
            };
            if let Some(Event::Denied(r)) = s.ask(b, a, now_ms()) {
                hero.documents.tell(tr(refusal_key(r)).to_owned());
            }
        }
        Some(Live::Guest {
            net: Some(n), s, ..
        }) => _ = n.send(s.ask(a).encode()),
        _ => {}
    }
}

pub(super) fn refusal_key(r: Refusal) -> &'static str {
    match r {
        Refusal::OnlyStarterEndsPrivate => "board.private.only_starter",
        Refusal::RevealMineOff => "board.private.reveal_mine_off",
        Refusal::VoteRunning => "board.livevote.running",
        Refusal::OnlyStarterEndsVote => "board.livevote.only_starter",
        Refusal::NothingToVote => "board.notice.nothing_to_vote",
        Refusal::BallotRefused => "board.livevote.refused",
    }
}

/// ADR-0182 — a facilitação da sala como o ECRÃ a desenha: o que é escondido de mim, o que é meu, e
/// os nomes no lugar dos pares (quem já saiu é «Someone»; quem acabou conta-se entre quem está).
pub(super) fn fac_view(
    fac: &sync::Fac,
    me: u64,
    host: u64,
    mine: BTreeSet<ElementId>,
    peers: &[LivePeer],
) -> LiveFac {
    let name = |p: u64| {
        peers.iter().find(|q| q.peer == p).map_or_else(
            || tr("board.private.someone").to_owned(),
            |q| q.name.clone(),
        )
    };
    let here = |p: &u64| peers.iter().any(|q| q.peer == *p);
    LiveFac {
        private: fac.private.as_ref().map(|p| LivePrivate {
            by: name(p.by),
            by_me: p.by == me,
            anonymous: p.anonymous,
            anyone_can_end: p.anyone_can_end,
            anyone_can_reveal: p.anyone_can_reveal,
            may_end: p.anyone_can_end || p.by == me || me == host,
            reveal_in_ms: p.reveal_in_ms,
        }),
        hidden_to_me: fac.hidden.difference(&mine).copied().collect(),
        mine_hidden: mine,
        vote: fac.vote.as_ref().map(|v| LiveVote {
            round: v.round,
            by: name(v.by),
            by_me: v.by == me,
            may_end: v.by == me || me == host,
            setup: v.setup,
            targets: v.targets.clone(),
            left_ms: v.left_ms,
            done: v.done.iter().filter(|p| here(p)).count(),
            people: peers.len(),
            used: v
                .used
                .iter()
                .map(|(p, n)| (name(*p), *n, v.done.contains(p)))
                .collect(),
            ended: v.ended,
        }),
    }
}

/// A vista nova da facilitação, com os avisos das viradas (o Miro avisa todos: o modo ligou, as
/// notas revelaram-se, uma votação começou — a quem a começou, não).
pub(super) fn announce(hero: &mut HeroScreen, fac: LiveFac) {
    let old = &hero.documents.live_view().fac;
    let mut say = Vec::new();
    match (&old.private, &fac.private) {
        (None, Some(p)) if !p.by_me => {
            say.push(tr_with("board.private.on", &[("name", &p.by)]));
        }
        (Some(_), None) => say.push(tr("board.private.revealed").to_owned()),
        _ => {}
    }
    let open = |f: &LiveFac| f.vote.as_ref().filter(|v| !v.ended).map(|v| v.round);
    if let Some(v) = fac
        .vote
        .as_ref()
        .filter(|v| !v.by_me && open(&fac) != open(old) && open(&fac).is_some())
    {
        let n = v.setup.votes.to_string();
        say.push(tr_with(
            "board.livevote.started",
            &[("name", &v.by), ("n", &n)],
        ));
    }
    for m in say {
        hero.documents.tell(m);
    }
    hero.documents.live_view_mut().fac = fac;
}
