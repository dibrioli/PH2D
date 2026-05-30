//! Inspector store-sync — runs once per frame at the start of paint,
//! ADR-0029 Phase C.1 port from
//! `ph2d_editor_core::screens::hero::inspector_sync`.
//!
//! Behavior unchanged from the legacy implementation; only the
//! signature switched from `(hero: &mut HeroScreen)` to
//! `(state: &mut InspectorState, host: &mut dyn PanelHostInternal)`.
//! Inspector snapshots that used to live on `hero.inspector.*` now
//! live in thread-locals owned by [`crate::state`].

use crate::state::{
    self, current_inspector_name, current_inspector_sprite, current_inspector_transform,
    current_inspector_visibility,
};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::{CheckboxValue, TextInputState};

pub(crate) fn sync_inspector_from_snapshots(
    inspector_state: &mut state::InspectorState,
    host: &mut dyn PanelHostInternal,
) {
    let transform = current_inspector_transform();
    let name = current_inspector_name();
    let visibility = current_inspector_visibility();
    let new_entity = transform
        .map(|i| i.entity_bits)
        .or_else(|| name.as_ref().map(|i| i.entity_bits))
        .or_else(|| visibility.map(|i| i.entity_bits));
    let entity_changed = new_entity != inspector_state.last_entity;
    let display_unit = host.project().display_unit;
    let ppm = host.project().pixels_per_meter;
    let pos_for_display = |m: f32| display_unit.from_meters(m, ppm) as f64;
    if entity_changed {
        host.store_mut().set_focus(None);
        let _ = host.store_mut().end_number_input_drag();
        host.store_mut().end_number_stepper_hold();
        if let Some(info) = transform {
            host.store_mut().set_number_value(
                ids::INSP_TRANSFORM_POS_X,
                pos_for_display(info.translation[0]),
            );
            host.store_mut().set_number_value(
                ids::INSP_TRANSFORM_POS_Y,
                pos_for_display(info.translation[1]),
            );
            host.store_mut().set_number_value(
                ids::INSP_TRANSFORM_ROT,
                info.rotation_rad.to_degrees() as f64,
            );
            host.store_mut()
                .set_number_value(ids::INSP_TRANSFORM_SCALE_X, info.scale[0] as f64);
            host.store_mut()
                .set_number_value(ids::INSP_TRANSFORM_SCALE_Y, info.scale[1] as f64);
            host.store_mut().set_number_value(
                ids::INSP_TRANSFORM_SKEW_X,
                info.skew_rad[0].to_degrees() as f64,
            );
            host.store_mut().set_number_value(
                ids::INSP_TRANSFORM_SKEW_Y,
                info.skew_rad[1].to_degrees() as f64,
            );
        }
        if let Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) = host.store_mut().get_mut(ids::INSP_ENTITY_NAME)
        {
            *state = TextInputState::Normal;
            text.clear();
            if let Some(info) = name.as_ref() {
                text.push_str(&info.name);
            }
            *caret = text.len();
            *selection_anchor = None;
        }
        inspector_state.last_entity = new_entity;
    } else {
        if let Some(info) = transform {
            host.store_mut().set_number_value(
                ids::INSP_TRANSFORM_POS_X,
                pos_for_display(info.translation[0]),
            );
            host.store_mut().set_number_value(
                ids::INSP_TRANSFORM_POS_Y,
                pos_for_display(info.translation[1]),
            );
            host.store_mut().set_number_value(
                ids::INSP_TRANSFORM_ROT,
                info.rotation_rad.to_degrees() as f64,
            );
            host.store_mut()
                .set_number_value(ids::INSP_TRANSFORM_SCALE_X, info.scale[0] as f64);
            host.store_mut()
                .set_number_value(ids::INSP_TRANSFORM_SCALE_Y, info.scale[1] as f64);
            host.store_mut().set_number_value(
                ids::INSP_TRANSFORM_SKEW_X,
                info.skew_rad[0].to_degrees() as f64,
            );
            host.store_mut().set_number_value(
                ids::INSP_TRANSFORM_SKEW_Y,
                info.skew_rad[1].to_degrees() as f64,
            );
        }
        let focused = host.store().focus_id() == Some(ids::INSP_ENTITY_NAME);
        let pending_name_edit = host
            .bus()
            .iter()
            .any(|a| matches!(a, EditorAction::InspectorNameEdit(_)));
        if !pending_name_edit
            && !focused
            && let Some(info) = name.as_ref()
            && let Some(InteractiveState::TextInput {
                text,
                caret,
                selection_anchor,
                ..
            }) = host.store_mut().get_mut(ids::INSP_ENTITY_NAME)
            && text.as_str() != info.name.as_str()
        {
            text.clear();
            text.push_str(&info.name);
            *caret = text.len();
            *selection_anchor = None;
        }
    }
    let pending_visibility_edit = host
        .bus()
        .iter()
        .any(|a| matches!(a, EditorAction::InspectorVisibilityEdit(_)));
    if !pending_visibility_edit
        && let Some(vis) = visibility
        && let Some(InteractiveState::Checkbox { value, .. }) =
            host.store_mut().get_mut(ids::INSP_VISIBILITY_CHECK)
    {
        *value = if vis.visible {
            CheckboxValue::Checked
        } else {
            CheckboxValue::Unchecked
        };
    }
    // W2 Sprite Inspector v2: reflect Sprite.flip_x/flip_y in the Flip
    // H/V checkboxes (unless a flip edit is mid-flight this frame, so we
    // don't clobber the user's just-toggled value before it commits).
    let pending_sprite_edit = host
        .bus()
        .iter()
        .any(|a| matches!(a, EditorAction::InspectorSpriteEdit { .. }));
    if !pending_sprite_edit && let Some(sp) = current_inspector_sprite() {
        for (id, on) in [
            (ids::INSP_SPRITE_FLIP_X, sp.flip_x),
            (ids::INSP_SPRITE_FLIP_Y, sp.flip_y),
        ] {
            if let Some(InteractiveState::Checkbox { value, .. }) = host.store_mut().get_mut(id) {
                *value = if on {
                    CheckboxValue::Checked
                } else {
                    CheckboxValue::Unchecked
                };
            }
        }
    }
}
