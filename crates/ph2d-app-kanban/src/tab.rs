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
    /// W6 — «Create card from template» no pé da lista.
    TemplateBtn(ListId),
    /// W6 — um alvo de um painel (filtro, pesquisa, edição rápida, diálogo).
    W6(crate::w6::Act),
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

/// A mola de um cartão, RELATIVA à lista onde está (`x` desde a borda dela, `y` desde o topo do
/// corpo): os cartões andam com a lista — a que abre espaço, a que voa, a que assenta.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CardSpring {
    /// A lista onde foi visto (mudar de lista re-baseia a mola: o cartão não salta).
    pub list: Option<ListId>,
    pub x: Follow,
    pub y: Follow,
}

/// As molas de uma aba.
#[derive(Clone, Debug, Default)]
pub(crate) struct Springs {
    /// Cada lista: `x` no espaço do conteúdo e `y` = o desvio do topo do quadro (`0` no lugar).
    pub lists: BTreeMap<ListId, (Follow, Follow)>,
    pub cards: BTreeMap<CardId, CardSpring>,
    /// Onde cada lista foi vista no último quadro: `(x do conteúdo, topo do corpo no ecrã)`.
    pub seen: BTreeMap<ListId, (f32, f32)>,
    /// A inclinação de quem voa (graus) e a sombra (0 → 1).
    pub tilt: Follow,
    /// A disposição inteira ainda se mexe (o hero repinta; o teste espera).
    pub moving: bool,
}

/// Uma aba de Kanban.
#[derive(Debug)]
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
    /// O que o último quadro PINTOU (as sondas leem-no).
    pub painted: std::cell::RefCell<Painted>,
    /// SMOKE 3: a lista a pôr na mão, e onde (o canto dela desde a origem das listas).
    pub smoke_hold: Option<(ListId, (f32, f32))>,
    /// O verso do cartão aberto (o modal do Trello, W3).
    pub back: Option<Box<crate::back::Back>>,
    /// J2 — o painel de automação (o Butler) aberto: um modal por cima do quadro e do verso.
    pub auto: Option<Box<crate::auto::Panel>>,
    /// W4 — a vista de um elemento do Quadro: o id dela (`None` = a aba, cujo id é o do Kanban).
    pub key: Option<u64>,
    /// W4 — a vista do elemento (rolagem, recolhidas) como o documento do Quadro a deu da última vez:
    /// outra mudança lá (desfazer, um colega) volta a ser a desta vista.
    pub board_seen: Option<ph2d_board_model::KanbanBoardView>,
    /// W4 — os quadros onde este Kanban está posto («Show on board»): `(quadro, elemento, nome)`.
    pub placements: Vec<(u64, u64, String)>,
    /// W6 — o filtro, os painéis e a edição rápida desta vista.
    pub w6: crate::w6::W6,
    /// W6 — as linhas que o último quadro mostrou (os espelhos e o filtro).
    pub rows: crate::rows::Rows,
    /// J1 — a vista à vista: `None` = o quadro; as outras são da `ph2d-kanban-views` (estado da VISTA).
    pub other: Option<ph2d_kanban_views::ViewKind>,
    pub views: ph2d_kanban_views::Views,
    /// J1 — px da vista → ecrã no último quadro, e onde o rato esteve (px da vista: a presença).
    pub to_screen: ph2d_vector::Affine,
    pub cursor: Option<(f32, f32)>,
}

