//! ⭐ **«Send to board…»** (MiroClone W15, ordem do dono 11/10: *«na hierarchy precisamos de uma
//! opção no menu do botão direito: "Enviar para o quadro..." e um modal para escolher o quadro que
//! recebe a img, vetor ou flip»*).
//!
//! O caminho inteiro: a linha do menu ([`crate::ids::CTX_MENU_HIER_SEND_TO_BOARD`], pintada só para
//! os tipos de [`goes_to_board`]) → `HierRequest::SendToBoard` → a shell confere o tipo e chama
//! [`open`] → este modal (busca + os quadros pela ordem das abas + «New board») →
//! `HierRequest::SendToBoardChosen` → a shell lê o conteúdo e a família do quadro entrega-o
//! (`ph2d_app_board::send_to_board::deliver`).
//!
//! ⚠️ A lista que se PINTA e a que o CLIQUE resolve saem da mesma [`visible`]: a linha `i` do modal
//! é sempre o quadro `i` dessa lista.

use super::HeroScreen;
use super::context_menu_overlay::{ROW_H, pad_y};
use crate::action_bus::{EditorAction, HierRequest};
use crate::documents::Documents;
use crate::icons::IconId;
use crate::ids;
use crate::interaction::{
    ContextMenuKind, ContextMenuRequest, HitIndex, InteractiveState, WidgetEvent, WidgetStore,
};
use crate::paint::{fill_rounded_rect, paint_icon, paint_text};
use crate::widget::{ButtonState, TextInput, TextInputState, paint_text_input_with_buffer};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_board_model::BoardId;
use ph2d_component_desc::ObjectKind;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, INLINE_ICON_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// ⭐ **A lei: que objectos um quadro recebe** — a imagem (os píxeis da sprite), o vector (um SVG,
/// nítido a qualquer zoom) e o Flip (o quadro actual). A shell lê-a para publicar as linhas
/// ([`WidgetStore::set_hierarchy_board_rows`]) e para conferir o pedido.
#[must_use]
pub const fn goes_to_board(kind: ObjectKind) -> bool {
    matches!(
        kind,
        ObjectKind::Image | ObjectKind::Vector | ObjectKind::Flip
    )
}

/// Abre o modal para a linha `row`: a busca limpa e com o teclado.
pub fn open(store: &mut WidgetStore, row: NodeId) {
    open_kind(store, ContextMenuKind::SendToBoard { row });
}

/// ⭐ Kanban D13 — o MESMO modal a escolher o Kanban cujo fundo passa a ser a imagem da linha `row`.
pub fn open_kanban_background(store: &mut WidgetStore, row: NodeId) {
    open_kind(store, ContextMenuKind::KanbanBackground { row });
}

/// ⭐ W23 B15 — o MESMO modal para a selecção do quadro `from` (`keep` = copiar, senão mover).
pub fn open_selection(store: &mut WidgetStore, from: BoardId, keep: bool) {
    open_kind(store, ContextMenuKind::SendSelection { from: from.0, keep });
}

fn open_kind(store: &mut WidgetStore, kind: ContextMenuKind) {
    store.register(
        ids::CTX_SEND_BOARD_SEARCH,
        InteractiveState::TextInput {
            state: TextInputState::Focused,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    store.set_focus(Some(ids::CTX_SEND_BOARD_SEARCH));
    store.mark_cancel_on_escape(ids::CTX_SEND_BOARD_SEARCH);
    store.open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind,
    });
}

/// O texto da busca.
fn query(store: &WidgetStore) -> &str {
    match store.get(ids::CTX_SEND_BOARD_SEARCH) {
        Some(InteractiveState::TextInput { text, .. }) => text.as_str(),
        _ => "",
    }
}

/// ⭐ Os quadros que o modal mostra, pela ordem das abas — os que contêm a busca (sem caixa), até
/// [`ids::CTX_SEND_BOARD_ROWS`] — e quantos casaram e ficaram de fora. `except` = o quadro de onde a
/// selecção sai (B15: mandá-la para ele mesmo não é escolha).
#[must_use]
pub fn visible(docs: &Documents, query: &str, except: Option<BoardId>) -> (Vec<BoardId>, usize) {
    let q = query.trim().to_lowercase();
    let all: Vec<BoardId> = docs
        .boards()
        .boards()
        .iter()
        .filter(|b| Some(b.id) != except)
        .filter(|b| q.is_empty() || b.name.to_lowercase().contains(&q))
        .map(|b| b.id)
        .collect();
    let cap = ids::CTX_SEND_BOARD_ROWS.len();
    let more = all.len().saturating_sub(cap);
    (all.into_iter().take(cap).collect(), more)
}

