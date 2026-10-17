//! O desenho do cartão do cronómetro e do painel da votação (módulo filho de [`super`]).

use std::time::Instant;

use super::super::super::HeroScreen;
use super::{FacButton, btn, column, fac_node};
use crate::documents::fac::VotePanel;
use crate::icons::IconId;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{
    fill_rounded_rect, paint_text, paint_text_centered, paint_text_elided, resolve,
};
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_board_edit::{TIMER_MAX, Timer, TimerRun, VOTES_MAX, VoteSetup};
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[path = "board_private_paint.rs"]
mod private;

#[path = "board_facilitate_history_paint.rs"]
mod history;

/// O que um botão mostra.
#[derive(Clone, Debug)]
enum Face {
    Icon(IconId),
    Text(String),
}

/// Um botão a pintar: o que é, onde, o que mostra, e se está aceso.
type Btn = (FacButton, Rect, Face, bool);

/// O que o painel da votação mostra (lido do editor e do documento antes de pintar).
enum VoteView {
    Setup(VoteSetup),
    Session {
        left: u32,
        /// A 2.ª linha: «n voted» (um ecrã) ou «n of m done» (ao vivo).
        voted: String,
        /// O relógio a andar, já em texto.
        clock: Option<String>,
        /// Pinta «I'm done» / «End for all» (ao vivo: se ainda não disse / se posso acabar).
        done: bool,
        end: bool,
        add1: bool,
        /// W12 — só para quem iniciou, ao vivo: uma linha por pessoa (os votos usados, se acabou).
        people: Vec<String>,
    },
}

/// `MM:SS`.
fn clock(t: &Timer, now: Instant) -> String {
    let (m, s) = t.display(now);
    format!("{m:02}:{s:02}")
}

