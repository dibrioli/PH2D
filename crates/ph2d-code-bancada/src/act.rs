//! ⭐ **O clique da grelha vira a edição do cartão Script do Inspector** (plano 13 Onda B, item 8):
//! a MESMA `EditorAction::InspectorComponentEdit` que o `ph2d-panel-inspector` empurra
//! (`event_script.rs`), logo o MESMO aplicador (`script_inspector::apply`) e o MESMO desfazer (o
//! projecto regista por diff, um passo por quadro).

use ph2d_code_viewmodel::{CellKind, Intent, Who};
use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::script_edits::ScriptFieldEdit as E;
use ph2d_script::props::PropDecl;

/// Um número escrito à mão (o ponto ou a vírgula decimal).
#[must_use]
pub fn parse_number(s: &str) -> Option<f64> {
    s.trim()
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
}

/// Um par escrito à mão, com a vírgula decimal ou o ponto: `x; y` (com `;` a vírgula é decimal — a
/// convenção das listas onde ela o é, Excel e LibreOffice), `x y` ou `x, y` (um espaço separa), `x,y`
/// (uma vírgula só), e `( )`/`[ ]` à volta. `1,5,2` é ambíguo e recusa-se.
#[must_use]
pub fn parse_vec2(s: &str) -> Option<[f64; 2]> {
    let s = s.trim().trim_start_matches(['(', '[']);
    let s = s.trim_end_matches([')', ']']);
    let parts: Vec<&str> = if s.contains(';') {
        s.split(';').collect()
    } else if s.split_whitespace().count() == 2 {
        s.split_whitespace()
            .map(|p| p.trim_end_matches(','))
            .collect()
    } else {
        s.split(',').collect()
    };
    match parts.as_slice() {
        [x, y] => Some([parse_number(x)?, parse_number(y)?]),
        _ => None,
    }
}

/// A edição que `intent` pede, ou `None` (não é uma edição, ou o texto não serve ao tipo).
/// `decls` = as declarações que o painel mostra (as pistas `min`/`max` limitam o número, como no
/// campo do Inspector).
#[must_use]
pub fn act(who: &Who, decls: Option<&[PropDecl]>, intent: &Intent) -> Option<EditorAction> {
    let Intent::Edit(h, c, value) = intent else {
        return None;
    };
    let holder = who.holders.get(*h)?;
    let col = who.columns.get(*c)?;
    let name = col.name.clone();
    let edit = match col.kind? {
        CellKind::Number => {
            let hint = decls
                .and_then(|d| d.iter().find(|d| d.name == name))
                .map(|d| d.hint.clone())
                .unwrap_or_default();
            let v = parse_number(value)?;
            let v = v
                .max(hint.min.unwrap_or(f64::MIN))
                .min(hint.max.unwrap_or(f64::MAX));
            E::SetNumber(name, v)
        }
        CellKind::Bool => E::SetBool(name, value.trim().parse().ok()?),
        CellKind::Text => E::SetText(name, value.clone()),
        CellKind::Choice => {
            col.options.iter().find(|o| *o == value)?;
            E::SetText(name, value.clone())
        }
        CellKind::Vec2 => E::SetVec2(name, parse_vec2(value)?),
        CellKind::Object => E::SetObject(name, value.trim().to_owned()),
        CellKind::Color => return None,
    };
    Some(EditorAction::InspectorComponentEdit {
        entity_bits: holder.bits,
        edit: ComponentEdit::Script(edit),
    })
}
