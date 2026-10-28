//! ⭐ A vista **Calendário** sobre uma [`Source`] — a de UM Kanban (W7) e a do projecto (W9, plano
//! §8.1): o MESMO widget, que só fala com a `Source`. Mês e semana; os itens pela entrega, o
//! intervalo início → entrega como barra; arrastar muda o dia (a hora fica); o «+» de um dia cria
//! um cartão nessa data, na lista que o compositor pergunta (a primeira por omissão, como o Trello).

mod input;
mod layout;
pub(crate) mod metrics;
mod paint;
mod pop;

pub use crate::item::Target;
pub use layout::{Day, Mode};

use ph2d_editor_core::doc_surface::KeyInput;
use ph2d_editor_core::widget::{Dropdown, LineField};
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_dates::{CivilDate, day_of};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

use crate::common::{Ctx, PointerAt, Reply};
use crate::item::{Item, ItemKey, Source};

/// O compositor do «+» de um dia: o título e a lista.
#[derive(Clone, Debug)]
pub(crate) struct Composer {
    pub day: CivilDate,
    pub field: LineField,
    pub list: Dropdown<usize>,
    pub targets: Vec<Target>,
}

/// O que se carregou num item (vira arrasto depois de [`metrics::DRAG_SLOP`]).
#[derive(Clone, Debug)]
pub(crate) struct Press {
    pub item: Item,
    /// O dia da célula onde se carregou: o deslocamento conta-se a partir dele.
    pub day: CivilDate,
    pub at: (f32, f32),
    /// O ponteiro dentro do pedaço agarrado, e o tamanho dele (o fantasma).
    pub grab: (f32, f32),
    pub size: (f32, f32),
    pub moved: bool,
}

/// As peças pintadas no último quadro (o rato lê-as).
#[derive(Clone, Debug, Default)]
pub(crate) struct Painted {
    /// O dia que fixou a vista pintada (a roda anda a partir dele).
    pub anchor: Option<CivilDate>,
    pub prev: Rect,
    pub next: Rect,
    pub today: Rect,
    pub month: Rect,
    pub week: Rect,
    pub cells: Vec<(Rect, CivilDate)>,
    /// Todos os itens da vista (o popover de um dia lê-os).
    pub items: Vec<Item>,
    /// Os pedaços à vista e o item de cada um.
    pub segs: Vec<(Rect, usize)>,
    pub more: Vec<(Rect, CivilDate, usize)>,
    pub plus: Option<(Rect, CivilDate)>,
    pub pop: pop::PopRects,
}

/// O estado do calendário (da VISTA).
#[derive(Debug, Default)]
pub struct CalendarState {
    pub mode: Mode,
    /// O dia que fixa o mês (ou a semana) à vista; `None` = hoje.
    pub anchor: Option<CivilDate>,
    pub(crate) hover: Option<(f32, f32)>,
    pub(crate) press: Option<Press>,
    pub(crate) composer: Option<Composer>,
    /// O popover com todos os itens de um dia (o «+N more»).
    pub(crate) day_pop: Option<CivilDate>,
    pub(crate) wheel: f32,
    pub(crate) painted: Painted,
}

impl CalendarState {
    /// O dia que fixa a vista.
    #[must_use]
    pub fn anchor_or_today(&self, cx: &Ctx<'_>) -> CivilDate {
        self.anchor.unwrap_or_else(|| day_of(cx.now.0, cx.zone))
    }

    /// O retângulo pintado da célula de `date` (px da vista).
    #[must_use]
    pub fn cell_rect(&self, date: CivilDate) -> Option<Rect> {
        self.painted
            .cells
            .iter()
            .find(|(_, d)| *d == date)
            .map(|(r, _)| *r)
    }

    /// Os pedaços pintados do item `key`.
    #[must_use]
    pub fn item_rects(&self, key: ItemKey) -> Vec<Rect> {
        self.painted
            .segs
            .iter()
            .filter(|(_, i)| self.painted.items.get(*i).is_some_and(|it| it.key == key))
            .map(|(r, _)| *r)
            .collect()
    }

    /// O «+N more» pintado no dia `date`: o retângulo e `N`.
    #[must_use]
    pub fn more_at(&self, date: CivilDate) -> Option<(Rect, usize)> {
        self.painted
            .more
            .iter()
            .find(|(_, d, _)| *d == date)
            .map(|(r, _, n)| (*r, *n))
    }

    /// O «+» à vista (o dia sob o rato).
    #[must_use]
    pub fn plus_rect(&self) -> Option<(Rect, CivilDate)> {
        self.painted.plus
    }

    /// O compositor está aberto, e para que dia.
    #[must_use]
    pub fn composing(&self) -> Option<CivilDate> {
        self.composer.as_ref().map(|c| c.day)
    }

    /// O popover de um dia está aberto, e para que dia.
    #[must_use]
    pub fn day_popover(&self) -> Option<CivilDate> {
        self.day_pop
    }
}

#[allow(clippy::too_many_arguments)]
pub fn paint(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    area: Rect,
    st: &mut CalendarState,
    src: &dyn Source,
    cx: &Ctx<'_>,
) {
    paint::paint(out, ts, theme, area, st, src, cx);
}

pub fn pointer(st: &mut CalendarState, src: &dyn Source, cx: &Ctx<'_>, p: PointerAt) -> Reply {
    input::pointer(st, src, cx, p)
}

pub fn wheel(st: &mut CalendarState, _at: (f32, f32), dy: f32, _shift: bool) -> Reply {
    input::wheel(st, dy)
}

pub fn key(
    st: &mut CalendarState,
    src: &dyn Source,
    cx: &Ctx<'_>,
    input: &KeyInput<'_>,
) -> Reply {
    input::key(st, src, cx, input)
}
