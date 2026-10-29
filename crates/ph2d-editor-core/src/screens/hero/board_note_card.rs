//! W11 do Kanban — **«Convert to card»**: a linha do menu de uma nota adesiva abre este modal, que
//! escolhe o Kanban e a lista ([`super::board_kanban_notes::choices`], primeiro as dos Kanbans postos no
//! Quadro); a linha clicada converte as notas seleccionadas ([`super::board_kanban_notes::notes_to_cards`],
//! um passo de desfazer do Quadro).
//!
//! ⚠️ A lista que se PINTA e a que o CLIQUE resolve saem da mesma [`visible`].

use super::HeroScreen;
use super::board_kanban_notes::{Choice, choices, notes_to_cards};
use super::context_menu_overlay::{ROW_H, pad_y};
use crate::ids;
use crate::interaction::{
    ContextMenuKind, ContextMenuRequest, InteractiveState, WidgetEvent, WidgetStore,
};
use crate::paint::{fill_rounded_rect, paint_text, resolve};
use crate::widget::{ButtonState, TextInput, TextInputState, paint_text_input_with_buffer};
use crate::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_model::Moment;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, TypeToken};
use ph2d_vector::VectorScene;

/// Abre o modal: a busca limpa e com o teclado.
pub(super) fn open(store: &mut WidgetStore) {
    store.register(
        ids::CTX_NOTE_CARD_SEARCH,
        InteractiveState::TextInput {
            state: TextInputState::Focused,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    store.set_focus(Some(ids::CTX_NOTE_CARD_SEARCH));
    store.mark_cancel_on_escape(ids::CTX_NOTE_CARD_SEARCH);
    store.open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::NotesToCard,
    });
}

fn is_open(store: &WidgetStore) -> bool {
    store
        .context_menu()
        .is_some_and(|m| m.kind == ContextMenuKind::NotesToCard)
}

fn query(store: &WidgetStore) -> &str {
    match store.get(ids::CTX_NOTE_CARD_SEARCH) {
        Some(InteractiveState::TextInput { text, .. }) => text.as_str(),
        _ => "",
    }
}

/// ⭐ As listas que o modal mostra — as que casam com a busca (no nome do Kanban ou da lista, sem
/// caixa), até [`ids::CTX_NOTE_CARD_ROWS`] — quantas casaram e ficaram de fora, e se há alguma.
#[must_use]
pub fn visible(hero: &HeroScreen) -> (Vec<Choice>, usize, bool) {
    let all = choices(hero);
    let any = !all.is_empty();
    let q = query(&hero.store).trim().to_lowercase();
    let hits: Vec<Choice> = all
        .into_iter()
        .filter(|c| {
            q.is_empty()
                || c.kanban_name.to_lowercase().contains(&q)
                || c.list_name.to_lowercase().contains(&q)
        })
        .collect();
    let cap = ids::CTX_NOTE_CARD_ROWS.len();
    let more = hits.len().saturating_sub(cap);
    (hits.into_iter().take(cap).collect(), more, any)
}

/// O instante de agora no relógio do Kanban (ms UTC).
fn now() -> Moment {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX));
    Moment(ms)
}

fn close(hero: &mut HeroScreen) {
    hero.store.close_context_menu();
    hero.store.consume_last_context_menu();
    hero.store.set_focus(None);
}

fn choose(hero: &mut HeroScreen, slot: usize) {
    let Some(c) = visible(hero).0.into_iter().nth(slot) else {
        return;
    };
    close(hero);
    notes_to_cards(hero, c.kanban, c.list, now());
}

/// As linhas do modal e a busca (`Enter` = a primeira lista, `Esc` = desistir). Corre no
/// pré-despacho. `true` = consumido.
pub(super) fn apply_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    if !is_open(&hero.store) {
        return false;
    }
    match event {
        WidgetEvent::Click(id) => {
            let Some(slot) = ids::CTX_NOTE_CARD_ROWS.iter().position(|r| *r == id) else {
                return false;
            };
            choose(hero, slot);
        }
        WidgetEvent::Submit(id) if id == ids::CTX_NOTE_CARD_SEARCH => choose(hero, 0),
        WidgetEvent::Cancel(id) if id == ids::CTX_NOTE_CARD_SEARCH => close(hero),
        _ => return false,
    }
    true
}

