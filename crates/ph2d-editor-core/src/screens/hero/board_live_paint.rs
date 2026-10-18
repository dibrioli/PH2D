//! O desenho do quadro ao vivo no canto (módulo filho de [`super`]): o botão «Live», a fila dos
//! avatares à esquerda dele, o selo «Following …» no topo, e o painel na coluna da equipa.

use super::super::super::super::HeroScreen;
use super::rows::{FIELDS, Row, rows};
use super::{Hit, LiveButton, ensure_fields, node, register};
use crate::documents::live::LiveRole;
use crate::icons::IconId;
use crate::paint::{
    fill_circle, fill_rounded_rect, paint_text_centered, paint_text_elided, resolve,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Affine, Circle, Stroke, VectorScene};

fn btn() -> f32 {
    super::super::super::facilitate::btn()
}

/// O botão «Live» do canto (de texto; aceso com o painel aberto).
pub fn paint_button(
    hero: &mut HeroScreen,
    id: NodeId,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
) {
    let on = hero.documents.live_view.panel.open;
    super::super::paint::text_button(
        scene,
        ts,
        &mut hero.hit_index,
        &hero.store,
        id,
        tr("board.live.button"),
        r,
        on,
        hero.theme,
    );
}

/// ⭐ Pinta o quadro ao vivo do canto: avatares, o selo de seguir, e o painel aberto.
pub fn paint(hero: &mut HeroScreen, area: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    if super::super::show::showing(hero) {
        return;
    }
    super::yield_to_team(hero);
    paint_badge(hero, area, scene, ts);
    if hero.documents.live_view.panel.open {
        paint_panel(hero, area, scene, ts);
    }
    // Os avatares DEPOIS do cartão: numa janela baixa (ou com muitos endereços) o cartão desce até à
    // fila deles, e o índice dá o clique ao último registado — antes o cartão engolia «seguir».
    paint_avatars(hero, area, scene, ts);
}

/// Um círculo na cor da pessoa com a inicial dela, com o contorno em `person-ink`.
fn avatar(scene: &mut VectorScene, ts: &mut TextSystem, r: Rect, name: &str, color: u8, t: Theme) {
    let (cx, cy, rad) = (r.x + r.w / 2.0, r.y + r.h / 2.0, r.w / 2.0);
    let ink = resolve(ColorToken::PersonInk, t);
    fill_circle(scene, cx, cy, rad, ink);
    let edge = StrokeToken::Thin.px();
    fill_circle(
        scene,
        cx,
        cy,
        rad - edge,
        resolve(ColorToken::person(color), t),
    );
    let initial: String = name
        .chars()
        .next()
        .into_iter()
        .flat_map(char::to_uppercase)
        .collect();
    paint_text_centered(ts, scene, &initial, r, TypeToken::Sm.px(), ink);
}

