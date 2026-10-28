//! ⭐ **Uma aba de Kanban** — a VISTA (o que é só desta janela: listas recolhidas, rolagem) e o estado
//! do gesto (o que paira, o arrasto, o campo aberto, as molas). Nada disto vai para o documento.
//!
//! [`place`] leva a disposição-alvo ([`crate::layout::targets`]) ao ecrã: mira as molas, avança-as e
//! grava a disposição VISTA, que o clique seguinte lê.

use crate::anim::Follow;
use crate::edit::Field;
use crate::layout::{CardBox, Layout, ListBox};
use crate::metrics as m;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{CardId, KanbanId, KanbanView, ListId};
use ph2d_spring::Spring;
use std::collections::BTreeMap;

/// Onde o rato está (o que o clique tomaria).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Hit {
    KanbanTitle,
    Star,
    BoardMenu,
    ListTitle(ListId),
    /// A cabeça da lista fora do título e dos botões: agarra a lista.
    ListHead(ListId),
    Collapse(ListId),
    ListMenu(ListId),
    AddCard(ListId),
    Card(CardId),
    CardEdit(CardId),
    /// Um rótulo na frente de um cartão (mostra/esconde o texto dos rótulos, o idioma do Trello).
    CardLabel(CardId),
    /// O botão «Add card»/«Add list» de um compositor, e o «×» dele.
    FieldAdd,
    FieldClose,
    Field,
    AddList,
    /// O fundo do quadro: arrastar rola na horizontal.
    Board,
    Menu(crate::menu::Row),
    Panel(crate::archive::Row),
}

/// O que o arrasto leva.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Lift {
    Card(CardId),
    List(ListId),
    /// O fundo do quadro (rola).
    Pan {
        scroll_x: f64,
    },
}

/// Um carregar que ainda não decidiu se é clique ou arrasto.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Press {
    pub at: (f32, f32),
    pub hit: Hit,
}

/// Um arrasto em curso.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Drag {
    pub lift: Lift,
    /// Onde o ponteiro está e onde agarrou (dentro do que voa).
    pub at: (f32, f32),
    pub grab: (f32, f32),
    /// O tamanho do que voa.
    pub size: (f32, f32),
    /// Para onde cai: `(lista, índice)` de um cartão, ou o índice de uma lista.
    pub to_card: Option<(ListId, usize)>,
    pub to_list: usize,
}

/// As molas de uma aba, no espaço do conteúdo.
#[derive(Clone, Debug, Default)]
pub(crate) struct Springs {
    pub lists: BTreeMap<ListId, Follow>,
    pub cards: BTreeMap<CardId, (Follow, Follow)>,
    /// A inclinação de quem voa (graus) e a sombra (0 → 1).
    pub tilt: Follow,
    /// A disposição inteira ainda se mexe (o hero repinta; o teste espera).
    pub moving: bool,
}

/// Uma aba de Kanban.
#[derive(Clone, Debug)]
pub(crate) struct Tab {
    pub view: KanbanView,
    pub layout: Layout,
    pub hover: Option<Hit>,
    pub press: Option<Press>,
    pub drag: Option<Drag>,
    pub field: Option<Field>,
    /// O cartão com o foco do teclado (o anel).
    pub focus: Option<CardId>,
    pub menu: Option<crate::menu::Menu>,
    pub panel: Option<crate::archive::Panel>,
    /// Os rótulos mostram o texto (clicar num rótulo alterna — o Trello).
    pub label_text: bool,
    pub springs: Springs,
    /// As alturas medidas: `(impressão do que pesa na altura, altura)`.
    pub heights: BTreeMap<CardId, (u64, f32)>,
    /// O cartão largado agora: parte de onde a mão o deixou (ecrã).
    pub landed: Option<(CardId, f32, f32)>,
}

impl Tab {
    pub fn new(kanban: KanbanId) -> Self {
        Self {
            view: KanbanView::of(kanban),
            layout: Layout::default(),
            hover: None,
            press: None,
            drag: None,
            field: None,
            focus: None,
            menu: None,
            panel: None,
            label_text: true,
            springs: Springs::default(),
            heights: BTreeMap::new(),
            landed: None,
        }
    }

    /// O id da aba (o do Kanban).
    pub fn id(&self) -> u64 {
        self.view.kanban.0
    }

