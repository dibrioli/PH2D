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

/// W13 (ADR-0182 §8, pontas da W12) — com «Make names anonymous» nenhum nome de OUTRA pessoa chega ao
/// ecrã: a lista e os avatares do painel, quem acolhe, quem iniciou a votação e os votos usados (que
/// se leem da lista). Os avisos de entrada e saída usam [`someone`].
pub(super) fn hide_names(fac: &sync::Fac, peers: &mut [LivePeer]) {
    if anonymous(fac) {
        for p in peers.iter_mut().filter(|p| !p.me) {
            tr("board.private.anonymous_name").clone_into(&mut p.name);
        }
    }
}

/// O modo privado está ligado com «Make names anonymous».
pub(super) fn anonymous(fac: &sync::Fac) -> bool {
    fac.private.as_ref().is_some_and(|p| p.anonymous)
}

/// O nome de um aviso (entrou, saiu, trouxe todos): «Someone» com os nomes anónimos.
pub(super) fn someone(hero: &HeroScreen, name: String) -> String {
    let fac = &hero.documents.live_view().fac;
    if fac.private.as_ref().is_some_and(|p| p.anonymous) {
        tr("board.private.someone").to_owned()
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use ph2d_board_model::VoteSetup;

    use super::*;

    fn peer(peer: u64, name: &str, me: bool) -> LivePeer {
        LivePeer {
            peer,
            name: name.into(),
            color: 0,
            me,
        }
    }

    /// A vista da sala (auditoria W12, interface F4/F5): quem já saiu é «Someone» (nunca um nome vazio);
    /// «n of m done» conta só quem ESTÁ; as linhas de cada um só a quem iniciou; o meu escondido não
    /// é «escondido de mim».
    #[test]
    fn the_view_names_who_left_and_counts_only_who_is_here() {
        let fac = sync::Fac {
            private: Some(sync::Private {
                by: 9,
                anonymous: false,
                anyone_can_end: false,
                anyone_can_reveal: false,
                reveal_in_ms: None,
            }),
            hidden: BTreeSet::from([ElementId(1), ElementId(2)]),
            vote: Some(sync::VoteRoom {
                round: 1,
                by: 1,
                setup: VoteSetup::default(),
                targets: BTreeSet::new(),
                left_ms: None,
                done: BTreeSet::from([2, 9]),
                used: BTreeMap::from([(2, 1), (9, 3)]),
                ended: false,
            }),
        };
        let peers = [peer(1, "Ana", true), peer(2, "Bia", false)];
        let v = fac_view(&fac, 1, 1, BTreeSet::from([ElementId(2)]), &peers);
        let p = v.private.unwrap();
        assert_eq!(p.by, tr("board.private.someone"));
        assert!(p.may_end, "o anfitrião acaba o que quem saiu começou");
        let vote = v.vote.unwrap();
        assert_eq!(
            (vote.done, vote.people),
            (1, 2),
            "quem saiu não conta em «done»"
        );
        assert!(vote.by_me);
        assert!(
            vote.used
                .contains(&(tr("board.private.someone").to_owned(), 3, true))
        );
        assert_eq!(v.hidden_to_me, BTreeSet::from([ElementId(1)]));
        assert_eq!(v.mine_hidden, BTreeSet::from([ElementId(2)]));
    }

    /// ⭐ W13 (pontas da W12): com «Make names anonymous» a lista diz «Anonymous» por cada OUTRO (o
    /// meu nome fica), e por ela quem iniciou a votação e os votos usados também; sem a opção, nada
    /// muda. ⛔ Controlo: a mesma sala sem a opção mostra «Bia».
    #[test]
    fn anonymous_names_never_reach_the_screen() {
        let room = |anonymous| sync::Fac {
            private: Some(sync::Private {
                by: 1,
                anonymous,
                anyone_can_end: false,
                anyone_can_reveal: false,
                reveal_in_ms: None,
            }),
            hidden: BTreeSet::new(),
            vote: Some(sync::VoteRoom {
                round: 1,
                by: 2,
                setup: VoteSetup::default(),
                targets: BTreeSet::new(),
                left_ms: None,
                done: BTreeSet::new(),
                used: BTreeMap::from([(2, 2)]),
                ended: false,
            }),
        };
        let anon = tr("board.private.anonymous_name").to_owned();
        for (on, other) in [(true, anon.as_str()), (false, "Bia")] {
            let fac = room(on);
            let mut peers = [peer(1, "Ana", true), peer(2, "Bia", false)];
            hide_names(&fac, &mut peers);
            assert_eq!(
                (peers[0].name.as_str(), peers[1].name.as_str()),
                ("Ana", other)
            );
            let v = fac_view(&fac, 1, 1, BTreeSet::new(), &peers);
            let vote = v.vote.unwrap();
            assert_eq!(vote.by, other, "quem iniciou");
            assert_eq!(
                vote.used,
                vec![(other.to_owned(), 2, false)],
                "os votos usados"
            );
        }
    }
}
