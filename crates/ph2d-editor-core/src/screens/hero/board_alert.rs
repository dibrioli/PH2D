//! ⭐ W23 B15 — **o aviso do quadro com um botão**: a faixa do topo do quadro (a das recusas,
//! [`super::board_bar_organize::paint_notice`]) com uma frase e, quando há o que fazer, o botão
//! (o «Undo» de um «Move to board…»). Pintar e clicar leem o MESMO [`layout`]; o mais novo dos dois
//! avisos (recusa ou este) é o que se vê.

use super::HeroScreen;
use crate::documents::BoardLive;
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
use std::time::Instant;

/// Quanto vive um aviso COM botão, em segundos: o tecto do Material para um *snackbar* com acção
/// (*«4–10 seconds»*) — dá tempo de decidir, não só de ler; um texto longo vive o que a leitura pede.
pub const LIFE_S: f32 = 10.0;

/// O que o botão faz.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    /// Desfaz os dois passos de um «Move to board…» — só enquanto ambos são o topo do desfazer do
    /// quadro deles (`*_at` = a hora do passo, [`ph2d_board_model::History::undo_at`]).
    UndoMove {
        from: BoardId,
        from_at: u64,
        to: BoardId,
        to_at: u64,
    },
    /// W23 B14 — abre o fio de uma menção ou resposta (no quadro dele).
    ShowThread {
        board: BoardId,
        thread: ph2d_board_model::ThreadId,
    },
}

impl Action {
    fn label(self) -> &'static str {
        match self {
            Self::UndoMove { .. } => "board.send.undo",
            Self::ShowThread { .. } => "board.comment.show",
        }
    }
}

/// Um aviso do quadro.
#[derive(Clone, Debug)]
pub(crate) struct Alert {
    pub(crate) text: String,
    pub(crate) action: Option<Action>,
    pub(crate) at: Instant,
}

impl Alert {
    pub(crate) fn new(text: String, action: Option<Action>) -> Self {
        Self {
            text,
            action,
            at: Instant::now(),
        }
    }

    fn life_s(&self) -> f32 {
        LIFE_S.max(crate::toast_reading::reading_ttl_s(&self.text))
    }
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

/// Onde ficam a faixa e o botão, na área do quadro `area`.
pub(crate) fn layout(
    text_system: &mut TextSystem,
    area: Rect,
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
    let w = (text_w + button_w.map_or(0.0, |b| gap + b)).min(area.w);
    let h = Spacing::Xl2.px();
    let r = Rect::new(area.x + (area.w - w) / 2.0, area.y + Spacing::Md.px(), w, h);
    let b = button_w.map(|bw| Rect::new(r.x + r.w - bw, r.y, bw, h));
    (r, b)
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
    let Some(alert) = shown(&hero.documents.live).cloned() else {
        return;
    };
    let button = alert.action.is_some_and(|a| can(hero, a));
    let theme: Theme = hero.theme;
    let (r, b) = layout(text_system, area, &alert, button);
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
