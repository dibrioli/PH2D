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
    self, current_inspector_name, current_inspector_ordering, current_inspector_sampling,
    current_inspector_sprite, current_inspector_transform, current_inspector_visibility,
    current_inspector_visibility_section,
};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{
    InspectorOrderingInfo, InspectorSpriteInfo, SpriteFieldEdit,
};
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
    // ⚠️ **FORA do `entity_changed`, de propósito** — o `Rope Length` de uma
    // polia é derivado pela PONTE, e um número que o produto muda não pode
    // ser sincado só quando a SELEÇÃO muda. Ver `sync_derived_rope_length`.
    crate::sync_physics::sync_derived_rope_length(host);
    let display_unit = host.project().display_unit;
    let ppm = host.project().pixels_per_meter;
    let pos_for_display = |m: f32| display_unit.from_meters(m, ppm) as f64;
    if entity_changed {
        host.store_mut().set_focus(None);
        let _ = host.store_mut().end_number_input_drag();
        host.store_mut().end_number_stepper_hold();
        // Close a tint picker bound to the PREVIOUS selection so its
        // last-picked color can't stream onto the newly-selected entity
        // (the host re-mirrors picker → widget_color every frame, so
        // reseeding the swatch alone can't win). Runs here — above the
        // `Some(sp)` sprite block — so it fires even when the new
        // selection has no Sprite (empty entity / note) and the sprite
        // block is skipped entirely.
        if matches!(host.store().picker_target(), Some(t) if is_sprite_color_swatch(t)) {
            host.store_mut().set_picker_target(None);
        }
        crate::sync_physics::sync_physics_fields(host);
        crate::sync_physics::sync_joint_fields(host);
        crate::sync_physics::sync_wheel_fields(host);
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
            // Reflect the live Transform into the boxes — but SKIP the box the user is currently
            // editing (focused for typing OR being drag-scrubbed). Without this guard the box the drag
            // writes gets overwritten every frame by the round-tripped ECS value (meters↔display unit
            // isn't bit-exact), so the drag delta is applied against a moving target → the move / rot /
            // scale TREMBLE on the canvas in real time (Enio 2026-06-26). Mirrors the Sampling /
            // Visibility blocks below, plus a drag guard so scrub is covered even without focus.
            let focus = host.store().focus_id();
            let drag = host.store().number_input_drag().map(|d| d.id);
            for (id, value) in [
                (
                    ids::INSP_TRANSFORM_POS_X,
                    pos_for_display(info.translation[0]),
                ),
                (
                    ids::INSP_TRANSFORM_POS_Y,
                    pos_for_display(info.translation[1]),
                ),
                (
                    ids::INSP_TRANSFORM_ROT,
                    info.rotation_rad.to_degrees() as f64,
                ),
                (ids::INSP_TRANSFORM_SCALE_X, info.scale[0] as f64),
                (ids::INSP_TRANSFORM_SCALE_Y, info.scale[1] as f64),
                (
                    ids::INSP_TRANSFORM_SKEW_X,
                    info.skew_rad[0].to_degrees() as f64,
                ),
                (
                    ids::INSP_TRANSFORM_SKEW_Y,
                    info.skew_rad[1].to_degrees() as f64,
                ),
            ] {
                if focus != Some(id) && drag != Some(id) {
                    host.store_mut().set_number_value(id, value);
                }
            }
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
        sync_sprite_fields(inspector_state, host, &sp, entity_changed);
    }
    // W3 §7 — reflect the ordering components (any Transform entity).
    if let Some(ord) = current_inspector_ordering() {
        sync_ordering_fields(host, &ord, entity_changed);
    }
    // W3 §9 — reflect the UV tiling/scroll NumberInputs.
    if let Some(samp) = current_inspector_sampling() {
        let focus = host.store().focus_id();
        let drag = host.store().number_input_drag().map(|d| d.id);
        for (id, value) in [
            (ids::INSP_SAMPLE_UV_SCALE_X, samp.uv_scale[0] as f64),
            (ids::INSP_SAMPLE_UV_SCALE_Y, samp.uv_scale[1] as f64),
            (ids::INSP_SAMPLE_UV_OFFSET_X, samp.uv_offset[0] as f64),
            (ids::INSP_SAMPLE_UV_OFFSET_Y, samp.uv_offset[1] as f64),
        ] {
            if focus != Some(id) && drag != Some(id) {
                host.store_mut().set_number_value(id, value);
            }
        }
    }
    // W3 §8 — reflect the Mask Alpha Cutoff + Enabler Rect NumberInputs.
    if let Some(vis) = current_inspector_visibility_section() {
        let focus = host.store().focus_id();
        let drag = host.store().number_input_drag().map(|d| d.id);
        for (id, value) in [
            (ids::INSP_VIS_ALPHA_CUTOFF, vis.alpha_cutoff as f64),
            (ids::INSP_VIS_RECT_X, vis.rect[0] as f64),
            (ids::INSP_VIS_RECT_Y, vis.rect[1] as f64),
            (ids::INSP_VIS_RECT_W, vis.rect[2] as f64),
            (ids::INSP_VIS_RECT_H, vis.rect[3] as f64),
        ] {
            if focus != Some(id) && drag != Some(id) {
                host.store_mut().set_number_value(id, value);
            }
        }
    }
}