/// A fila dos avatares à esquerda do «Live» (só com sessão). Clicar no de outra pessoa segue-a.
fn paint_avatars(hero: &mut HeroScreen, area: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    let theme = hero.theme;
    hero.documents.live_view.panel.avatars.clear();
    let lv = &hero.documents.live_view;
    if lv.role == LiveRole::Off || lv.peers.is_empty() {
        return;
    }
    let nav = super::super::super::button_rects(super::super::super::corner(hero, area));
    let Some(live) = nav
        .iter()
        .find(|(b, _)| *b == super::super::super::NavButton::Live)
        .map(|(_, r)| *r)
    else {
        return;
    };
    let (s, g) = (live.h, ph2d_tokens::control_gap_px());
    // Cabem na metade esquerda da fila; o resto vira «+n».
    let room = (((live.x - area.x) / 2.0 / (s + g)).floor() as usize).max(1);
    let peers = lv.peers.clone();
    let following = lv.following;
    let shown = if peers.len() > room {
        room - 1
    } else {
        peers.len()
    };
    let mut rects = Vec::new();
    let mut x = live.x - Spacing::Md.px() - s;
    for _ in 0..shown + usize::from(shown < peers.len()) {
        rects.push(Rect::new(x, live.y, s, s));
        x -= s + g;
    }
    super::super::super::panel(scene, &rects, theme);
    for (i, p) in peers.iter().take(shown).enumerate() {
        let r = rects[i];
        avatar(scene, ts, r, &p.name, p.color, theme);
        hero.documents.live_view.panel.avatars.push(p.peer);
        if following == Some(p.peer) {
            let ring = Stroke::new(f64::from(StrokeToken::Thick.px()));
            let c = Circle::new(
                (f64::from(r.x + r.w / 2.0), f64::from(r.y + r.h / 2.0)),
                f64::from(r.w / 2.0 + Spacing::Xxs.px()),
            );
            let accent = resolve(ColorToken::Accent, theme);
            scene
                .inner_mut()
                .stroke(&ring, Affine::IDENTITY, accent, None, &c);
        }
        if p.me {
            continue;
        }
        let id = node(Hit::Avatar(i));
        register(&mut hero.store, id);
        // W12 — com os nomes anónimos não se segue ninguém (Miro PM: «Follow is no longer available»).
        let anon = hero
            .documents
            .live_view()
            .fac
            .private
            .as_ref()
            .is_some_and(|p| p.anonymous);
        let tip = if anon {
            tr("board.private.no_follow").to_owned()
        } else if following == Some(p.peer) {
            tr("board.live.unfollow_tip").to_owned()
        } else {
            tr_with("board.live.follow_tip", &[("name", &p.name)])
        };
        hero.store.set_tooltip(id, tip);
        hero.hit_index.register(id, r);
    }
    if shown < peers.len() {
        let r = rects[shown];
        let more = tr_with(
            "board.live.more",
            &[("n", &(peers.len() - shown).to_string())],
        );
        fill_circle(
            scene,
            r.x + r.w / 2.0,
            r.y + r.h / 2.0,
            r.w / 2.0,
            resolve(ColorToken::Bg3, theme),
        );
        let fg = resolve(ColorToken::Text1, theme);
        paint_text_centered(ts, scene, &more, r, TypeToken::Xs.px(), fg);
        let id = node(Hit::Button(LiveButton::Everyone));
        register(&mut hero.store, id);
        hero.store
            .set_tooltip(id, tr("board.live.everyone_tip").to_owned());
        hero.hit_index.register(id, r);
    }
    hero.documents.live_view.panel.avatars_shown = shown;
}

/// c21 — o selo «Following {name}» no topo ao centro (com o × que larga), e a moldura da área na
/// cor da pessoa (o do Excalidraw).
fn paint_badge(hero: &mut HeroScreen, area: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    let theme = hero.theme;
    let lv = &hero.documents.live_view;
    let Some(f) = lv.following else {
        return;
    };
    let Some((name, color)) = lv
        .peers
        .iter()
        .find(|p| p.peer == f)
        .map(|p| (p.name.clone(), p.color))
    else {
        return;
    };
    let fill = resolve(ColorToken::person(color), theme);
    let ink = resolve(ColorToken::PersonInk, theme);
    let edge = StrokeToken::Thick.px();
    let frame = Rect::new(
        area.x + edge / 2.0,
        area.y + edge / 2.0,
        area.w - edge,
        area.h - edge,
    );
    crate::paint::stroke_rect(scene, frame, edge, fill);
    let text = tr_with("board.live.following", &[("name", &name)]);
    let (s, font) = (btn(), TypeToken::Sm.px());
    let tw = crate::paint::rect_for_label(ts.prefix_width(&text, font));
    let w = (tw + s).min(area.w);
    // Duas filas abaixo do topo: a primeira é dos avisos, a segunda das frases da equipa.
    let y = area.y + Spacing::Md.px() + 2.0 * (Spacing::Xl2.px() + Spacing::Xs.px());
    let r = Rect::new(area.x + (area.w - w) / 2.0, y, w, s);
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, r, radius, fill);
    hero.hit_index
        .register(node(Hit::Button(LiveButton::Badge)), r);
    paint_text_centered(ts, scene, &text, Rect::new(r.x, r.y, r.w - s, s), font, ink);
    super::super::super::button_ink(
        scene,
        &mut hero.hit_index,
        &hero.store,
        node(Hit::Button(LiveButton::Unfollow)),
        IconId::Close,
        Rect::new(r.x + r.w - s, r.y, s, s),
        false,
        theme,
        ColorToken::PersonInk,
    );
}

/// A altura de uma fila (sem o vão).
fn row_h(row: &Row, ts: &mut TextSystem, w: f32) -> f32 {
    let s = btn();
    match row {
        Row::Text(t, font, _) => ts.layout(t, font.px(), w).height().max(s),
        Row::Code(_) => 2.0 * s,
        Row::End => 0.0,
        _ => s,
    }
}

