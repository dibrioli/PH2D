//! ⭐ **O QUE SE FAZ A UMA ABA DE QUADRO** (MiroClone, W0b) — renomear no lugar, o menu do botão
//! direito (*Rename · Duplicate · Delete…*), a pergunta antes de apagar e o arrasto que reordena.
//!
//! [`super::document_tabs`] responde *«que abas há e onde»*; isto responde *«o que acontece quando o
//! artista pega numa»*.
//!
//! - **Renomear é NO LUGAR** (duplo-clique ou *Rename*): a aba vira o campo, com o nome
//!   seleccionado. `Enter` ou clicar fora grava; `Esc` desiste; um nome vazio não grava.
//! - ⚠️ **O botão direito e o arrasto entram ANTES do despacho** ([`pointer`], chamado pelo
//!   `HeroScreen::handle_pointer*`): o id de uma aba é derivado do `BoardId` e o despacho não sabe
//!   que é uma aba — o hero sabe, porque tem os quadros.
//! - **A largada é julgada contra a geometria do quadro ANTERIOR** (o `HitIndex` ainda não foi
//!   limpo), a lei de `slot_tabs_drag::resolve_tab_drop`. A marca que se pinta durante o arrasto
//!   sai da MESMA [`drop_slot`].

use super::HeroScreen;
use super::context_menu_overlay::{ROW_H, pad_y};
use super::document_tabs::{Target, register_board, tab_node_id};
use crate::documents::{Documents, TabDrag};
use crate::ids;
use crate::interaction::{
    ContextMenuKind, ContextMenuRequest, HitIndex, InteractiveState, TAB_DRAG_THRESHOLD_PX,
    WidgetEvent, WidgetStore,
};
use crate::paint::{fill_rounded_rect, paint_text, resolve};
use crate::widget::{Button, paint_button};
use crate::zones::Rect;
use ph2d_board_model::BoardId;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// O quadro cuja aba está sob `(x, y)`, pelo `HitIndex` do último quadro pintado.
fn board_at(hero: &HeroScreen, x: f32, y: f32) -> Option<BoardId> {
    let id = hero.hit_index.hit(x, y)?;
    hero.documents
        .boards()
        .boards()
        .iter()
        .map(|b| b.id)
        .find(|b| tab_node_id(*b) == id)
}

/// ⭐ **O ponteiro chega aqui antes do despacho.** `true` = consumido (o despacho não o vê).
///
/// Só o botão direito numa aba de quadro é consumido (abre o menu dela). O arrasto apenas OBSERVA:
/// o clique que troca de aba continua a ser do despacho, e um empurrão de poucos px ainda é clique.
pub fn pointer(hero: &mut HeroScreen, e: PointerEvent) -> bool {
    match e.kind {
        PointerKind::Down => {
            let Some(board) = board_at(hero, e.x, e.y) else {
                return false;
            };
            match e.button {
                PointerButton::Secondary => {
                    hero.store.open_context_menu(ContextMenuRequest {
                        x: e.x,
                        y: e.y,
                        kind: ContextMenuKind::BoardTab { board: board.0 },
                    });
                    true
                }
                PointerButton::Primary => {
                    hero.documents.tab_drag = Some(TabDrag {
                        board,
                        start: (e.x, e.y),
                        cursor: (e.x, e.y),
                    });
                    false
                }
                _ => false,
            }
        }
        PointerKind::Move => {
            if let Some(d) = hero.documents.tab_drag.as_mut() {
                d.cursor = (e.x, e.y);
            }
            false
        }
        PointerKind::Up => {
            let Some(d) = hero.documents.tab_drag.take() else {
                return false;
            };
            let rects = |b: BoardId| hero.hit_index.rect_for(tab_node_id(b));
            let bar = hero.last_layout.map(|l| l.top_bar);
            if let Some(to) = bar.and_then(|bar| drop_index(&hero.documents, d, bar, rects)) {
                hero.documents.move_tab(d.board, to);
                hero.documents.activate(Some(d.board));
            }
            false
        }
    }
}

/// `true` quando o dedo já andou o bastante para isto deixar de ser um clique.
fn is_drag(d: TabDrag) -> bool {
    let (dx, dy) = (d.cursor.0 - d.start.0, d.cursor.1 - d.start.1);
    dx.hypot(dy) >= TAB_DRAG_THRESHOLD_PX
}

/// ⭐⭐ **A posição onde a aba arrastada cai**, entre as OUTRAS abas: a da primeira cujo meio está
/// à direita do dedo (a lei de `slot_tabs_drag::tab_row_drop`). Uma aba sem rect conta como à
/// esquerda — empurra a largada para mais tarde, nunca para antes de uma aba que o artista viu.
pub fn drop_slot(order: &[BoardId], x: f32, rect: impl Fn(BoardId) -> Option<Rect>) -> usize {
    order
        .iter()
        .position(|b| rect(*b).is_some_and(|r| x < r.x + r.w * 0.5))
        .unwrap_or(order.len())
}

