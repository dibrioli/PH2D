//! W12 (ADR-0182) — a facilitação numa SESSÃO AO VIVO (módulo filho de [`super`]): a votação entre
//! pessoas (a sessão do editor segue a rodada da sala; cada mudança do meu boletim vai ao anfitrião, e
//! os votos dos outros nunca chegam aqui) e o modo privado (o painel, e as notas escondidas de mim, que
//! não se escrevem nem se votam). A sala decide; o ecrã pede.

use std::time::Instant;

use super::super::super::HeroScreen;
use super::FacButton;
use crate::documents::fac::{PrivateSetup, VotePanel};
use crate::documents::live::{LiveFac, LiveRequest, LiveRole};
use ph2d_board_edit::{Notice, VoteSession};
use ph2d_board_model::BoardId;

/// O quadro activo, se é o da sala e estou nela.
pub(super) fn live_board(hero: &HeroScreen) -> Option<BoardId> {
    let id = hero.documents.active_board()?.id;
    let lv = hero.documents.live_view();
    let on = matches!(lv.role, LiveRole::Hosting { .. } | LiveRole::Guest { .. });
    (on && lv.board == Some(id)).then_some(id)
}

/// O painel do modo privado do quadro activo está aberto?
#[must_use]
pub fn private_open(hero: &HeroScreen) -> bool {
    hero.documents
        .active_board()
        .is_some_and(|b| hero.documents.live.fac.private.contains_key(&b.id))
}

/// O botão «Private» da barra: abre ou fecha o painel (fechá-lo NÃO desliga o modo).
pub fn toggle_private(hero: &mut HeroScreen) {
    let Some(id) = hero.documents.active_board().map(|b| b.id) else {
        return;
    };
    let p = &mut hero.documents.live.fac.private;
    if p.remove(&id).is_none() {
        p.insert(id, PrivateSetup::default());
    }
}

/// ⭐ Antes de desenhar: o que é escondido de mim chega ao editor (não escreve nem vota) e ao desenho
/// (sem o texto, com o olho); a sessão de voto do editor segue a rodada da sala; o meu boletim sai
/// quando muda.
pub(super) fn follow(hero: &mut HeroScreen) {
    let theme = hero.theme;
    let here = live_board(hero).is_some();
    // «I'm done» passa o ecrã à pessoa seguinte num ecrã; ao vivo só avisa a sala.
    let tip = if here {
        "board.livevote.done_tip"
    } else {
        "board.vote.done_tip"
    };
    hero.store
        .set_tooltip(super::fac_node(FacButton::VoteDone), ph2d_i18n::tr(tip));
    let fac = if here {
        hero.documents.live_view().fac.clone()
    } else {
        LiveFac::default()
    };
    let mut asks = Vec::new();
    let Some((board, live)) = hero.documents.active_parts() else {
        return;
    };
    let ed = super::super::super::board_view::editor(&mut live.editor, theme);
    ed.set_unread(fac.hidden_to_me.clone());
    live.render_cache.unread = fac.hidden_to_me;
    live.render_cache.mine_private = fac.mine_hidden;
    let (f, id) = (&mut live.fac, board.id);
    match &fac.vote {
        Some(v) if !v.ended => {
            // Uma votação nova da sala: o painel abre a todos (o Miro avisa todos ao começar).
            if f.live_round.get(&id) != Some(&v.round) {
                ed.put_vote(Some(VoteSession::live(v.setup, v.targets.clone())));
                f.live_round.insert(id, v.round);
                f.live_done.remove(&id);
                f.live_sent.remove(&id);
                f.vote_clock.remove(&id);
                f.panels.insert(id, VotePanel::Session);
            }
            if let Some(s) = ed.vote() {
                let now = (s.ballot().clone(), f.live_done.contains(&id));
                if f.live_sent.get(&id) != Some(&now) {
                    asks.push(LiveRequest::Ballot {
                        round: v.round,
                        votes: now.0.clone(),
                        done: now.1,
                    });
                    f.live_sent.insert(id, now);
                }
            }
        }
        // Acabou: os totais já chegaram ao quadro; a sessão sai e o painel mostra o resultado.
        Some(v) if f.live_round.get(&id) == Some(&v.round) && ed.vote().is_some() => {
            ed.put_vote(None);
            f.live_done.remove(&id);
            f.live_sent.remove(&id);
            f.panels.insert(id, VotePanel::Results { all: false });
        }
        // Saí da sala a meio: a sessão da sala não fica a fingir de votação deste ecrã.
        None if !here && f.live_round.remove(&id).is_some() => {
            ed.put_vote(None);
            f.live_done.remove(&id);
            f.live_sent.remove(&id);
            f.panels.remove(&id);
        }
        _ => {}
    }
    for a in asks {
        hero.documents.ask_live(a);
    }
}

