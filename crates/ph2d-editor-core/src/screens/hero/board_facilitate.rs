//! ⭐ **A facilitação de um quadro** (MiroClone W7, `docs/MiroClone/05_facilitacao.md`): o cartão do
//! cronómetro e o painel da votação, no canto de cima à direita (o do Miro; à esquerda do painel de
//! Frames quando ele está aberto). Módulo filho de [`super`] (a navegação): a shell só compõe.
//!
//! As leis vivem no `ph2d-board-edit` (`Timer`, `VoteSession`); aqui, onde ficam, o que pintam e o que
//! cada clique faz. ⚠️ Um botão que não faria nada no estado de agora NÃO se pinta.

use std::time::{Duration, Instant};

use super::super::HeroScreen;
use crate::documents::fac::{Fac, VotePanel};
use crate::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_board_edit::{Timer, TimerRun, VOTES_MAX, VoteSetup};
use ph2d_board_model::BoardId;
use ph2d_tokens::Spacing;

#[path = "board_facilitate_paint.rs"]
mod paint;
pub use paint::paint;

#[path = "board_facilitate_live.rs"]
mod live;
pub use live::{private_open, toggle_private};

/// Um botão do cartão ou do painel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FacButton {
    TimerMinus,
    TimerPlus,
    TimerPlay,
    TimerPause,
    TimerStop,
    TimerAdd1,
    TimerAdd5,
    TimerSound,
    TimerMinimize,
    TimerExpand,
    TimerClose,
    TimerConfirmStop,
    TimerCancelStop,
    VotesMinus,
    VotesPlus,
    OnePerObject,
    MinutesMinus,
    MinutesPlus,
    VoteStart,
    VoteDone,
    VoteEnd,
    VoteAdd1,
    VoteSeeAll,
    VoteClear,
    VoteClose,
    /// V4 — no resultado: abrir uma votação NOVA (que, no fim, tira este do quadro).
    VoteNew,
    /// O fundo do cartão e do painel: toma o clique (um clique no mostrador não cai no quadro).
    Surface,
    /// W12 — o painel do modo privado: as duas opções de ligar, ligar, a opção de quem ligou,
    /// revelar as minhas, desligar para todos, e fechar o painel.
    PrivateAnonymous,
    PrivateAnyoneEnd,
    PrivateOn,
    PrivateAnyoneReveal,
    PrivateRevealMine,
    PrivateOff,
    PrivateClose,
}

const ALL: [FacButton; 34] = [
    FacButton::TimerMinus,
    FacButton::TimerPlus,
    FacButton::TimerPlay,
    FacButton::TimerPause,
    FacButton::TimerStop,
    FacButton::TimerAdd1,
    FacButton::TimerAdd5,
    FacButton::TimerSound,
    FacButton::TimerMinimize,
    FacButton::TimerExpand,
    FacButton::TimerClose,
    FacButton::TimerConfirmStop,
    FacButton::TimerCancelStop,
    FacButton::VotesMinus,
    FacButton::VotesPlus,
    FacButton::OnePerObject,
    FacButton::MinutesMinus,
    FacButton::MinutesPlus,
    FacButton::VoteStart,
    FacButton::VoteDone,
    FacButton::VoteEnd,
    FacButton::VoteAdd1,
    FacButton::VoteSeeAll,
    FacButton::VoteClear,
    FacButton::VoteClose,
    FacButton::VoteNew,
    FacButton::Surface,
    FacButton::PrivateAnonymous,
    FacButton::PrivateAnyoneEnd,
    FacButton::PrivateOn,
    FacButton::PrivateAnyoneReveal,
    FacButton::PrivateRevealMine,
    FacButton::PrivateOff,
    FacButton::PrivateClose,
];

/// ⛔ Bits 40–47 diferentes dos saltos da navegação (`0x4e61_1…`–`0x4e61_4…`): o idioma dela.
const SALT: u64 = 0x4e61_5000_0000_0000;

#[must_use]
pub fn fac_node(b: FacButton) -> NodeId {
    let i = ALL.iter().position(|x| *x == b).expect("na tabela") as u64;
    NodeId(SALT ^ (i + 1))
}

fn button_of(id: NodeId) -> Option<FacButton> {
    let i = (id.0 ^ SALT).checked_sub(1)?;
    ALL.get(usize::try_from(i).ok()?).copied()
}