    pub fn scroll_y(&self, list: ListId) -> f32 {
        #[allow(clippy::cast_possible_truncation)]
        self.view.scroll_y.get(&list).map_or(0.0, |v| *v as f32)
    }
}

fn shift(r: Rect, dx: f32, dy: f32) -> Rect {
    Rect::new(r.x + dx, r.y + dy, r.w, r.h)
}

/// ⭐ A disposição VISTA: mira as molas nos alvos, avança `dt` e leva tudo ao ecrã (com a rolagem).
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn place(tab: &mut Tab, target: Layout, dt: f64, law: Option<Spring>) -> Layout {
    let ox = target.board.x + m::BOARD_PAD - tab.view.scroll_x as f32;
    let mut moving = false;
    let landed = tab.landed.take();
    let mut lists = Vec::with_capacity(target.lists.len());
    for lb in target.lists {
        let f = tab.springs.lists.entry(lb.id).or_default();
        f.aim(f64::from(lb.rect.x));
        moving |= f.advance(dt, law);
        let lx = f.pos as f32;
        let dx = ox + lx - lb.rect.x;
        let sy = tab.scroll_y(lb.id);
        let body_y = lb.body.y + m::LIST_BODY_TOP - sy;
        let mut cards = Vec::with_capacity(lb.cards.len());
        for c in &lb.cards {
            let (fx, fy) = tab.springs.cards.entry(c.id).or_default();
            if let Some((id, x, y)) = landed
                && id == c.id
            {
                fx.jump(f64::from(x - ox));
                fy.jump(f64::from(y - body_y));
            }
            fx.aim(f64::from(c.rect.x));
            fy.aim(f64::from(c.rect.y));
            moving |= fx.advance(dt, law);
            moving |= fy.advance(dt, law);
            cards.push(CardBox {
                id: c.id,
                rect: Rect::new(
                    ox + fx.pos as f32,
                    body_y + fy.pos as f32,
                    c.rect.w,
                    c.rect.h,
                ),
            });
        }
        let card_dy = |r: Rect| Rect::new(r.x + dx, body_y + r.y, r.w, r.h);
        lists.push(ListBox {
            id: lb.id,
            rect: shift(lb.rect, dx, 0.0),
            collapsed: lb.collapsed,
            title: shift(lb.title, dx, 0.0),
            collapse: shift(lb.collapse, dx, 0.0),
            menu: shift(lb.menu, dx, 0.0),
            body: shift(lb.body, dx, 0.0),
            content_h: lb.content_h,
            cards,
            slot: lb.slot.map(card_dy),
            composer: lb.composer.map(card_dy),
            add: shift(lb.add, dx, 0.0),
            close: lb.close.map(|r| shift(r, dx, 0.0)),
            count: lb.count,
        });
    }
    // O que já não está na disposição larga a mola (um cartão arquivado não volta a voar de lá).
    let alive_l: std::collections::BTreeSet<ListId> = lists.iter().map(|l| l.id).collect();
    tab.springs.lists.retain(|id, _| alive_l.contains(id));
    let alive_c: std::collections::BTreeSet<CardId> = lists
        .iter()
        .flat_map(|l| l.cards.iter().map(|c| c.id))
        .collect();
    let lifted = match tab.drag.map(|d| d.lift) {
        Some(crate::tab::Lift::Card(id)) => Some(id),
        _ => None,
    };
    tab.springs
        .cards
        .retain(|id, _| alive_c.contains(id) || Some(*id) == lifted);
    let tilt_to = if matches!(
        tab.drag.map(|d| d.lift),
        Some(Lift::Card(_) | Lift::List(_))
    ) {
        m::FLIGHT_TILT_DEG
    } else {
        0.0
    };
    // A inclinação nasce a 0 (o 1.º alvo de uma mola é a posição dela), e só então mira.
    if !tab.springs.tilt.seeded {
        tab.springs.tilt.aim(0.0);
    }
    tab.springs.tilt.aim(tilt_to);
    moving |= tab.springs.tilt.advance(dt, law);
    tab.springs.moving = moving;
    Layout {
        area: target.area,
        header: target.header,
        title: target.title,
        star: target.star,
        board_menu: target.board_menu,
        board: target.board,
        lists,
        add_list: shift(target.add_list, ox, 0.0),
        add_list_field: target.add_list_field.map(|r| shift(r, ox, 0.0)),
        add_list_close: target.add_list_close.map(|r| shift(r, ox, 0.0)),
        content_w: target.content_w,
    }
}
