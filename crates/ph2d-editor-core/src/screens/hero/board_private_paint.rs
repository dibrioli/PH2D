//! W12 (ADR-0182) — o painel do MODO PRIVADO (módulo filho do desenho da facilitação): o diálogo do
//! Miro (`facilitacao_miro_notas.md` §6) — as duas opções e «Turn on private mode for all»; ligado,
//! quem o ligou, quantas notas estão escondidas, a opção de quem ligou, «Reveal all my sticky notes»
//! e «Turn off private mode for all» (ou a contagem de 5 s). Fora de uma sessão ao vivo o painel diz
//! como lá chegar — um botão que não faria nada não se pinta.

use super::super::super::super::HeroScreen;
use super::{Btn, FacButton, Face, btn, buttons, card, fac_node};
use crate::documents::fac::PrivateSetup;
use crate::documents::live::LiveRole;
use crate::icons::IconId;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{paint_text, paint_text_elided, resolve};
use crate::zones::Rect;
use ph2d_board_model::BoardId;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// O que o painel mostra.
pub(super) enum PrivateView {
    /// Fora de uma sessão ao vivo.
    NoSession,
    /// Numa sessão, mas este não é o quadro da sala.
    OtherBoard,
    /// Desligado: as opções de ligar.
    Setup(PrivateSetup),
    On {
        /// «Fulano turned on private mode» (ou «You …»).
        by: String,
        by_me: bool,
        hidden: usize,
        anyone_reveal: bool,
        /// «Reveal all my sticky notes»: com a opção e notas minhas escondidas.
        reveal_mine: bool,
        may_end: bool,
        /// Segundos até revelar.
        countdown: Option<u64>,
    },
}

/// O painel do quadro `id`, se está aberto.
pub(super) fn view(hero: &HeroScreen, id: BoardId) -> Option<PrivateView> {
    let setup = *hero.documents.live.fac.private.get(&id)?;
    if super::super::fac_live::live_board(hero).is_none() {
        let role = &hero.documents.live_view().role;
        let on = matches!(role, LiveRole::Hosting { .. } | LiveRole::Guest { .. });
        return Some(if on {
            PrivateView::OtherBoard
        } else {
            PrivateView::NoSession
        });
    }
    let f = &hero.documents.live_view().fac;
    Some(match &f.private {
        None => PrivateView::Setup(setup),
        Some(p) => PrivateView::On {
            by: if p.by_me {
                tr("board.private.started_by_you").to_owned()
            } else {
                tr_with("board.private.started_by", &[("name", &p.by)])
            },
            by_me: p.by_me,
            hidden: f.hidden_to_me.len() + f.mine_hidden.len(),
            anyone_reveal: p.anyone_can_reveal,
            reveal_mine: p.anyone_can_reveal && !f.mine_hidden.is_empty(),
            may_end: p.may_end,
            countdown: p.reveal_in_ms.map(|ms| ms.div_ceil(1000)),
        },
    })
}

/// ⭐ Pinta o painel em `at` (a largura da coluna); devolve a altura.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    hit: &mut HitIndex,
    store: &WidgetStore,
    theme: Theme,
    v: &PrivateView,
    at: Rect,
) -> f32 {
    let (s, pad, g) = (btn(), Spacing::Xs.px(), ph2d_tokens::control_gap_px());
    let lines = lines(v);
    let h = height(v);
    let r = Rect::new(at.x, at.y, at.w, h);
    card(scene, r, theme, ColorToken::BgElev);
    hit.register(fac_node(FacButton::Surface), r);
    let (text1, text2) = (
        resolve(ColorToken::Text1, theme),
        resolve(ColorToken::Text2, theme),
    );
    let row = |i: usize| r.y + pad + i as f32 * (s + g);
    let label_y = |i: usize| row(i) + (s - TypeToken::Sm.px()) / 2.0 - pad / 2.0;
    let x0 = r.x + Spacing::Sm.px();
    let right = r.x + r.w - pad;
    let title = tr("board.private.title");
    paint_text(
        ts,
        scene,
        title,
        x0,
        label_y(0),
        TypeToken::Md.px(),
        f32::INFINITY,
        text1,
    );
    let mut list: Vec<Btn> = vec![(
        FacButton::PrivateClose,
        Rect::new(right - s, row(0), s, s),
        Face::Icon(IconId::Close),
        false,
    )];
    for (k, line) in lines.iter().enumerate() {
        let i = k + 1;
        match line {
            Line::Text(t) => {
                let w = r.w - 2.0 * pad;
                paint_text_elided(ts, scene, t, x0, label_y(i), TypeToken::Sm.px(), w, text2);
            }
            Line::Toggle(key, b, on) => {
                let w = r.w - 2.0 * s;
                paint_text_elided(
                    ts,
                    scene,
                    tr(key),
                    x0,
                    label_y(i),
                    TypeToken::Sm.px(),
                    w,
                    text2,
                );
                let icon = if *on { IconId::Success } else { IconId::Circle };
                list.push((
                    *b,
                    Rect::new(right - s, row(i), s, s),
                    Face::Icon(icon),
                    *on,
                ));
            }
            Line::Button(key, b, on) => {
                let rr = Rect::new(r.x + pad, row(i), r.w - 2.0 * pad, s);
                list.push((*b, rr, Face::Text(tr(key).into()), *on));
            }
        }
    }
    buttons(scene, ts, hit, store, theme, list);
    h
}

