//! ⭐ Editar no lugar: os popovers de uma célula e o lote que cada escolha pede. A tabela NUNCA
//! escreve: cada função devolve um [`Plan`] com o cartão inteiro reposto (`KanbanOp::put`).

use ph2d_editor_core::widget::{Dropdown, DropdownOption};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_model::{
    Card, CardId, Due, FieldId, FieldKind, FieldValue, Kanban, KanbanId, KanbanOp, LabelId,
    ListId, Member, Moment, OptionId, Plan, Record,
};

use super::metrics as m;
use crate::datepick::{Picker, node};

/// O popover aberto sobre uma célula.
#[derive(Clone, Debug)]
pub enum Pop {
    /// A lista do cartão (o `Dropdown` do editor-core, ancorado na célula).
    List {
        card: CardId,
        dd: Dropdown<ListId>,
        chip: Rect,
    },
    /// As etiquetas do Kanban, cada uma alterna.
    Labels { card: CardId, at: Rect },
    /// As pessoas, cada uma alterna.
    Members { card: CardId, at: Rect },
    /// As datas pelo `DatePicker`.
    Dates {
        card: CardId,
        picker: Box<Picker>,
        at: Rect,
    },
    /// Um campo de escolha (o `Dropdown`; `None` = limpar).
    Option {
        card: CardId,
        field: FieldId,
        dd: Dropdown<Option<OptionId>>,
        chip: Rect,
    },
}

impl Pop {
    #[must_use]
    pub fn card(&self) -> CardId {
        match self {
            Self::List { card, .. }
            | Self::Labels { card, .. }
            | Self::Members { card, .. }
            | Self::Dates { card, .. }
            | Self::Option { card, .. } => *card,
        }
    }
}

/// O `Dropdown` das listas de `k`, aberto, com a do cartão escolhida.
#[must_use]
pub fn list_dropdown(k: &Kanban, card: &Card) -> Dropdown<ListId> {
    let opts = k
        .lists_in_order()
        .into_iter()
        .map(|l| DropdownOption::new(node(0x100 ^ l.id.0), l.id, l.title.clone()))
        .collect();
    let mut dd = Dropdown::new(node(0x200), tr("kanban.views.table.col.list"), opts).open(true);
    dd.select(card.list);
    dd
}

/// O `Dropdown` de um campo de escolha: «—» (limpar) e as opções.
#[must_use]
pub fn option_dropdown(k: &Kanban, card: &Card, f: FieldId) -> Option<Dropdown<Option<OptionId>>> {
    let def = k.fields.get(&f)?;
    let FieldKind::Dropdown(opts) = &def.kind else {
        return None;
    };
    let mut v = vec![DropdownOption::new(
        node(0x300),
        None,
        tr("kanban.views.table.none"),
    )];
    v.extend(
        opts.iter()
            .map(|o| DropdownOption::new(node(0x300 ^ o.id.0), Some(o.id), o.text.clone())),
    );
    let mut dd = Dropdown::new(node(0x400), def.name.clone(), v).open(true);
    dd.select(match card.fields.get(&f) {
        Some(FieldValue::Option(o)) => Some(*o),
        _ => None,
    });
    Some(dd)
}

fn put(kanban: KanbanId, before: &Card, after: Card) -> Plan {
    let mut p = Plan::default();
    if after != *before {
        p.push(kanban, KanbanOp::put(Record::Card(after)));
    }
    p
}

/// Leva o cartão para o fim da lista `to` (o idioma do «Move» do Trello).
#[must_use]
pub fn move_to(k: &Kanban, card: &Card, to: ListId) -> Plan {
    if card.list == to || !k.lists.contains_key(&to) {
        return Plan::default();
    }
    let mut n = card.clone();
    n.list = to;
    n.pos = k.pos_after_cards(to);
    put(k.id, card, n)
}

/// Alterna a etiqueta `l`.
#[must_use]
pub fn toggle_label(k: &Kanban, card: &Card, l: LabelId) -> Plan {
    let mut n = card.clone();
    if let Some(i) = n.labels.iter().position(|x| *x == l) {
        n.labels.remove(i);
    } else {
        n.labels.push(l);
    }
    put(k.id, card, n)
}

/// Alterna o membro `who`.
#[must_use]
pub fn toggle_member(k: &Kanban, card: &Card, who: &Member) -> Plan {
    let mut n = card.clone();
    if let Some(i) = n.members.iter().position(|x| x == who) {
        n.members.remove(i);
    } else {
        n.members.push(who.clone());
    }
    put(k.id, card, n)
}

/// As datas que o seletor diz ao «Save» (o «concluído» fica como estava).
#[must_use]
pub fn set_dates(
    k: &Kanban,
    card: &Card,
    start: Option<Moment>,
    due: Option<(Moment, Option<u32>)>,
) -> Plan {
    let mut n = card.clone();
    n.start = start;
    n.due = due.map(|(at, remind)| Due {
        at,
        done: card.due.is_some_and(|d| d.done),
        remind_before_min: remind,
    });
    put(k.id, card, n)
}

/// Põe (ou limpa, `None`) o valor do campo `f`.
#[must_use]
pub fn set_field(k: &Kanban, card: &Card, f: FieldId, v: Option<FieldValue>) -> Plan {
    let mut n = card.clone();
    match v {
        Some(v) => {
            n.fields.insert(f, v);
        }
        None => {
            n.fields.remove(&f);
        }
    }
    put(k.id, card, n)
}

/// O retângulo de um popover de `h` de altura ancorado sob a célula `cell`, dentro de `bounds`
/// (sobe acima da célula se não cabe por baixo).
#[must_use]
pub fn place(cell: Rect, h: f32, bounds: Rect) -> Rect {
    let x = cell.x.min(bounds.x + bounds.w - m::POP_W).max(bounds.x);
    let below = cell.y + cell.h + m::POP_GAP;
    let y = if below + h <= bounds.y + bounds.h {
        below
    } else {
        (cell.y - m::POP_GAP - h).max(bounds.y)
    };
    Rect::new(x, y, m::POP_W, h)
}

/// A altura de um popover de `n` linhas com título.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn list_pop_h(n: usize) -> f32 {
    m::POP_PAD * 2.0 + m::POP_ROW_H * (n as f32 + 1.0)
}

/// O rect da linha `i` (a `0` é o título) de um popover de linhas.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn pop_row(at: Rect, i: usize) -> Rect {
    Rect::new(
        at.x + m::POP_PAD,
        at.y + m::POP_PAD + m::POP_ROW_H * i as f32,
        at.w - 2.0 * m::POP_PAD,
        m::POP_ROW_H,
    )
}
