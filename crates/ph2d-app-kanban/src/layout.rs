//! ⭐ **Onde fica cada coisa** — a disposição do quadro num quadro de imagem. Pintar e clicar leem a
//! MESMA (a do último quadro pintado, com as molas aplicadas): o que se vê é o que se agarra.
//!
//! Duas passadas: [`targets`] diz onde cada lista e cartão DEVE estar (com o espaço que o arrasto
//! abre e o campo de texto no lugar); o pintor mira as molas nesses alvos e grava a disposição VISTA.
//! As molas vivem no espaço do conteúdo (sem a rolagem): rolar é imediato, assentar é que é mola.

use crate::edit::{Field, Target};
use crate::metrics as m;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{CardId, Kanban, KanbanView, ListId};

/// Um cartão na disposição.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CardBox {
    pub id: CardId,
    pub rect: Rect,
}

/// Uma lista na disposição.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ListBox {
    pub id: ListId,
    pub rect: Rect,
    pub collapsed: bool,
    /// O título (clicar edita).
    pub title: Rect,
    pub collapse: Rect,
    pub menu: Rect,
    /// O recorte dos cartões (rola por dentro).
    pub body: Rect,
    /// A altura dos cartões todos (o fundo da rolagem).
    pub content_h: f32,
    pub cards: Vec<CardBox>,
    /// O espaço aberto pelo arrasto, se cai aqui.
    pub slot: Option<Rect>,
    /// O compositor de cartão aberto nesta lista (a caixa do texto).
    pub composer: Option<Rect>,
    /// «Add a card» — ou, com o compositor aberto, os botões dele.
    pub add: Rect,
    pub close: Option<Rect>,
    pub count: usize,
}

/// O quadro inteiro.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Layout {
    pub area: Rect,
    pub header: Rect,
    pub title: Rect,
    pub star: Rect,
    pub board_menu: Rect,
    /// A área das listas (abaixo do cabeçalho).
    pub board: Rect,
    pub lists: Vec<ListBox>,
    /// «Add another list» — ou o compositor de lista.
    pub add_list: Rect,
    pub add_list_field: Option<Rect>,
    pub add_list_close: Option<Rect>,
    /// A largura do conteúdo (o fundo da rolagem horizontal).
    pub content_w: f32,
}

impl Layout {
    pub fn list(&self, id: ListId) -> Option<&ListBox> {
        self.lists.iter().find(|l| l.id == id)
    }

    pub fn card(&self, id: CardId) -> Option<(&ListBox, CardBox)> {
        self.lists
            .iter()
            .find_map(|l| l.cards.iter().find(|c| c.id == id).map(|c| (l, *c)))
    }
}

/// O que o arrasto em curso pede à disposição.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Lifted {
    /// Um cartão fora do fluxo; o espaço dele abre em `to` (lista, índice), com `h` de altura.
    Card {
        id: CardId,
        to: Option<(ListId, usize)>,
        h: f32,
    },
    /// Uma lista fora do fluxo; o espaço dela abre no índice `to`.
    List { id: ListId, to: usize, w: f32 },
}

/// A altura de cada cartão (o pintor mede-a e guarda-a) e a do texto do campo aberto.
pub(crate) trait Heights {
    fn card(&mut self, k: &Kanban, id: CardId) -> f32;
    fn field(&mut self, text: &str, width: f32) -> f32;
    /// A largura de um título de lista numa linha.
    fn title_w(&mut self, text: &str) -> f32;
}

/// O cabeçalho: o título à esquerda, a estrela, o menu à direita.
fn header(area: Rect, title_w: f32) -> (Rect, Rect, Rect, Rect) {
    let h = Rect::new(area.x, area.y, area.w, m::HEADER_H);
    let cy = h.y + (m::HEADER_H - m::BUTTON_H) / 2.0;
    let title = Rect::new(h.x + m::HEADER_PAD_L, cy, title_w, m::BUTTON_H);
    let star = Rect::new(
        title.x + title.w + m::HEADER_GAP,
        cy,
        m::BUTTON_H,
        m::BUTTON_H,
    );
    let menu = Rect::new(
        h.x + h.w - m::HEADER_PAD_R - m::BUTTON_H,
        cy,
        m::BUTTON_H,
        m::BUTTON_H,
    );
    (h, title, star, menu)
}

/// A cabeça de uma lista (o título e os dois botões).
fn head_h() -> f32 {
    m::LIST_HEAD_TOP + m::LINE_H + m::LIST_HEAD_BOTTOM
}

/// O pé de uma lista: «Add a card» (ou os botões do compositor).
fn foot_h() -> f32 {
    m::LIST_FOOT_TOP + m::BUTTON_H + m::LIST_FOOT_BOTTOM
}

