//! **O recibo do Remove da física** (ficha f1): «Physics Body removed · Undo». Vive enquanto o Desfazer
//! ainda é o do Remove: mudar de objecto, mover o objecto ou mexer de novo na física apaga-o. O «Undo» é
//! o Desfazer global (`EditorAction::UndoStep`), o mesmo do Ctrl+Z, e o diff repõe corpo e forma.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::InspectorTransformInfo;
use std::cell::RefCell;

/// O que foi removido e o objecto como estava.
#[derive(Clone, PartialEq)]
struct Recibo {
    entity_bits: u64,
    forma: bool,
    pose: Option<InspectorTransformInfo>,
}

thread_local! {
    static RECIBO: RefCell<Option<Recibo>> = const { RefCell::new(None) };
}

fn limpa() {
    RECIBO.with(|r| *r.borrow_mut() = None);
}

/// A chave da frase do recibo à vista neste objecto (`true` = era só a forma), se houver.
#[must_use]
pub(crate) fn a_vista() -> Option<bool> {
    let pose = crate::state::current_inspector_transform();
    RECIBO.with(|r| {
        r.borrow()
            .as_ref()
            .filter(|c| {
                pose.as_ref()
                    .is_some_and(|p| p.entity_bits == c.entity_bits)
            })
            .map(|c| c.forma)
    })
}

/// Uma vez por quadro: outro objecto, o objecto movido ou a física de volta (o Desfazer já correu)
/// tiram a validade do recibo.
pub(crate) fn valida() {
    let pose = crate::state::current_inspector_transform();
    let fisica = crate::state::current_inspector_physics();
    let vale = RECIBO.with(|r| {
        r.borrow().as_ref().is_none_or(|c| {
            pose.as_ref().is_some_and(|p| Some(p) == c.pose.as_ref())
                && !fisica
                    .as_ref()
                    .is_some_and(|f| f.entity_bits == c.entity_bits && f.has_collider)
        })
    });
    if !vale {
        limpa();
    }
}

/// O clique no «Set Scale to 1», no «Undo» do recibo, e o registo do Remove (o Remove segue o seu
/// caminho: devolve `false`). Qualquer outra edição da física apaga o recibo.
pub(crate) fn click(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = ev else {
        if matches!(ev, WidgetEvent::ValueChanged(_)) {
            limpa();
        }
        return false;
    };
    if id == crate::ids::INSP_PHYS_REMOVED_UNDO {
        if a_vista().is_some() {
            host.bus_mut().push(EditorAction::UndoStep { redo: false });
        }
        limpa();
        return true;
    }
    if id == crate::ids::INSP_PHYS_SCALE_ONE {
        if let Some(mut t) = crate::state::current_inspector_transform() {
            // O espelho é uma escolha do artista: fica.
            t.scale = t.scale.map(|s| if s < 0.0 { -1.0 } else { 1.0 });
            host.bus_mut().push(EditorAction::InspectorTransformEdit(t));
        }
        return true;
    }
    if id == crate::ids::INSP_PHYS_REMOVE
        && let Some(f) = crate::state::current_inspector_physics()
        && (f.has_body || f.has_collider)
    {
        RECIBO.with(|r| {
            *r.borrow_mut() = Some(Recibo {
                entity_bits: f.entity_bits,
                forma: !f.has_body,
                pose: crate::state::current_inspector_transform()
                    .filter(|p| p.entity_bits == f.entity_bits),
            });
        });
    }
    false
}
