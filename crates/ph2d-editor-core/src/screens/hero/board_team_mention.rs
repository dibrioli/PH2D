//! ⭐ W23 B14 — **o `@` do compositor** (doc 17 §2.3; o Miro: *«type @ and choose a person»*): escrever
//! `@` (no início ou depois de um espaço) abre por baixo do campo a lista de quem se pode chamar
//! ([`ph2d_board_model::Team::people`] e «everyone»), filtrada pelo que se escreve a seguir; ↑/↓ andam,
//! `Enter` ou o clique escolhem. Escolher troca o `@…` pelo nome e CHAMA a pessoa (a menção é um
//! dado: [`ph2d_board_model::Who`]); escrever o nome à mão não chama ninguém (o Miro).

use super::super::super::HeroScreen;
use super::{Hit, me, node, register};
use crate::interaction::InteractiveState;
use crate::paint::{fill_rounded_rect, paint_text_elided, resolve};
use crate::zones::Rect;
use ph2d_board_model::{EVERYONE, Who};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, TypeToken};
use ph2d_vector::VectorScene;

/// Quantas pessoas a lista mostra (as que casam primeiro; escrever mais estreita-a).
pub(super) const ROWS: usize = 6;

/// O `@…` que o cursor está a escrever: o byte do `@` e o que vem depois dele até ao cursor. `None` =
/// o cursor não está numa menção (o `@` não abre uma palavra, ou há uma quebra de linha pelo meio).
#[must_use]
pub fn query(text: &str, caret: usize) -> Option<(usize, String)> {
    let before = text.get(..caret)?;
    let at = before.rfind('@')?;
    let q = &before[at + 1..];
    let opens = at == 0 || before[..at].ends_with(char::is_whitespace);
    (opens && !q.contains('\n') && !q.contains('@')).then(|| (at, q.to_owned()))
}

/// Quem casa com `q` (sem caixa, pelo início do nome ou de uma palavra dele), e «everyone».
#[must_use]
pub fn matches(people: &[String], q: &str) -> Vec<Who> {
    let q = q.to_lowercase();
    let hit = |name: &str| {
        let n = name.to_lowercase();
        n.starts_with(&q) || n.split_whitespace().any(|w| w.starts_with(&q))
    };
    let mut v: Vec<Who> = people
        .iter()
        .filter(|p| hit(p))
        .map(|p| Who::Person(p.clone()))
        .collect();
    if EVERYONE[1..].starts_with(&q) {
        v.push(Who::Everyone);
    }
    v.truncate(ROWS);
    v
}

/// O campo do compositor: texto e cursor (`focused` = só com o teclado nele — a pintura; o `Enter`
/// chega depois de o despacho largar o foco).
fn field(hero: &HeroScreen, focused: bool) -> Option<(String, usize)> {
    let id = crate::ids::BOARD_COMMENT_INPUT;
    if focused && hero.store.focus_id() != Some(id) {
        return None;
    }
    match hero.store.get(id) {
        Some(InteractiveState::TextInput { text, caret, .. }) => Some((text.clone(), *caret)),
        _ => None,
    }
}

/// A lista aberta agora: onde começa o `@` e quem ela mostra.
fn open_now(hero: &HeroScreen, focused: bool) -> Option<(usize, usize, Vec<Who>)> {
    let (text, caret) = field(hero, focused)?;
    let (at, q) = query(&text, caret)?;
    let present: Vec<String> = hero
        .documents
        .live_view
        .peers
        .iter()
        .filter(|p| !p.me)
        .map(|p| p.name.clone())
        .collect();
    let people = hero
        .documents
        .active_board()?
        .team
        .people(&me(hero), &present);
    let found = matches(&people, &q);
    (!found.is_empty()).then_some((at, caret, found))
}

