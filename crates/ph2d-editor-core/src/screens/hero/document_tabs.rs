//! ⭐⭐ **AS ABAS DE DOCUMENTO** — `[Scene] [Board 1] [Board 2] [+]` na barra de menus (MiroClone).
//!
//! > *«Os quadros não devem aparecer na hierarquia mas devem aparecer como abas na barra superior
//! > do APP.»* — o dono, 2026-10-05 (`docs/MiroClone/02_plano.md` §1.1).
//!
//! Ocupam o vazio entre os títulos dos menus e as abas de LAYOUT ([`super::layout_tabs`], encostadas
//! à direita), com o idioma visual delas. Clicar troca o que a área central mostra; o `+` cria um
//! quadro e abre-o.
//!
//! ⚠️ **O id da aba de um quadro é DERIVADO do `BoardId`** (XOR com um salto, a lei do
//! `slot_tabs::tab_node_id`), e regista-se quando o quadro NASCE ou CHEGA de um ficheiro
//! ([`register_board`]): sem `InteractiveState` uma aba é pintada e nasce morta.

use super::HeroScreen;
use crate::icons::IconId;
use crate::ids;
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, paint_text_centered, rect_for_label, resolve};
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_board_model::{BoardId, BoardSet};
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, INLINE_ICON_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// ⛔ O salto que separa o id da aba de um QUADRO de tudo o resto (bijecção: não cria colisão nova).
const BOARD_TAB_SALT: u64 = 0xb0a2_d7ab_0000_0003;

/// O id do controlo da aba do quadro `id`.
#[must_use]
pub fn tab_node_id(id: BoardId) -> NodeId {
    NodeId(BOARD_TAB_SALT ^ id.0)
}

/// O que uma aba abre.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Scene,
    Board(BoardId),
    New,
}

impl Target {
    #[must_use]
    pub fn node(self) -> NodeId {
        match self {
            Target::Scene => ids::DOC_TAB_SCENE,
            Target::Board(id) => tab_node_id(id),
            Target::New => ids::DOC_TAB_NEW,
        }
    }
}

/// ⭐ **A ÚNICA porta da geometria** — o pintor, o hit e os testes leem daqui.
///
/// Da esquerda para a direita a partir de `start_x`, até `end_x`. As abas de quadro partilham o que
/// sobra depois de `Scene` e `+`; um nome que não cabe é encurtado pelo pintor. Devolve vazio se
/// nem `Scene` e `+` cabem — uma aba por cima de um menu abriria o menu errado. `editing` = a aba
/// que é o campo de renomear e o texto dele: ela mede pelo maior dos dois, para o campo crescer.
#[must_use]
pub fn tab_rects(
    bar: Rect,
    start_x: f32,
    end_x: f32,
    boards: &BoardSet,
    editing: Option<(BoardId, &str)>,
    text_system: &mut TextSystem,
) -> Vec<(Target, Rect)> {
    let font = TypeToken::Sm.px();
    let scene_w = rect_for_label(text_system.prefix_width(tr("board.tab.scene"), font));
    let new_w = bar.h;
    let free = end_x - start_x - scene_w - new_w;
    if free < 0.0 {
        return Vec::new();
    }
    let n = boards.boards().len();
    let share = if n == 0 { 0.0 } else { free / n as f32 };
    let mut out = Vec::with_capacity(n + 2);
    let mut x = start_x;
    let mut push = |t: Target, w: f32, out: &mut Vec<(Target, Rect)>| {
        out.push((t, Rect::new(x, bar.y, w, bar.h)));
        x += w;
    };
    push(Target::Scene, scene_w, &mut out);
    for b in boards.boards() {
        let mut text_w = text_system.prefix_width(&b.name, font);
        if let Some((_, buf)) = editing.filter(|(id, _)| *id == b.id) {
            text_w = text_w.max(text_system.prefix_width(buf, font));
        }
        let natural = rect_for_label(text_w);
        push(Target::Board(b.id), natural.min(share), &mut out);
    }
    push(Target::New, new_w, &mut out);
    out
}

/// Regista as duas abas fixas. Chamado pelo `pre_populate` do hero.
pub fn populate(store: &mut WidgetStore) {
    for id in [ids::DOC_TAB_SCENE, ids::DOC_TAB_NEW] {
        register(store, id);
    }
    super::document_tabs_menu::populate(store);
}

/// Regista a aba de um quadro que acabou de nascer ou de chegar de um ficheiro.
pub fn register_board(store: &mut WidgetStore, id: BoardId) {
    if store.button_state(tab_node_id(id)).is_none() {
        register(store, tab_node_id(id));
    }
}

