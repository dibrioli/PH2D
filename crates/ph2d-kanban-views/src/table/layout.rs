//! As colunas, a ordem das linhas e a geometria da tabela (o que pinta e o que o rato lê é a MESMA
//! conta: [`Grid`]).

use std::cmp::Ordering;

use ph2d_editor_core::zones::Rect;
use ph2d_kanban_filter::normalize;
use ph2d_kanban_model::{Card, CardId, FieldId, FieldKind, FieldValue, Kanban};

use super::metrics as m;
use crate::common::{Ctx, visible_cards};

/// Uma coluna da tabela.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Column {
    Card,
    List,
    Labels,
    Members,
    Due,
    /// Um campo personalizado.
    Field(FieldId),
}

impl Column {
    #[must_use]
    pub fn width(self) -> f32 {
        match self {
            Self::Card => m::CARD_W,
            Self::List => m::LIST_W,
            Self::Labels => m::LABELS_W,
            Self::Members => m::MEMBERS_W,
            Self::Due => m::DUE_W,
            Self::Field(_) => m::FIELD_W,
        }
    }
}

/// As colunas de `k`: as cinco do Trello, depois os campos pela ordem deles.
#[must_use]
pub fn columns(k: &Kanban) -> Vec<Column> {
    let mut v = vec![
        Column::Card,
        Column::List,
        Column::Labels,
        Column::Members,
        Column::Due,
    ];
    let mut fields: Vec<_> = k.fields.values().filter(|f| !f.meta.deleted).collect();
    fields.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
    v.extend(fields.into_iter().map(|f| Column::Field(f.id)));
    v
}

/// A ordem pedida: uma coluna e o sentido (`desc`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sort {
    pub by: Column,
    pub desc: bool,
}

/// A chave de ordenação de um cartão numa coluna, calculada UMA vez por cartão (comparar com
/// `normalize` dentro do `sort_by` custava `799 ms` a 20 000 cartões em debug; ver `MEDICOES.md`).
#[derive(Clone, Debug, PartialEq)]
enum Key {
    Text(String),
    Int(i64),
    Num(f64),
}

impl Key {
    fn order(&self, o: &Self) -> Ordering {
        match (self, o) {
            (Self::Text(a), Self::Text(b)) => a.cmp(b),
            (Self::Int(a), Self::Int(b)) => a.cmp(b),
            (Self::Num(a), Self::Num(b)) => a.total_cmp(b),
            _ => Ordering::Equal,
        }
    }
}

fn first_label(k: &Kanban, c: &Card) -> Option<String> {
    c.labels
        .iter()
        .filter_map(|l| k.labels.get(l))
        .filter(|l| !l.meta.deleted)
        .map(|l| normalize(&l.name))
        .min()
}

#[allow(clippy::cast_possible_wrap)]
fn field_key(k: &Kanban, f: FieldId, c: &Card) -> Option<Key> {
    Some(match c.fields.get(&f)? {
        FieldValue::Number(x) => Key::Num(*x),
        FieldValue::Date(x) => Key::Int(x.0),
        FieldValue::Checkbox(x) => Key::Int(i64::from(!*x)),
        FieldValue::Text(x) => Key::Text(normalize(x)),
        FieldValue::Option(o) => match k.fields.get(&f).map(|d| &d.kind) {
            Some(FieldKind::Dropdown(opts)) => Key::Int(opts.iter().position(|p| p.id == *o)? as i64),
            _ => return None,
        },
    })
}

/// A chave de `c` na coluna `by` (`None` = vazio: fica no fim nos dois sentidos).
#[allow(clippy::cast_possible_wrap)]
fn key(k: &Kanban, lists: &[ph2d_kanban_model::ListId], by: Column, c: &Card) -> Option<Key> {
    match by {
        Column::Card => Some(Key::Text(normalize(&c.title))),
        Column::List => lists.iter().position(|l| *l == c.list).map(|i| Key::Int(i as i64)),
        Column::Labels => first_label(k, c).map(Key::Text),
        Column::Members => c.members.iter().map(|m| normalize(&m.0)).min().map(Key::Text),
        Column::Due => c.due.map(|d| Key::Int(d.at.0)),
        Column::Field(f) => field_key(k, f, c),
    }
}