/// O registo do pintor num quadro.
#[derive(Clone, Debug, Default)]
pub(crate) struct Painted {
    /// Cada título de lista pintado: `(lista, texto mostrado, tamanho, peso)`.
    pub titles: Vec<(ListId, String, f32, f32)>,
    /// A lista em voo: `(lista, rect no ecrã antes de inclinar, inclinação °, cartões pintados nela)`.
    pub flight: Option<(ListId, Rect, f64, Vec<CardId>)>,
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
            painted: std::cell::RefCell::default(),
            smoke_hold: None,
            back: None,
            auto: None,
            key: None,
            board_seen: None,
            placements: Vec::new(),
            w6: crate::w6::W6::default(),
            rows: crate::rows::Rows::default(),
            other: None,
            views: ph2d_kanban_views::Views::default(),
            to_screen: ph2d_vector::Affine::IDENTITY,
            cursor: None,
        }
    }

    /// A vista de um elemento do Quadro (id `key`, fora do espaço dos `KanbanId`).
    pub fn on_board(kanban: KanbanId, key: u64) -> Self {
        Self {
            key: Some(key),
            ..Self::new(kanban)
        }
    }

    /// O id da aba (o do Kanban) ou da vista de um elemento do Quadro.
    pub fn id(&self) -> u64 {
        self.key.unwrap_or(self.view.kanban.0)
    }

    pub fn scroll_y(&self, list: ListId) -> f32 {
        #[allow(clippy::cast_possible_truncation)]
        self.view.scroll_y.get(&list).map_or(0.0, |v| *v as f32)
    }
}

fn shift(r: Rect, dx: f32, dy: f32) -> Rect {
    Rect::new(r.x + dx, r.y + dy, r.w, r.h)
}

/// A origem das listas no ecrã: `(x 0 do conteúdo, topo das listas)`.
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn origin(tab: &Tab, board: Rect) -> (f32, f32) {
    (
        board.x + m::BOARD_PAD - tab.view.scroll_x as f32,
        board.y + m::BOARD_PAD,
    )
}

/// ⭐ A lista em voo está onde a mão a leva (`ghost` = o canto dela no ecrã): as molas saltam para
/// lá a cada quadro, e ao largar partem daí — ela assenta com mola, com os cartões dentro.
pub(crate) fn pin_list(springs: &mut Springs, id: ListId, ghost: (f32, f32), (ox, oy): (f32, f32)) {
    let (fx, fy) = springs.lists.entry(id).or_default();
    fx.jump(f64::from(ghost.0 - ox));
    fy.jump(f64::from(ghost.1 - oy));
}

/// O que todas as listas de um quadro partilham em [`place`].
struct Ctx<'a> {
    origin: (f32, f32),
    dt: f64,
    law: Option<Spring>,
    landed: Option<(CardId, f32, f32)>,
    /// Onde cada lista foi vista no quadro anterior (o cartão que muda de lista re-baseia-se).
    prev: &'a BTreeMap<ListId, (f32, f32)>,
}