/// A lista estava à vista na última pintura (o `Enter` do compositor escolhe, não publica): o `Enter`
/// faz o que se vê.
pub(super) fn is_open(hero: &HeroScreen) -> bool {
    hero.documents.live.team.mention_shown && open_now(hero, false).is_some()
}

/// Escolhe a pessoa `n` da lista: o `@…` vira o nome dela (e um espaço) e ela fica chamada.
pub(super) fn choose(hero: &mut HeroScreen, n: usize) {
    let Some((at, caret, found)) = open_now(hero, false) else {
        return;
    };
    let Some(who) = found.get(n).cloned() else {
        return;
    };
    let id = crate::ids::BOARD_COMMENT_INPUT;
    if let Some(InteractiveState::TextInput {
        text,
        caret: c,
        selection_anchor,
        ..
    }) = hero.store.get_mut(id)
    {
        let label = format!("{} ", who.label());
        text.replace_range(at..caret, &label);
        *c = at + label.len();
        *selection_anchor = None;
    }
    let team = &mut hero.documents.live.team;
    team.calling.push(who);
    team.mention_pick = 0;
    // O teclado continua no compositor (o `Enter` do despacho largou-o).
    hero.store.set_focus(Some(id));
    hero.store.mark_cancel_on_escape(id);
}

/// ⭐ Pinta a lista por baixo do campo `field` (ou por cima, se não cabe na área `area`) e arma as
/// setas do campo para ela. Sem lista, larga as setas.
pub(super) fn paint(
    hero: &mut HeroScreen,
    field_r: Rect,
    area: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
) {
    let id = crate::ids::BOARD_COMMENT_INPUT;
    let Some((_, _, found)) = open_now(hero, true) else {
        hero.store.arm_list_nav(None);
        let team = &mut hero.documents.live.team;
        (team.mention_pick, team.mention_shown) = (0, false);
        return;
    };
    hero.store.arm_list_nav(Some(id));
    hero.documents.live.team.mention_shown = true;
    let n = found.len();
    let steps = hero.store.take_list_steps(id);
    let pick = &mut hero.documents.live.team.mention_pick;
    *pick = (*pick as i64 + i64::from(steps)).rem_euclid(n as i64) as usize;
    let pick = *pick;
    let theme = hero.theme;
    let row_h = field_r.h;
    let h = row_h * n as f32;
    let below = field_r.y + field_r.h + Spacing::Xs.px();
    let y = if below + h <= area.y + area.h {
        below
    } else {
        field_r.y - Spacing::Xs.px() - h
    };
    let list = Rect::new(field_r.x, y, field_r.w, h);
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, list, radius, resolve(ColorToken::BgElev, theme));
    crate::paint::stroke_frame(
        scene,
        list,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        ph2d_tokens::StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    let font = TypeToken::Sm.px();
    let pad = Spacing::Md.px();
    for (i, who) in found.iter().enumerate() {
        let r = Rect::new(list.x, list.y + row_h * i as f32, list.w, row_h);
        let hid = node(Hit::Mention(i));
        register(&mut hero.store, hid);
        hero.hit_index.register(hid, r);
        if i == pick || hero.store.hot_id() == Some(hid) {
            let rr = crate::paint::frame_radius(theme, Radius::Sm.px());
            fill_rounded_rect(scene, r, rr, resolve(ColorToken::Bg2, theme));
        }
        let label = match who {
            Who::Person(name) => name.clone(),
            Who::Everyone => ph2d_i18n::tr("board.comment.everyone").to_owned(),
        };
        paint_text_elided(
            ts,
            scene,
            &label,
            r.x + pad,
            r.y + (r.h - font) / 2.0,
            font,
            r.w - 2.0 * pad,
            resolve(ColorToken::Text1, theme),
        );
    }
}

/// O `Enter` do compositor com a lista aberta: escolhe a realçada.
pub(super) fn submit(hero: &mut HeroScreen) {
    let pick = hero.documents.live.team.mention_pick;
    choose(hero, pick);
}