/// A largada de um arrasto: `None` se não passou o limiar, ou se o dedo saiu da faixa da barra
/// (largar longe é desistir, como nas abas de painel).
fn drop_index(
    docs: &Documents,
    d: TabDrag,
    bar: Rect,
    rect: impl Fn(BoardId) -> Option<Rect>,
) -> Option<usize> {
    let (x, y) = d.cursor;
    if !is_drag(d) || y < bar.y - bar.h || y > bar.y + bar.h * 2.0 {
        return None;
    }
    Some(drop_slot(&others(docs, d.board), x, rect))
}

/// As abas de quadro na ordem, sem a arrastada.
fn others(docs: &Documents, dragged: BoardId) -> Vec<BoardId> {
    docs.boards()
        .boards()
        .iter()
        .map(|b| b.id)
        .filter(|b| *b != dragged)
        .collect()
}

/// A marca que diz onde a aba arrastada vai cair — `None` sem arrasto que caia. Usa as abas que
/// o pintor acabou de medir (`tabs`), pela MESMA [`drop_index`].
pub(super) fn drop_caret(docs: &Documents, bar: Rect, tabs: &[(Target, Rect)]) -> Option<Rect> {
    let d = docs.tab_drag?;
    let rect = |b: BoardId| {
        tabs.iter()
            .find(|(t, _)| *t == Target::Board(b))
            .map(|(_, r)| *r)
    };
    let at = drop_index(docs, d, bar, rect)?;
    let others = others(docs, d.board);
    let x = match (
        others.get(at),
        at.checked_sub(1).and_then(|i| others.get(i)),
    ) {
        (Some(next), _) => rect(*next)?.x,
        (None, Some(prev)) => rect(*prev).map(|r| r.x + r.w)?,
        (None, None) => rect(d.board)?.x,
    };
    // A espessura da marca das abas de painel (`slot_tabs_drag::DROP_CARET_PX`): o mesmo «é aqui».
    let w = ph2d_tokens::StrokeToken::Thick.px();
    Some(Rect::new(x - w * 0.5, bar.y, w, bar.h))
}

/// Põe a aba de `board` em modo renomear: o campo com o nome inteiro seleccionado e o teclado.
pub fn begin_rename(hero: &mut HeroScreen, board: BoardId) {
    let Some(name) = hero.documents.boards().get(board).map(|b| b.name.clone()) else {
        return;
    };
    let caret = name.len();
    hero.store.register(
        ids::DOC_TAB_RENAME_INPUT,
        InteractiveState::TextInput {
            state: crate::widget::TextInputState::Focused,
            text: name,
            caret,
            selection_anchor: Some(0),
        },
    );
    hero.store.set_focus(Some(ids::DOC_TAB_RENAME_INPUT));
    hero.store.mark_cancel_on_escape(ids::DOC_TAB_RENAME_INPUT);
    hero.documents.renaming = Some(board);
}

/// Grava o que está no campo (aparado; vazio não grava) e sai do modo renomear.
fn commit_rename(hero: &mut HeroScreen) {
    let Some(board) = hero.documents.renaming.take() else {
        return;
    };
    let text = match hero.store.get(ids::DOC_TAB_RENAME_INPUT) {
        Some(InteractiveState::TextInput { text, .. }) => text.trim().to_owned(),
        _ => String::new(),
    };
    if !text.is_empty() {
        hero.documents.rename(board, text);
    }
}

/// O quadro a que o menu aberto (ou o que o Down acabou de fechar) se refere.
fn menu_board(store: &WidgetStore) -> Option<BoardId> {
    match store
        .context_menu()
        .or_else(|| store.last_context_menu())?
        .kind
    {
        ContextMenuKind::BoardTab { board } | ContextMenuKind::ConfirmDeleteBoard { board } => {
            Some(BoardId(board))
        }
        _ => None,
    }
}

