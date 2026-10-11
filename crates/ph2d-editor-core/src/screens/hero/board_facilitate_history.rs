//! ⭐ **O histórico das votações no painel** (W13, [ADR-0183]) — módulo filho de [`super`]: o separador
//! «Results» (o toggle «Show results on the board», a lista das sessões e a página), abrir uma sessão,
//! renomeá-la, apagá-la (o 2.º clique) e acabar uma votação de UM ecrã para o histórico. O que se muda
//! é a equipa do quadro (fora do desfazer); ao vivo viaja pela captura da equipa.
//!
//! [ADR-0183]: ../../../../../docs/architecture/decisions/0183-o-historico-das-votacoes-e-um-instantaneo-da-equipa.md

use ph2d_a11y::NodeId;
use ph2d_board_edit::{Editor, VoteSetup};
use ph2d_board_model::{Board, BoardId, Poll, PollId};
use ph2d_i18n::tr_with;

use super::super::super::HeroScreen;
use super::{ALL, FacButton, SALT};
use crate::documents::fac::{Fac, VotePanel};
use crate::ids::BOARD_POLL_NAME_INPUT;
use crate::interaction::WidgetEvent;

/// A fatia de ids das filas da lista, logo depois dos botões do painel. Não é um tecto de filas (a
/// página mostra as que cabem e as setas passam): é o espaço de ids que [`row_of`] reconhece, para não
/// roubar o clique de outro widget — nenhum ecrã mostra 1 024 filas de um botão.
const ROW_IDS: usize = 1024;

/// O id da fila `k` da página da lista.
#[must_use]
pub fn row_node(k: usize) -> NodeId {
    NodeId(SALT ^ (ALL.len() + 1 + k) as u64)
}

/// A fila de um id de [`row_node`].
pub(super) fn row_of(id: NodeId) -> Option<usize> {
    let i = usize::try_from((id.0 ^ SALT).checked_sub(1)?).ok()?;
    i.checked_sub(ALL.len()).filter(|k| *k < ROW_IDS)
}

/// O título de uma sessão: o nome que lhe deram, ou «Voting N» no idioma do ecrã.
#[must_use]
pub(crate) fn title(p: &Poll) -> String {
    if p.name.is_empty() {
        tr_with("board.vote.session_default", &[("n", &p.number)])
    } else {
        p.name.clone()
    }
}

/// V4 — acaba a votação de UM ecrã: a sessão vai para o histórico, passa a ser a mostrada, e o painel
/// abre-a. `false` = não havia votação.
pub(super) fn end_here(board: &mut Board, ed: &mut Editor, f: &mut Fac, by: &str) -> bool {
    let Some((totals, voters)) = ed.end_vote() else {
        return false;
    };
    let at = super::super::team::now_unix();
    let poll = board.team.add_poll(totals, by.trim(), voters, at);
    f.panels
        .insert(board.id, VotePanel::Results { poll, all: false });
    true
}

/// O painel depois de a votação da SALA acabar: a sessão que o anfitrião acabou de escrever (a
/// mostrada), senão a lista.
pub(super) fn after_room_end(board: &Board, f: &mut Fac) {
    let panel = match board.team.shown.or(board.team.polls.last().map(|p| p.id)) {
        Some(poll) => VotePanel::Results { poll, all: false },
        None => VotePanel::History,
    };
    f.panels.insert(board.id, panel);
}

/// As sessões da página da lista de `board`, da mais nova à mais velha: `(fila, sessão)`.
pub(super) fn page(f: &Fac, board: &Board) -> Vec<PollId> {
    let from = f.history_from.get(&board.id).copied().unwrap_or(0);
    board
        .team
        .polls_newest_first()
        .skip(from)
        .take(f.history_page.max(1))
        .map(|p| p.id)
        .collect()
}

/// O campo do nome: Enter (ou sair dele) grava, Esc larga.
pub(super) fn field_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    match event {
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id) if id == BOARD_POLL_NAME_INPUT => {
            let name = super::super::field_text(&hero.store, BOARD_POLL_NAME_INPUT);
            if let Some((bid, poll)) = hero.documents.live.fac.renaming_poll.take()
                && let Some(b) = hero.documents.active_board_mut().filter(|b| b.id == bid)
            {
                b.team.rename_poll(poll, &name);
            }
            true
        }
        WidgetEvent::Cancel(id) if id == BOARD_POLL_NAME_INPUT => {
            hero.documents.live.fac.renaming_poll = None;
            true
        }
        WidgetEvent::TextChanged(id) => id == BOARD_POLL_NAME_INPUT,
        _ => false,
    }
}

/// A fila `k` da página: abre a sessão (e, com o toggle ligado, o quadro passa a mostrá-la — a
/// figura do Miro: os selos são os da sessão aberta).
pub(super) fn row_click(hero: &mut HeroScreen, k: usize) {
    let Some((board, live)) = hero.documents.active_parts() else {
        return;
    };
    let f = &mut live.fac;
    let Some(poll) = page(f, board).get(k).copied() else {
        return;
    };
    f.panels
        .insert(board.id, VotePanel::Results { poll, all: false });
    if board.team.shown.is_some() {
        board.team.show_poll(Some(poll));
    }
}

/// Os botões do separador «Results» e da sessão aberta. `armed` = o lixo que estava armado antes
/// deste clique; o título renomeia só ao duplo clique (o idioma das versões). `false` = não é um
/// destes botões.
pub(super) fn click(
    hero: &mut HeroScreen,
    b: FacButton,
    armed: Option<(BoardId, PollId)>,
    double: bool,
) -> bool {
    let Some((board, live)) = hero.documents.active_parts() else {
        return false;
    };
    let (f, id) = (&mut live.fac, board.id);
    let open = match f.panels.get(&id) {
        Some(VotePanel::Results { poll, .. }) => Some(*poll),
        _ => None,
    };
    match b {
        FacButton::VoteTabNew => {
            f.panels.insert(id, VotePanel::Setup(VoteSetup::default()));
        }
        FacButton::VoteTabResults | FacButton::PollBack => {
            f.panels.insert(id, VotePanel::History);
        }
        FacButton::ShowOnBoard => {
            let next = if board.team.shown.is_some() {
                None
            } else {
                open.or_else(|| board.team.polls_newest_first().next().map(|p| p.id))
            };
            board.team.show_poll(next);
        }
        FacButton::PollRename => {
            let Some(poll) = open.filter(|_| double) else {
                return true;
            };
            let name = board.team.poll(poll).map(title).unwrap_or_default();
            f.renaming_poll = Some((id, poll));
            super::super::open_field(&mut hero.store, BOARD_POLL_NAME_INPUT, name);
        }
        FacButton::PollDelete => {
            let Some(poll) = open else {
                return true;
            };
            if armed != Some((id, poll)) {
                f.confirm_poll = Some((id, poll));
                return true;
            }
            board.team.remove_poll(poll);
            if f.renaming_poll == Some((id, poll)) {
                f.renaming_poll = None;
            }
            f.panels.insert(id, VotePanel::History);
        }
        FacButton::PollUp | FacButton::PollDown => {
            let n = board.team.polls.len();
            let step = f.history_page.max(1);
            let from = f.history_from.entry(id).or_default();
            *from = if b == FacButton::PollUp {
                from.saturating_sub(step)
            } else {
                (*from + step).min(n.saturating_sub(1))
            };
        }
        _ => return false,
    }
    true
}
