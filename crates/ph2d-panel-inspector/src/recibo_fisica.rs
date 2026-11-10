//! **O recibo do Remove da física** (ficha f1): «Physics Body removed · Undo». Vive enquanto o próximo
//! Desfazer é o do Remove, e a shell publica o topo da fila a cada quadro ([`publica_o_topo`]): qualquer
//! edição depois dele, ou um Desfazer de outra fila, apaga-o. O «Undo» é o Desfazer global
//! (`EditorAction::UndoStep`), o mesmo do Ctrl+Z, e o diff repõe corpo e forma.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::InspectorTransformInfo;
use std::cell::{Cell, RefCell};

/// O que foi removido e o objecto como estava.
#[derive(Clone, PartialEq)]
struct Recibo {
    entity_bits: u64,
    forma: bool,
    pose: Option<InspectorTransformInfo>,
    /// O topo da fila quando se clicou: o 1.º topo diferente é o passo do Remove.
    topo_antes: Option<u64>,
    /// O passo do Remove, quando já chegou à fila.
    passo: Option<Option<u64>>,
}

thread_local! {
    static RECIBO: RefCell<Option<Recibo>> = const { RefCell::new(None) };
    static TOPO: Cell<Option<u64>> = const { Cell::new(None) };
}

fn limpa() {
    RECIBO.with(|r| *r.borrow_mut() = None);
}

/// Uma vez por quadro, depois da fila andar: o degrau que o próximo Desfazer tiraria, e se é a fila global
/// quem responde. O recibo só sobrevive enquanto esse degrau é o do Remove.
pub fn publica_o_topo(topo: Option<u64>, global_responde: bool) {
    TOPO.with(|t| t.set(topo));
    RECIBO.with(|r| {
        let mut r = r.borrow_mut();
        let vale = r.as_mut().is_some_and(|c| {
            if c.passo.is_none() && topo != c.topo_antes {
                c.passo = Some(topo);
            }
            global_responde && c.passo.is_none_or(|p| p == topo)
        });
        if !vale {
            *r = None;
        }
    });
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
        // Quem estica: o corpo, na opção B — não a forma escolhida.
        if let Some(mut t) = crate::state::current_inspector_physics().and_then(|f| f.estica) {
            // O espelho é uma escolha do artista: fica.
            t.scale = t.scale.map(|s| if s < 0.0 { -1.0 } else { 1.0 });
            host.bus_mut().push(EditorAction::InspectorTransformEdit(t));
        }
        return true;
    }
    if id == crate::ids::INSP_PHYS_REMOVE
        && let Some(f) = crate::state::current_inspector_physics()
        && crate::sections::physics_cartao::oferece_remove(&f)
    {
        RECIBO.with(|r| {
            *r.borrow_mut() = Some(Recibo {
                entity_bits: f.entity_bits,
                forma: !f.has_body,
                pose: crate::state::current_inspector_transform()
                    .filter(|p| p.entity_bits == f.entity_bits),
                topo_antes: TOPO.with(Cell::get),
                passo: None,
            });
        });
    }
    false
}