/// ⭐ Pinta o cartão e o painel do quadro activo (os relógios andam em [`super::tick`]).
pub fn paint(hero: &mut HeroScreen, area: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    hero.documents.live.fac.column = None;
    let now = Instant::now();
    let theme = hero.theme;
    let nav = &hero.documents.live.nav;
    let (x, y, w) = column(area, nav.frames_open, nav.search_open);
    let Some(board) = hero.documents.active_board() else {
        return;
    };
    let id = board.id;
    // As linhas que cabem por baixo do que já ocupa a coluna (o cronómetro, o modo privado): um
    // painel nunca sai do ecrã nem passa por cima do seguinte.
    let gap = ph2d_tokens::section_gap_px();
    let private_view = private::view(hero, id);
    let timer = hero.documents.live.fac.timers.get(&id).cloned();
    let taken = timer.as_ref().map_or(0.0, |t| timer_card_height(t) + gap)
        + private_view
            .as_ref()
            .map_or(0.0, |p| private::height(p) + gap);
    let step = btn() + ph2d_tokens::control_gap_px();
    // O chão da coluna: acima dos botões do canto e do minimapa aberto (a lista do histórico passava
    // por baixo deles — foto a9 da cena 14).
    let corner = super::super::corner(hero, area);
    let mut floor = super::super::button_rects(corner)
        .iter()
        .map(|(_, r)| r.y)
        .fold(area.y + area.h, f32::min);
    if hero.documents.live.nav.minimap_open {
        floor = floor.min(super::super::minimap_rect(corner).y);
    }
    // Um painel de `fit` filas da lista tem `fit + 3` filas e a moldura do cartão (`2·pad − vão`).
    // As listas paginam até UMA fila; o que nem assim cabe (uma janela baixa) rola, abaixo.
    let frame = 2.0 * Spacing::Xs.px() - ph2d_tokens::control_gap_px();
    let fit = (((floor - gap - y - taken - frame) / step).floor() as usize)
        .saturating_sub(3)
        .max(1);
    let view = match hero.documents.live.fac.panels.get(&id).copied() {
        None => None,
        Some(VotePanel::Setup(s)) => Some(VoteView::Setup(s)),
        Some(VotePanel::Session) => {
            let fac = &hero.documents.live.fac;
            let room = super::fac_live::room_session(hero, id, fit);
            hero.documents
                .live
                .editor
                .as_ref()
                .and_then(|e| e.vote())
                .map(|s| match room {
                    // W12 — ao vivo: quantos acabaram, o relógio da sala, e (a quem iniciou) cada um.
                    Some(r) => VoteView::Session {
                        left: s.left(),
                        voted: r.voted,
                        clock: r.clock,
                        done: r.done,
                        end: r.end,
                        add1: false,
                        people: r.people,
                    },
                    None => {
                        // O relógio da votação só se mostra (e o «+1 min» só se pinta) a andar.
                        let on = fac.vote_clock.contains_key(&id)
                            && fac.timers.get(&id).is_some_and(|t| {
                                matches!(
                                    t.run(),
                                    TimerRun::Running { .. } | TimerRun::Paused { .. }
                                )
                            });
                        VoteView::Session {
                            left: s.left(),
                            voted: tr_with("board.vote.voted", &[("n", &s.voters())]),
                            clock: fac.timers.get(&id).filter(|_| on).map(|t| clock(t, now)),
                            done: true,
                            end: true,
                            add1: on,
                            people: Vec::new(),
                        }
                    }
                })
        }
        Some(VotePanel::Results { .. } | VotePanel::History) => None,
    };
    let hist = history::view(hero, id, fit);
    let heights = [
        timer.as_ref().map(timer_card_height),
        view.as_ref().map(vote_height),
        hist.as_ref().map(history::height),
        private_view.as_ref().map(private::height),
    ];
    let cards = heights.iter().flatten().count();
    if cards == 0 {
        return;
    }
    let content = heights.iter().flatten().sum::<f32>() + gap * (cards - 1) as f32;
    // O que não cabe acima do chão ROLA pela porta da rolagem (o desenho e o clique recortados pela
    // mesma banda); a barra toma a largura dela à esquerda, nunca por cima dos botões dos cartões.
    let visible = (floor - gap - y).max(0.0);
    let scrolls = crate::widget::scrollbar_is_needed(
        crate::widget::scroll_area::with_tail(content, visible),
        visible,
    );
    let bar = if scrolls {
        w - crate::widget::scrollbar_track_rect(Rect::new(0.0, 0.0, w, w)).x
    } else {
        0.0
    };
    let x = x - bar;
    let band = Rect::new(x, y, w + bar, visible);
    hero.documents.live.fac.column = Some((id, band));
    let (hit, store) = (&mut hero.hit_index, &hero.store);
    let key = super::scroll_key();
    let bar_id = crate::widget::BOARD_FACILITATE_SCROLLBAR_ID;
    let body = crate::widget::scroll_area::open_with(scene, hit, store, key, bar_id, band);
    let top = body.top();
    let mut y = top;
    if let Some(t) = timer {
        let at = Rect::new(x, y, w, 0.0);
        y += paint_timer(scene, ts, hit, store, theme, &t, now, at) + gap;
    }
    if let Some(v) = view {
        y += paint_vote(scene, ts, hit, store, theme, &v, Rect::new(x, y, w, 0.0)) + gap;
    }
    if let Some(v) = hist {
        y += history::paint(scene, ts, hit, store, theme, &v, Rect::new(x, y, w, 0.0)) + gap;
    }
    if let Some(p) = private_view {
        y += private::paint(scene, ts, hit, store, theme, &p, Rect::new(x, y, w, 0.0)) + gap;
    }
    debug_assert!(
        (y - gap - top - content).abs() < 0.5,
        "a altura somada antes de pintar é a pintada"
    );
    crate::widget::scroll_area::close_parts(body, scene, hit, store, content, theme)
        .publish(&mut hero.store);
}

/// O fundo de um cartão.
fn card(scene: &mut VectorScene, r: Rect, theme: Theme, token: ColorToken) {
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, r, radius, resolve(token, theme));
}

/// Pinta e regista os botões.
fn buttons(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    hit: &mut HitIndex,
    store: &WidgetStore,
    theme: Theme,
    list: Vec<Btn>,
) {
    for (b, r, face, on) in list {
        let id = fac_node(b);
        match face {
            Face::Icon(icon) => super::super::button(scene, hit, store, id, icon, r, on, theme),
            Face::Text(label) => {
                let hover = matches!(
                    store.button_state(id),
                    Some(ButtonState::Hovered | ButtonState::Pressed | ButtonState::Focused)
                );
                let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
                let bg = if on {
                    ColorToken::Accent
                } else if hover {
                    ColorToken::Bg3
                } else {
                    ColorToken::Bg2
                };
                fill_rounded_rect(scene, r, radius, resolve(bg, theme));
                let fg = resolve(
                    if on {
                        ColorToken::AccentFg
                    } else {
                        ColorToken::Text1
                    },
                    theme,
                );
                paint_text_centered(ts, scene, &label, r, TypeToken::Sm.px(), fg);
                hit.register(id, r);
            }
        }
    }
}