/// Onde cada fila cai a partir de `(x, y)` com a largura `w`: o rect de cada fila, os cartões, e a
/// altura toda (com o vão da última). Dentro de um cartão as filas recuam a folga dele de cada lado;
/// o título fica por cima do cartão, no do painel (o modelo de `widget::section_cards`).
fn place<'a>(
    rows: impl IntoIterator<Item = &'a Row>,
    ts: &mut TextSystem,
    x: f32,
    y: f32,
    w: f32,
) -> (Vec<Rect>, Vec<Rect>, f32) {
    let (s, g, cp) = (
        btn(),
        ph2d_tokens::control_gap_px(),
        ph2d_tokens::card_pad_px(),
    );
    let mut at = Vec::new();
    let mut cards = Vec::new();
    let mut top: Option<f32> = None;
    let mut cy = y;
    for row in rows {
        match row {
            Row::Section(_) => {
                at.push(Rect::new(x, cy, w, s));
                top = Some(cy + s);
                cy += s + cp;
            }
            Row::End => {
                let t = top.take().unwrap_or(cy);
                let bottom = cy - g + cp;
                cards.push(Rect::new(x, t, w, bottom - t));
                at.push(Rect::new(x, bottom, w, 0.0));
                cy = bottom + g;
            }
            _ => {
                let inset = if top.is_some() { cp } else { 0.0 };
                let rw = w - 2.0 * inset;
                let rh = row_h(row, ts, rw);
                at.push(Rect::new(x + inset, cy, rw, rh));
                cy += rh + g;
            }
        }
    }
    (at, cards, cy - y)
}

/// ⭐ O painel na coluna da equipa: as filas fixas inteiras, e as listas (achados, pessoas) cortadas
/// ao que cabe por cima dos botões do canto.
fn paint_panel(hero: &mut HeroScreen, area: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    let theme = hero.theme;
    ensure_fields(&mut hero.store);
    let (x, y0, w) = super::super::panels::column(hero, area);
    let corner = super::super::super::corner(hero, area);
    let room = (super::super::panels::bottom(corner) - y0).max(0.0);
    let (s, pad, g) = (btn(), Spacing::Sm.px(), ph2d_tokens::control_gap_px());
    let inner = w - 2.0 * pad;
    let all = rows(hero);
    let (_, _, fixed_h) = place(all.iter().filter(|r| !r.is_list()), ts, x + pad, 0.0, inner);
    let fixed = pad * 2.0 + s + g + fixed_h;
    let lists = all.iter().filter(|r| r.is_list()).count();
    let mut fit = ((room - fixed) / (s + g)).floor().max(0.0) as usize;
    // A lista que não cabe acaba em «+n» (antes cortava-se calada); a fila dele sai do que cabe.
    if lists > fit {
        fit = fit.saturating_sub(1);
    }
    let mut seen = 0;
    let mut kept = Vec::with_capacity(all.len());
    for r in all {
        if r.is_list() {
            seen += 1;
            if seen > fit {
                if seen == fit + 1 {
                    let more = tr_with("board.live.more", &[("n", &(lists - fit).to_string())]);
                    kept.push(if matches!(r, Row::Person(..)) {
                        Row::Button(LiveButton::MorePeople, more, false)
                    } else {
                        Row::Text(more, TypeToken::Xs, ColorToken::Text3)
                    });
                }
                continue;
            }
        }
        kept.push(r);
    }
    let all = kept;
    hero.documents.live_view.panel.people_shown =
        all.iter().filter(|r| matches!(r, Row::Person(..))).count();
    let top = y0 + pad + s + g;
    let (at, cards, body) = place(&all, ts, x + pad, top, inner);
    let h = pad * 2.0 + s + g + body;
    let card = Rect::new(x, y0, w, h);
    super::super::panels::card(scene, card, theme);
    crate::screens::hero::board_nav::fit::cover(hero, card);
    hero.hit_index
        .register(node(Hit::Button(LiveButton::Surface)), card);
    // Os cartões de cada caminho de ligação (e o das pessoas), por baixo das filas deles.
    let tone = crate::widget::section_cards::CardDepth::Section.fill(theme);
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    for c in &cards {
        fill_rounded_rect(scene, *c, radius, tone);
    }
    hero.documents.live_view.panel.sections = cards;
    // O título e o ×.
    let y = y0 + pad;
    super::super::super::button(
        scene,
        &mut hero.hit_index,
        &hero.store,
        node(Hit::Button(LiveButton::Close)),
        IconId::Close,
        Rect::new(x + w - pad - s, y, s, s),
        false,
        theme,
    );
    let font = TypeToken::Sm.px();
    paint_text_elided(
        ts,
        scene,
        tr("board.live.button"),
        x + pad,
        super::super::text_y(y, s, font),
        font,
        inner - s - g,
        resolve(ColorToken::Text1, theme),
    );
    let mut found = Vec::new();
    for (row, r) in all.iter().zip(at) {
        paint_row(hero, row, r, scene, ts, &mut found);
    }
    hero.documents.live_view.panel.found = found;
}

