//! **O despacho da secção RAY SENSOR** (suplente #21).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — o mesmo padrão das irmãs.
//!
//! # ⚠️ O CLIQUE afirma o contrário do que está no ECRÃ, e o ecrã vem do SNAPSHOT
//!
//! Nunca do store, que guarda o visual. É a lei que a §11 pagou com um report: ler o store fazia o
//! primeiro clique depois de trocar de objecto mandar o valor do objecto **anterior**.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::ray_edits::RayFieldEdit;

/// Despacha um evento da secção RAY SENSOR. `true` = consumido.
pub(crate) fn apply_ray_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_ray() else {
        return false;
    };
    let bits = info.entity_bits;

    // A camada pela lista (a mesma do collider) e os quadros «Sees» (a linha da matriz do mundo).
    if let WidgetEvent::Click(id) = ev {
        if let Some(n) = crate::ids::INSP_RAY_LAYER_OPTS
            .iter()
            .position(|&o| o == id)
        {
            #[allow(clippy::cast_possible_truncation)]
            push(host, bits, RayFieldEdit::Layer(n as u32));
            return true;
        }
        let quem = &crate::sections::physics_colide::DO_RAIO;
        #[allow(clippy::cast_possible_truncation)]
        let a = info.layer.min(u32::from(u8::MAX)) as u8;
        if crate::sections::physics_colide::clique_da_fila(host, id, quem, a, true) {
            return true;
        }
    }

    if let WidgetEvent::TextChanged(id) = ev {
        // ⚠️ **O NOME cru** — quem o apara é quem o lê (a regra do `SignalOnHit`).
        let nome = host.store().text(id).unwrap_or("").to_string();
        let edit = match id {
            crate::ids::INSP_RAY_ON_ENTER => RayFieldEdit::OnEnter(nome),
            crate::ids::INSP_RAY_ON_EXIT => RayFieldEdit::OnExit(nome),
            _ => return false,
        };
        push(host, bits, edit);
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev {
        let v = crate::unidade_mundo::le(host, id).unwrap_or(0.0);
        #[allow(clippy::cast_possible_truncation)]
        let f = v as f32;
        // ⚠️ **Um `match` e não um `if` por campo** — com `if`s, o terceiro acaba a escrever no
        // primeiro (a lição dos três campos de texto da tabela de acções).
        let edit = match id {
            crate::ids::INSP_RAY_ORIGIN_X => RayFieldEdit::OriginX(f),
            crate::ids::INSP_RAY_ORIGIN_Y => RayFieldEdit::OriginY(f),
            crate::ids::INSP_RAY_DIR_X => RayFieldEdit::DirX(f),
            crate::ids::INSP_RAY_DIR_Y => RayFieldEdit::DirY(f),
            crate::ids::INSP_RAY_REACH => RayFieldEdit::Reach(f),
            _ => return false,
        };
        push(host, bits, edit);
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: RayFieldEdit) {
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits,
        edit: ComponentEdit::Ray(edit),
    });
}