/// Regista a linha do menu e as do modal (focáveis desde o arranque).
pub(super) fn populate(store: &mut WidgetStore) {
    store.register(
        ids::CTX_NOTE_CARD_SEARCH,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    for id in [ids::CTX_MENU_BOARD_EL_TO_CARD]
        .into_iter()
        .chain(ids::CTX_NOTE_CARD_ROWS)
    {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// ⭐ **O modal** — centrado: o título, a busca e as listas («Kanban › lista»). No-op se fechado.
pub(super) fn paint(
    hero: &mut HeroScreen,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    viewport: Rect,
) {
    if !is_open(&hero.store) {
        return;
    }
    let (rows, more, any) = visible(hero);
    let theme = hero.theme;
    let store = &hero.store;
    let hit_index = &mut hero.hit_index;
    let (text, caret, anchor, ti_state) = match store.get(ids::CTX_NOTE_CARD_SEARCH) {
        Some(InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            state,
        }) => (text.as_str(), *caret, *selection_anchor, *state),
        _ => ("", 0, None, TextInputState::Normal),
    };
    let gap = Spacing::Xs.px();
    let list_rows = rows.len().max(1) + usize::from(more > 0);
    let w = super::context_menu_dialogs::DIALOG_W;
    let h = pad_y() * 2.0 + ROW_H * (2 + list_rows) as f32 + gap * 2.0; // LITERAL-PX-OK: 2 linhas fixas e 2 vãos (título · busca)
    let rect = Rect::new(
        (viewport.x + (viewport.w - w) * 0.5).max(viewport.x),
        (viewport.y + (viewport.h - h) * 0.5).max(viewport.y),
        w,
        h,
    );
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    crate::paint::stroke_frame(
        scene,
        rect,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        1.0,
        resolve(ColorToken::Border, theme),
    );
    let x = rect.x + Spacing::Xs.px();
    let inner_w = rect.w - Spacing::Xs.px() * 2.0;
    let pad_x = Spacing::Md.px();
    let font = TypeToken::Sm.px();
    let tw = inner_w - pad_x * 2.0;
    let ink = |t: ColorToken| resolve(t, theme);
    let line = |ts: &mut TextSystem, scene: &mut VectorScene, s: &str, y: f32, c: ColorToken| {
        paint_text(
            ts,
            scene,
            s,
            x + pad_x,
            y + (ROW_H - font) * 0.5,
            font,
            tw,
            ink(c),
        );
    };
    let mut y = rect.y + pad_y();
    line(
        text_system,
        scene,
        tr("kanban.links.to_card_title"),
        y,
        ColorToken::Text1,
    );
    y += ROW_H + gap;
    let search = Rect::new(x, y, inner_w, ROW_H);
    hit_index.register(ids::CTX_NOTE_CARD_SEARCH, search);
    let ti = TextInput::new(ids::CTX_NOTE_CARD_SEARCH, "")
        .placeholder(tr("kanban.links.to_card_search"))
        .visual((ti_state, store.hover_live(ids::CTX_NOTE_CARD_SEARCH)));
    paint_text_input_with_buffer(
        &ti,
        Some(text),
        Some(caret),
        anchor,
        search,
        scene,
        text_system,
        theme,
    );
    y += ROW_H + gap;
    for (i, c) in rows.iter().enumerate() {
        let id = ids::CTX_NOTE_CARD_ROWS[i];
        let r = Rect::new(x, y, inner_w, ROW_H);
        hit_index.register(id, r);
        if Some(id) == store.hot_id() {
            let rr = crate::paint::frame_radius(theme, Radius::Sm.px());
            fill_rounded_rect(scene, r, rr, ink(ColorToken::Bg2));
        }
        let label = tr_with(
            "kanban.links.to_card_row",
            &[("kanban", &c.kanban_name), ("list", &c.list_name)],
        );
        line(text_system, scene, &label, y, ColorToken::Text1);
        y += ROW_H;
    }
    if rows.is_empty() {
        let none = if any {
            tr("chrome.no_matches")
        } else {
            tr("kanban.links.to_card_empty")
        };
        line(text_system, scene, none, y, ColorToken::Text3);
        y += ROW_H;
    }
    if more > 0 {
        let n = more.to_string();
        let hint = tr_with("board.send.more", &[("count", &n)]);
        line(text_system, scene, &hint, y, ColorToken::Text3);
    }
}

#[cfg(test)]
#[path = "board_note_card_tests.rs"]
mod tests;