fn paint_row(
    hero: &mut HeroScreen,
    row: &Row,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    found: &mut Vec<String>,
) {
    let theme = hero.theme;
    let s = btn();
    let button = |hero: &mut HeroScreen,
                  scene: &mut VectorScene,
                  ts: &mut TextSystem,
                  id,
                  label: &str,
                  on| {
        super::super::paint::text_button(
            scene,
            ts,
            &mut hero.hit_index,
            &hero.store,
            id,
            label,
            r,
            on,
            theme,
        );
    };
    match row {
        Row::Section(sec) => {
            let font = TypeToken::Sm.px();
            paint_text_elided(
                ts,
                scene,
                tr(sec.title_key()),
                r.x,
                super::super::text_y(r.y, s, font),
                font,
                r.w,
                resolve(ColorToken::Text1, theme),
            );
        }
        Row::End => {}
        Row::Text(t, font, ink) => {
            crate::paint::paint_text_block(
                ts,
                scene,
                t,
                r.x,
                r.y,
                font.px(),
                r.w,
                resolve(*ink, theme),
            );
        }
        Row::Code(code) => {
            fill_rounded_rect(
                scene,
                r,
                crate::paint::frame_radius(theme, Radius::Md.px()),
                resolve(ColorToken::Bg2, theme),
            );
            let fg = resolve(ColorToken::Text1, theme);
            paint_text_centered(ts, scene, code, r, TypeToken::Xl2.px(), fg);
        }
        Row::Button(b, label, main) => button(hero, scene, ts, node(Hit::Button(*b)), label, *main),
        Row::Field(key, id) => {
            let font = TypeToken::Xs.px();
            // A coluna dos rótulos é a do MAIS LARGO dos campos do painel (todos alinham): a 40 %
            // fixa cortava «Address (optional)» (foto b0 da cena 12). Nunca mais de 60 % da fila.
            let widest =
                crate::paint::label_column_width(ts, font, FIELDS.iter().map(|(k, _)| tr(k)));
            let lw = (widest + Spacing::Sm.px()).min(r.w * 0.6); // LITERAL-PX-OK: tecto de 60 % da fila
            paint_text_elided(
                ts,
                scene,
                tr(key),
                r.x,
                super::super::text_y(r.y, s, font),
                font,
                lw - Spacing::Xs.px(),
                resolve(ColorToken::Text2, theme),
            );
            let f = Rect::new(r.x + lw, r.y, r.w - lw, s);
            super::super::super::paint_field(
                *id,
                f,
                scene,
                ts,
                theme,
                &mut hero.hit_index,
                &hero.store,
            );
        }
        Row::Found(i, line) => {
            let Some(addr) = hero
                .documents
                .live_view
                .found
                .get(*i)
                .map(|f| f.addr.clone())
            else {
                return;
            };
            let id = node(Hit::Found(found.len()));
            found.push(addr);
            register(&mut hero.store, id);
            hero.store.set_tooltip(id, tr("board.live.found_tip"));
            button(hero, scene, ts, id, line, false);
        }
        Row::Person(name, color) => {
            let (dot, text_x) = (Spacing::Xs.px(), Spacing::Xl.px());
            fill_circle(
                scene,
                r.x + Spacing::Sm.px(),
                r.y + s / 2.0,
                dot,
                resolve(ColorToken::person(*color), theme),
            );
            let font = TypeToken::Sm.px();
            paint_text_elided(
                ts,
                scene,
                name,
                r.x + text_x,
                super::super::text_y(r.y, s, font),
                font,
                r.w - text_x,
                resolve(ColorToken::Text1, theme),
            );
        }
    }
}