/// Uma lista levada ao ecrã: a mola dela (ou a mão, se voa) e a dos cartões, relativa a ela.
#[allow(clippy::cast_possible_truncation)]
fn place_list(
    tab: &mut Tab,
    lb: ListBox,
    c: &Ctx<'_>,
    pin: Option<(f32, f32)>,
    moving: &mut bool,
) -> ListBox {
    let (ox, oy) = c.origin;
    {
        let (fx, fy) = tab.springs.lists.entry(lb.id).or_default();
        fx.aim(f64::from(lb.rect.x));
        fy.aim(f64::from(lb.rect.y - oy));
    }
    if let Some(g) = pin {
        pin_list(&mut tab.springs, lb.id, g, c.origin);
    } else if let Some((fx, fy)) = tab.springs.lists.get_mut(&lb.id) {
        *moving |= fx.advance(c.dt, c.law);
        *moving |= fy.advance(c.dt, c.law);
    }
    let (lx, ly) = tab
        .springs
        .lists
        .get(&lb.id)
        .map_or((lb.rect.x, lb.rect.y), |(fx, fy)| {
            (fx.pos as f32, oy + fy.pos as f32)
        });
    let (dx, dy) = (ox + lx - lb.rect.x, ly - lb.rect.y);
    let list_x = ox + lx;
    let body_y = lb.body.y + dy + m::LIST_BODY_TOP - tab.scroll_y(lb.id);
    tab.springs.seen.insert(lb.id, (lx, body_y));
    let mut cards = Vec::with_capacity(lb.cards.len());
    for cb in &lb.cards {
        let s = tab.springs.cards.entry(cb.id).or_default();
        if let Some(from) = s.list
            && from != lb.id
            && let (Some(a), Some(b)) = (c.prev.get(&from), c.prev.get(&lb.id))
        {
            s.x.pos += f64::from(a.0 - b.0);
            s.y.pos += f64::from(a.1 - b.1);
        }
        s.list = Some(lb.id);
        if let Some((id, x, y)) = c.landed
            && id == cb.id
        {
            s.x.jump(f64::from(x - list_x));
            s.y.jump(f64::from(y - body_y));
        }
        s.x.aim(f64::from(cb.rect.x - lb.rect.x));
        s.y.aim(f64::from(cb.rect.y));
        *moving |= s.x.advance(c.dt, c.law);
        *moving |= s.y.advance(c.dt, c.law);
        cards.push(CardBox {
            id: cb.id,
            rect: Rect::new(
                list_x + s.x.pos as f32,
                body_y + s.y.pos as f32,
                cb.rect.w,
                cb.rect.h,
            ),
        });
    }
    let card_dy = |r: Rect| Rect::new(r.x + dx, body_y + r.y, r.w, r.h);
    let s = |r: Rect| shift(r, dx, dy);
    ListBox {
        id: lb.id,
        rect: s(lb.rect),
        collapsed: lb.collapsed,
        title: s(lb.title),
        collapse: s(lb.collapse),
        menu: s(lb.menu),
        body: s(lb.body),
        content_h: lb.content_h,
        cards,
        slot: lb.slot.map(card_dy),
        composer: lb.composer.map(card_dy),
        add: s(lb.add),
        close: lb.close.map(s),
        count: lb.count,
        limit: lb.limit.map(s),
        total: lb.total,
        template: lb.template.map(s),
    }
}

/// ⭐ A disposição VISTA: mira as molas nos alvos, avança `dt` e leva tudo ao ecrã (com a rolagem).
/// A lista em voo vai aonde a mão a leva; as outras abrem-lhe o espaço com mola.
pub(crate) fn place(tab: &mut Tab, target: Layout, dt: f64, law: Option<Spring>) -> Layout {
    let ghost = tab
        .drag
        .filter(|d| matches!(d.lift, Lift::List(_)))
        .map(|d| (d.at.0 - d.grab.0, d.at.1 - d.grab.1));
    let prev = std::mem::take(&mut tab.springs.seen);
    let c = Ctx {
        origin: origin(tab, target.board),
        dt,
        law,
        landed: tab.landed.take(),
        prev: &prev,
    };
    let mut moving = false;
    let mut lists = Vec::with_capacity(target.lists.len());
    for lb in target.lists {
        lists.push(place_list(tab, lb, &c, None, &mut moving));
    }
    let flying = target
        .flying
        .map(|lb| place_list(tab, lb, &c, ghost, &mut moving));
    // O que já não está na disposição larga a mola (um cartão arquivado não volta a voar de lá).
    let alive_l: std::collections::BTreeSet<ListId> =
        lists.iter().chain(&flying).map(|l| l.id).collect();
    tab.springs.lists.retain(|id, _| alive_l.contains(id));
    let alive_c: std::collections::BTreeSet<CardId> = lists
        .iter()
        .chain(&flying)
        .flat_map(|l| l.cards.iter().map(|c| c.id))
        .collect();
    let lifted = match tab.drag.map(|d| d.lift) {
        Some(Lift::Card(id)) => Some(id),
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
    let ox = c.origin.0;
    Layout {
        area: target.area,
        header: target.header,
        title: target.title,
        star: target.star,
        board_menu: target.board_menu,
        board: target.board,
        lists,
        flying,
        list_slot: target.list_slot.map(|r| shift(r, ox, 0.0)),
        add_list: shift(target.add_list, ox, 0.0),
        add_list_field: target.add_list_field.map(|r| shift(r, ox, 0.0)),
        add_list_close: target.add_list_close.map(|r| shift(r, ox, 0.0)),
        content_w: target.content_w,
    }
}