/// ⭐ As linhas da tabela: os cartões à vista (o filtro), pela ordem do quadro ou pela pedida. A
/// ordem é estável: o que empata fica pela ordem do quadro; `desc` inverte só a comparação (os
/// vazios continuam no fim, como no Trello).
#[must_use]
pub fn rows(k: &Kanban, cx: &Ctx<'_>, sort: Option<Sort>) -> Vec<CardId> {
    let cards = visible_cards(k, cx);
    let Some(s) = sort else {
        return cards.into_iter().map(|c| c.id).collect();
    };
    let lists: Vec<_> = k.lists_in_order().into_iter().map(|l| l.id).collect();
    let mut keyed: Vec<(Option<Key>, CardId)> =
        cards.into_iter().map(|c| (key(k, &lists, s.by, c), c.id)).collect();
    keyed.sort_by(|(a, _), (b, _)| match (a, b) {
        (Some(a), Some(b)) => {
            let o = a.order(b);
            if s.desc { o.reverse() } else { o }
        }
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    });
    keyed.into_iter().map(|(_, id)| id).collect()
}

/// ⭐ A geometria de um quadro de imagem da tabela.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Grid {
    /// O cartão branco da tabela.
    pub sheet: Rect,
    pub header: Rect,
    /// A área das linhas (abaixo do cabeçalho).
    pub body: Rect,
    /// Cada coluna e o `x` (já com a rolagem horizontal) e a largura.
    pub cols: Vec<(Column, f32, f32)>,
    pub scroll_y: f32,
    pub scroll_x: f32,
    pub rows: usize,
}

impl Grid {
    #[must_use]
    pub fn new(area: Rect, cols: &[Column], rows: usize, scroll: (f32, f32)) -> Self {
        let sheet = Rect::new(
            area.x + m::PAD,
            area.y + m::PAD,
            (area.w - 2.0 * m::PAD).max(0.0),
            (area.h - 2.0 * m::PAD).max(0.0),
        );
        let header = Rect::new(sheet.x, sheet.y, sheet.w, m::HEADER_H.min(sheet.h));
        let body = Rect::new(
            sheet.x,
            header.y + header.h,
            sheet.w,
            (sheet.h - header.h).max(0.0),
        );
        let total_w: f32 = cols.iter().map(|c| c.width()).sum();
        let mut g = Self {
            sheet,
            header,
            body,
            cols: Vec::new(),
            scroll_y: 0.0,
            scroll_x: 0.0,
            rows,
        };
        g.scroll_y = scroll.1.clamp(0.0, g.max_scroll_y());
        g.scroll_x = scroll.0.clamp(0.0, (total_w - sheet.w).max(0.0));
        let mut x = sheet.x - g.scroll_x;
        for c in cols {
            g.cols.push((*c, x, c.width()));
            x += c.width();
        }
        g
    }

    /// A altura de todas as linhas.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn content_h(&self) -> f32 {
        self.rows as f32 * m::ROW_H
    }

    #[must_use]
    pub fn max_scroll_y(&self) -> f32 {
        (self.content_h() - self.body.h).max(0.0)
    }

    /// ⭐ As linhas que se veem (rolagem virtual): `[first, end)`.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn visible(&self) -> std::ops::Range<usize> {
        let first = (self.scroll_y / m::ROW_H).floor().max(0.0) as usize;
        let end = ((self.scroll_y + self.body.h) / m::ROW_H).ceil().max(0.0) as usize;
        first.min(self.rows)..end.min(self.rows)
    }

    /// O rect da linha `i` (pode estar fora do corpo).
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn row(&self, i: usize) -> Rect {
        Rect::new(
            self.body.x,
            self.body.y + i as f32 * m::ROW_H - self.scroll_y,
            self.body.w,
            m::ROW_H,
        )
    }

    /// A célula `(linha, coluna)`.
    #[must_use]
    pub fn cell(&self, i: usize, col: Column) -> Option<Rect> {
        let r = self.row(i);
        self.cols
            .iter()
            .find(|c| c.0 == col)
            .map(|&(_, x, w)| Rect::new(x, r.y, w, r.h))
    }

    /// A coluna sob `x`.
    #[must_use]
    pub fn col_at(&self, x: f32) -> Option<Column> {
        if x < self.sheet.x || x > self.sheet.x + self.sheet.w {
            return None;
        }
        self.cols
            .iter()
            .find(|&&(_, cx, w)| x >= cx && x < cx + w)
            .map(|c| c.0)
    }

    /// A linha sob `y` (só dentro do corpo).
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn row_at(&self, y: f32) -> Option<usize> {
        if !(self.body.y..self.body.y + self.body.h).contains(&y) {
            return None;
        }
        let i = ((y - self.body.y + self.scroll_y) / m::ROW_H).floor();
        (i >= 0.0 && (i as usize) < self.rows).then_some(i as usize)
    }
}