/// O campo de renomear, as linhas do menu e os botões da pergunta. Corre no pré-despacho.
pub(super) fn apply_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let id = match event {
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id) if id == ids::DOC_TAB_RENAME_INPUT => {
            commit_rename(hero);
            return true;
        }
        // ⚠️ O `Esc` emite `Cancel` E DEPOIS `Blur`: largar o alvo aqui é o que faz o `Blur` não gravar.
        WidgetEvent::Cancel(id) if id == ids::DOC_TAB_RENAME_INPUT => {
            hero.documents.renaming = None;
            return true;
        }
        WidgetEvent::Click(id) => id,
        _ => return false,
    };
    let rows = [
        ids::CTX_MENU_BOARD_RENAME,
        ids::CTX_MENU_BOARD_DUPLICATE,
        ids::CTX_MENU_BOARD_DELETE,
        ids::CTX_MENU_BOARD_DELETE_CONFIRM,
        ids::CTX_MENU_BOARD_DELETE_CANCEL,
    ];
    if !rows.contains(&id) {
        return false;
    }
    let board = menu_board(&hero.store);
    hero.store.close_context_menu();
    hero.store.consume_last_context_menu();
    let Some(board) = board.filter(|b| hero.documents.boards().get(*b).is_some()) else {
        return true;
    };
    if id == ids::CTX_MENU_BOARD_RENAME {
        begin_rename(hero, board);
    } else if id == ids::CTX_MENU_BOARD_DUPLICATE {
        let name = hero.documents.boards().get(board).map_or("", |b| &b.name);
        let name = tr_with("board.tab.copy_name", &[("name", &name)]);
        if let Some(copy) = hero.documents.duplicate(board, name) {
            register_board(&mut hero.store, copy);
        }
    } else if id == ids::CTX_MENU_BOARD_DELETE {
        hero.store.open_context_menu(ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: ContextMenuKind::ConfirmDeleteBoard { board: board.0 },
        });
    } else if id == ids::CTX_MENU_BOARD_DELETE_CONFIRM && hero.documents.remove(board).is_some() {
        hero.store.unregister(tab_node_id(board));
    }
    true
}

/// Regista o campo e as linhas (focáveis desde o arranque). Chamado pelo `document_tabs::populate`.
pub(super) fn populate(store: &mut WidgetStore) {
    store.register(
        ids::DOC_TAB_RENAME_INPUT,
        InteractiveState::TextInput {
            state: crate::widget::TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    for id in [
        ids::CTX_MENU_BOARD_RENAME,
        ids::CTX_MENU_BOARD_DUPLICATE,
        ids::CTX_MENU_BOARD_DELETE,
        ids::CTX_MENU_BOARD_DELETE_CONFIRM,
        ids::CTX_MENU_BOARD_DELETE_CANCEL,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: crate::widget::ButtonState::Normal,
            },
        );
    }
}

/// ⭐ **A pergunta antes de apagar** — centrada, com o NOME do quadro (por isso pinta-a quem tem os
/// quadros, logo a seguir ao overlay dos menus). No-op se a pergunta não está aberta.
pub fn paint_confirm_delete(
    docs: &Documents,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    viewport: Rect,
) {
    let Some(ContextMenuKind::ConfirmDeleteBoard { board }) = store.context_menu().map(|r| r.kind)
    else {
        return;
    };
    let Some(name) = docs.boards().get(BoardId(board)).map(|b| b.name.as_str()) else {
        return;
    };
    let gap = Spacing::Xs.px();
    let w = super::context_menu_dialogs::DIALOG_W;
    let h = pad_y() * 2.0 + ROW_H * 3.0 + gap * 2.0; // LITERAL-PX-OK: 3 linhas (título · aviso · botões), 2 vãos
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
    let x = rect.x + Spacing::Md.px();
    let inner_w = rect.w - Spacing::Md.px() * 2.0;
    let font = TypeToken::Sm.px();
    let mut y = rect.y + pad_y();
    let title = tr_with("board.dialog.delete_title", &[("name", &name)]);
    for (text, token) in [
        (title.as_str(), ColorToken::Text1),
        (tr("board.dialog.delete_hint"), ColorToken::Text3),
    ] {
        let ty = y + (ROW_H - font) * 0.5;
        paint_text(
            text_system,
            scene,
            text,
            x,
            ty,
            font,
            inner_w,
            resolve(token, theme),
        );
        y += ROW_H + gap;
    }
    let bw = (inner_w - gap) * 0.5;
    let cancel = Rect::new(x, y, bw, ROW_H);
    let delete = Rect::new(x + bw + gap, y, bw, ROW_H);
    let buttons = [
        (
            ids::CTX_MENU_BOARD_DELETE_CANCEL,
            cancel,
            tr("board.dialog.cancel"),
            false,
        ),
        (
            ids::CTX_MENU_BOARD_DELETE_CONFIRM,
            delete,
            tr("board.dialog.delete"),
            true,
        ),
    ];
    for (id, r, label, danger) in buttons {
        hit_index.register(id, r);
        let b = Button::new(id, label).visual(store.button_visual(id));
        let b = if danger { b.danger() } else { b };
        paint_button(&b, r, scene, text_system, theme);
    }
}
