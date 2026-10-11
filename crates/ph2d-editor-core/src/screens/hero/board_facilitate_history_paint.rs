//! O desenho do histórico das votações (W13, ADR-0183) — módulo filho do desenho do painel: o
//! separador «Results» (o toggle e a lista) e uma sessão aberta (o título que se renomeia, quando e
//! quem, as linhas do resultado). As linhas nunca pintam o texto de uma nota que não leio (ADR-0182).

use ph2d_board_model::{BoardId, Element, ElementId};
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

use super::super::super::super::HeroScreen;
use super::super::history::{row_node, title};
use super::super::{FacButton, btn};
use super::{Btn, Face, buttons, card};
use crate::documents::fac::VotePanel;
use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text, paint_text_centered, paint_text_elided, resolve};
use crate::widget::ButtonState;
use crate::zones::Rect;

/// O que o histórico mostra (lido do quadro antes de pintar).
pub(super) enum HistView {
    /// O separador «Results»: o toggle, a página da lista (`(título, quando)`) e as setas.
    List {
        shown_on: bool,
        rows: Vec<(String, String)>,
        up: bool,
        down: bool,
    },
    /// Uma sessão aberta. `hidden` = as linhas que «See all» não mostra (não cabem na área).
    Session {
        title: String,
        sub: String,
        rows: Vec<(String, u32)>,
        all: bool,
        more: bool,
        hidden: usize,
        renaming: bool,
        armed: bool,
        shown_on: bool,
    },
}

/// Os dois separadores do Miro na fila `(a, b)`: «New session» · «Results» (o aceso é o de agora).
pub(super) fn tabs(list: &mut Vec<Btn>, a: Rect, b: Rect, results: bool) {
    list.push((
        FacButton::VoteTabNew,
        a,
        Face::Text(tr("board.vote.tab_new").into()),
        !results,
    ));
    list.push((
        FacButton::VoteTabResults,
        b,
        Face::Text(tr("board.vote.results").into()),
        results,
    ));
}

/// O texto de uma linha do resultado: a 1.ª linha do texto da nota; o marcador se não a leio; e
/// «Deleted item» se já não está no quadro (o instantâneo guarda-a, ADR-0183 §2).
fn row_text(el: Option<&Element>, unread: bool) -> String {
    let Some(el) = el else {
        return tr("board.vote.deleted_row").to_owned();
    };
    if unread {
        return tr("board.vote.hidden_row").to_owned();
    }
    let t = el
        .shape()
        .map_or("", |s| s.text.as_str())
        .lines()
        .next()
        .unwrap_or("")
        .trim();
    if t.is_empty() {
        tr("board.vote.untitled").to_owned()
    } else {
        t.to_owned()
    }
}

