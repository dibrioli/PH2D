//! **A semente da secção CAMERA** — a assinatura que decide se os números dela se re-semeiam, e a
//! própria semente.
//!
//! ⚠️ **Mudou-se do `sync_sections` na W7 da paralaxe**, pelo tecto de LOC do painel (`602/600`):
//! a assinatura nasceu aqui e a semente que ela governa veio ter com ela — *um corte por
//! responsabilidade*, e não por número de linhas.
//!
//! ⚠️ **Irmão do [`crate::sync_sections`] por CAP de FICHEIRO**, e pela mesma lei das irmãs
//! (`sync_ray`, `sync_parallax`): *só o que SEMEIA um widget entra*. As caixas espelham o mundo
//! todo o quadro e ficam fora, e as leituras derivadas (quantas câmeras há, se esta manda) também —
//! elas não semeiam número nenhum.

use std::hash::{Hash, Hasher};

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::InspectorCameraInfo;
use ph2d_editor_core::widget::CheckboxValue;

use crate::sync_sections::write_text;

pub(crate) fn assinatura(cam: &InspectorCameraInfo) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    cam.entity_bits.hash(&mut h);
    cam.camera.priority.hash(&mut h);
    let mut numeros = vec![
        cam.camera.height_world,
        cam.camera.offset[0],
        cam.camera.offset[1],
        cam.camera.dolly,
    ];
    // ⚠️ **A PRESENÇA de cada bloco entra**, e não só o valor: anexar o seguidor ou a cerca com os
    // valores de fábrica tem de re-semear o quadro em que as fileiras nascem.
    cam.follow.is_some().hash(&mut h);
    cam.limits.is_some().hash(&mut h);
    if let Some(f) = cam.follow.as_ref() {
        numeros.extend(f.damping);
        numeros.extend(f.dead_zone);
        numeros.extend(f.lookahead);
        numeros.extend(f.offset);
        f.target.hash(&mut h);
    }
    if let Some(l) = cam.limits.as_ref() {
        numeros.extend(l.min);
        numeros.extend(l.max);
    }
    for v in numeros {
        v.to_bits().hash(&mut h);
    }
    h.finish()
}

/// Semeia os campos da secção CAMERA a partir do snapshot.
///
/// ⚠️ **As CAIXAS espelham o mundo todo o quadro** (como as do timer): a secção decide a partir do
/// SNAPSHOT e o store aqui só publica o estado para a árvore de acessibilidade. Deixá-las numa
/// aresta punha-as a mentir a quem as lê por ali.
///
/// ⚠️ **Os NÚMEROS e o TEXTO são de ARESTA** — reescrevê-los por quadro apagaria o que o artista
/// está a digitar antes de o commit da shell chegar.
pub(crate) fn sync_camera_fields(
    host: &mut dyn PanelHostInternal,
    cam: &ph2d_editor_core::screens::hero::InspectorCameraInfo,
    seed: bool,
) {
    for (id, on) in [
        (crate::ids::INSP_CAMERA_ACTIVE, cam.camera.active),
        (crate::ids::INSP_CAMERA_PREVIEW, cam.preview_on),
    ] {
        if let Some(InteractiveState::Checkbox { value, .. }) = host.store_mut().get_mut(id) {
            *value = if on {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            };
        }
    }
    for (bit, &id) in crate::ids::INSP_CAMERA_CULL_BIT.iter().enumerate() {
        let on = cam.camera.cull_mask & (1u32 << bit) != 0;
        if let Some(InteractiveState::Checkbox { value, .. }) = host.store_mut().get_mut(id) {
            *value = if on {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            };
        }
    }
    if !seed {
        return;
    }
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    let mut numeros: Vec<(ph2d_a11y::NodeId, f64)> = vec![
        (
            crate::ids::INSP_CAMERA_HEIGHT,
            f64::from(cam.camera.height_world),
        ),
        (
            crate::ids::INSP_CAMERA_OFFSET_X,
            f64::from(cam.camera.offset[0]),
        ),
        (
            crate::ids::INSP_CAMERA_OFFSET_Y,
            f64::from(cam.camera.offset[1]),
        ),
        (
            crate::ids::INSP_CAMERA_PRIORITY,
            f64::from(cam.camera.priority),
        ),
        (crate::ids::INSP_CAMERA_DOLLY, f64::from(cam.camera.dolly)),
    ];
    if let Some(f) = cam.follow.as_ref() {
        numeros.extend([
            (crate::ids::INSP_CAMERA_DAMP_X, f64::from(f.damping[0])),
            (crate::ids::INSP_CAMERA_DAMP_Y, f64::from(f.damping[1])),
            (crate::ids::INSP_CAMERA_DEAD_X, f64::from(f.dead_zone[0])),
            (crate::ids::INSP_CAMERA_DEAD_Y, f64::from(f.dead_zone[1])),
            (crate::ids::INSP_CAMERA_LOOK_X, f64::from(f.lookahead[0])),
            (crate::ids::INSP_CAMERA_LOOK_Y, f64::from(f.lookahead[1])),
            (crate::ids::INSP_CAMERA_FOLLOW_OFF_X, f64::from(f.offset[0])),
            (crate::ids::INSP_CAMERA_FOLLOW_OFF_Y, f64::from(f.offset[1])),
        ]);
        write_text(host, crate::ids::INSP_CAMERA_TARGET, &f.target);
    }
    if let Some(l) = cam.limits.as_ref() {
        numeros.extend([
            (crate::ids::INSP_CAMERA_MIN_X, f64::from(l.min[0])),
            (crate::ids::INSP_CAMERA_MIN_Y, f64::from(l.min[1])),
            (crate::ids::INSP_CAMERA_MAX_X, f64::from(l.max[0])),
            (crate::ids::INSP_CAMERA_MAX_Y, f64::from(l.max[1])),
        ]);
    }
    for (id, v) in numeros {
        if focus != Some(id) && drag != Some(id) {
            host.store_mut().set_number_value(id, v);
        }
    }
}
