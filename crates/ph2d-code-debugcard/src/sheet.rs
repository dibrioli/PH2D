//! As linhas da Sheet (a tabela da casa, [`ph2d_ui_table`]): `Locals · N`, as locais; `self · N
//! fields`, os campos; as filhas de uma tabela com ▸, abertas sozinhas se uma delas mudou.

use crate::{DebugCard, VarGroup, VarRow, kind_name, shown_value};
use ph2d_i18n::{tr, tr_with};
use ph2d_script::ValueKind;
use ph2d_ui_table::{Cell, Column, Ink, Row};

/// As três colunas do Stopped: Name · Value (números à direita) · Type (ocultável).
#[must_use]
pub fn columns() -> Vec<Column> {
    vec![
        Column::new("name", tr("code.debug.col.name")).mono().flex(),
        Column::new("value", tr("code.debug.col.value"))
            .mono()
            .end(),
        Column::new("type", tr("code.debug.col.type")).hideable(),
    ]
}

/// O tom do valor pelo tipo (o DevTools); o que mudou é âmbar.
#[must_use]
pub fn value_ink(r: &VarRow) -> Ink {
    if r.old.is_some() {
        return Ink::Changed;
    }
    match r.kind {
        ValueKind::Number | ValueKind::Vector => Ink::Number,
        ValueKind::String => Ink::Text,
        ValueKind::Boolean => Ink::Boolean,
        ValueKind::Nil => Ink::Nil,
        ValueKind::Table => Ink::Muted,
        ValueKind::Function | ValueKind::Coroutine | ValueKind::Buffer | ValueKind::Userdata => {
            Ink::Function
        }
    }
}

/// A coluna Type: o nome Luau, e «· changed» no que mudou.
#[must_use]
pub fn type_text(r: &VarRow) -> String {
    let k = kind_name(r.kind);
    if r.old.is_some() {
        tr_with("code.debug.type_changed", &[("type", &k)])
    } else {
        k.to_owned()
    }
}

/// ⭐ As linhas da Sheet de `c`.
#[must_use]
pub fn sheet_rows(c: &DebugCard) -> Vec<Row> {
    let mut out = Vec::new();
    for (group, vars) in c.groups() {
        if vars.is_empty() {
            continue;
        }
        let top = vars.iter().filter(|v| v.depth == 0).count();
        let (key, title, count) = match group {
            VarGroup::Locals => (
                "#locals",
                tr("code.debug.group.locals"),
                tr_with("code.debug.group_count", &[("count", &top)]),
            ),
            VarGroup::This => (
                "#self",
                tr("code.debug.group.self"),
                tr_with("code.debug.group_fields", &[("count", &top)]),
            ),
        };
        out.push(Row {
            key: key.into(),
            group: Some(count),
            cells: vec![Cell::new(title, Ink::Plain)],
            expandable: true,
            default_open: true,
            ..Row::default()
        });
        for (i, v) in vars.iter().enumerate() {
            // Uma tabela com uma filha que mudou abre-se sozinha (o âmbar não fica escondido).
            let open = vars[i + 1..]
                .iter()
                .take_while(|w| w.depth > v.depth)
                .any(|w| w.old.is_some());
            out.push(Row {
                key: v.key.clone(),
                depth: v.depth + 1,
                cells: vec![
                    Cell::new(&v.name, Ink::Name),
                    Cell {
                        old: v.old.clone(),
                        ..Cell::new(shown_value(v), value_ink(v))
                    },
                    Cell::new(type_text(v), Ink::Muted),
                ],
                changed: v.old.is_some(),
                expandable: v.expandable,
                default_open: open,
                ..Row::default()
            });
        }
    }
    out
}
