//! ⭐ W23 B15 — **o aviso do quadro com um botão**: a faixa do topo do quadro (a das recusas,
//! [`super::board_bar_organize::paint_notice`]) com uma frase e, quando há o que fazer, o botão
//! (o «Undo» de um «Move to board…»). Pintar e clicar leem o MESMO [`layout`]; o mais novo dos dois
//! avisos (recusa ou este) é o que se vê.

use super::HeroScreen;
use crate::documents::BoardLive;
use crate::documents::alert::{Action, Alert};
use crate::ids;
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text_centered, rect_for_label, resolve};
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_board_model::BoardId;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[cfg(test)]
thread_local! {
    /// O aviso do topo (este ou o da recusa) que a última pintura pôs no ecrã.
    pub(crate) static BANNER: std::cell::Cell<Option<Rect>> = const { std::cell::Cell::new(None) };
}

/// O aviso que se vê agora: vivo e mais novo que a última recusa.
pub(crate) fn shown(live: &BoardLive) -> Option<&Alert> {
    let a = live.alert.as_ref()?;
    let fresh = a.at.elapsed().as_secs_f32() < a.life_s();
    let newer = live.notice.is_none_or(|(_, t)| t <= a.at);
    (fresh && newer).then_some(a)
}

/// O botão ainda faz o que diz: os dois passos continuam no topo do desfazer de cada quadro.
fn can(hero: &HeroScreen, action: Action) -> bool {
    match action {
        Action::UndoMove {
            from,
            from_at,
            to,
            to_at,
        } => {
            let at = |b: BoardId| hero.documents.live.histories.get(&b)?.undo_at();
            at(from) == Some(from_at) && at(to) == Some(to_at)
        }
        Action::ShowThread { board, thread } => hero
            .documents
            .boards()
            .get(board)
            .is_some_and(|b| b.team.thread(thread).is_some()),
    }
}

/// Onde ficam a faixa e o botão, na área do quadro `area`, fora do chrome `keep` ([`banner_rect`]).
pub(crate) fn layout(
    text_system: &mut TextSystem,
    (area, keep): (Rect, &[Rect]),
    alert: &Alert,
    button: bool,
) -> (Rect, Option<Rect>) {
    let font = TypeToken::Sm.px();
    let gap = Spacing::Md.px();
    let text_w = rect_for_label(text_system.prefix_width(&alert.text, font));
    let button_w = alert
        .action
        .filter(|_| button)
        .map(|a| rect_for_label(text_system.prefix_width(tr(a.label()), font)));
    let r = banner_rect(area, keep, text_w + button_w.map_or(0.0, |b| gap + b));
    let b = button_w.map(|bw| Rect::new(r.x + r.w - bw, r.y, bw, r.h));
    (r, b)
}

/// ⭐ W25 — **onde fica um aviso do topo** de largura `w` (este e o da recusa): na altura mais alta que não
/// toca o chrome já pintado (`keep`: os painéis da direita, as gavetas) nem a barra curta, o mais perto do
/// meio da área à direita dela; sem nenhuma livre, no topo, por cima (antes ficava sempre no topo e tapava o
/// painel «Frames»).
pub(crate) fn banner_rect(area: Rect, keep: &[Rect], w: f32) -> Rect {
    let tools = super::board_bar::toolbar_rects(area);
    let left = tools.iter().map(|(_, r)| r.x + r.w).fold(area.x, f32::max) + Spacing::Md.px();
    let tools = super::float_bar::panel_of(tools.into_iter().map(|(_, r)| r));
    let keep: Vec<Rect> = keep.iter().copied().chain(tools).collect();
    let (h, top) = (Spacing::Xl2.px(), area.y + Spacing::Md.px());
    let w = w.min(area.x + area.w - left);
    let mut ys = super::float_bar::heights_beside(&keep, h);
    ys.retain(|&y| y > top);
    ys.sort_by(f32::total_cmp);
    ys.insert(0, top);
    let cx = (left + area.x + area.w) / 2.0;
    super::float_bar::rect_clear_of(area, &keep, (w, h), &ys, cx)
        .unwrap_or_else(|| Rect::new(cx - w / 2.0, top, w, h))
}

/// Regista o botão (focável desde o arranque).
pub(super) fn populate(store: &mut WidgetStore) {
    store.register(
        ids::BOARD_ALERT_ACTION,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

/// Pinta o aviso no topo da área do quadro. No-op sem aviso visível.
pub(super) fn paint(
    hero: &mut HeroScreen,
    area: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
) {
    #[cfg(test)]
    BANNER.with(|b| b.set(None));
    let Some(alert) = shown(&hero.documents.live).cloned() else {
        return;
    };
    let button = alert.action.is_some_and(|a| can(hero, a));
    let theme: Theme = hero.theme;
    let keep = super::board_nav::fit::painted(&hero.documents.live.nav);
    let (r, b) = layout(text_system, (area, &keep), &alert, button);
    super::board_nav::fit::float(&mut hero.documents.live.nav, r);
    #[cfg(test)]
    BANNER.with(|b| b.set(Some(r)));
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, r, radius, resolve(ColorToken::BgElev, theme));
    let font = TypeToken::Sm.px();
    let text_r = b.map_or(r, |b| Rect::new(r.x, r.y, b.x - r.x, r.h));
    paint_text_centered(
        text_system,
        scene,
        &alert.text,
        text_r,
        font,
        resolve(ColorToken::Text1, theme),
    );
    if let Some(b) = b {
        register(&mut hero.hit_index, b);
        if hero.store.hot_id() == Some(ids::BOARD_ALERT_ACTION) {
            let rr = crate::paint::frame_radius(theme, Radius::Sm.px());
            fill_rounded_rect(scene, b, rr, resolve(ColorToken::Bg2, theme));
        }
        let label = alert.action.map_or("", |a| tr(a.label()));
        paint_text_centered(
            text_system,
            scene,
            label,
            b,
            font,
            resolve(ColorToken::Accent, theme),
        );
    }
}

fn register(hit: &mut HitIndex, b: Rect) {
    hit.register(ids::BOARD_ALERT_ACTION, b);
}

/// O clique no botão. Corre no pré-despacho; `true` = consumido.
pub(super) fn apply_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let (WidgetEvent::Click(id) | WidgetEvent::DoubleClick(id)) = event else {
        return false;
    };
    if id != ids::BOARD_ALERT_ACTION {
        return false;
    }
    let Some(action) = shown(&hero.documents.live).and_then(|a| a.action) else {
        return true;
    };
    hero.documents.live.alert = None;
    if !can(hero, action) {
        return true;
    }
    match action {
        Action::UndoMove { from, to, .. } => {
            hero.documents.undo_on(to);
            hero.documents.undo_on(from);
            hero.documents.activate(Some(from));
        }
        Action::ShowThread { board, thread } => {
            super::board_nav::team::show_thread(hero, board, thread);
        }
    }
    true
}