/// ⭐ Os Kanbans que o modal mostra (D13), pela ordem do projecto — os que contêm a busca — até
/// [`ids::CTX_SEND_BOARD_ROWS`], e quantos ficaram de fora.
#[must_use]
pub fn visible_kanbans(docs: &Documents, query: &str) -> (Vec<(u64, String)>, usize) {
    let q = query.trim().to_lowercase();
    let set = docs.kanbans();
    let all: Vec<(u64, String)> = set
        .iter()
        .filter(|k| q.is_empty() || k.header.name.to_lowercase().contains(&q))
        .map(|k| (k.id.0, k.header.name.clone()))
        .collect();
    let cap = ids::CTX_SEND_BOARD_ROWS.len();
    let more = all.len().saturating_sub(cap);
    (all.into_iter().take(cap).collect(), more)
}

/// O que o modal aberto vai mandar: o conteúdo de uma linha da Hierarquia (para um quadro, ou para o
/// fundo de um Kanban), ou a selecção de um quadro.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Row(NodeId),
    Selection { from: BoardId, keep: bool },
    Kanban(NodeId),
}

impl Source {
    /// O quadro que a lista esconde (o de onde a selecção sai).
    fn except(self) -> Option<BoardId> {
        match self {
            Self::Row(_) | Self::Kanban(_) => None,
            Self::Selection { from, .. } => Some(from),
        }
    }

    /// A lista que o modal mostra e o clique resolve (`id`, nome), e quantos casaram e ficaram de fora.
    fn items(self, docs: &Documents, query: &str) -> (Vec<(u64, String)>, usize) {
        if matches!(self, Self::Kanban(_)) {
            return visible_kanbans(docs, query);
        }
        let (boards, more) = visible(docs, query, self.except());
        let named = boards
            .into_iter()
            .map(|b| {
                (
                    b.0,
                    docs.boards()
                        .get(b)
                        .map_or_else(String::new, |d| d.name.clone()),
                )
            })
            .collect();
        (named, more)
    }

    /// A busca e a linha «novo» (as chaves).
    fn search_and_new(self) -> (&'static str, &'static str) {
        match self {
            Self::Kanban(_) => ("kanban.bg.pick_search", "kanban.bg.pick_new"),
            _ => ("board.send.search", "board.send.new_board"),
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Row(_) => "board.send.title",
            Self::Selection { keep: true, .. } => "board.send.copy_title",
            Self::Selection { keep: false, .. } => "board.send.move_title",
            Self::Kanban(_) => "kanban.bg.pick_title",
        }
    }
}

/// O que o modal aberto vai mandar.
#[must_use]
pub fn source(store: &WidgetStore) -> Option<Source> {
    match store.context_menu()?.kind {
        ContextMenuKind::SendToBoard { row } => Some(Source::Row(row)),
        ContextMenuKind::SendSelection { from, keep } => Some(Source::Selection {
            from: BoardId(from),
            keep,
        }),
        ContextMenuKind::KanbanBackground { row } => Some(Source::Kanban(row)),
        _ => None,
    }
}

/// O quadro (ou o Kanban) escolhido (`None` = um novo): a linha da Hierarquia segue para a shell; a
/// selecção de um quadro vai na pintura seguinte (`board_send::run_pending`, com o moldador). O modal
/// fecha.
fn choose(hero: &mut HeroScreen, src: Source, chosen: Option<u64>) {
    let board = chosen.map(BoardId);
    hero.store.close_context_menu();
    hero.store.consume_last_context_menu();
    hero.store.set_focus(None);
    match src {
        Source::Row(row) => {
            hero.bus
                .push(EditorAction::Hierarchy(HierRequest::SendToBoardChosen {
                    row,
                    board: board.map(|b| b.0),
                }))
        }
        Source::Selection { from, keep } => {
            hero.documents.live.send = Some(crate::documents::alert::Send {
                from,
                to: board,
                keep,
            });
        }
        Source::Kanban(row) => hero.bus.push(EditorAction::Hierarchy(
            HierRequest::KanbanBackgroundChosen {
                row,
                kanban: chosen,
            },
        )),
    }
}

