//! ⭐ A vista **Calendário** sobre uma [`Source`] — a de UM Kanban (W7) e a do projecto (W9, plano
//! §8.1): o MESMO widget, que só fala com a `Source`. Mês, semana, dia e agenda. No mês os itens
//! caem pela entrega e o intervalo início → entrega é uma barra; arrastar muda o dia (a hora fica);
//! o «+» de um dia cria um cartão nessa data, na lista que o compositor pergunta (a primeira por
//! omissão, como o Trello). Semana e dia têm a grelha de HORAS: a faixa «dia inteiro», os blocos
//! pela hora e pela duração, arrastar move (encaixe de 15 min), a borda de baixo estica o fim e
//! arrastar no vazio cria. A agenda lista os próximos itens, por dia.

mod agenda;
mod hours;
mod hours_input;
mod hours_paint;
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
use ph2d_kanban_model::Moment;
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
    /// Na grelha de horas: o início e a entrega do bloco arrastado (ou só a entrega, num clique).
    pub start: Option<Moment>,
    pub due: Option<Moment>,
    /// Onde o painel abre (junto ao bloco); `None` = junto ao dia.
    pub at_y: Option<f32>,
}

/// Um gesto na grelha de horas.
#[derive(Clone, Debug)]
pub(crate) enum HourDrag {
    /// Mover um bloco: `grab` = os minutos entre o topo dele e o rato.
    Move { item: Item, top: Moment, has_start: bool, grab: f32 },
    /// Esticar o fim (a borda de baixo); `top` fica.
    Resize { item: Item, top: Moment },
    /// Arrastar no vazio: um bloco novo desde `from` (minutos) no dia `day`.
    Create { day: CivilDate, from: f32 },
}

#[derive(Clone, Debug)]
pub(crate) struct Hold {
    pub drag: HourDrag,
    pub at: (f32, f32),
    pub moved: bool,
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
    /// Os botões do seletor de modo.
    pub modes: Vec<(Rect, Mode)>,
    pub cells: Vec<(Rect, CivilDate)>,
    /// A grelha de horas pintada (semana e dia).
    pub hours: Option<hours::HourGeo>,
    pub blocks: Vec<(Rect, hours::Block)>,
    /// As linhas da agenda (pela ordem) e a janela dela.
    pub agenda: Vec<(Rect, usize)>,
    pub agenda_view: Rect,
    pub agenda_h: f32,
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
    /// A rolagem das horas (`None` = ainda não abriu: abre nas 8:00 ou na hora actual).
    pub(crate) scroll: Option<f32>,
    pub(crate) agenda_scroll: f32,
    pub(crate) hold: Option<Hold>,
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

    /// Os pedaços pintados do item `key` (barras, pastilhas e blocos de tempo).
    #[must_use]
    pub fn item_rects(&self, key: ItemKey) -> Vec<Rect> {
        let is = |i: usize| self.painted.items.get(i).is_some_and(|it| it.key == key);
        let segs = self.painted.segs.iter().filter(|(_, i)| is(*i)).map(|(r, _)| *r);
        let blocks = self.painted.blocks.iter().filter(|(_, b)| is(b.item)).map(|(r, _)| *r);
        segs.chain(blocks).collect()
    }

    /// ⭐ O instante sob `(x, y)` na grelha de horas pintada (semana ou dia), encaixado a 15 min;
    /// `None` fora das horas ou no mês e na agenda.
    #[must_use]
    pub fn time_at(&self, x: f32, y: f32) -> Option<Moment> {
        self.painted.hours.as_ref()?.time_at(x, y)
    }

    /// As linhas da agenda pintadas, pela ordem: o retângulo e o item.
    #[must_use]
    pub fn agenda_rows(&self) -> Vec<(Rect, ItemKey)> {
        self.painted
            .agenda
            .iter()
            .filter_map(|(r, i)| Some((*r, self.painted.items.get(*i)?.key)))
            .collect()
    }

    /// O y de `hour:minute` na grelha de horas pintada (para quem quer apontar uma hora).
    #[must_use]
    pub fn hour_y(&self, hour: u8, minute: u8) -> Option<f32> {
        let g = self.painted.hours.as_ref()?;
        Some(g.y_of(f32::from(hour) * 60.0 + f32::from(minute)))
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

/// ⭐ O instante sob `(x, y)` na grelha de horas (o Planner larga um bloco aí).
#[must_use]
pub fn time_at(st: &CalendarState, x: f32, y: f32) -> Option<Moment> {
    st.time_at(x, y)
}

pub fn key(
    st: &mut CalendarState,
    src: &dyn Source,
    cx: &Ctx<'_>,
    input: &KeyInput<'_>,
) -> Reply {
    input::key(st, src, cx, input)
}
