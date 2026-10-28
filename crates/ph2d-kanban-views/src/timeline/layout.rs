//! As faixas e as filas da Linha do tempo (puras): que cartões vão em que faixa, em que fila, de que
//! dia a que dia; e a régua dia ↔ px.

use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_dates::{CivilDate, Zone, day_of};
use ph2d_kanban_model::{Card, CardId, Kanban, LabelId, ListId, Member};

use super::GroupBy;
use crate::common::{Ctx, people, visible_cards};

/// A chave de uma faixa.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LaneKey {
    List(ListId),
    Member(Member),
    Label(LabelId),
    NoMember,
    NoLabel,
}

/// Os dias de um cartão na linha do tempo, inclusive (só a entrega = um dia).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub card: CardId,
    pub start: CivilDate,
    pub end: CivilDate,
}

/// Uma faixa: o título e os cartões já arrumados em filas sem sobreposição.
#[derive(Clone, Debug, PartialEq)]
pub struct Lane {
    pub key: LaneKey,
    pub title: String,
    pub rows: Vec<Vec<Span>>,
}

impl Lane {
    /// Quantos cartões a faixa tem.
    #[must_use]
    pub fn count(&self) -> usize {
        self.rows.iter().map(Vec::len).sum()
    }
}

/// Os dias de `c` no fuso, se ele tem alguma data.
#[must_use]
pub fn span(c: &Card, zone: &Zone) -> Option<Span> {
    let end = c.due.map(|d| d.at).or(c.start)?;
    let end = day_of(end.0, zone);
    let start = c
        .start
        .map(|s| day_of(s.0, zone))
        .filter(|s| *s <= end)
        .unwrap_or(end);
    Some(Span {
        card: c.id,
        start,
        end,
    })
}

/// Arruma os intervalos em filas: cada um na primeira fila cujo último acabou antes dele.
fn pack(mut spans: Vec<Span>) -> Vec<Vec<Span>> {
    spans.sort_by(|a, b| a.start.cmp(&b.start).then(a.end.cmp(&b.end)));
    let mut rows: Vec<Vec<Span>> = Vec::new();
    for s in spans {
        match rows
            .iter_mut()
            .find(|r| r.last().is_some_and(|l| l.end < s.start))
        {
            Some(r) => r.push(s),
            None => rows.push(vec![s]),
        }
    }
    rows
}

/// As faixas do agrupamento `group` e os cartões sem datas (a gaveta), pelo filtro da aba.
#[must_use]
pub fn lanes(k: &Kanban, cx: &Ctx<'_>, group: GroupBy) -> (Vec<Lane>, Vec<CardId>) {
    let cards = visible_cards(k, cx);
    let undated = cards
        .iter()
        .filter(|c| c.start.is_none() && c.due.is_none())
        .map(|c| c.id)
        .collect();
    let dated: Vec<(&Card, Span)> = cards
        .iter()
        .filter_map(|c| span(c, cx.zone).map(|s| (*c, s)))
        .collect();
    let lane = |key: LaneKey, title: String, keep: &dyn Fn(&Card) -> bool| Lane {
        key,
        title,
        rows: pack(
            dated
                .iter()
                .filter(|(c, _)| keep(c))
                .map(|(_, s)| *s)
                .collect(),
        ),
    };
    let mut out = Vec::new();
    match group {
        GroupBy::List => {
            for l in k.lists_in_order() {
                out.push(lane(LaneKey::List(l.id), l.title.clone(), &|c| {
                    c.list == l.id
                }));
            }
        }
        GroupBy::Member => {
            for m in people(k, cx) {
                let has = dated.iter().any(|(c, _)| c.members.contains(&m));
                if has {
                    let key = LaneKey::Member(m.clone());
                    out.push(lane(key, m.0.clone(), &|c| c.members.contains(&m)));
                }
            }
            out.push(lane(
                LaneKey::NoMember,
                tr("kanban.views.timeline.no_members").to_owned(),
                &|c| c.members.is_empty(),
            ));
        }
        GroupBy::Label => {
            let mut labels: Vec<_> = k.labels.values().filter(|l| !l.meta.deleted).collect();
            labels.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
            for l in labels {
                let has = dated.iter().any(|(c, _)| c.labels.contains(&l.id));
                if has {
                    let title = if l.name.is_empty() {
                        tr("kanban.views.timeline.unnamed_label").to_owned()
                    } else {
                        l.name.clone()
                    };
                    out.push(lane(LaneKey::Label(l.id), title, &|c| {
                        c.labels.contains(&l.id)
                    }));
                }
            }
            out.push(lane(
                LaneKey::NoLabel,
                tr("kanban.views.timeline.no_labels").to_owned(),
                &|c| {
                    c.labels
                        .iter()
                        .all(|l| !k.labels.get(l).is_some_and(|x| !x.meta.deleted))
                },
            ));
        }
    }
    (out, undated)
}

/// A régua: o dia ↔ o px, sobre a grelha de barras.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ruler {
    /// A grelha das barras (à direita da coluna dos nomes, abaixo da escala).
    pub grid: Rect,
    /// O dia de índice `0`.
    pub anchor: CivilDate,
    pub ppd: f32,
    /// Px da rolagem horizontal (a partir do `anchor`).
    pub scroll_x: f32,
}

impl Ruler {
    /// O índice do dia `d` (dias desde o `anchor`).
    #[must_use]
    pub fn index(&self, d: CivilDate) -> i64 {
        self.anchor.days_until(d)
    }

    /// O x do começo do dia `d`.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn x_of(&self, d: CivilDate) -> f32 {
        self.grid.x + self.index(d) as f32 * self.ppd - self.scroll_x
    }

    /// O índice do dia sob `x`.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn index_at(&self, x: f32) -> i64 {
        ((x - self.grid.x + self.scroll_x) / self.ppd).floor() as i64
    }

    /// O dia sob `x`.
    #[must_use]
    pub fn day_at(&self, x: f32) -> CivilDate {
        self.anchor.add_days(self.index_at(x))
    }

    /// A barra de `s` (x e largura).
    #[must_use]
    pub fn bar_x(&self, s: &Span) -> (f32, f32) {
        let x0 = self.x_of(s.start);
        (x0, self.x_of(s.end.add_days(1)) - x0)
    }
}
