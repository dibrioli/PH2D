//! ⭐ A vista **Linha do tempo** (a Timeline do Trello): faixas por lista, membro ou etiqueta; barras
//! início → entrega numa escala de dias com zoom Dia/Semana/Mês/Trimestre e a linha de hoje; arrastar
//! a barra move as duas datas, arrastar uma ponta muda só essa; os cartões sem datas esperam numa
//! gaveta e ganham datas ao serem largados num dia. A vista só pede: o lote volta no [`Reply`].

mod drawer;
mod gesture;
mod layout;
pub(crate) mod metrics;
mod paint;
mod scale;

pub use gesture::{Part, clamp_delta, moved, plan, rescheduled};
pub use layout::{Lane, LaneKey, Ruler, Span, lanes, span};

use ph2d_editor_core::doc_surface::{Key, KeyInput};
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_dates::{CivilDate, day_of};
use ph2d_kanban_model::{CardId, CardRef, KanbanId, KanbanSet};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

use crate::common::{Ctx, PointerAt, PointerKind, Reply};
use metrics as m;

/// O zoom da escala.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Zoom {
    Day,
    #[default]
    Week,
    Month,
    Quarter,
}

impl Zoom {
    pub const ALL: [Self; 4] = [Self::Day, Self::Week, Self::Month, Self::Quarter];

    /// Px por dia.
    #[must_use]
    pub fn ppd(self) -> f32 {
        m::PX_PER_DAY[self as usize]
    }

    fn key(self) -> &'static str {
        match self {
            Self::Day => "kanban.views.timeline.zoom.day",
            Self::Week => "kanban.views.timeline.zoom.week",
            Self::Month => "kanban.views.timeline.zoom.month",
            Self::Quarter => "kanban.views.timeline.zoom.quarter",
        }
    }

    fn step(self, inward: bool) -> Self {
        let i = self as usize;
        let j = if inward {
            i.saturating_sub(1)
        } else {
            (i + 1).min(Self::ALL.len() - 1)
        };
        Self::ALL[j]
    }
}

/// O agrupamento das faixas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GroupBy {
    #[default]
    List,
    Member,
    Label,
}

impl GroupBy {
    pub const ALL: [Self; 3] = [Self::List, Self::Member, Self::Label];

    fn key(self) -> &'static str {
        match self {
            Self::List => "kanban.views.timeline.group.list",
            Self::Member => "kanban.views.timeline.group.member",
            Self::Label => "kanban.views.timeline.group.label",
        }
    }
}

/// Um botão da barra de ferramentas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Zoom(Zoom),
    Group(GroupBy),
    Today,
    Drawer,
    DrawerClose,
}

/// Uma barra pintada.
#[derive(Clone, Debug, PartialEq)]
pub struct Bar {
    pub card: CardId,
    pub rect: Rect,
    pub lane: LaneKey,
    pub span: Span,
}

/// O que o último quadro pintou (o rato lê ISTO, nunca uma segunda conta).
#[derive(Clone, Debug, Default)]
pub struct Painted {
    pub ruler: Option<Ruler>,
    pub buttons: Vec<(Button, Rect)>,
    pub bars: Vec<Bar>,
    /// Cada faixa: a banda inteira (nomes + grelha), em px da vista.
    pub lanes: Vec<(LaneKey, Rect)>,
    /// A janela das faixas (rola na vertical).
    pub body: Rect,
    pub content_h: f32,
    pub drawer: Option<Rect>,
    pub drawer_cards: Vec<(CardId, Rect)>,
    pub drawer_content_h: f32,
}

impl Painted {
    /// As barras do cartão `card` (uma por faixa em que ele aparece).
    pub fn bars_of(&self, card: CardId) -> impl Iterator<Item = &Bar> {
        self.bars.iter().filter(move |b| b.card == card)
    }

    /// O rect de um botão.
    #[must_use]
    pub fn button(&self, b: Button) -> Option<Rect> {
        self.buttons.iter().find(|(x, _)| *x == b).map(|(_, r)| *r)
    }

