//! **O despacho da secção CAMERA** (TOP-20 #7, W3).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — mesmo padrão do [`crate::event_audio`].
//!
//! # ⚠️ Ela não tem linha aberta, e por isso não escreve no `InspectorState`
//!
//! Um objecto tem UMA câmera, UM seguidor e UMA cerca. Sem lista não há escolha de linha a lembrar,
//! e este despacho só traduz gestos em edições.
//!
//! # ⚠️ O CLIQUE afirma o contrário do que está no ECRÃ, e o ecrã vem do SNAPSHOT
//!
//! Nunca do store, que guarda o visual. É a lei que a §11 pagou com um report: ler o store fazia o
//! primeiro clique depois de trocar de objecto mandar o valor do objecto **anterior**.
//!
//! # ⚠️ Os pares de eixo viajam JUNTOS
//!
//! `Offset`, `Damping`, `Dead Zone`, `Lookahead` e os cantos da cerca são `[f32; 2]` **no
//! componente**, e o descritor espelha a ESTRUTURA. ⇒ mexer no `X` manda o par inteiro, com o `Y`
//! lido do snapshot. ⛔ Mandar meio par obrigaria a shell a ler o outro eixo do mundo, e é assim
//! que dois escritores do mesmo campo nascem.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::CameraFieldEdit;

use crate::state;

/// Despacha um evento da secção CAMERA. `true` = consumido.
#[allow(clippy::too_many_lines)]
pub(crate) fn apply_camera_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = state::current_inspector_camera() else {
        return false;
    };
    let bits = info.entity_bits;

    if let WidgetEvent::Toggled(id) = ev {
        // ⭐ **A pré-visualização é a única que não escreve no documento** — ver o modelo.
        if id == ids::INSP_CAMERA_PREVIEW {
            push(host, bits, CameraFieldEdit::Preview(!info.preview_on));
            return true;
        }
        if id == ids::INSP_CAMERA_ACTIVE {
            push(host, bits, CameraFieldEdit::Active(!info.camera.active));
            return true;
        }
        if let Some(bit) = ids::INSP_CAMERA_CULL_BIT.iter().position(|&b| b == id) {
            // ⚠️ **O estado vem do SNAPSHOT**, e a edição é o bit INVERTIDO dele.
            let ligado = info.camera.cull_mask & (1u32 << bit) != 0;
            push(
                host,
                bits,
                CameraFieldEdit::CullBit(u8::try_from(bit).unwrap_or(0), !ligado),
            );
            return true;
        }
        return false;
    }

    if let WidgetEvent::TextChanged(id) = ev
        && id == ids::INSP_CAMERA_TARGET
    {
        let text = host.store().text(id).unwrap_or("").to_string();
        push(host, bits, CameraFieldEdit::Target(text));
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev {
        let v = host.store().number_value(id).unwrap_or(0.0);
        #[allow(clippy::cast_possible_truncation)]
        let f = v as f32;
        let follow = info.follow.as_ref();
        let limits = info.limits.as_ref();
        // ⚠️ **Um `if` por campo seria como o terceiro acaba a escrever no primeiro** — a mesma
        // razão que pôs os três campos de texto da tabela de acções numa porta só.
        let edit = match id {
            ids::INSP_CAMERA_HEIGHT => CameraFieldEdit::Height(f),
            #[allow(clippy::cast_possible_truncation)]
            ids::INSP_CAMERA_PRIORITY => CameraFieldEdit::Priority(v as i32),
            ids::INSP_CAMERA_OFFSET_X => CameraFieldEdit::Offset([f, info.camera.offset[1]]),
            ids::INSP_CAMERA_OFFSET_Y => CameraFieldEdit::Offset([info.camera.offset[0], f]),
            ids::INSP_CAMERA_DAMP_X => {
                CameraFieldEdit::Damping([f, follow.map_or(0.0, |x| x.damping[1])])
            }
            ids::INSP_CAMERA_DAMP_Y => {
                CameraFieldEdit::Damping([follow.map_or(0.0, |x| x.damping[0]), f])
            }
            ids::INSP_CAMERA_DEAD_X => {
                CameraFieldEdit::DeadZone([f, follow.map_or(0.0, |x| x.dead_zone[1])])
            }
            ids::INSP_CAMERA_DEAD_Y => {
                CameraFieldEdit::DeadZone([follow.map_or(0.0, |x| x.dead_zone[0]), f])
            }
            ids::INSP_CAMERA_LOOK_X => {
                CameraFieldEdit::Lookahead([f, follow.map_or(0.0, |x| x.lookahead[1])])
            }
            ids::INSP_CAMERA_LOOK_Y => {
                CameraFieldEdit::Lookahead([follow.map_or(0.0, |x| x.lookahead[0]), f])
            }
            ids::INSP_CAMERA_FOLLOW_OFF_X => {
                CameraFieldEdit::FollowOffset([f, follow.map_or(0.0, |x| x.offset[1])])
            }
            ids::INSP_CAMERA_FOLLOW_OFF_Y => {
                CameraFieldEdit::FollowOffset([follow.map_or(0.0, |x| x.offset[0]), f])
            }
            ids::INSP_CAMERA_MIN_X => {
                CameraFieldEdit::LimitMin([f, limits.map_or(0.0, |x| x.min[1])])
            }
            ids::INSP_CAMERA_MIN_Y => {
                CameraFieldEdit::LimitMin([limits.map_or(0.0, |x| x.min[0]), f])
            }
            ids::INSP_CAMERA_MAX_X => {
                CameraFieldEdit::LimitMax([f, limits.map_or(0.0, |x| x.max[1])])
            }
            ids::INSP_CAMERA_MAX_Y => {
                CameraFieldEdit::LimitMax([limits.map_or(0.0, |x| x.max[0]), f])
            }
            _ => return false,
        };
        push(host, bits, edit);
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: CameraFieldEdit) {
    host.bus_mut()
        .push(EditorAction::InspectorCameraEdit { entity_bits, edit });
}

/// Repõe o visual de uma caixa depois do clique — o valor vivo é o do snapshot.
#[allow(dead_code)]
fn demote(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Checkbox { state, .. }) = host.store_mut().get_mut(id) {
        *state = ph2d_editor_core::widget::CheckboxState::Normal;
    }
}
