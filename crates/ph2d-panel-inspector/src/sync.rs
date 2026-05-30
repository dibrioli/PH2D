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
use ph2d_editor_core::widget::{CheckboxValue, SliderState, TextInputState};

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
    // W2 Sprite Inspector v2 — reflect the editable Sprite fields.
    if let Some(sp) = current_inspector_sprite() {
        // Checkboxes (Flip H/V, Tint Fill): seed ONLY on entity switch.
        // A checkbox toggles its own stored value on click, and `sync`
        // runs AFTER the bus is drained, so an every-frame reseed from
        // the (still-stale-until-commit) snapshot would revert the
        // just-toggled value for one frame — a visible flicker (audit
        // F2). Between switches the widget holds the truth; the next
        // snapshot reflects the commit. Mirrors the Name TextInput.
        if entity_changed {
            for (id, on) in [
                (ids::INSP_SPRITE_FLIP_X, sp.flip_x),
                (ids::INSP_SPRITE_FLIP_Y, sp.flip_y),
                (ids::INSP_SPRITE_TINT_FILL, sp.tint_fill),
            ] {
                if let Some(InteractiveState::Checkbox { value, .. }) = host.store_mut().get_mut(id)
                {
                    *value = if on {
                        CheckboxValue::Checked
                    } else {
                        CheckboxValue::Unchecked
                    };
                }
            }
        }
        // Numeric fields — every frame (so external changes reflect),
        // skipping the field the user is actively editing. Matches the
        // Transform NumberInputs' tolerated 1-frame post-commit lag.
        let focus = host.store().focus_id();
        for (id, value) in [
            (ids::INSP_SPRITE_HFRAMES, sp.hframes as f64),
            (ids::INSP_SPRITE_VFRAMES, sp.vframes as f64),
            (ids::INSP_SPRITE_FRAME, sp.frame as f64),
        ] {
            if focus != Some(id) {
                host.store_mut().set_number_value(id, value);
            }
        }
        // Opacity Slider (0..1 storage) + linked percent chip. Skip while
        // the slider is being dragged or the chip is focused so we don't
        // fight the user's input.
        let dragging = matches!(
            host.store().slider(ids::INSP_SPRITE_OPACITY),
            Some((SliderState::Dragging, _))
        );
        if !dragging && focus != Some(ids::INSP_SPRITE_OPACITY_CHIP) {
            if let Some(InteractiveState::Slider { value, .. }) =
                host.store_mut().get_mut(ids::INSP_SPRITE_OPACITY)
            {
                *value = sp.opacity;
            }
            // Chip lives in display space (percent) per the integer map.
            host.store_mut()
                .set_number_value(ids::INSP_SPRITE_OPACITY_CHIP, (sp.opacity * 100.0) as f64);
        }
    }
}
