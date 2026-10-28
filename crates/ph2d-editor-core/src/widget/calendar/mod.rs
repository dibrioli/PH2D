//! [`Calendar`] — a grelha de um mês (6 × 7) que o CHAMADOR entrega.
//!
//! O widget não faz contas de calendário: as 42 células (`{ano, mês, dia, do mês, hoje}`), o
//! título do mês e os nomes dos dias da semana chegam de fora (a crate de datas e a i18n do
//! chamador). Ele guarda a escolha, o intervalo realçado e o foco do teclado, e devolve
//! [`CalendarEvent`]s — mudar de mês é um pedido ao chamador, que responde com
//! [`Calendar::set_month`].

mod paint;

pub use paint::{CalendarHit, CalendarLayout, cell_node_id, paint_calendar};

use crate::doc_surface::Key;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_host::Modifiers;

/// Colunas da grelha (os dias de uma semana).
pub const CALENDAR_COLS: usize = 7;
/// Linhas da grelha — seis semanas cobrem qualquer mês, comece em que dia começar.
pub const CALENDAR_ROWS: usize = 6;
/// Células da grelha.
pub const CALENDAR_CELLS: usize = CALENDAR_COLS * CALENDAR_ROWS;

/// Um dia civil. A ordem derivada (ano, mês, dia) é a cronológica — o widget compara, não soma.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CalendarDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl CalendarDate {
    #[must_use]
    pub const fn new(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }
}

/// Uma célula da grelha, tal como o chamador a calculou.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct CalendarCell {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    /// Pertence ao mês mostrado (as outras são os dias vizinhos, esbatidos).
    pub in_month: bool,
    pub is_today: bool,
}

impl CalendarCell {
    #[must_use]
    pub const fn date(&self) -> CalendarDate {
        CalendarDate::new(self.year, self.month, self.day)
    }
}

/// O que o calendário pede ao chamador.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CalendarEvent {
    /// O dia escolhido (clique ou `Enter`).
    Pick(CalendarDate),
    /// Mostrar o mês anterior (seta, `PageUp` ou o foco a sair pelo topo).
    PrevMonth,
    /// Mostrar o mês seguinte.
    NextMonth,
    /// Voltar ao mês de hoje.
    Today,
}

/// Para onde o foco vai quando o chamador entregar o mês novo.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Pending {
    /// `delta` dias a partir de uma data que as duas grelhas partilham.
    Shift(CalendarDate, isize),
    /// O mesmo número de dia no mês novo (o `PageUp`/`PageDown`), preso ao último dia dele.
    SameDay(u8),
}

#[derive(Clone, Debug)]
pub struct Calendar {
    pub id: NodeId,
    /// O título do mês, já traduzido («outubro de 2026»).
    pub title: String,
    /// Os nomes curtos dos dias, pela ordem das colunas (o início da semana é do chamador).
    pub weekdays: [String; CALENDAR_COLS],
    /// Rótulos dos três botões do cabeçalho: anterior, seguinte, hoje.
    pub prev_label: String,
    pub next_label: String,
    pub today_label: String,
    pub cells: [CalendarCell; CALENDAR_CELLS],
    pub selected: Option<CalendarDate>,
    /// O intervalo realçado, inclusivo nas duas pontas (início → entrega).
    pub range: Option<(CalendarDate, CalendarDate)>,
    /// A célula com o foco do teclado.
    pub focus: Option<CalendarDate>,
    /// O calendário tem o foco do teclado (pinta o anel).
    pub focused: bool,
    /// A célula/botão sob o rato.
    pub hover: Option<CalendarHit>,
    pending: Option<Pending>,
}