/// Uma linha do painel.
enum Line {
    Text(String),
    /// A frase (chave), o botão e se está ligado.
    Toggle(&'static str, FacButton, bool),
    /// Um botão largo (chave, botão, aceso).
    Button(&'static str, FacButton, bool),
}

/// As linhas do painel, pela ordem (a 0 é o título, que não está aqui).
fn lines(v: &PrivateView) -> Vec<Line> {
    let mut lines: Vec<Line> = Vec::new();
    match v {
        PrivateView::NoSession => {
            lines.push(Line::Text(tr("board.private.needs_live").to_owned()));
            lines.push(Line::Text(tr("board.private.needs_live_how").to_owned()));
        }
        PrivateView::OtherBoard => {
            lines.push(Line::Text(tr("board.private.other_board").to_owned()));
            lines.push(Line::Text(tr("board.private.other_board_how").to_owned()));
        }
        PrivateView::Setup(p) => {
            lines.push(Line::Text(tr("board.private.explain").to_owned()));
            lines.push(Line::Text(tr("board.private.explain_2").to_owned()));
            lines.push(Line::Toggle(
                "board.private.anonymous",
                FacButton::PrivateAnonymous,
                p.anonymous,
            ));
            lines.push(Line::Toggle(
                "board.private.anyone_end",
                FacButton::PrivateAnyoneEnd,
                p.anyone_can_end,
            ));
            lines.push(Line::Button(
                "board.private.turn_on",
                FacButton::PrivateOn,
                true,
            ));
        }
        PrivateView::On {
            by,
            by_me,
            hidden,
            anyone_reveal,
            reveal_mine,
            may_end,
            countdown,
        } => {
            lines.push(Line::Text(by.clone()));
            lines.push(Line::Text(tr_with(
                "board.private.hidden_count",
                &[("n", hidden)],
            )));
            if *by_me {
                lines.push(Line::Toggle(
                    "board.private.anyone_reveal",
                    FacButton::PrivateAnyoneReveal,
                    *anyone_reveal,
                ));
            }
            if *reveal_mine {
                lines.push(Line::Button(
                    "board.private.reveal_mine",
                    FacButton::PrivateRevealMine,
                    false,
                ));
            }
            match countdown {
                Some(n) => lines.push(Line::Text(tr_with("board.private.revealing", &[("s", n)]))),
                None if *may_end => {
                    lines.push(Line::Button(
                        "board.private.turn_off",
                        FacButton::PrivateOff,
                        true,
                    ));
                }
                None => {}
            }
        }
    }
    lines
}

/// A altura do painel (quem se põe por baixo, ou conta o que cabe, pergunta aqui).
pub(super) fn height(v: &PrivateView) -> f32 {
    let (s, pad, g) = (btn(), Spacing::Xs.px(), ph2d_tokens::control_gap_px());
    let rows = lines(v).len() + 1;
    pad * 2.0 + rows as f32 * s + (rows - 1) as f32 * g
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::live::LiveRole;
    use ph2d_a11y::NodeId;

    /// Fora de uma sessão o painel ensina a abrir o Live; numa sessão mas noutro quadro, diz que o
    /// modo é do quadro da sala (auditoria W12, interface F6).
    #[test]
    fn outside_the_room_board_the_panel_says_why() {
        let mut hero = HeroScreen::new(NodeId(1));
        let room = hero.documents.create("Sala".into());
        let other = hero.documents.create("Outro".into());
        hero.documents.activate(Some(other));
        hero.documents.live.fac.private.insert(other, PrivateSetup::default());
        assert!(matches!(view(&hero, other), Some(PrivateView::NoSession)));
        let v = hero.documents.live_view_mut();
        v.role = LiveRole::Hosting {
            code: "ABC-DEF".into(),
            addrs: Vec::new(),
        };
        v.board = Some(room);
        assert!(matches!(view(&hero, other), Some(PrivateView::OtherBoard)));
        hero.documents.activate(Some(room));
        hero.documents.live.fac.private.insert(room, PrivateSetup::default());
        assert!(matches!(view(&hero, room), Some(PrivateView::Setup(_))));
    }
}