fn register(store: &mut WidgetStore, id: NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

/// ⭐ **Carregar um projecto** traz os quadros dele: troca-os, regista as abas e volta à `Scene`.
pub fn load(hero: &mut HeroScreen, boards: BoardSet) {
    for b in boards.boards() {
        register_board(&mut hero.store, b.id);
    }
    hero.documents.replace(boards);
}

/// Pinta as abas e regista os alvos.
#[allow(clippy::too_many_arguments)]
pub fn paint(
    bar: Rect,
    start_x: f32,
    end_x: f32,
    hero_docs: &crate::documents::Documents,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let active = hero_docs.active().map_or(Target::Scene, Target::Board);
    let editing = hero_docs.renaming.zip(rename_buffer(store));
    let tabs = tab_rects(
        bar,
        start_x,
        end_x,
        hero_docs.boards(),
        editing,
        text_system,
    );
    let dragged = hero_docs.tab_drag.map(|d| Target::Board(d.board));
    for &(t, r) in &tabs {
        if let (Target::Board(b), Some((_, buf))) =
            (t, editing.filter(|(e, _)| Target::Board(*e) == t))
        {
            debug_assert_eq!(Some(b), hero_docs.renaming);
            paint_rename_field(r, buf, scene, text_system, theme, hit_index, store);
            continue;
        }
        let id = t.node();
        let is_on = t == active;
        let state = store.button_state(id).unwrap_or(ButtonState::Normal);
        let bg = if is_on {
            Some(ColorToken::AccentSoft)
        } else if Some(t) == dragged
            || matches!(
                state,
                ButtonState::Hovered | ButtonState::Focused | ButtonState::Pressed
            )
        {
            Some(ColorToken::BgElev)
        } else {
            None
        };
        if let Some(bg) = bg {
            let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
            fill_rounded_rect(scene, r, radius, resolve(bg, theme));
        }
        let fg = resolve(
            if is_on {
                ColorToken::Accent
            } else {
                ColorToken::Text2
            },
            theme,
        );
        match t {
            Target::New => {
                let s = INLINE_ICON_PX;
                let icon = Rect::new(r.x + (r.w - s) / 2.0, r.y + (r.h - s) / 2.0, s, s);
                paint_icon(scene, IconId::Plus, icon, fg, StrokeToken::Default.px());
            }
            Target::Scene => {
                paint_text_centered(
                    text_system,
                    scene,
                    tr("board.tab.scene"),
                    r,
                    TypeToken::Sm.px(),
                    fg,
                );
            }
            Target::Board(b) => {
                let name = hero_docs.boards().get(b).map_or("", |b| b.name.as_str());
                paint_text_centered(text_system, scene, name, r, TypeToken::Sm.px(), fg);
            }
        }
        hit_index.register(id, r);
    }
    if let Some(caret) = super::document_tabs_menu::drop_caret(hero_docs, bar, &tabs) {
        scene.fill_rect(
            crate::paint::rect_to_vello(caret),
            resolve(ColorToken::Accent, theme),
        );
    }
}

/// O texto do campo de renomear (o `store` é o dono do buffer enquanto se escreve).
fn rename_buffer(store: &WidgetStore) -> Option<&str> {
    match store.get(ids::DOC_TAB_RENAME_INPUT) {
        Some(InteractiveState::TextInput { text, .. }) => Some(text.as_str()),
        _ => None,
    }
}

/// A aba em modo renomear: o campo de texto no lugar dela, com o mesmo pintor dos outros campos.
fn paint_rename_field(
    r: Rect,
    buf: &str,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let id = ids::DOC_TAB_RENAME_INPUT;
    let Some(InteractiveState::TextInput {
        caret,
        selection_anchor,
        state,
        ..
    }) = store.get(id)
    else {
        return;
    };
    hit_index.register(id, r);
    let field = crate::widget::TextInput::new(id, "").visual((*state, store.hover_live(id)));
    let clip = crate::paint::rect_to_vello(r);
    scene.push_clip(&clip);
    crate::widget::paint_text_input_with_buffer(
        &field,
        Some(buf),
        Some(*caret),
        *selection_anchor,
        r,
        scene,
        text_system,
        theme,
    );
    scene.pop_layer();
}

/// O recuo entre os títulos dos menus e a primeira aba, e entre a última e as abas de layout.
#[must_use]
pub fn gap_px() -> f32 {
    Spacing::Lg.px()
}

/// ⭐ **Clicar numa aba troca o documento** — corre no pré-despacho (ids derivados não chegam a um
/// handler de chrome).
///
/// ⚠️ **O 2.º clique rápido chega como `DoubleClick`, não como `Click`** (medido no teste do clique
/// real): sem o aceitar, carregar duas vezes depressa no `+` criaria UM quadro só.
pub fn apply_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    if super::document_tabs_menu::apply_event(hero, event) {
        return true;
    }
    let (WidgetEvent::Click(id) | WidgetEvent::DoubleClick(id)) = event else {
        return false;
    };
    if id == ids::DOC_TAB_SCENE {
        hero.documents.activate(None);
        return true;
    }
    if id == ids::DOC_TAB_NEW {
        let n = hero.documents.boards().boards().len() + 1;
        let name = ph2d_i18n::tr_with("board.tab.default_name", &[("n", &n)]);
        let board = hero.documents.create(name);
        register_board(&mut hero.store, board);
        return true;
    }
    let hit = hero
        .documents
        .boards()
        .boards()
        .iter()
        .map(|b| b.id)
        .find(|b| tab_node_id(*b) == id);
    if let Some(board) = hit {
        hero.documents.activate(Some(board));
        // ⭐ O 2.º clique rápido numa aba de quadro renomeia-a no lugar (o idioma das folhas de
        // uma planilha); o 1.º já a abriu.
        if matches!(event, WidgetEvent::DoubleClick(_)) {
            super::document_tabs_menu::begin_rename(hero, board);
        }
        return true;
    }
    false
}

#[cfg(test)]
#[path = "document_tabs_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "document_tabs_menu_tests.rs"]
mod menu_tests;
