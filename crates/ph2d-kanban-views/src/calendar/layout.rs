//! ⭐ A disposição do calendário — PURA: os dias à vista, a geometria das células e onde cada item
//! cai (faixas por semana, «+N more» pelo espaço MEDIDO da célula).

use ph2d_editor_core::zones::Rect;
use ph2d_kanban_dates::{CalendarMonth, CivilDate, Weekday, Zone, day_of};

use super::metrics as m;
use crate::item::Item;

/// Mês, semana e dia (os dois com a grelha de horas) ou a agenda (a lista dos próximos).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Mode {
    #[default]
    Month,
    Week,
    Day,
    Agenda,
}

impl Mode {
    pub const ALL: [Self; 4] = [Self::Month, Self::Week, Self::Day, Self::Agenda];

    /// Semana e dia: a grelha de horas.
    #[must_use]
    pub fn hours(self) -> bool {
        matches!(self, Self::Week | Self::Day)
    }
}

/// Um dia à vista.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Day {
    pub date: CivilDate,
    /// Do mês mostrado (no modo semana, todos).
    pub in_month: bool,
    pub today: bool,
}

/// O primeiro dia da semana de `d` que começa em `ws`.
#[must_use]
pub fn week_of(d: CivilDate, ws: Weekday) -> CivilDate {
    let back = (7 + i64::from(d.weekday().monday_index()) - i64::from(ws.monday_index())) % 7;
    d.add_days(-back)
}

/// Os dias à vista: as semanas do mês de `anchor` (sem a última se for toda do mês seguinte) ou a
/// semana de `anchor`.
#[must_use]
pub fn days(mode: Mode, anchor: CivilDate, today: CivilDate, ws: Weekday) -> Vec<Day> {
    match mode {
        Mode::Month => {
            let Some(month) = CalendarMonth::new(anchor.year, anchor.month, Some(today), ws) else {
                return Vec::new();
            };
            let mut v: Vec<Day> = month
                .cells()
                .iter()
                .filter_map(|c| {
                    Some(Day {
                        date: CivilDate::new(c.year, c.month, c.day)?,
                        in_month: c.in_month,
                        today: c.is_today,
                    })
                })
                .collect();
            while v.len() > 7 && v[v.len() - 7..].iter().all(|d| !d.in_month) {
                v.truncate(v.len() - 7);
            }
            v
        }
        Mode::Week | Mode::Day | Mode::Agenda => {
            let (first, n) = match mode {
                Mode::Week => (week_of(anchor, ws), 7),
                Mode::Day => (anchor, 1),
                _ => (anchor, m::AGENDA_DAYS),
            };
            (0..n)
                .map(|i| {
                    let date = first.add_days(i);
                    Day {
                        date,
                        in_month: true,
                        today: date == today,
                    }
                })
                .collect()
        }
    }
}

/// A geometria: o cabeçalho, a fila dos dias da semana e a grelha.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Geo {
    pub header: Rect,
    pub weekdays: Rect,
    pub grid: Rect,
    pub rows: usize,
}

impl Geo {
    #[must_use]
    pub fn of(area: Rect, n_days: usize) -> Self {
        let x = area.x + m::PAD;
        let w = (area.w - 2.0 * m::PAD).max(0.0);
        let header = Rect::new(x, area.y, w, m::HEADER_H);
        let weekdays = Rect::new(x, header.y + header.h, w, m::WEEKDAY_H);
        let top = weekdays.y + weekdays.h;
        let grid = Rect::new(x, top, w, (area.y + area.h - m::PAD - top).max(0.0));
        Self {
            header,
            weekdays,
            grid,
            rows: n_days.div_ceil(7).max(1),
        }
    }

    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn cell(&self, i: usize) -> Rect {
        let (cw, ch) = (self.grid.w / 7.0, self.grid.h / self.rows as f32);
        let (r, c) = (i / 7, i % 7);
        Rect::new(
            self.grid.x + c as f32 * cw,
            self.grid.y + r as f32 * ch,
            cw,
            ch,
        )
    }

    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn cell_h(&self) -> f32 {
        self.grid.h / self.rows as f32
    }
}

/// Um pedaço de um item numa semana: as colunas `c0..=c1` da fila `row`, na faixa `lane`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seg {
    pub item: usize,
    pub row: usize,
    pub c0: usize,
    pub c1: usize,
    pub lane: usize,
    /// O item começa / acaba neste pedaço (a esquina redonda).
    pub starts: bool,
    pub ends: bool,
}