/// Reflect the §7 ordering snapshot onto the Inspector widgets.
/// Checkboxes seed on entity switch (diverging → Indeterminate);
/// integer NumberInputs reflect every frame except the focused one
/// (same rhythm as the sprite/transform fields).
fn sync_ordering_fields(
    host: &mut dyn PanelHostInternal,
    ord: &InspectorOrderingInfo,
    entity_changed: bool,
) {
    if entity_changed {
        for (id, on, mixed) in [
            (
                ids::INSP_ORDER_Z_RELATIVE,
                ord.z_as_relative,
                ord.mixed.z_as_relative,
            ),
            (
                ids::INSP_ORDER_SHOW_BEHIND,
                ord.show_behind_parent,
                ord.mixed.show_behind_parent,
            ),
            (
                ids::INSP_ORDER_YSORT_ENABLED,
                ord.y_sort_enabled,
                ord.mixed.y_sort_enabled,
            ),
            (
                ids::INSP_ORDER_SORTING_GROUP,
                ord.sorting_group,
                ord.mixed.sorting_group,
            ),
            (
                ids::INSP_ORDER_SORT_AT_ROOT,
                ord.sort_at_root,
                ord.mixed.sort_at_root,
            ),
            (
                ids::INSP_ORDER_TOP_LEVEL,
                ord.top_level,
                ord.mixed.top_level,
            ),
        ] {
            if let Some(InteractiveState::Checkbox { value, .. }) = host.store_mut().get_mut(id) {
                *value = if mixed {
                    CheckboxValue::Indeterminate
                } else if on {
                    CheckboxValue::Checked
                } else {
                    CheckboxValue::Unchecked
                };
            }
        }
    }
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    for (id, value) in [
        (ids::INSP_ORDER_Z_INDEX, ord.z_index.unwrap_or(0) as f64),
        (ids::INSP_ORDER_ORDER_IN_LAYER, ord.order_in_layer as f64),
        (ids::INSP_ORDER_AXIS_X, ord.y_sort_axis[0] as f64),
        (ids::INSP_ORDER_AXIS_Y, ord.y_sort_axis[1] as f64),
    ] {
        if focus != Some(id) && drag != Some(id) {
            host.store_mut().set_number_value(id, value);
        }
    }
    // Sorting Layer dropdown selected index — seed on entity switch only
    // (the option click drives it between switches).
    if entity_changed
        && let Some(InteractiveState::Dropdown { selected_index, .. }) =
            host.store_mut().get_mut(ids::INSP_ORDER_SORTING_LAYER)
    {
        *selected_index = Some(ord.sorting_layer as usize);
    }
}