/// A dica de cada botão.
fn tooltip_key(b: FacButton) -> &'static str {
    match b {
        FacButton::TimerMinus => "board.timer.minus",
        FacButton::TimerPlus => "board.timer.plus",
        FacButton::TimerPlay => "board.timer.start",
        FacButton::TimerPause => "board.timer.pause",
        FacButton::TimerStop => "board.timer.stop",
        FacButton::TimerAdd1 => "board.timer.add1_tip",
        FacButton::TimerAdd5 => "board.timer.add5_tip",
        FacButton::TimerSound => "board.timer.sound",
        FacButton::TimerMinimize => "board.timer.minimize",
        FacButton::TimerExpand => "board.timer.expand",
        FacButton::TimerClose => "board.timer.close",
        FacButton::TimerConfirmStop => "board.timer.stop_confirm",
        FacButton::TimerCancelStop => "board.timer.cancel",
        FacButton::VotesMinus | FacButton::VotesPlus => "board.vote.votes",
        FacButton::OnePerObject => "board.vote.one_per",
        FacButton::MinutesMinus | FacButton::MinutesPlus => "board.vote.minutes",
        FacButton::VoteStart => "board.vote.start",
        FacButton::VoteDone => "board.vote.done_tip",
        FacButton::VoteEnd => "board.vote.end",
        FacButton::VoteAdd1 => "board.timer.add1_tip",
        FacButton::VoteSeeAll => "board.vote.see_all",
        FacButton::VoteClear => "board.vote.clear",
        FacButton::VoteClose => "board.vote.close",
        FacButton::VoteNew => "board.vote.new",
        FacButton::Surface => "board.vote.surface",
        FacButton::PrivateAnonymous => "board.private.anonymous_tip",
        FacButton::PrivateAnyoneEnd => "board.private.anyone_end_tip",
        FacButton::PrivateOn => "board.private.turn_on",
        FacButton::PrivateAnyoneReveal => "board.private.anyone_reveal_tip",
        FacButton::PrivateRevealMine => "board.private.reveal_mine_tip",
        FacButton::PrivateOff => "board.private.turn_off_tip",
        FacButton::PrivateClose => "board.vote.close",
    }
}