/// ⭐ Onde cada lista e cartão DEVE estar, no espaço do CONTEÚDO (sem rolagem: `x` desde o início
/// das listas, `y` desde o topo do corpo de cada lista). `title_w` = a largura do título medida.
#[allow(clippy::too_many_lines)]
pub(crate) fn targets(
    k: &Kanban,
    view: &KanbanView,
    area: Rect,
    title_w: f32,
    field: Option<&Field>,
    lifted: Option<Lifted>,
    heights: &mut dyn Heights,
) -> Layout {
    let (hdr, title, star, board_menu) = header(area, title_w);
    let board = Rect::new(
        area.x,
        area.y + m::HEADER_H,
        area.w,
        (area.h - m::HEADER_H).max(0.0),
    );
    let top = board.y + m::BOARD_PAD;
    let max_h = (board.h - 2.0 * m::BOARD_PAD).max(head_h() + foot_h());
    let mut lists = Vec::new();
    let mut x = 0.0_f32;
    let lifted_list = match lifted {
        Some(Lifted::List { id, .. }) => Some(id),
        _ => None,
    };
    // Os cartões agrupados por lista numa passagem (pedir `cards_in` por lista custava listas × N:
    // 63–77 ms dos 144 a 100 000 cartões em 10 listas, medido 28/10). A ordem é a do `cards_in`.
    let mut by_list: std::collections::BTreeMap<ListId, Vec<&ph2d_kanban_model::Card>> =
        std::collections::BTreeMap::new();
    for c in k.cards.values().filter(|c| !c.meta.deleted && !c.archived) {
        by_list.entry(c.list).or_default().push(c);
    }
    for v in by_list.values_mut() {
        v.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
    }
    let order: Vec<_> = k
        .lists_in_order()
        .into_iter()
        .filter(|l| Some(l.id) != lifted_list)
        .collect();
    for (i, l) in order.iter().enumerate() {
        if let Some(Lifted::List { to, w, .. }) = lifted
            && to == i
        {
            x += w + m::LIST_GAP;
        }
        let collapsed = view.collapsed.contains(&l.id);
        let cards_all = by_list.remove(&l.id).unwrap_or_default();
        let count = cards_all.len();
        if collapsed {
            let tw = heights.title_w(&l.title);
            let rect = Rect::new(x, top, m::COLLAPSED_W, max_h.min(collapsed_h(tw)));
            let btn = Rect::new(
                x + (m::COLLAPSED_W - m::LIST_ICON_BTN) / 2.0,
                top + m::LIST_HEAD_TOP,
                m::LIST_ICON_BTN,
                m::LIST_ICON_BTN,
            );
            lists.push(ListBox {
                id: l.id,
                rect,
                collapsed: true,
                title: rect,
                collapse: btn,
                menu: Rect::new(0.0, 0.0, 0.0, 0.0),
                body: Rect::new(0.0, 0.0, 0.0, 0.0),
                content_h: 0.0,
                cards: Vec::new(),
                slot: None,
                composer: None,
                add: Rect::new(0.0, 0.0, 0.0, 0.0),
                close: None,
                count,
            });
            x += m::COLLAPSED_W + m::LIST_GAP;
            continue;
        }
        let inner_x = x + m::LIST_BODY_X;
        let menu = Rect::new(
            x + m::LIST_W - m::LIST_HEAD_RIGHT - m::LIST_ICON_BTN,
            top + m::LIST_HEAD_TOP - (m::LIST_ICON_BTN - m::LINE_H) / 2.0,
            m::LIST_ICON_BTN,
            m::LIST_ICON_BTN,
        );
        let collapse = Rect::new(
            menu.x - m::LIST_ICON_BTN,
            menu.y,
            m::LIST_ICON_BTN,
            m::LIST_ICON_BTN,
        );
        let title = Rect::new(
            x + m::LIST_HEAD_LEFT,
            top + m::LIST_HEAD_TOP,
            (collapse.x - m::LIST_ICON_GAP - x - m::LIST_HEAD_LEFT).max(0.0),
            m::LINE_H,
        );
        // Os cartões, um a seguir ao outro, com o espaço do arrasto e o compositor no meio.
        let mut y = 0.0_f32;
        let mut cards = Vec::new();
        let mut slot = None;
        let mut composer = None;
        let composing = field.and_then(|f| match f.target {
            Target::NewCard { list, after } if list == l.id => Some((after, f.text.as_str())),
            _ => None,
        });
        let lifted_card = match lifted {
            Some(Lifted::Card { id, .. }) => Some(id),
            _ => None,
        };
        let drop_at = match lifted {
            Some(Lifted::Card {
                to: Some((list, i)),
                h,
                ..
            }) if list == l.id => Some((i, h)),
            _ => None,
        };
        let push_gap = |y: &mut f32, h: f32| {
            let r = Rect::new(inner_x, *y, m::CARD_W, h);
            *y += h + m::CARD_GAP;
            r
        };
        let mut shown = 0usize;
        for c in &cards_all {
            if Some(c.id) == lifted_card {
                continue;
            }
            if let Some((i, h)) = drop_at
                && i == shown
            {
                slot = Some(push_gap(&mut y, h));
            }
            let editing = field.is_some_and(|f| f.target == Target::CardTitle(c.id));
            let h = if editing {
                let f = field.map_or("", |f| f.text.as_str());
                (heights.field(f, m::CARD_W - 2.0 * m::CARD_PAD_X) + 2.0 * m::CARD_PAD_Y)
                    .max(heights.card(k, c.id))
            } else {
                heights.card(k, c.id)
            };
            cards.push(CardBox {
                id: c.id,
                rect: Rect::new(inner_x, y, m::CARD_W, h),
            });
            y += h + m::CARD_GAP;
            shown += 1;
            if let Some((Some(after), text)) = composing
                && after == c.id
            {
                let h = composer_h(heights, text);
                composer = Some(push_gap(&mut y, h));
            }
        }
        if let Some((i, h)) = drop_at
            && i >= shown
        {
            slot = Some(push_gap(&mut y, h));
        }
        if let Some((None, text)) = composing {
            let h = composer_h(heights, text);
            composer = Some(push_gap(&mut y, h));
        }
        let content_h = (y - m::CARD_GAP).max(0.0);
        let body_room = (max_h - head_h() - foot_h()).max(0.0);
        let body_h = content_h.min(body_room) + m::LIST_BODY_TOP + m::LIST_BODY_BOTTOM;
        let body = Rect::new(x, top + head_h(), m::LIST_W, body_h);
        let foot_y = body.y + body.h + m::LIST_FOOT_TOP;
        let (add, close) = if composing.is_some() {
            let add = Rect::new(inner_x, foot_y, m::CARD_W / 2.0, m::BUTTON_H);
            let close = Rect::new(
                add.x + add.w + m::LIST_ICON_GAP,
                foot_y,
                m::BUTTON_H,
                m::BUTTON_H,
            );
            (add, Some(close))
        } else {
            (Rect::new(inner_x, foot_y, m::CARD_W, m::BUTTON_H), None)
        };
        let rect = Rect::new(x, top, m::LIST_W, head_h() + body_h + foot_h());
        lists.push(ListBox {
            id: l.id,
            rect,
            collapsed: false,
            title,
            collapse,
            menu,
            body,
            content_h,
            cards,
            slot,
            composer,
            add,
            close,
            count,
        });
        x += m::LIST_W + m::LIST_GAP;
    }
    if let Some(Lifted::List { to, w, .. }) = lifted
        && to >= order.len()
    {
        x += w + m::LIST_GAP;
    }
    let new_list = field.is_some_and(|f| f.target == Target::NewList);
    let (add_list, add_list_field, add_list_close) = if new_list {
        let text = field.map_or("", |f| f.text.as_str());
        let fh = heights
            .field(text, m::CARD_W - 2.0 * m::FIELD_PAD_X)
            .max(m::LINE_H)
            + 2.0 * m::FIELD_PAD_Y;
        let f = Rect::new(x + m::LIST_BODY_X, top + m::LIST_BODY_X, m::CARD_W, fh);
        let btn_y = f.y + f.h + m::LIST_FOOT_TOP;
        let add = Rect::new(f.x, btn_y, m::CARD_W / 2.0, m::BUTTON_H);
        let close = Rect::new(
            add.x + add.w + m::LIST_ICON_GAP,
            btn_y,
            m::BUTTON_H,
            m::BUTTON_H,
        );
        (add, Some(f), Some(close))
    } else {
        (Rect::new(x, top, m::LIST_W, m::NEW_LIST_H), None, None)
    };
    Layout {
        area,
        header: hdr,
        title,
        star,
        board_menu,
        board,
        lists,
        add_list,
        add_list_field,
        add_list_close,
        // As duas margens: a da esquerda (a origem das listas) e a da direita, depois do «Add list».
        content_w: x + m::LIST_W + 2.0 * m::BOARD_PAD,
    }
}

