//! `ph2d-kanban-views` — as vistas de um Kanban além do quadro (TrelloClone W7, o «Views» do Trello
//! Premium): **Tabela**, **Calendário**, **Linha do tempo** e **Painel**. Sem Mapa (dono 27/10).
//!
//! O estado de cada vista ([`Views`]) é da VISTA, nunca do documento. Uma vista pinta e responde ao
//! rato e ao teclado; o que muda o documento volta num [`Reply::plan`] que o CHAMADOR aplica
//! (`apply_logged` por Kanban, ou `KanbanSet::apply_plan`) — a vista nunca escreve.
//!
//! O Calendário lê [`Item`]s de uma [`Source`]: aqui a de UM Kanban ([`KanbanSource`]); a W9 junta
//! todos os Kanbans e os eventos do projecto noutra `Source`, sobre o MESMO widget (plano §8.1).

mod common;
pub mod datepick;
#[doc(hidden)]
pub mod fixture;
mod item;

pub mod calendar;
pub mod dashboard;
pub mod table;
pub mod timeline;

pub use common::{Ctx, PointerAt, PointerKind, Reply};
pub use item::{Item, ItemKey, ItemTone, KanbanSource, Source, Target};

use ph2d_editor_core::doc_surface::KeyInput;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{KanbanId, KanbanSet};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{Affine, Point, VectorScene};

/// As vistas que o seletor oferece além do Quadro.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ViewKind {
    #[default]
    Table,
    Calendar,
    Timeline,
    Dashboard,
}

impl ViewKind {
    pub const ALL: [Self; 4] = [Self::Table, Self::Calendar, Self::Timeline, Self::Dashboard];

    /// O nome no seletor (pela i18n).
    #[must_use]
    pub fn label(self) -> &'static str {
        ph2d_i18n::tr(match self {
            Self::Table => "kanban.views.table",
            Self::Calendar => "kanban.views.calendar",
            Self::Timeline => "kanban.views.timeline",
            Self::Dashboard => "kanban.views.dashboard",
        })
    }
}

/// ⭐ O estado das quatro vistas de UMA aba (ou de um elemento do Quadro) e qual está à vista.
#[derive(Debug, Default)]
pub struct Views {
    pub kind: ViewKind,
    pub table: table::TableState,
    pub calendar: calendar::CalendarState,
    pub timeline: timeline::TimelineState,
    pub dashboard: dashboard::DashboardState,
    /// Px da vista → ecrã, do último quadro pintado (o rato volta por ela).
    to_screen: Affine,
}

impl Views {
    /// Pinta a vista à vista na área `rect` (px da vista), levada ao ecrã por `to_screen`.
    #[allow(clippy::too_many_arguments)]
    pub fn paint(
        &mut self,
        scene: &mut VectorScene,
        to_screen: Affine,
        rect: Rect,
        set: &KanbanSet,
        kanban: KanbanId,
        ts: &mut TextSystem,
        theme: Theme,
        cx: &Ctx<'_>,
    ) {
        self.to_screen = to_screen;
        let identity = to_screen == Affine::IDENTITY;
        let mut local = VectorScene::new();
        let out = if identity { &mut *scene } else { &mut local };
        match self.kind {
            ViewKind::Table => table::paint(out, ts, theme, rect, &mut self.table, set, kanban, cx),
            ViewKind::Calendar => {
                let src = KanbanSource::new(set, kanban, cx);
                calendar::paint(out, ts, theme, rect, &mut self.calendar, &src, cx);
            }
            ViewKind::Timeline => {
                timeline::paint(out, ts, theme, rect, &mut self.timeline, set, kanban, cx);
            }
            ViewKind::Dashboard => {
                dashboard::paint(out, ts, theme, rect, &mut self.dashboard, set, kanban, cx);
            }
        }
        if !identity {
            scene.append_transformed(&local, to_screen);
        }
    }

    /// O ponto do ecrã em px da vista.
    #[must_use]
    pub fn to_view(&self, x: f32, y: f32) -> (f32, f32) {
        let p = self.to_screen.inverse() * Point::new(f64::from(x), f64::from(y));
        #[allow(clippy::cast_possible_truncation)]
        (p.x as f32, p.y as f32)
    }

    /// Carregar, mexer ou largar em `(x, y)` do ECRÃ.
    pub fn pointer(
        &mut self,
        set: &KanbanSet,
        kanban: KanbanId,
        cx: &Ctx<'_>,
        kind: PointerKind,
        x: f32,
        y: f32,
    ) -> Reply {
        let (x, y) = self.to_view(x, y);
        let p = PointerAt { kind, x, y };
        match self.kind {
            ViewKind::Table => table::pointer(&mut self.table, set, kanban, cx, p),
            ViewKind::Calendar => {
                let src = KanbanSource::new(set, kanban, cx);
                calendar::pointer(&mut self.calendar, &src, cx, p)
            }
            ViewKind::Timeline => timeline::pointer(&mut self.timeline, set, kanban, cx, p),
            ViewKind::Dashboard => dashboard::pointer(&mut self.dashboard, set, kanban, cx, p),
        }
    }

    /// A roda em `(x, y)` do ECRÃ: `dy` vertical, `shift` = horizontal (o idioma do hero), `ctrl` =
    /// zoom (a linha do tempo).
    pub fn wheel(
        &mut self,
        set: &KanbanSet,
        kanban: KanbanId,
        cx: &Ctx<'_>,
        (x, y): (f32, f32),
        dy: f32,
        (shift, ctrl): (bool, bool),
    ) -> Reply {
        let (x, y) = self.to_view(x, y);
        match self.kind {
            ViewKind::Table => table::wheel(&mut self.table, (x, y), dy, shift),
            ViewKind::Calendar => calendar::wheel(&mut self.calendar, (x, y), dy, shift),
            ViewKind::Timeline => {
                timeline::wheel(&mut self.timeline, set, kanban, cx, (x, y), dy, (shift, ctrl))
            }
            ViewKind::Dashboard => Reply::default(),
        }
    }

    /// Uma tecla.
    pub fn key(
        &mut self,
        set: &KanbanSet,
        kanban: KanbanId,
        cx: &Ctx<'_>,
        input: &KeyInput<'_>,
    ) -> Reply {
        match self.kind {
            ViewKind::Table => table::key(&mut self.table, set, kanban, cx, input),
            ViewKind::Calendar => {
                let src = KanbanSource::new(set, kanban, cx);
                calendar::key(&mut self.calendar, &src, cx, input)
            }
            ViewKind::Timeline => timeline::key(&mut self.timeline, set, kanban, cx, input),
            ViewKind::Dashboard => Reply::default(),
        }
    }
}