impl Calendar {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            title: String::new(),
            weekdays: Default::default(),
            prev_label: String::new(),
            next_label: String::new(),
            today_label: String::new(),
            cells: [CalendarCell::default(); CALENDAR_CELLS],
            selected: None,
            range: None,
            focus: None,
            focused: false,
            hover: None,
            pending: None,
        }
    }

    /// Os rótulos do cabeçalho e os nomes dos dias (i18n do chamador).
    #[must_use]
    pub fn labels(
        mut self,
        weekdays: [String; CALENDAR_COLS],
        prev: impl Into<String>,
        next: impl Into<String>,
        today: impl Into<String>,
    ) -> Self {
        self.weekdays = weekdays;
        self.prev_label = prev.into();
        self.next_label = next.into();
        self.today_label = today.into();
        self
    }

    /// Entrega o mês a mostrar. Um foco pedido por uma tecla que saiu da grelha anterior pousa
    /// agora; sem pedido, o foco que já não está na grelha vai para o dia escolhido ou para hoje.
    pub fn set_month(&mut self, title: impl Into<String>, cells: [CalendarCell; CALENDAR_CELLS]) {
        self.title = title.into();
        self.cells = cells;
        let landed = match self.pending.take() {
            Some(Pending::Shift(anchor, delta)) => self
                .index_of(anchor)
                .and_then(|i| i.checked_add_signed(delta))
                .filter(|i| *i < CALENDAR_CELLS),
            Some(Pending::SameDay(day)) => {
                self.cells.iter().rposition(|c| c.in_month && c.day <= day)
            }
            None => self.focus.and_then(|f| self.index_of(f)),
        };
        self.focus = Some(
            landed
                .or_else(|| self.selected.and_then(|s| self.index_of(s)))
                .or_else(|| self.cells.iter().position(|c| c.is_today))
                .or_else(|| self.cells.iter().position(|c| c.in_month))
                .map_or(self.cells[0].date(), |i| self.cells[i].date()),
        );
    }

    #[must_use]
    pub fn index_of(&self, d: CalendarDate) -> Option<usize> {
        self.cells.iter().position(|c| c.date() == d)
    }

    /// `d` está dentro do intervalo realçado.
    #[must_use]
    pub fn in_range(&self, d: CalendarDate) -> bool {
        self.range.is_some_and(|(a, b)| {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            lo <= d && d <= hi
        })
    }

    /// Um clique em `(x, y)` dentro de `rect`.
    pub fn pointer_down(
        &mut self,
        rect: crate::zones::Rect,
        x: f32,
        y: f32,
    ) -> Option<CalendarEvent> {
        match CalendarLayout::of(rect).hit(x, y)? {
            CalendarHit::Cell(i) => {
                let d = self.cells[i].date();
                self.focus = Some(d);
                self.selected = Some(d);
                Some(CalendarEvent::Pick(d))
            }
            CalendarHit::Prev => Some(CalendarEvent::PrevMonth),
            CalendarHit::Next => Some(CalendarEvent::NextMonth),
            CalendarHit::Today => Some(CalendarEvent::Today),
        }
    }

    /// O rato a passar: devolve `true` quando o realce mudou (pede um quadro).
    pub fn pointer_move(&mut self, rect: crate::zones::Rect, x: f32, y: f32) -> bool {
        let h = CalendarLayout::of(rect).hit(x, y);
        let changed = h != self.hover;
        self.hover = h;
        changed
    }

    /// Uma tecla com o calendário focado. `(tomou a tecla, evento)`.
    pub fn key(&mut self, key: Key, mods: Modifiers) -> (bool, Option<CalendarEvent>) {
        let _ = mods;
        let cols = CALENDAR_COLS as isize;
        let step = match key {
            Key::Left => -1,
            Key::Right => 1,
            Key::Up => -cols,
            Key::Down => cols,
            Key::PageUp | Key::PageDown => {
                let day = self.focus.map_or(1, |f| f.day);
                self.pending = Some(Pending::SameDay(day));
                let ev = if key == Key::PageUp {
                    CalendarEvent::PrevMonth
                } else {
                    CalendarEvent::NextMonth
                };
                return (true, Some(ev));
            }
            Key::Enter | Key::Space => {
                let Some(d) = self.focus else {
                    return (false, None);
                };
                self.selected = Some(d);
                return (true, Some(CalendarEvent::Pick(d)));
            }
            _ => return (false, None),
        };
        let from = self
            .focus
            .and_then(|f| self.index_of(f))
            .unwrap_or_default();
        match from
            .checked_add_signed(step)
            .filter(|i| *i < CALENDAR_CELLS)
        {
            Some(i) => {
                self.focus = Some(self.cells[i].date());
                (true, None)
            }
            None => {
                // Sai da grelha: o mês vizinho partilha a célula de partida.
                self.pending = Some(Pending::Shift(self.cells[from].date(), step));
                let ev = if step < 0 {
                    CalendarEvent::PrevMonth
                } else {
                    CalendarEvent::NextMonth
                };
                (true, Some(ev))
            }
        }
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Grid)
            .label(&self.title)
            .bounds(x, y, w, h)
            .focusable(true)
            .action(Action::Focus)
            .build()
    }
}