/// As linhas do modal, a busca (`Enter` = o primeiro quadro da lista, `Esc` = desistir). Corre no
/// pré-despacho. `true` = consumido.
pub(super) fn apply_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let Some(src) = source(&hero.store) else {
        return false;
    };
    let pick = |hero: &HeroScreen, slot: usize| {
        src.items(&hero.documents, query(&hero.store))
            .0
            .get(slot)
            .map(|(id, _)| *id)
    };
    match event {
        WidgetEvent::Click(id) if id == ids::CTX_SEND_BOARD_NEW => choose(hero, src, None),
        WidgetEvent::Click(id) => {
            let Some(slot) = ids::CTX_SEND_BOARD_ROWS.iter().position(|r| *r == id) else {
                return false;
            };
            if let Some(board) = pick(hero, slot) {
                choose(hero, src, Some(board));
            }
        }
        WidgetEvent::Submit(id) if id == ids::CTX_SEND_BOARD_SEARCH => {
            if let Some(board) = pick(hero, 0) {
                choose(hero, src, Some(board));
            }
        }
        WidgetEvent::Cancel(id) if id == ids::CTX_SEND_BOARD_SEARCH => {
            hero.store.close_context_menu();
            hero.store.consume_last_context_menu();
        }
        _ => return false,
    }
    true
}