/// Regista os botões (pintado ⇒ clicável). Chamado pelo `board_nav::populate`.
pub fn populate(store: &mut WidgetStore) {
    for b in ALL {
        let id = fac_node(b);
        if store.button_state(id).is_none() {
            store.register(
                id,
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
        store.set_tooltip(id, ph2d_i18n::tr(tooltip_key(b)));
    }
}

/// O lado de um botão do cartão e do painel (o das barras do quadro).
pub(crate) fn btn() -> f32 {
    Spacing::Xl2.px()
}

/// A largura do cartão e do painel, em botões.
const CARD_BTNS: f32 = 8.0; // LITERAL-PX-OK: largura em botões (cabem −, +, ▶ e os três do canto)

/// As filas que a busca ocupa no topo da área (o campo, as etiquetas e «n found»).
const SEARCH_ROWS: f32 = 3.0; // LITERAL-PX-OK: filas da busca, em botões

/// ⭐ F7 — a coluna da facilitação: o canto de cima à direita da área, à esquerda do painel de Frames
/// quando ele está aberto (nunca por cima dele), e por baixo da busca quando ela está aberta (a
/// auditoria W7 viu o × da busca por cima do cartão). `(x, y, largura)`.
#[must_use]
pub fn column(area: Rect, frames_open: bool, search_open: bool) -> (f32, f32, f32) {
    let w = btn() * CARD_BTNS;
    let mut right = area.x + area.w - Spacing::Md.px();
    if frames_open {
        right -= super::frames_panel_width() + Spacing::Md.px();
    }
    let mut top = area.y + Spacing::Md.px();
    if search_open {
        top += SEARCH_ROWS * (btn() + Spacing::Xs.px());
    }
    (right - w, top, w)
}

/// O quadro activo e a facilitação dele.
fn active(hero: &mut HeroScreen) -> Option<(BoardId, &mut Fac)> {
    let id = hero.documents.active_board()?.id;
    Some((id, &mut hero.documents.live.fac))
}

/// O cartão do cronómetro do quadro activo está aberto?
#[must_use]
pub fn timer_open(hero: &HeroScreen) -> bool {
    hero.documents
        .active_board()
        .is_some_and(|b| hero.documents.live.fac.timers.contains_key(&b.id))
}

/// O painel de votação do quadro activo está aberto?
#[must_use]
pub fn vote_open(hero: &HeroScreen) -> bool {
    hero.documents
        .active_board()
        .is_some_and(|b| hero.documents.live.fac.panels.contains_key(&b.id))
}

/// Há uma votação aberta no quadro activo?
#[must_use]
pub fn vote_in_session(hero: &HeroScreen) -> bool {
    hero.documents.active_board().is_some()
        && hero
            .documents
            .live
            .editor
            .as_ref()
            .is_some_and(|e| e.vote().is_some())
}

/// O botão «Timer» da barra curta: abre o cartão (armado) ou fecha-o (e com ele o relógio).
pub fn toggle_timer(hero: &mut HeroScreen) {
    if let Some((id, f)) = active(hero) {
        if f.timers.remove(&id).is_none() {
            f.timers.insert(id, Timer::default());
        }
        f.vote_clock.remove(&id);
    }
}

/// O botão «Voting» da barra curta: abre o painel (a sessão aberta, senão o resultado, senão a
/// abertura) ou fecha-o — fechar NÃO acaba uma sessão (fica à espera de «End for all»).
pub fn toggle_vote(hero: &mut HeroScreen) {
    let theme = hero.theme;
    let has_session = hero.documents.active_parts().is_some_and(|(_, live)| {
        super::super::board_view::editor(&mut live.editor, theme)
            .vote()
            .is_some()
    });
    let has_results = hero
        .documents
        .active_board()
        .is_some_and(|b| !ph2d_board_edit::vote_results(&b.doc).is_empty());
    if let Some((id, f)) = active(hero)
        && f.panels.remove(&id).is_none()
    {
        let p = if has_session {
            VotePanel::Session
        } else if has_results {
            VotePanel::Results { all: false }
        } else {
            VotePanel::Setup(VoteSetup::default())
        };
        f.panels.insert(id, p);
    }
}

/// ⭐ Uma vez por pintura, com QUALQUER aba activa (a Cena incluída — F1: trocar de aba não pára um
/// relógio): os cronómetros andam e o zero pede a campainha (F6). Os de um quadro apagado ou de um
/// projecto trocado já saíram por `Documents::remove`/`replace` (a porta única).
pub fn tick(hero: &mut HeroScreen, now: Instant) {
    let f = &mut hero.documents.live.fac;
    for t in f.timers.values_mut() {
        if t.tick(now) {
            f.chime = true;
        }
    }
}

/// ⭐ Antes de desenhar o quadro activo (os `+` e o esconder do resultado leem a sessão DELE — a
/// auditoria W7 viu um quadro com a sessão do anterior): a sessão de voto segue o SEU quadro (a aba
/// mudou ⇒ estaciona-se e volta a do activo), larga os alvos apagados ou escondidos (V2), e o zero do
/// relógio da votação acaba-a (V6) — nunca a meio de um gesto (o desfazer dele guardaria o antes).
pub fn follow(hero: &mut HeroScreen) {
    let theme = hero.theme;
    let Some((board, live)) = hero.documents.active_parts() else {
        return;
    };
    let f = &mut live.fac;
    let ed = super::super::board_view::editor(&mut live.editor, theme);
    if f.in_editor != Some(board.id) {
        if let (Some(old), Some(s)) = (f.in_editor, ed.take_vote()) {
            f.parked.insert(old, s);
        }
        ed.put_vote(f.parked.remove(&board.id));
        f.in_editor = Some(board.id);
    }
    ed.prune_vote(&board.doc);
    let done = f
        .timers
        .get(&board.id)
        .is_some_and(|t| matches!(t.run(), TimerRun::Done { .. }));
    if done && !ed.is_busy() && f.vote_clock.remove(&board.id).is_some() && ed.vote().is_some() {
        let history = live.histories.entry(board.id).or_default();
        ed.end_vote(&mut board.doc, history);
        f.panels.insert(board.id, VotePanel::Results { all: false });
    }
    live::follow(hero);
}

/// ⭐ Os cliques do cartão e do painel. Corre no pré-despacho (ids derivados).
pub fn apply_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let (WidgetEvent::Click(id) | WidgetEvent::DoubleClick(id)) = event else {
        return false;
    };
    let Some(b) = button_of(id) else {
        return false;
    };
    // W12 — numa sessão ao vivo, votar e o modo privado são pedidos à sala.
    if live::click(hero, b) {
        return true;
    }
    let now = Instant::now();
    let theme = hero.theme;
    let Some((board, live)) = hero.documents.active_parts() else {
        return true;
    };
    let f = &mut live.fac;
    if let Some(t) = f.timers.get_mut(&board.id) {
        match b {
            FacButton::TimerMinus => _ = t.step(false),
            FacButton::TimerPlus => _ = t.step(true),
            FacButton::TimerPlay => t.play(now),
            FacButton::TimerPause => t.pause(now),
            FacButton::TimerStop => t.confirm_stop = true,
            FacButton::TimerConfirmStop => {
                t.stop();
                // O ■ larga o relógio da votação: a sessão segue sem relógio (auditoria W7).
                f.vote_clock.remove(&board.id);
            }
            FacButton::TimerCancelStop => t.confirm_stop = false,
            FacButton::TimerAdd1 | FacButton::VoteAdd1 => _ = t.add(now, Duration::from_secs(60)),
            FacButton::TimerAdd5 => _ = t.add(now, Duration::from_secs(300)),
            FacButton::TimerSound => t.sound = !t.sound,
            FacButton::TimerMinimize => t.minimized = true,
            FacButton::TimerExpand => t.minimized = false,
            _ => {}
        }
    }
    if b == FacButton::TimerClose {
        f.timers.remove(&board.id);
        f.vote_clock.remove(&board.id);
        return true;
    }
    let ed = super::super::board_view::editor(&mut live.editor, theme);
    let history = live.histories.entry(board.id).or_default();
    let panel = f.panels.get(&board.id).copied();
    match (b, panel) {
        (FacButton::VotesMinus | FacButton::VotesPlus, Some(VotePanel::Setup(mut s))) => {
            let up = b == FacButton::VotesPlus;
            let next = if up { s.votes + 1 } else { s.votes - 1 };
            s.votes = next.clamp(1, VOTES_MAX); // CLAMP-OK: constantes, 1 ≤ VOTES_MAX (99)
            f.panels.insert(board.id, VotePanel::Setup(s));
        }
        (FacButton::OnePerObject, Some(VotePanel::Setup(mut s))) => {
            s.one_per_object = !s.one_per_object;
            f.panels.insert(board.id, VotePanel::Setup(s));
        }
        (FacButton::MinutesMinus | FacButton::MinutesPlus, Some(VotePanel::Setup(mut s))) => {
            // O passo dos minutos é o do cronómetro (F2), e abaixo de 1 é 0 = sem relógio.
            let up = b == FacButton::MinutesPlus;
            s.minutes = match (s.minutes, up) {
                (0, true) => 1,
                (0 | 1, false) => 0,
                (m, _) => ph2d_board_edit::step_minutes(m, up),
            };
            f.panels.insert(board.id, VotePanel::Setup(s));
        }
        (FacButton::VoteStart, Some(VotePanel::Setup(s))) => {
            if ed.start_vote(&board.doc, s) {
                f.panels.insert(board.id, VotePanel::Session);
                if s.minutes > 0 {
                    let mut t = Timer::with_minutes(s.minutes);
                    t.play(now);
                    f.timers.insert(board.id, t);
                    f.vote_clock.insert(board.id, true);
                }
            }
        }
        (FacButton::VoteDone, _) => _ = ed.next_voter(),
        (FacButton::VoteEnd, _) => {
            ed.end_vote(&mut board.doc, history);
            f.vote_clock.remove(&board.id);
            f.panels.insert(board.id, VotePanel::Results { all: false });
        }
        (FacButton::VoteSeeAll, Some(VotePanel::Results { all })) => {
            f.panels.insert(board.id, VotePanel::Results { all: !all });
        }
        (FacButton::VoteClear, _) => {
            ed.clear_votes(&mut board.doc, history);
            f.panels
                .insert(board.id, VotePanel::Setup(VoteSetup::default()));
        }
        (FacButton::VoteClose, _) => _ = f.panels.remove(&board.id),
        (FacButton::VoteNew, Some(VotePanel::Results { .. })) => {
            f.panels
                .insert(board.id, VotePanel::Setup(VoteSetup::default()));
        }
        _ => {}
    }
    if let Some(n) = ed.take_notice() {
        live.notice = Some((n.key(), now));
    }
    true
}