/// A caixa do compositor de cartão para `text`.
fn composer_h(heights: &mut dyn Heights, text: &str) -> f32 {
    (heights
        .field(text, m::CARD_W - 2.0 * m::CARD_PAD_X)
        .max(m::LINE_H)
        + 2.0 * m::CARD_PAD_Y)
        .max(m::COMPOSER_MIN_H)
}

/// Uma lista recolhida: o botão, o título na vertical (`title_w` = a largura dele deitado) e a
/// contagem.
fn collapsed_h(title_w: f32) -> f32 {
    m::LIST_HEAD_TOP
        + m::LIST_ICON_BTN
        + m::LIST_ICON_GAP
        + title_w
        + m::LIST_ICON_GAP
        + m::COUNT_H
        + m::LIST_HEAD_TOP
}

/// O quadro rola na horizontal e cada lista na vertical: o fundo de cada rolagem.
pub(crate) fn max_scroll_x(l: &Layout) -> f32 {
    (l.content_w - l.board.w).max(0.0)
}

pub(crate) fn max_scroll_y(lb: &ListBox) -> f32 {
    if lb.collapsed {
        return 0.0;
    }
    (lb.content_h - (lb.body.h - m::LIST_BODY_TOP - m::LIST_BODY_BOTTOM)).max(0.0)
}