/// Regista a linha do menu e as do modal (focáveis desde o arranque).
pub(super) fn populate(store: &mut WidgetStore) {
    store.register(
        ids::CTX_SEND_BOARD_SEARCH,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    let buttons = [
        ids::CTX_MENU_HIER_SEND_TO_BOARD,
        ids::CTX_MENU_HIER_KANBAN_BACKGROUND,
        ids::CTX_SEND_BOARD_NEW,
    ];
    for id in buttons.into_iter().chain(ids::CTX_SEND_BOARD_ROWS) {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// ⭐ **O modal** — centrado: o título, a busca, os quadros (o activo com o ponto), «New board».
/// Pinta-o quem tem os quadros, logo a seguir ao overlay dos menus. No-op se não está aberto.
pub fn paint(
    docs: &Documents,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    viewport: Rect,
) {
    let Some(src) = source(store) else {
        return;
    };
    let (query, caret, anchor, ti_state) = match store.get(ids::CTX_SEND_BOARD_SEARCH) {
        Some(InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            state,
        }) => (text.as_str(), *caret, *selection_anchor, *state),
        _ => ("", 0, None, TextInputState::Normal),
    };
    let (boards, more) = src.items(docs, query);
    let (search_key, new_key) = src.search_and_new();
    let gap = Spacing::Xs.px();
    // título · busca · quadros (≥ 1: a frase «nada casa») · «mais N» · risco · «New board».
    let list_rows = boards.len().max(1) + usize::from(more > 0);
    let w = super::context_menu_dialogs::DIALOG_W;
    let h = pad_y() * 2.0 + ROW_H * (3 + list_rows) as f32 + gap * 3.0; // LITERAL-PX-OK: 3 linhas fixas e 3 vãos (título · busca · risco)
    let rect = Rect::new(
        (viewport.x + (viewport.w - w) * 0.5).max(viewport.x),
        (viewport.y + (viewport.h - h) * 0.5).max(viewport.y),
        w,
        h,
    );
    let superficie = crate::widget::lista_flutuante::Superficie::de(theme);
    let _tinta = superficie.veste();
    superficie.moldura_de_caixa(scene, rect);
    let x = rect.x + Spacing::Xs.px();
    let inner_w = rect.w - Spacing::Xs.px() * 2.0;
    let pad_x = Spacing::Md.px();
    let font = TypeToken::Sm.px();
    let text_at = |r: Rect| r.y + (r.h - font) * 0.5;
    let mut y = rect.y + pad_y();
    let title = Rect::new(x, y, inner_w, ROW_H);
    let ink = |t: ColorToken| crate::widget::card_ink::tinta(t, theme);
    let tw = inner_w - pad_x * 2.0;
    let title_text = tr(src.title());
    paint_text(
        text_system,
        scene,
        title_text,
        x + pad_x,
        text_at(title),
        font,
        tw,
        ink(ColorToken::Text1),
    );
    y += ROW_H + gap;
    let search = Rect::new(x, y, inner_w, ROW_H);
    hit_index.register(ids::CTX_SEND_BOARD_SEARCH, search);
    let ti = TextInput::new(ids::CTX_SEND_BOARD_SEARCH, "")
        .placeholder(tr(search_key))
        .visual((ti_state, store.hover_live(ids::CTX_SEND_BOARD_SEARCH)));
    paint_text_input_with_buffer(
        &ti,
        Some(query),
        Some(caret),
        anchor,
        search,
        scene,
        text_system,
        theme,
    );
    y += ROW_H + gap;
    let active = docs.active().filter(|_| !matches!(src, Source::Kanban(_)));
    let hot = |id: NodeId, r: Rect, scene: &mut VectorScene| {
        if Some(id) == store.hot_id() {
            let rr = crate::paint::frame_radius(theme, Radius::Sm.px());
            fill_rounded_rect(scene, r, rr, ink(ColorToken::Bg2));
        }
    };
    let dot_col = pad_x + INLINE_ICON_PX;
    for (i, (b, name)) in boards.iter().enumerate() {
        let id = ids::CTX_SEND_BOARD_ROWS[i];
        let r = Rect::new(x, y, inner_w, ROW_H);
        hit_index.register(id, r);
        hot(id, r, scene);
        if Some(BoardId(*b)) == active {
            // O quadro activo leva o ponto do acento, como a lista de cenas.
            let c = ph2d_vector::Circle::new(
                ph2d_vector::Point::new(f64::from(r.x + pad_x), f64::from(r.y + r.h * 0.5)),
                f64::from(StrokeToken::Thick.px()),
            );
            scene.inner_mut().fill(
                ph2d_vector::Fill::NonZero,
                ph2d_vector::Affine::IDENTITY,
                &ph2d_vector::Brush::Solid(ink(ColorToken::Accent)),
                None,
                &c,
            );
        }
        paint_text(
            text_system,
            scene,
            name,
            r.x + dot_col,
            text_at(r),
            font,
            tw,
            ink(ColorToken::Text1),
        );
        y += ROW_H;
    }
    if boards.is_empty() {
        let r = Rect::new(x, y, inner_w, ROW_H);
        let none = tr("chrome.no_matches");
        paint_text(
            text_system,
            scene,
            none,
            r.x + dot_col,
            text_at(r),
            font,
            tw,
            ink(ColorToken::Text3),
        );
        y += ROW_H;
    }
    if more > 0 {
        let r = Rect::new(x, y, inner_w, ROW_H);
        let n = more.to_string();
        let hint = tr_with("board.send.more", &[("count", &n)]);
        paint_text(
            text_system,
            scene,
            &hint,
            r.x + dot_col,
            text_at(r),
            font,
            tw,
            ink(ColorToken::Text3),
        );
        y += ROW_H;
    }
    // O risco antes do «New board»: criar não é escolher um dos de cima.
    let line = Rect::new(x, y + gap * 0.5, inner_w, StrokeToken::Hairline.px());
    fill_rounded_rect(scene, line, 0.0, ink(ColorToken::Border));
    y += gap;
    let r = Rect::new(x, y, inner_w, ROW_H);
    hit_index.register(ids::CTX_SEND_BOARD_NEW, r);
    hot(ids::CTX_SEND_BOARD_NEW, r, scene);
    let icon = Rect::new(
        r.x + pad_x * 0.5,
        r.y + (r.h - INLINE_ICON_PX) * 0.5,
        INLINE_ICON_PX,
        INLINE_ICON_PX,
    );
    paint_icon(
        scene,
        IconId::Add,
        icon,
        ink(ColorToken::Text2),
        StrokeToken::Default.px(),
    );
    let new = tr(new_key);
    paint_text(
        text_system,
        scene,
        new,
        r.x + dot_col,
        text_at(r),
        font,
        tw,
        ink(ColorToken::Text1),
    );
}

#[cfg(test)]
#[path = "send_to_board_tests.rs"]
mod tests;