/// ⭐ Os cliques que, ao vivo, são pedidos à sala (e os do painel do modo privado). `false` = não é
/// daqui (segue a votação de um ecrã).
pub(super) fn click(hero: &mut HeroScreen, b: FacButton) -> bool {
    let Some(id) = hero.documents.active_board().map(|b| b.id) else {
        return false;
    };
    let live = live_board(hero).is_some();
    let fac = &mut hero.documents.live.fac;
    let ask = match b {
        FacButton::PrivateClose => {
            fac.private.remove(&id);
            None
        }
        FacButton::PrivateAnonymous | FacButton::PrivateAnyoneEnd => {
            if let Some(s) = fac.private.get_mut(&id) {
                if b == FacButton::PrivateAnonymous {
                    s.anonymous = !s.anonymous;
                } else {
                    s.anyone_can_end = !s.anyone_can_end;
                }
            }
            None
        }
        FacButton::PrivateOn => fac.private.get(&id).filter(|_| live).map(|s| {
            LiveRequest::PrivateOn {
                anonymous: s.anonymous,
                anyone_can_end: s.anyone_can_end,
            }
        }),
        FacButton::PrivateAnyoneReveal => {
            let on = hero
                .documents
                .live_view()
                .fac
                .private
                .as_ref()
                .is_some_and(|p| p.anyone_can_reveal);
            Some(LiveRequest::AnyoneCanReveal(!on))
        }
        FacButton::PrivateRevealMine => Some(LiveRequest::RevealMine),
        FacButton::PrivateOff => Some(LiveRequest::PrivateOff),
        FacButton::VoteStart if live => return start_vote(hero, id),
        FacButton::VoteDone if live => {
            fac.live_done.insert(id);
            None
        }
        FacButton::VoteEnd if live => Some(LiveRequest::VoteEnd),
        // O relógio da sala é do anfitrião: o «+1 min» não se pinta ao vivo.
        FacButton::VoteAdd1 if live => None,
        _ => return false,
    };
    if let Some(a) = ask.filter(|_| live) {
        hero.documents.ask_live(a);
    }
    true
}

/// «Start voting» ao vivo: os alvos (a selecção, ou o quadro inteiro) vão à sala, que abre a votação
/// a todos; o painel passa à sessão quando ela chega.
fn start_vote(hero: &mut HeroScreen, id: BoardId) -> bool {
    let theme = hero.theme;
    let Some(VotePanel::Setup(setup)) = hero.documents.live.fac.panels.get(&id).copied() else {
        return true;
    };
    let Some((board, live)) = hero.documents.active_parts() else {
        return true;
    };
    let ed = super::super::super::board_view::editor(&mut live.editor, theme);
    let ask = VoteSession::new(&board.doc, ed.selection(), setup).map(|s| LiveRequest::VoteStart {
        setup,
        targets: s.targets().clone(),
    });
    match ask {
        Some(a) => hero.documents.ask_live(a),
        None => live.notice = Some((Notice::NothingToVote.key(), Instant::now())),
    }
    true
}