/// ⭐ F2–F7 — o cartão do cronómetro em `at` (a largura da coluna); devolve a altura.
#[allow(clippy::too_many_arguments)]
fn paint_timer(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    hit: &mut HitIndex,
    store: &WidgetStore,
    theme: Theme,
    t: &Timer,
    now: Instant,
    at: Rect,
) -> f32 {
    let (s, pad, g) = (btn(), Spacing::Xs.px(), ph2d_tokens::control_gap_px());
    let text = clock(t, now);
    if t.minimized {
        // F7 — a pílula com o resto: clicá-la volta ao cartão.
        let w = s * 3.0; // LITERAL-PX-OK: três botões de largo cabem `00:00` a `Sm`
        let r = Rect::new(at.x + at.w - w, at.y, w, s);
        let list = vec![(FacButton::TimerExpand, r, Face::Text(text), false)];
        buttons(scene, ts, hit, store, theme, list);
        return s;
    }
    let done = matches!(t.run(), TimerRun::Done { .. });
    let blink = t.blink_off(now);
    // F6 — no zero o cartão treme e acende-se; F4 — pausado, o mostrador pisca.
    let shake = if done && blink { pad } else { 0.0 };
    let h = timer_height();
    let r = Rect::new(at.x + shake, at.y, at.w, h);
    card(
        scene,
        r,
        theme,
        if done && !blink {
            ColorToken::AccentSoft
        } else {
            ColorToken::BgElev
        },
    );
    // O fundo toma o clique (registado ANTES dos botões, que ganham por cima dele).
    hit.register(fac_node(FacButton::Surface), r);
    let ink = if matches!(t.run(), TimerRun::Paused { .. }) && blink {
        ColorToken::Text3
    } else if done {
        ColorToken::Accent
    } else {
        ColorToken::Text1
    };
    paint_text(
        ts,
        scene,
        &text,
        r.x + Spacing::Sm.px(),
        r.y + pad,
        TypeToken::Xl2.px(),
        f32::INFINITY,
        resolve(ink, theme),
    );
    let mut list: Vec<Btn> = Vec::new();
    let corner = [
        (FacButton::TimerClose, IconId::Close),
        (FacButton::TimerMinimize, IconId::Minimize),
        (
            FacButton::TimerSound,
            if t.sound {
                IconId::Volume2
            } else {
                IconId::VolumeX
            },
        ),
    ];
    for (i, (b, icon)) in corner.into_iter().enumerate() {
        let bx = r.x + r.w - pad - (i as f32 + 1.0) * s - i as f32 * g;
        list.push((b, Rect::new(bx, r.y + pad, s, s), Face::Icon(icon), false));
    }
    let row_y = r.y + pad + 2.0 * s + g;
    let mut x = r.x + pad;
    let mut push = |b: FacButton, face: Face, width: f32, on: bool| {
        list.push((b, Rect::new(x, row_y, width, s), face, on));
        x += width + g;
    };
    let wide = s * 2.0; // LITERAL-PX-OK: os botões de texto (+1m, Cancel) têm dois botões de largo
    if t.confirm_stop {
        push(
            FacButton::TimerConfirmStop,
            Face::Text(tr("board.timer.stop_confirm").into()),
            wide * 2.0,
            true,
        );
        push(
            FacButton::TimerCancelStop,
            Face::Text(tr("board.timer.cancel").into()),
            wide * 1.5, // LITERAL-PX-OK: «Cancel» em três botões de largo
            false,
        );
    } else {
        match t.run() {
            TimerRun::Set => {
                // Nos limites (1 e 99 minutos) o botão que não mexeria não se pinta.
                if t.minutes() > 1 {
                    push(FacButton::TimerMinus, Face::Icon(IconId::Minus), s, false);
                }
                if u64::from(t.minutes()) < TIMER_MAX.as_secs() / 60 {
                    push(FacButton::TimerPlus, Face::Icon(IconId::Plus), s, false);
                }
                push(FacButton::TimerPlay, Face::Icon(IconId::Play), s, true);
            }
            TimerRun::Running { .. } | TimerRun::Paused { .. } => {
                let running = matches!(t.run(), TimerRun::Running { .. });
                if running {
                    push(FacButton::TimerPause, Face::Icon(IconId::Pause), s, true);
                } else {
                    push(FacButton::TimerPlay, Face::Icon(IconId::Play), s, true);
                }
                push(FacButton::TimerStop, Face::Icon(IconId::Stop), s, false);
                push(
                    FacButton::TimerAdd1,
                    Face::Text(tr("board.timer.add1").into()),
                    wide,
                    false,
                );
                push(
                    FacButton::TimerAdd5,
                    Face::Text(tr("board.timer.add5").into()),
                    wide,
                    false,
                );
            }
            TimerRun::Done { .. } => {
                push(FacButton::TimerPlay, Face::Icon(IconId::Play), s, true);
                push(FacButton::TimerStop, Face::Icon(IconId::Stop), s, false);
            }
        }
    }
    buttons(scene, ts, hit, store, theme, list);
    h
}