    fn lane_at(&self, y: f32) -> Option<&LaneKey> {
        self.lanes
            .iter()
            .find(|(_, r)| r.y <= y && y < r.y + r.h)
            .map(|(k, _)| k)
    }
}

/// O que a mão leva.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Grab {
    Card(CardId, Part),
    Pan { scroll: (f32, f32) },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Drag {
    pub grab: Grab,
    pub from: (f32, f32),
    pub at: (f32, f32),
    pub moved: bool,
}

/// O estado da linha do tempo (da VISTA).
#[derive(Debug)]
pub struct TimelineState {
    pub zoom: Zoom,
    pub group: GroupBy,
    pub drawer_open: bool,
    /// Px a partir do dia âncora (o «hoje» do primeiro quadro).
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub drawer_scroll: f32,
    anchor: Option<CivilDate>,
    painted: Painted,
    pub(crate) drag: Option<Drag>,
    pub(crate) hover: Option<(CardId, Part)>,
}

impl Default for TimelineState {
    fn default() -> Self {
        Self {
            zoom: Zoom::default(),
            group: GroupBy::default(),
            drawer_open: true,
            scroll_x: 0.0,
            scroll_y: 0.0,
            drawer_scroll: 0.0,
            anchor: None,
            painted: Painted::default(),
            drag: None,
            hover: None,
        }
    }
}

impl TimelineState {
    /// O que o último quadro pintou.
    #[must_use]
    pub fn painted(&self) -> &Painted {
        &self.painted
    }

    /// Muda o zoom mantendo o dia do meio no meio.
    pub fn set_zoom(&mut self, z: Zoom) {
        if let Some(r) = self.painted.ruler {
            let mid = (self.scroll_x + r.grid.w / 2.0) / r.ppd;
            self.scroll_x = mid * z.ppd() - r.grid.w / 2.0;
        }
        self.zoom = z;
    }

