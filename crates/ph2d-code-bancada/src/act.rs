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
        CellKind::Vec2 => {
            let mut it = value.split([',', ' ']).filter(|p| !p.trim().is_empty());
            let x = parse_number(it.next()?)?;
            let y = parse_number(it.next()?)?;
            it.next().is_none().then_some(())?;
            E::SetVec2(name, [x, y])
        }
        CellKind::Object => E::SetObject(name, value.trim().to_owned()),
        CellKind::Color => return None,
    };
    Some(EditorAction::InspectorComponentEdit {
        entity_bits: holder.bits,
        edit: ComponentEdit::Script(edit),
    })
}