/// ⭐ V1–V4 — o painel da votação em `at` (a largura da coluna).
#[allow(clippy::too_many_arguments)]
fn paint_vote(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    hit: &mut HitIndex,
    store: &WidgetStore,
    theme: Theme,
    v: &VoteView,
    at: Rect,
) -> f32 {
    let (s, pad, g) = (btn(), Spacing::Xs.px(), ph2d_tokens::control_gap_px());
    let h = vote_height(v);
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
    let title = tr("board.vote.title");
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
        FacButton::VoteClose,
        Rect::new(right - s, row(0), s, s),
        Face::Icon(IconId::Close),
        false,
    )];
    let wide = |i: usize, k: usize, n: usize| {
        let w = (r.w - 2.0 * pad - (n as f32 - 1.0) * g) / n as f32;
        Rect::new(r.x + pad + k as f32 * (w + g), row(i), w, s)
    };
    match v {
        VoteView::Setup(setup) => {
            history::tabs(&mut list, wide(1, 0, 2), wide(1, 1, 2), false);
            // Até ao `−` do contador (3 botões e 2 vãos da borda) — «Votes per person» saía cortado.
            let label_w = right - 3.0 * s - 3.0 * g - x0; // LITERAL-PX-OK: o contador, 3 botões e 3 vãos
            paint_text_elided(
                ts,
                scene,
                tr("board.vote.votes"),
                x0,
                label_y(2),
                TypeToken::Sm.px(),
                label_w,
                text2,
            );
            let steps = (FacButton::VotesMinus, FacButton::VotesPlus);
            let ends = (setup.votes > 1, setup.votes < VOTES_MAX);
            let value = setup.votes.to_string();
            stepper(
                scene,
                ts,
                &mut list,
                right,
                row(2),
                steps,
                ends,
                &value,
                text1,
            );
            paint_text_elided(
                ts,
                scene,
                tr("board.vote.one_per"),
                x0,
                label_y(3),
                TypeToken::Sm.px(),
                r.w - 2.0 * s,
                text2,
            );
            let icon = if setup.one_per_object {
                IconId::Success
            } else {
                IconId::Circle
            };
            list.push((
                FacButton::OnePerObject,
                Rect::new(right - s, row(3), s, s),
                Face::Icon(icon),
                setup.one_per_object,
            ));
            paint_text_elided(
                ts,
                scene,
                tr("board.vote.minutes"),
                x0,
                label_y(4),
                TypeToken::Sm.px(),
                label_w,
                text2,
            );
            let m = if setup.minutes == 0 {
                tr("board.vote.no_clock").to_owned()
            } else {
                setup.minutes.to_string()
            };
            let steps = (FacButton::MinutesMinus, FacButton::MinutesPlus);
            let ends = (
                setup.minutes > 0,
                u64::from(setup.minutes) < TIMER_MAX.as_secs() / 60,
            );
            stepper(scene, ts, &mut list, right, row(4), steps, ends, &m, text1);
            list.push((
                FacButton::VoteStart,
                wide(5, 0, 1),
                Face::Text(tr("board.vote.start").into()),
                true,
            ));
        }
        VoteView::Session {
            left,
            voted,
            clock,
            done,
            end,
            add1,
            people,
        } => {
            let left_text = if *left == 1 {
                tr("board.vote.left_one").to_owned()
            } else {
                tr_with("board.vote.left", &[("n", left)])
            };
            paint_text(
                ts,
                scene,
                &left_text,
                x0,
                row(1),
                TypeToken::Lg.px(),
                f32::INFINITY,
                text1,
            );
            paint_text(
                ts,
                scene,
                voted,
                x0,
                label_y(2),
                TypeToken::Sm.px(),
                f32::INFINITY,
                text2,
            );
            if let Some(c) = clock {
                let cr = Rect::new(right - 2.0 * s, row(2), 2.0 * s, s);
                paint_text_centered(ts, scene, c, cr, TypeToken::Sm.px(), text1);
            }
            let both: Vec<(FacButton, &str, bool)> = [
                (*done, FacButton::VoteDone, "board.vote.done", false),
                (*end, FacButton::VoteEnd, "board.vote.end", true),
            ]
            .into_iter()
            .filter(|(show, ..)| *show)
            .map(|(_, b, key, on)| (b, key, on))
            .collect();
            if both.is_empty() {
                // Ao vivo, depois de «I'm done» e sem poder acabar: espera-se pelo fim.
                let wait = tr("board.livevote.waiting");
                let w = r.w - 2.0 * pad;
                paint_text_elided(
                    ts,
                    scene,
                    wait,
                    x0,
                    label_y(3),
                    TypeToken::Sm.px(),
                    w,
                    text2,
                );
            }
            for (k, (b, key, on)) in both.iter().enumerate() {
                list.push((*b, wide(3, k, both.len()), Face::Text(tr(key).into()), *on));
            }
            let mut i = 4;
            if *add1 {
                list.push((
                    FacButton::VoteAdd1,
                    wide(4, 0, 1),
                    Face::Text(tr("board.vote.add1").into()),
                    false,
                ));
                i += 1;
            }
            for p in people {
                let w = r.w - 2.0 * pad;
                paint_text_elided(ts, scene, p, x0, label_y(i), TypeToken::Sm.px(), w, text2);
                i += 1;
            }
        }
    }
    buttons(scene, ts, hit, store, theme, list);
    h
}