/// Onde os itens caem: os pedaços à vista e, por dia, quantos ficaram de fora.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Placed {
    pub segs: Vec<Seg>,
    /// `(índice do dia, escondidos)`, só os dias com algum escondido.
    pub more: Vec<(usize, usize)>,
}

/// O intervalo de dias (índices em `days`) que o item ocupa, preso à vista, e se ele começa e acaba
/// dentro dela.
fn span(days: &[Day], it: &Item, zone: &Zone) -> Option<(usize, usize, bool, bool)> {
    let first = days.first()?.date;
    let n = i64::try_from(days.len()).ok()?;
    let a = first.days_until(day_of(it.start.unwrap_or(it.end).0, zone));
    let b = first.days_until(day_of(it.end.0, zone)).max(a);
    if b < 0 || a >= n {
        return None;
    }
    Some((
        usize::try_from(a.max(0)).ok()?,
        usize::try_from(b.min(n - 1)).ok()?,
        a >= 0,
        b < n,
    ))
}

/// Quantas faixas de `lane_h` cabem numa célula de `cell_h` (com ou sem a linha «+N more»).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[must_use]
pub fn lanes_fit(cell_h: f32, lane_h: f32, with_more: bool) -> usize {
    let mut avail = cell_h - m::DAY_NUM_H - m::CELL_PAD;
    if with_more {
        avail -= m::MORE_H;
    }
    ((avail + m::LANE_GAP) / (lane_h + m::LANE_GAP)).max(0.0).floor() as usize
}

/// ⭐ Põe os itens nas faixas: por semana, os longos primeiro; um pedaço só se vê se a sua faixa
/// cabe — senão conta no «+N more» de cada dia que ele toca.
#[must_use]
pub fn place(days: &[Day], items: &[Item], zone: &Zone, cell_h: f32, lane_h: f32) -> Placed {
    let rows = days.len().div_ceil(7);
    let mut raw: Vec<Seg> = Vec::new();
    for (i, it) in items.iter().enumerate() {
        let Some((a, b, in_a, in_b)) = span(days, it, zone) else {
            continue;
        };
        for row in a / 7..=b / 7 {
            let (lo, hi) = (a.max(row * 7), b.min(row * 7 + 6));
            raw.push(Seg {
                item: i,
                row,
                c0: lo % 7,
                c1: hi % 7,
                lane: 0,
                starts: in_a && lo == a,
                ends: in_b && hi == b,
            });
        }
    }
    raw.sort_by(|x, y| {
        (x.row, x.c0)
            .cmp(&(y.row, y.c0))
            .then((y.c1 - y.c0).cmp(&(x.c1 - x.c0)))
            .then(items[x.item].end.cmp(&items[y.item].end))
            .then(items[x.item].title.cmp(&items[y.item].title))
    });
    let mut used: Vec<[Vec<bool>; 7]> = (0..rows).map(|_| Default::default()).collect();
    for s in &mut raw {
        let cols = &mut used[s.row];
        let mut lane = 0;
        while (s.c0..=s.c1).any(|c| cols[c].get(lane).copied().unwrap_or(false)) {
            lane += 1;
        }
        for col in &mut cols[s.c0..=s.c1] {
            if col.len() <= lane {
                col.resize(lane + 1, false);
            }
            col[lane] = true;
        }
        s.lane = lane;
    }
    let all = lanes_fit(cell_h, lane_h, false);
    let some = lanes_fit(cell_h, lane_h, true);
    let mut hidden = vec![0usize; days.len()];
    let mut segs = Vec::new();
    for row in 0..rows {
        let need = raw
            .iter()
            .filter(|s| s.row == row)
            .map(|s| s.lane + 1)
            .max()
            .unwrap_or(0);
        let limit = if need <= all { all } else { some };
        for s in raw.iter().filter(|s| s.row == row) {
            if s.lane < limit {
                segs.push(*s);
            } else {
                for c in s.c0..=s.c1 {
                    hidden[row * 7 + c] += 1;
                }
            }
        }
    }
    let more = hidden
        .into_iter()
        .enumerate()
        .filter(|(_, n)| *n > 0)
        .collect();
    Placed { segs, more }
}

/// Os itens que tocam o dia `date`, pela ordem da vista (os longos primeiro, depois pela hora).
#[must_use]
pub fn on_day<'a>(items: &'a [Item], date: CivilDate, zone: &Zone) -> Vec<&'a Item> {
    let mut v: Vec<&Item> = items
        .iter()
        .filter(|it| {
            day_of(it.start.unwrap_or(it.end).0, zone) <= date && date <= day_of(it.end.0, zone)
        })
        .collect();
    v.sort_by(|a, b| a.end.cmp(&b.end).then(a.title.cmp(&b.title)));
    v
}
