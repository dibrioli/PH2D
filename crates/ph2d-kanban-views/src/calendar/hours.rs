//! ⭐ A grelha de HORAS (semana e dia) — PURA: que itens vão à faixa «dia inteiro» e quais são
//! blocos de tempo, onde cada bloco cai (a hora de início, a altura pela duração, lado a lado
//! quando se sobrepõem) e as contas hora ↔ px com o encaixe de 15 min.

use ph2d_editor_core::zones::Rect;
use ph2d_kanban_dates::{CivilDate, Zone, civil, day_of, instant};
use ph2d_kanban_model::Moment;

use super::layout::Day;
use super::metrics as m;
use crate::item::Item;

pub(crate) const DAY_MIN: f32 = 24.0 * 60.0;
const MIN_PER_HOUR: f32 = 60.0;
const MS_PER_MIN: i64 = 60_000;

/// Como um item entra na grelha de horas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    /// Vários dias, ou um dia sem hora (meia-noite e sem início): a faixa de cima.
    AllDay,
    /// Um bloco: do início (no mesmo dia da entrega) à entrega; sem início, um bloco curto na entrega.
    Timed { top: Moment, has_start: bool },
}

#[must_use]
pub(crate) fn kind(it: &Item, zone: &Zone) -> Kind {
    let end_day = day_of(it.end.0, zone);
    match it.start {
        Some(s) if day_of(s.0, zone) == end_day => Kind::Timed {
            top: s,
            has_start: true,
        },
        Some(_) => Kind::AllDay,
        None => {
            let c = civil(it.end.0, zone);
            if c.hour == 0 && c.minute == 0 {
                Kind::AllDay
            } else {
                Kind::Timed {
                    top: it.end,
                    has_start: false,
                }
            }
        }
    }
}

/// Os minutos desde a meia-noite local de `ms`.
#[must_use]
pub(crate) fn minutes(ms: i64, zone: &Zone) -> f32 {
    let c = civil(ms, zone);
    f32::from(c.hour) * MIN_PER_HOUR + f32::from(c.minute)
}

/// `min` minutos encaixados a 15, dentro do dia.
#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub(crate) fn snap(min: f32) -> i64 {
    let s = m::SNAP_MIN;
    let n = (f64::from(min) / s as f64).round() as i64 * s;
    n.clamp(0, (DAY_MIN as i64) - s)
}

/// O instante local `date` + `min` minutos (`None` se não existir).
#[must_use]
pub(crate) fn at(date: CivilDate, min: i64, zone: &Zone) -> Option<Moment> {
    let (h, mi) = (u8::try_from(min / 60).ok()?, u8::try_from(min % 60).ok()?);
    instant(date, h, mi, zone).map(Moment)
}

/// A duração em ms.
#[must_use]
pub(crate) fn mins_ms(min: i64) -> i64 {
    min * MS_PER_MIN
}

/// Onde a grelha está pintada: o y da meia-noite e as colunas dos dias.
#[derive(Clone, Debug, Default)]
pub(crate) struct HourGeo {
    /// A janela das horas (sem a coluna das horas).
    pub body: Rect,
    /// O y da meia-noite (o topo das horas menos a rolagem).
    pub top: f32,
    /// Cada dia: a coluna visível das horas e a data.
    pub cols: Vec<(Rect, CivilDate)>,
    pub zone: Option<Zone>,
}

impl HourGeo {
    #[must_use]
    pub fn y_of(&self, min: f32) -> f32 {
        self.top + min / MIN_PER_HOUR * m::HOUR_H
    }

    #[must_use]
    pub fn min_of(&self, y: f32) -> f32 {
        (y - self.top) / m::HOUR_H * MIN_PER_HOUR
    }

    /// A coluna sob `x` (dentro das horas ou da faixa).
    #[must_use]
    pub fn col_at(&self, x: f32) -> Option<(usize, CivilDate)> {
        self.cols
            .iter()
            .position(|(r, _)| x >= r.x && x < r.x + r.w)
            .map(|i| (i, self.cols[i].1))
    }

    /// ⭐ O instante sob `(x, y)` nas horas, encaixado a 15 min (`None` fora delas).
    #[must_use]
    pub fn time_at(&self, x: f32, y: f32) -> Option<Moment> {
        if !self.body.contains(x, y) {
            return None;
        }
        let (_, date) = self.col_at(x)?;
        at(date, snap(self.min_of(y)), self.zone.as_ref()?)
    }
}

/// Um bloco de tempo posto na grelha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Block {
    pub item: usize,
    pub col: usize,
    pub top_min: f32,
    /// O fundo PINTADO (pelo menos a altura mínima medida).
    pub bot_min: f32,
    pub lane: usize,
    pub lanes: usize,
    pub has_start: bool,
}

/// ⭐ Divide os itens: os blocos (por coluna, lado a lado quando se tocam) e os de «dia inteiro».
#[must_use]
pub(crate) fn place(days: &[Day], items: &[Item], zone: &Zone, min_minutes: f32) -> (Vec<Block>, Vec<usize>) {
    let mut all_day = Vec::new();
    let mut blocks: Vec<Block> = Vec::new();
    for (i, it) in items.iter().enumerate() {
        match kind(it, zone) {
            Kind::AllDay => all_day.push(i),
            Kind::Timed { top, has_start } => {
                let d = day_of(top.0, zone);
                let Some(col) = days.iter().position(|x| x.date == d) else {
                    continue;
                };
                let t = minutes(top.0, zone);
                let e = if has_start {
                    let span = (it.end.0 - top.0) as f32 / (MS_PER_MIN as f32);
                    t + span.max(0.0)
                } else {
                    t
                };
                blocks.push(Block {
                    item: i,
                    col,
                    top_min: t,
                    bot_min: e.max(t + min_minutes).min(DAY_MIN),
                    lane: 0,
                    lanes: 1,
                    has_start,
                });
            }
        }
    }
    blocks.sort_by(|a, b| {
        (a.col, a.top_min)
            .partial_cmp(&(b.col, b.top_min))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.bot_min.partial_cmp(&a.bot_min).unwrap_or(std::cmp::Ordering::Equal))
    });
    let mut start = 0;
    while start < blocks.len() {
        let col = blocks[start].col;
        let mut ends: Vec<f32> = Vec::new();
        let mut reach = f32::NEG_INFINITY;
        let mut end = start;
        while end < blocks.len() && blocks[end].col == col && (end == start || blocks[end].top_min < reach) {
            let b = &mut blocks[end];
            let lane = ends.iter().position(|e| *e <= b.top_min).unwrap_or(ends.len());
            if lane == ends.len() {
                ends.push(b.bot_min);
            } else {
                ends[lane] = b.bot_min;
            }
            b.lane = lane;
            reach = reach.max(b.bot_min);
            end += 1;
        }
        for b in &mut blocks[start..end] {
            b.lanes = ends.len();
        }
        start = end;
    }
    (blocks, all_day)
}

/// O retângulo de um bloco (lado a lado: a coluna partida pelas faixas do grupo).
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub(crate) fn block_rect(g: &HourGeo, b: &Block) -> Rect {
    let col = g.cols.get(b.col).map_or(g.body, |(r, _)| *r);
    let inner = col.w - 2.0 * m::BLOCK_GAP;
    let w = inner / b.lanes as f32;
    let y0 = g.y_of(b.top_min);
    let y1 = g.y_of(b.bot_min);
    Rect::new(
        col.x + m::BLOCK_GAP + w * b.lane as f32,
        y0,
        (w - m::BLOCK_GAP).max(0.0),
        (y1 - y0 - m::BLOCK_GAP).max(0.0),
    )
}