/// ⭐ O que o painel do quadro `id` mostra do histórico (`fit` = as filas que cabem por baixo do
/// título e dos botões); regista as filas da página. `None` = o painel não é do histórico.
pub(super) fn view(hero: &mut HeroScreen, id: BoardId, fit: usize) -> Option<HistView> {
    let now = super::super::super::team::now_unix();
    let (board, live) = hero.documents.active_parts()?;
    let panel = live.fac.panels.get(&id).copied()?;
    let empty = std::collections::BTreeSet::new();
    let unread = live.editor.as_ref().map_or(&empty, |e| e.unread());
    let shown_on = board.team.shown.is_some();
    let f = &mut live.fac;
    match panel {
        VotePanel::History => {
            let n = board.team.polls.len();
            f.history_page = fit;
            let from = f.history_from.entry(id).or_default();
            *from = (*from).min(n.saturating_sub(1));
            let from = *from;
            let rows: Vec<(String, String)> = board
                .team
                .polls_newest_first()
                .skip(from)
                .take(fit)
                .map(|p| {
                    let when = if p.at > 0 {
                        super::super::super::team::ago(p.at, now)
                    } else {
                        String::new()
                    };
                    (title(p), when)
                })
                .collect();
            for k in 0..rows.len() {
                let rid = row_node(k);
                if hero.store.button_state(rid).is_none() {
                    hero.store.register(
                        rid,
                        InteractiveState::Button {
                            state: ButtonState::Normal,
                        },
                    );
                }
                hero.store.set_tooltip(rid, tr("board.vote.open_tip"));
            }
            Some(HistView::List {
                shown_on,
                up: from > 0,
                down: from + fit < n,
                rows,
            })
        }
        VotePanel::Results { poll, all } => {
            let Some(p) = board.team.poll(poll) else {
                // Apagada (aqui ou ao vivo): o painel volta à lista.
                f.panels.insert(id, VotePanel::History);
                return None;
            };
            let res = ph2d_board_edit::vote_results(&board.doc, p);
            // `fit` desconta o título, os botões e «…and n more»; a sessão tem mais duas filas fixas
            // (quando e quem, e o toggle).
            let n = if all {
                res.len().min(fit.saturating_sub(2).max(1))
            } else {
                res.len().min(3)
            };
            let line = |(e, v): &(ElementId, u32)| {
                (row_text(board.doc.get(*e), unread.contains(e)), *v)
            };
            let mut sub = Vec::new();
            if p.at > 0 {
                sub.push(super::super::super::team::ago(p.at, now));
            }
            if !p.by.is_empty() {
                sub.push(tr_with("board.vote.by", &[("name", &p.by)]));
            }
            Some(HistView::Session {
                title: title(p),
                sub: sub.join(" · "),
                rows: res.iter().take(n).map(line).collect(),
                all,
                more: res.len() > 3,
                hidden: if all { res.len() - n } else { 0 },
                renaming: f.renaming_poll == Some((id, poll)),
                armed: f.confirm_poll == Some((id, poll)),
                shown_on,
            })
        }
        _ => None,
    }
}