/// BulkSelect-aware reflection of the editable `Sprite` fields onto the
/// Inspector widgets — extracted from `sync_inspector_from_snapshots` for
/// the `architecture_panel_loc_cap` 200-LOC fn cap (logic verbatim). `sp`
/// is the live primary snapshot; `entity_changed` gates the
/// once-per-selection checkbox seed.
fn sync_sprite_fields(
    inspector_state: &mut state::InspectorState,
    host: &mut dyn PanelHostInternal,
    sp: &InspectorSpriteInfo,
    entity_changed: bool,
) {
    // Checkboxes (Flip H/V, Tint Fill): seed ONLY on entity switch.
    // A checkbox toggles its own stored value on click, and `sync`
    // runs AFTER the bus is drained, so an every-frame reseed from
    // the (still-stale-until-commit) snapshot would revert the
    // just-toggled value for one frame — a visible flicker (audit
    // F2). Between switches the widget holds the truth; the next
    // snapshot reflects the commit. Mirrors the Name TextInput.
    // Reseed on a selection CHANGE — the primary switching
    // (`entity_changed`) OR the selection size changing (BulkSelect:
    // adding/removing an extra). A diverging field seeds Indeterminate
    // ("Mixed"); toggling it then dispatches a definite value to the
    // whole selection (which un-mixes it next frame).
    let selection_changed =
        entity_changed || sp.selected_count != inspector_state.last_selected_count;
    inspector_state.last_selected_count = sp.selected_count;
    if selection_changed {
        for (id, on, mixed) in [
            (ids::INSP_SPRITE_FLIP_X, sp.flip_x, sp.mixed.flip_x),
            (ids::INSP_SPRITE_FLIP_Y, sp.flip_y, sp.mixed.flip_y),
            (ids::INSP_SPRITE_TINT_FILL, sp.tint_fill, sp.mixed.tint_fill),
            (
                ids::INSP_REGION_ENABLED,
                sp.region_enabled,
                sp.mixed.region_enabled,
            ),
            (
                ids::INSP_REGION_FILTER_CLIP,
                sp.region_filter_clip,
                sp.mixed.region_filter_clip,
            ),
            (ids::INSP_SPRITE_CENTERED, sp.centered, sp.mixed.centered),
        ] {
            if let Some(InteractiveState::Checkbox { value, .. }) = host.store_mut().get_mut(id) {
                *value = if mixed {
                    CheckboxValue::Indeterminate
                } else if on {
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
    let drag = host.store().number_input_drag().map(|d| d.id);
    for (id, value, mixed) in [
        (
            ids::INSP_SPRITE_HFRAMES,
            sp.hframes as f64,
            sp.mixed.hframes,
        ),
        (
            ids::INSP_SPRITE_VFRAMES,
            sp.vframes as f64,
            sp.mixed.vframes,
        ),
        (ids::INSP_SPRITE_FRAME, sp.frame as f64, sp.mixed.frame),
        (
            ids::INSP_REGION_X,
            sp.region_rect[0] as f64,
            sp.mixed.region_x,
        ),
        (
            ids::INSP_REGION_Y,
            sp.region_rect[1] as f64,
            sp.mixed.region_y,
        ),
        (
            ids::INSP_REGION_W,
            sp.region_rect[2] as f64,
            sp.mixed.region_w,
        ),
        (
            ids::INSP_REGION_H,
            sp.region_rect[3] as f64,
            sp.mixed.region_h,
        ),
        (
            ids::INSP_SPRITE_OFFSET_X,
            sp.offset[0] as f64,
            sp.mixed.offset_x,
        ),
        (
            ids::INSP_SPRITE_OFFSET_Y,
            sp.offset[1] as f64,
            sp.mixed.offset_y,
        ),
    ] {
        if focus != Some(id) && drag != Some(id) {
            // Mixed → blank the field (no misleading single value, and
            // a blank blur reverts cleanly — never stomps). Otherwise
            // reflect the primary's value every frame.
            if mixed {
                host.store_mut().blank_number_input(id);
            } else {
                host.store_mut().set_number_value(id, value);
            }
        }
    }
    // Opacity Slider (0..1 storage) + linked percent chip. Skip while
    // the slider is being dragged or the chip is focused so we don't
    // fight the user's input.
    let dragging = matches!(
        host.store().slider(ids::INSP_SPRITE_OPACITY),
        Some((SliderState::Dragging, _))
    );
    if !dragging
        && focus != Some(ids::INSP_SPRITE_OPACITY_CHIP)
        && drag != Some(ids::INSP_SPRITE_OPACITY_CHIP)
    {
        // The slider track can't show "Mixed", so it parks at the
        // primary's value; the blank percent chip is the Mixed signal.
        if let Some(InteractiveState::Slider { value, .. }) =
            host.store_mut().get_mut(ids::INSP_SPRITE_OPACITY)
        {
            *value = sp.opacity;
        }
        if sp.mixed.opacity {
            host.store_mut()
                .blank_number_input(ids::INSP_SPRITE_OPACITY_CHIP);
        } else {
            // Chip lives in display space (percent) per the integer map.
            host.store_mut().set_number_value(
                ids::INSP_SPRITE_OPACITY_CHIP,
                (sp.opacity * 100.0) as f64, // LITERAL-PX-OK: opacity percent scale
            );
        }
    }
    // Tint / Self Tint swatches. The BlenderColorPicker round-trips
    // the chosen color through `widget_color(<swatch>)` (mirrored
    // each frame from the picker in `hero.rs`, BEFORE this panel
    // paints). Two regimes:
    //   • picker targets THIS swatch → the user is actively picking:
    //     dispatch the divergence as a Sprite edit (live preview,
    //     same 1-frame post-commit lag the Opacity slider tolerates).
    //     Byte-compare against the committed channel so sub-1/255
    //     float dust doesn't spin the bus every frame, and so the
    //     stream stops the instant the commit lands (round-trip is
    //     exact: u8 → /255 → ×255 round-nearest → same u8).
    //   • otherwise → keep the swatch fill in lock-step with the
    //     committed channel so undo / external edits reflect.
    // On an entity switch the top `entity_changed` block has already
    // closed any tint picker bound to the prior selection, so this
    // reads `None` on the switch frame and the loop reseeds both
    // swatches from the new sprite's committed channels.
    let picker_target = host.store().picker_target();
    for (swatch_id, chan) in [
        (ids::INSP_SPRITE_TINT_SWATCH, sp.tint),
        (ids::INSP_SPRITE_SELF_TINT_SWATCH, sp.self_tint),
    ] {
        let committed = state::tint_f32_to_u8(chan);
        if picker_target == Some(swatch_id) {
            if let Some(picked) = host.store().widget_color(swatch_id)
                && picked != committed
            {
                let new_chan = state::tint_u8_to_f32(picked);
                let edit = if swatch_id == ids::INSP_SPRITE_TINT_SWATCH {
                    SpriteFieldEdit::Tint(new_chan)
                } else {
                    SpriteFieldEdit::SelfTint(new_chan)
                };
                host.bus_mut().push(EditorAction::InspectorSpriteEdit {
                    entity_bits: sp.entity_bits,
                    edit,
                });
            }
        } else {
            host.store_mut().set_widget_color(swatch_id, committed);
        }
    }
    // Per-corner tint swatches (TL, TR, BL, BR). Same regime as
    // Tint/Self, but `PerCornerTint` has no per-index variant, so the
    // edit carries the WHOLE [[f32;4];4] array with just the picked
    // corner replaced.
    let corner_ids = [
        ids::INSP_SPRITE_CORNER_TL,
        ids::INSP_SPRITE_CORNER_TR,
        ids::INSP_SPRITE_CORNER_BL,
        ids::INSP_SPRITE_CORNER_BR,
    ];
    for (i, &corner_id) in corner_ids.iter().enumerate() {
        let committed = state::tint_f32_to_u8(sp.per_corner_tint[i]);
        if picker_target == Some(corner_id) {
            if let Some(picked) = host.store().widget_color(corner_id)
                && picked != committed
            {
                let mut arr = sp.per_corner_tint;
                arr[i] = state::tint_u8_to_f32(picked);
                host.bus_mut().push(EditorAction::InspectorSpriteEdit {
                    entity_bits: sp.entity_bits,
                    edit: SpriteFieldEdit::PerCornerTint(arr),
                });
            }
        } else {
            host.store_mut().set_widget_color(corner_id, committed);
        }
    }
}

/// True for any of the 6 Sprite color-swatch ids (Tint, Self Tint, and
/// the 4 per-corner swatches) whose picker is bound to the current
/// selection — used to close the picker on an entity switch so the prior
/// sprite's picked color can't stream onto the next one.
fn is_sprite_color_swatch(id: ph2d_a11y::NodeId) -> bool {
    matches!(
        id,
        ids::INSP_SPRITE_TINT_SWATCH
            | ids::INSP_SPRITE_SELF_TINT_SWATCH
            | ids::INSP_SPRITE_CORNER_TL
            | ids::INSP_SPRITE_CORNER_TR
            | ids::INSP_SPRITE_CORNER_BL
            | ids::INSP_SPRITE_CORNER_BR
    )
}