/// Um contador `[−] n [+]` encostado a `right` na linha de cima `y`.
#[allow(clippy::too_many_arguments)]
fn stepper(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    list: &mut Vec<Btn>,
    right: f32,
    y: f32,
    (minus, plus): (FacButton, FacButton),
    (can_down, can_up): (bool, bool),
    value: &str,
    ink: ph2d_vector::Color,
) {
    let (s, g) = (btn(), ph2d_tokens::control_gap_px());
    // Nos limites o botão que não mexeria não se pinta (controlo morto).
    if can_down {
        let x = right - 3.0 * s - 2.0 * g; // LITERAL-PX-OK: o `−` a três botões da borda
        list.push((
            minus,
            Rect::new(x, y, s, s),
            Face::Icon(IconId::Minus),
            false,
        ));
    }
    if can_up {
        list.push((
            plus,
            Rect::new(right - s, y, s, s),
            Face::Icon(IconId::Plus),
            false,
        ));
    }
    let mid = Rect::new(right - 2.0 * s - g, y, s, s);
    paint_text_centered(ts, scene, value, mid, TypeToken::Sm.px(), ink);
}

/// A altura do cartão do cronómetro aberto: o mostrador (dois botões) e a fila dos botões.
pub(super) fn timer_height() -> f32 {
    let (s, pad, g) = (btn(), Spacing::Xs.px(), ph2d_tokens::control_gap_px());
    pad + 2.0 * s + g + s + pad
}

/// A altura do cronómetro como ele se pinta: o cartão, ou a pílula (F7).
fn timer_card_height(t: &Timer) -> f32 {
    if t.minimized { btn() } else { timer_height() }
}

/// A altura do painel da votação.
fn vote_height(v: &VoteView) -> f32 {
    let (s, pad, g) = (btn(), Spacing::Xs.px(), ph2d_tokens::control_gap_px());
    let rows = match v {
        VoteView::Setup(_) => 6,
        VoteView::Session { add1, people, .. } => 4 + usize::from(*add1) + people.len(),
    };
    pad * 2.0 + rows as f32 * s + (rows - 1) as f32 * g
}
