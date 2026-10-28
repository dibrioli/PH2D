//! ⭐ **O que o painel conta** — puro, sobre os cartões que o filtro deixa ver
//! ([`crate::common::visible_cards`]). As janelas da entrega são as do `ph2d-kanban-filter`
//! ([`due_in`]) partidas para não se sobreporem: cada cartão cai num só balde.

use ph2d_kanban_filter::{DueWindow, due_in};
use ph2d_kanban_model::{Card, Hue, Kanban, LabelColor, LabelId, ListId, Member};
use ph2d_tokens::{ColorToken as T, Theme};

/// ⭐ A ordem categórica das listas SEM cor — validada pela skill de dataviz
/// (`validate_palette.js`, 28/10): degrau `bold` no claro sobre `kanban-card` e `normal` no escuro
/// passam faixa de claridade, croma, separação CVD e o piso da visão normal entre vizinhas. Uma
/// lista sem cor além da série fica cinzenta (nunca um matiz reciclado). Tabela: `MEDICOES.md` §W7.
pub const SERIES: [Hue; 5] = [Hue::Yellow, Hue::Blue, Hue::Red, Hue::Purple, Hue::Lime];

use crate::common::{Ctx, avatar_hue, list_token, visible_cards};

/// Quem é uma barra (a cor segue a ENTIDADE, nunca a posição).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Who {
    /// A lista, a cor dela, e — se não tem — o matiz da série que lhe cabe ([`SERIES`]) e se esse
    /// matiz já é o de uma lista COM cor (então leva o outro degrau: duas listas nunca iguais).
    List(ListId, Option<Hue>, Option<(Hue, bool)>),
    Label(LabelId, Option<LabelColor>),
    Member(Member),
    Due(Bucket),
    /// Os cartões sem etiqueta / sem membro.
    None,
    /// O que não coube, dobrado numa barra só.
    Other,
}

impl Who {
    /// O token da marca no tema (o escuro e o claro levam degraus diferentes da paleta).
    #[must_use]
    pub fn token(&self, theme: Theme) -> T {
        match self {
            Self::List(_, Some(hue), _) => list_token(Some(*hue)),
            Self::List(_, None, series) => series.map_or(T::KanbanBlackSubtle, |(h, taken)| {
                // As listas COM cor vestem o `normal`; a série veste o `bold` no claro, e no escuro
                // também quando o matiz já é de uma lista com cor.
                let shade = if theme.is_dark() && !taken { 1 } else { 2 };
                ph2d_tokens::kanban::LABELS[shade]
                    [Hue::ALL.iter().position(|x| *x == h).unwrap_or(0)]
            }),
            Self::Label(_, Some(c)) => crate::common::label_token(*c),
            Self::Label(_, None) | Self::None | Self::Other => T::KanbanBlackSubtle,
            Self::Member(m) => crate::common::label_token(LabelColor {
                hue: avatar_hue(m),
                shade: ph2d_kanban_model::Shade::Normal,
            }),
            Self::Due(b) => b.token(),
        }
    }
}

/// Uma barra (ou fatia): o nome, a contagem e quem é.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bar {
    pub name: String,
    pub count: usize,
    pub who: Who,
}

/// Os baldes da entrega (o «Cards per due date» do Trello), pela ordem do gráfico.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bucket {
    Complete,
    Overdue,
    NextDay,
    NextWeek,
    Later,
    NoDates,
}

impl Bucket {
    pub const ALL: [Self; 6] = [
        Self::Complete,
        Self::Overdue,
        Self::NextDay,
        Self::NextWeek,
        Self::Later,
        Self::NoDates,
    ];

    /// O balde de `card`: concluído vence; depois as janelas do filtro, da mais curta à mais longa.
    #[must_use]
    pub fn of(card: &Card, cx: &Ctx<'_>) -> Self {
        match card.due {
            None => Self::NoDates,
            Some(d) if d.done => Self::Complete,
            Some(_) if due_in(DueWindow::Overdue, card, cx.now, cx.zone) => Self::Overdue,
            Some(_) if due_in(DueWindow::NextDay, card, cx.now, cx.zone) => Self::NextDay,
            Some(_) if due_in(DueWindow::NextWeek, card, cx.now, cx.zone) => Self::NextWeek,
            Some(_) => Self::Later,
        }
    }

    /// A cor de ESTADO (reservada: concluído, atrasado, em breve) ou um neutro.
    #[must_use]
    pub fn token(self) -> T {
        match self {
            Self::Complete => T::KanbanDueDone,
            Self::Overdue => T::KanbanDueOver,
            Self::NextDay => T::KanbanDueSoon,
            Self::NextWeek => T::KanbanSky,
            Self::Later => T::KanbanBlueSubtle,
            Self::NoDates => T::KanbanBlackSubtle,
        }
    }

    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::Complete => "kanban.views.dashboard.due.complete",
            Self::Overdue => "kanban.views.dashboard.due.overdue",
            Self::NextDay => "kanban.views.dashboard.due.next_day",
            Self::NextWeek => "kanban.views.dashboard.due.next_week",
            Self::Later => "kanban.views.dashboard.due.later",
            Self::NoDates => "kanban.views.dashboard.due.none",
        }
    }
}