    /// Põe hoje no meio da grelha (no próximo quadro).
    pub fn go_today(&mut self) {
        self.anchor = None;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn paint(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    area: Rect,
    st: &mut TimelineState,
    set: &KanbanSet,
    kanban: KanbanId,
    cx: &Ctx<'_>,
) {
    let Some(k) = set.get(kanban) else {
        return;
    };
    paint::paint(out, ts, theme, area, st, k, cx);
}

/// Hoje como âncora, com hoje a um terço da grelha (o Trello abre com hoje à vista e o futuro à
/// frente).
fn settle_anchor(st: &mut TimelineState, cx: &Ctx<'_>, grid_w: f32) {
    if st.anchor.is_none() {
        st.anchor = Some(day_of(cx.now.0, cx.zone));
        st.scroll_x = -grid_w / 3.0;
    }
}

fn press(st: &mut TimelineState, p: PointerAt) -> Reply {
    let at = (p.x, p.y);
    if let Some(b) = st
        .painted
        .buttons
        .iter()
        .find(|(_, r)| r.contains(p.x, p.y))
        .map(|(b, _)| *b)
    {
        match b {
            Button::Zoom(z) => st.set_zoom(z),
            Button::Group(g) => {
                st.group = g;
                st.scroll_y = 0.0;
            }
            Button::Today => st.go_today(),
            Button::Drawer => st.drawer_open = !st.drawer_open,
            Button::DrawerClose => st.drawer_open = false,
        }
        return Reply::taken();
    }
    let grab = if let Some((id, _)) = st
        .painted
        .drawer_cards
        .iter()
        .find(|(_, r)| r.contains(p.x, p.y))
    {
        Grab::Card(*id, Part::Drawer)
    } else if let Some(b) = st
        .painted
        .bars
        .iter()
        .rev()
        .find(|b| b.rect.contains(p.x, p.y))
    {
        Grab::Card(b.card, part_at(&b.rect, p.x))
    } else if st.painted.body.contains(p.x, p.y) {
        Grab::Pan {
            scroll: (st.scroll_x, st.scroll_y),
        }
    } else {
        return Reply::default();
    };
    st.drag = Some(Drag {
        grab,
        from: at,
        at,
        moved: false,
    });
    Reply::taken()
}

/// A parte da barra sob `x`: as pontas são pegas de [`m::HANDLE_W`] (numa barra estreita, o meio
/// fica para o corpo).
fn part_at(r: &Rect, x: f32) -> Part {
    let h = m::HANDLE_W.min(r.w / 3.0);
    if x < r.x + h {
        Part::Start
    } else if x > r.x + r.w - h {
        Part::End
    } else {
        Part::Body
    }
}

fn release(st: &mut TimelineState, set: &KanbanSet, kanban: KanbanId, cx: &Ctx<'_>) -> Reply {
    let Some(d) = st.drag.take() else {
        return Reply::default();
    };
    let Grab::Card(card, part) = d.grab else {
        return Reply::taken();
    };
    if !d.moved {
        if part == Part::Drawer {
            return Reply::taken();
        }
        return Reply::open(CardRef { kanban, card });
    }
    let (Some(r), Some(k)) = (st.painted.ruler, set.get(kanban)) else {
        return Reply::taken();
    };
    if part == Part::Drawer && !r.grid.contains(d.at.0, d.at.1) {
        return Reply::taken();
    }
    let delta = r.index_at(d.at.0) - r.index_at(d.from.0);
    let lane = st.painted.lane_at(d.at.1).cloned();
    let plan = plan(
        k,
        kanban,
        card,
        part,
        delta,
        r.day_at(d.at.0),
        lane.as_ref(),
        cx.zone,
    );
    Reply::with_plan(plan)
}

pub fn pointer(
    st: &mut TimelineState,
    set: &KanbanSet,
    kanban: KanbanId,
    cx: &Ctx<'_>,
    p: PointerAt,
) -> Reply {
    match p.kind {
        PointerKind::Down => press(st, p),
        PointerKind::Up => release(st, set, kanban, cx),
        PointerKind::Move => {
            if let Some(d) = st.drag.as_mut() {
                d.at = (p.x, p.y);
                let (dx, dy) = (p.x - d.from.0, p.y - d.from.1);
                d.moved |= dx.abs() > m::DRAG_SLOP || dy.abs() > m::DRAG_SLOP;
                if let Grab::Pan { scroll } = d.grab {
                    st.scroll_x = scroll.0 - dx;
                    st.scroll_y = scroll.1 - dy;
                }
                return Reply::taken();
            }
            let h = st
                .painted
                .bars
                .iter()
                .rev()
                .find(|b| b.rect.contains(p.x, p.y))
                .map(|b| (b.card, part_at(&b.rect, p.x)));
            if h == st.hover {
                return Reply::default();
            }
            st.hover = h;
            Reply {
                redraw: true,
                ..Reply::default()
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn wheel(
    st: &mut TimelineState,
    _set: &KanbanSet,
    _kanban: KanbanId,
    _cx: &Ctx<'_>,
    at: (f32, f32),
    dy: f32,
    shift: bool,
) -> Reply {
    let step = dy * m::WHEEL_PX;
    if st.painted.drawer.is_some_and(|r| r.contains(at.0, at.1)) {
        st.drawer_scroll += step;
    } else if shift {
        st.scroll_x += step;
    } else {
        st.scroll_y += step;
    }
    Reply::taken()
}

pub fn key(
    st: &mut TimelineState,
    _set: &KanbanSet,
    _kanban: KanbanId,
    _cx: &Ctx<'_>,
    input: &KeyInput<'_>,
) -> Reply {
    if !input.pressed {
        return Reply::default();
    }
    match input.key {
        Key::Escape if st.drag.is_some() => {
            st.drag = None;
            Reply::taken()
        }
        Key::Char('+' | '=') if input.mods.ctrl => {
            st.set_zoom(st.zoom.step(true));
            Reply::taken()
        }
        Key::Char('-') if input.mods.ctrl => {
            st.set_zoom(st.zoom.step(false));
            Reply::taken()
        }
        Key::Char('t' | 'T') if !input.mods.ctrl => {
            st.go_today();
            Reply::taken()
        }
        _ => Reply::default(),
    }
}