/// ⭐ Pinta o histórico em `at` (a largura da coluna); devolve a altura.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    hit: &mut HitIndex,
    store: &WidgetStore,
    theme: Theme,
    v: &HistView,
    at: Rect,
) -> f32 {
    let (s, pad, g) = (btn(), Spacing::Xs.px(), ph2d_tokens::control_gap_px());
    let rows = match v {
        HistView::List { rows, .. } => 3 + rows.len().max(1),
        HistView::Session { rows, hidden, .. } => 4 + rows.len().max(1) + usize::from(*hidden > 0),
    };
    let h = pad * 2.0 + rows as f32 * s + (rows - 1) as f32 * g;
    let r = Rect::new(at.x, at.y, at.w, h);
    card(scene, r, theme, ColorToken::BgElev);
    hit.register(super::fac_node(FacButton::Surface), r);
    let (text1, text2) = (
        resolve(ColorToken::Text1, theme),
        resolve(ColorToken::Text2, theme),
    );
    let row = |i: usize| r.y + pad + i as f32 * (s + g);
    let label_y = |i: usize| row(i) + (s - TypeToken::Sm.px()) / 2.0 - pad / 2.0;
    let x0 = r.x + Spacing::Sm.px();
    let right = r.x + r.w - pad;
    let wide = |i: usize, k: usize, n: usize| {
        let w = (r.w - 2.0 * pad - (n as f32 - 1.0) * g) / n as f32;
        Rect::new(r.x + pad + k as f32 * (w + g), row(i), w, s)
    };
    let icon = |b: FacButton, x: f32, i: IconId, on: bool| -> Btn {
        (b, Rect::new(x, row(0), s, s), Face::Icon(i), on)
    };
    let mut list: Vec<Btn> = vec![icon(FacButton::VoteClose, right - s, IconId::Close, false)];
    // O toggle «Show results on the board» na fila `i` (o do Miro: no separador e na sessão).
    let toggle = |list: &mut Vec<Btn>, scene: &mut VectorScene, ts: &mut TextSystem, i, on| {
        let w = r.w - 2.0 * s;
        let key = "board.vote.show_on_board";
        paint_text_elided(ts, scene, tr(key), x0, label_y(i), TypeToken::Sm.px(), w, text2);
        let face = Face::Icon(if on { IconId::Success } else { IconId::Circle });
        list.push((FacButton::ShowOnBoard, Rect::new(right - s, row(i), s, s), face, on));
    };
    match v {
        HistView::List {
            shown_on,
            rows,
            up,
            down,
        } => {
            let md = TypeToken::Md.px();
            paint_text(ts, scene, tr("board.vote.title"), x0, label_y(0), md, f32::INFINITY, text1);
            let mut x = right - s;
            for (b, i, on) in [(FacButton::PollDown, IconId::ChevronDown, *down), (FacButton::PollUp, IconId::ChevronUp, *up)] {
                // Uma seta que não mexeria não se pinta (controlo morto).
                if on {
                    x -= s + g;
                    list.push(icon(b, x, i, false));
                }
            }
            tabs(&mut list, wide(1, 0, 2), wide(1, 1, 2), true);
            toggle(&mut list, scene, ts, 2, *shown_on);
            if rows.is_empty() {
                let none = tr("board.vote.no_sessions");
                paint_text_elided(ts, scene, none, x0, label_y(3), TypeToken::Sm.px(), r.w, text2);
            }
            let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
            for (k, (name, when)) in rows.iter().enumerate() {
                let rr = wide(3 + k, 0, 1);
                let id = row_node(k);
                let hover = matches!(
                    store.button_state(id),
                    Some(ButtonState::Hovered | ButtonState::Pressed | ButtonState::Focused)
                );
                let bg = if hover { ColorToken::Bg3 } else { ColorToken::Bg2 };
                fill_rounded_rect(scene, rr, radius, resolve(bg, theme));
                let (sm, ty) = (TypeToken::Sm.px(), label_y(3 + k));
                // A data com a largura dela (a «2 d ago» saía «2 d a…»): o nome é que se corta.
                let when_w = crate::paint::rect_for_label(ts.prefix_width(when, sm)) + pad;
                let name_w = rr.w - when_w - 2.0 * pad;
                paint_text_elided(ts, scene, name, rr.x + pad, ty, sm, name_w, text1);
                let wr = Rect::new(rr.x + rr.w - when_w, rr.y, when_w - pad, s);
                paint_text_centered(ts, scene, when, wr, sm, text2);
                hit.register(id, rr);
            }
        }
        HistView::Session {
            title,
            sub,
            rows,
            all,
            more,
            hidden,
            renaming,
            armed,
            shown_on,
        } => {
            list.push(icon(FacButton::PollBack, r.x + pad, IconId::ChevronLeft, false));
            let del = right - 2.0 * s - g;
            list.push(icon(FacButton::PollDelete, del, IconId::Trash, *armed));
            // O título: duplo clique renomeia (o idioma das versões); a renomear, o campo ocupa-o.
            let tx = r.x + pad + s + g;
            let tr_ = Rect::new(tx, row(0), del - g - tx, s);
            if *renaming {
                let input = crate::ids::BOARD_POLL_NAME_INPUT;
                super::super::super::paint_field(input, tr_, scene, ts, theme, hit, store);
            } else {
                let md = TypeToken::Md.px();
                paint_text_elided(ts, scene, title, tr_.x, label_y(0), md, tr_.w, text1);
                hit.register(super::fac_node(FacButton::PollRename), tr_);
            }
            let sm = TypeToken::Sm.px();
            // O lixo armado diz-se aqui (a dica só aparece com o rato parado em cima).
            let (line, ink) = if *armed {
                (tr("board.vote.delete_confirm"), resolve(ColorToken::Danger, theme))
            } else {
                (sub.as_str(), text2)
            };
            paint_text_elided(ts, scene, line, x0, label_y(1), sm, r.w - 2.0 * pad, ink);
            toggle(&mut list, scene, ts, 2, *shown_on);
            let cw = 2.5 * s; // LITERAL-PX-OK: a coluna de «99 votes» em botões
            for (i, (text, n)) in rows.iter().enumerate() {
                let count = if *n == 1 {
                    tr("board.vote.one_vote").to_owned()
                } else {
                    tr_with("board.vote.n_votes", &[("n", n)])
                };
                let line = format!("{}. {text}", i + 1);
                let lw = r.w - cw - 2.0 * pad;
                paint_text_elided(ts, scene, &line, x0, label_y(i + 3), sm, lw, text1);
                let cr = Rect::new(right - cw, row(i + 3), cw, s);
                paint_text_centered(ts, scene, &count, cr, sm, text2);
            }
            if rows.is_empty() {
                let none = tr("board.vote.no_votes");
                paint_text_elided(ts, scene, none, x0, label_y(3), sm, r.w, text2);
            }
            if *hidden > 0 {
                let line = tr_with("board.vote.more_rows", &[("n", hidden)]);
                let y = label_y(rows.len() + 3);
                paint_text(ts, scene, &line, x0, y, sm, f32::INFINITY, text2);
            }
            let last = 3 + rows.len().max(1) + usize::from(*hidden > 0);
            let n = 1 + usize::from(*more);
            list.push((
                FacButton::VoteNew,
                wide(last, 0, n),
                Face::Text(tr("board.vote.new_short").into()),
                true,
            ));
            if *more {
                let key = if *all { "board.vote.top3" } else { "board.vote.see_all" };
                list.push((FacButton::VoteSeeAll, wide(last, 1, n), Face::Text(tr(key).into()), false));
            }
        }
    }
    buttons(scene, ts, hit, store, theme, list);
    h
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::*;
    use ph2d_board_model::{
        BoardOp, BoardSet, Element, Rgba, RichText, Shape, ShapeType, Style,
    };

    /// ⭐ As linhas de uma sessão nunca pintam o texto de uma nota que não leio (o marcador) nem
    /// fingem uma nota apagada (a linha diz «(deleted)»); a que leio mostra a 1.ª linha dela. A
    /// gravação da cena não vê o texto dos painéis: prova-se pela função que o decide.
    #[test]
    fn a_row_never_paints_a_hidden_or_a_deleted_note() {
        let mut set = BoardSet::default();
        let id = set.create("R".into());
        let b = set.get_mut(id).unwrap();
        let mut put = |b: &mut ph2d_board_model::Board, text: &str| {
            let el = Element::new_shape(
                b.doc.mint_id(),
                b.doc.z_on_top(),
                Shape {
                    kind: ShapeType::Sticky,
                    style: Style::new(None, None, Rgba([0, 0, 0, 255])),
                    text: RichText::plain(text),
                },
                [0.0, 0.0, 100.0, 100.0],
            );
            let id = el.id;
            let _ = BoardOp::Put(el).apply(&mut b.doc);
            id
        };
        let (read, secret, gone) = (put(b, "visível\n2.ª"), put(b, "segredo antigo"), put(b, "x"));
        let _ = BoardOp::Delete(gone).apply(&mut b.doc);
        let poll = b
            .team
            .add_poll(BTreeMap::from([(read, 3), (secret, 2), (gone, 1)]), "", 1, 0);
        let mut hero = HeroScreen::new(ph2d_a11y::NodeId(1));
        crate::screens::hero::document_tabs::load(&mut hero, set);
        hero.documents.activate(Some(id));
        let theme = hero.theme;
        let (_, live) = hero.documents.active_parts().unwrap();
        crate::screens::hero::board_view::editor(&mut live.editor, theme)
            .set_unread(BTreeSet::from([secret]));
        live.fac
            .panels
            .insert(id, VotePanel::Results { poll, all: true });
        let Some(HistView::Session { rows, .. }) = view(&mut hero, id, 10) else {
            panic!("a sessão aberta");
        };
        let rows: Vec<(&str, u32)> = rows.iter().map(|(t, n)| (t.as_str(), *n)).collect();
        assert_eq!(
            rows,
            [
                ("visível", 3),
                (tr("board.vote.hidden_row"), 2),
                (tr("board.vote.deleted_row"), 1)
            ]
        );
    }
}