/// Os quatro gráficos do painel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Chart {
    PerList,
    PerLabel,
    PerMember,
    PerDue,
}

impl Chart {
    pub const ALL: [Self; 4] = [Self::PerList, Self::PerLabel, Self::PerMember, Self::PerDue];

    /// As categorias PARTEM os cartões (cada um num só)? Só aí um anel diz a verdade.
    #[must_use]
    pub fn partitions(self) -> bool {
        matches!(self, Self::PerList | Self::PerDue)
    }

    #[must_use]
    pub fn title_key(self) -> &'static str {
        match self {
            Self::PerList => "kanban.views.dashboard.per_list",
            Self::PerLabel => "kanban.views.dashboard.per_label",
            Self::PerMember => "kanban.views.dashboard.per_member",
            Self::PerDue => "kanban.views.dashboard.per_due",
        }
    }
}

/// Tudo o que o painel mostra.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub total: usize,
    pub per_list: Vec<Bar>,
    pub per_label: Vec<Bar>,
    pub per_member: Vec<Bar>,
    pub per_due: Vec<Bar>,
}

impl Stats {
    #[must_use]
    pub fn bars(&self, c: Chart) -> &[Bar] {
        match c {
            Chart::PerList => &self.per_list,
            Chart::PerLabel => &self.per_label,
            Chart::PerMember => &self.per_member,
            Chart::PerDue => &self.per_due,
        }
    }

    /// A contagem de um balde da entrega.
    #[must_use]
    pub fn due(&self, b: Bucket) -> usize {
        self.per_due
            .iter()
            .find(|x| x.who == Who::Due(b))
            .map_or(0, |x| x.count)
    }
}

/// ⭐ Conta os cartões à vista de `k`.
#[must_use]
pub fn stats(k: &Kanban, cx: &Ctx<'_>) -> Stats {
    let cards = visible_cards(k, cx);
    let uncolored: Vec<ListId> = k
        .lists_in_order()
        .into_iter()
        .filter(|l| l.color.is_none())
        .map(|l| l.id)
        .collect();
    let own: Vec<Hue> = k.lists_in_order().iter().filter_map(|l| l.color).collect();
    let per_list = k
        .lists_in_order()
        .into_iter()
        .map(|l| Bar {
            name: l.title.clone(),
            count: cards.iter().filter(|c| c.list == l.id).count(),
            who: Who::List(
                l.id,
                l.color,
                uncolored
                    .iter()
                    .position(|x| *x == l.id)
                    .and_then(|i| SERIES.get(i))
                    .map(|h| (*h, own.contains(h))),
            ),
        })
        .collect();
    let mut labels: Vec<_> = k.labels.values().filter(|l| !l.meta.deleted).collect();
    labels.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
    let mut per_label: Vec<Bar> = labels
        .into_iter()
        .map(|l| Bar {
            name: l.name.clone(),
            count: cards.iter().filter(|c| c.labels.contains(&l.id)).count(),
            who: Who::Label(l.id, l.color),
        })
        .collect();
    per_label.sort_by_key(|a| std::cmp::Reverse(a.count));
    push_none(
        &mut per_label,
        cards.iter().filter(|c| c.labels.is_empty()).count(),
        "kanban.views.dashboard.no_label",
    );
    let mut per_member: Vec<Bar> = crate::common::people(k, cx)
        .into_iter()
        .map(|m| Bar {
            name: m.0.clone(),
            count: cards.iter().filter(|c| c.members.contains(&m)).count(),
            who: Who::Member(m),
        })
        .collect();
    per_member.sort_by_key(|a| std::cmp::Reverse(a.count));
    push_none(
        &mut per_member,
        cards.iter().filter(|c| c.members.is_empty()).count(),
        "kanban.views.dashboard.no_member",
    );
    let per_due = Bucket::ALL
        .into_iter()
        .map(|b| Bar {
            name: ph2d_i18n::tr(b.key()).to_owned(),
            count: cards.iter().filter(|c| Bucket::of(c, cx) == b).count(),
            who: Who::Due(b),
        })
        .collect();
    Stats {
        total: cards.len(),
        per_list,
        per_label,
        per_member,
        per_due,
    }
}

fn push_none(v: &mut Vec<Bar>, n: usize, key: &str) {
    if n > 0 {
        v.push(Bar {
            name: ph2d_i18n::tr(key).to_owned(),
            count: n,
            who: Who::None,
        });
    }
}

/// Cabem `rows` barras: as primeiras `rows - 1` e o resto dobrado em «Other» (nunca uma barra a
/// mais do que cabe, nunca uma cor inventada).
#[must_use]
pub fn fold(bars: &[Bar], rows: usize) -> Vec<Bar> {
    if bars.len() <= rows {
        return bars.to_vec();
    }
    if rows == 0 {
        return Vec::new();
    }
    let keep = rows - 1;
    let mut out: Vec<Bar> = bars[..keep].to_vec();
    out.push(Bar {
        name: ph2d_i18n::tr_with(
            "kanban.views.dashboard.other",
            &[("n", &(bars.len() - keep))],
        ),
        count: bars[keep..].iter().map(|b| b.count).sum(),
        who: Who::Other,
    });
    out
}
