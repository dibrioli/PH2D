//! Inspector store-sync — once per frame, before `paint_inspector`.
//!
//! Extracted from `paint_hero_screen` (Track E3) so the giant paint
//! loop in [`super::super::hero`] doesn't carry the 100-line snapshot-
//! reconciliation logic inline. Behavior is byte-identical to the
//! previous in-loop implementation.
//!
//! What this does each frame, when the Inspector panel is the one
//! about to paint:
//!
//! 1. Detect whether the selected entity changed since last frame
//!    (any of `inspector_transform / inspector_name / inspector_visibility`
//!    can drive the entity-id signal).
//! 2. If it changed: drop focus, cancel any in-flight NumberInput
//!    drag/stepper-hold (audit fix #3 from M14.A), force-rewrite the
//!    5 Transform NumberInput buffers + the editable name TextInput
//!    buffer from the new snapshot, reset that TextInput's state to
//!    `Normal` (audit pass #2), and store the new entity-id.
//! 3. If it didn't change: focus-guarded refresh of the 5 Transform
//!    fields so gizmo-driven mutations propagate to the non-focused
//!    NumberInputs while the user keeps typing in the focused one.
//! 4. Sync the Visibility checkbox value from `inspector_visibility`
//!    unless `pending_visibility_edit` is already queued — that
//!    would mean the user just clicked and the shell hasn't drained
//!    yet, so the snapshot is one frame stale (audit CRITICAL #4).

use super::HeroScreen;
use super::ids;
use crate::interaction::InteractiveState;

/// Reconcile the WidgetStore's Inspector slots against the host-
/// supplied snapshots (`hero.inspector_*`).
///
/// See the module docs for the three-phase contract (entity-changed
/// force-rewrite vs. same-entity focus-guarded refresh, plus the
/// Visibility skip-when-pending guard).
pub(super) fn sync_inspector_from_snapshots(hero: &mut HeroScreen) {
    // M14.A: when the selected entity changes, force-rewrite the 5
    // Transform NumberInput buffers from the new snapshot AND end
    // any in-flight drag/stepper-hold — otherwise the orphaned state
    // would keep ticking against the new entity with the old
    // `start_value` (audit fix #3). Same selection-id is reused for
    // Name and Visibility so any of those snapshots can drive the
    // entity_changed flag.
    let new_entity = hero
        .inspector_transform
        .map(|i| i.entity_bits)
        .or_else(|| hero.inspector_name.as_ref().map(|i| i.entity_bits))
        .or_else(|| hero.inspector_visibility.map(|i| i.entity_bits));
    let entity_changed = new_entity != hero.last_inspector_entity;
    if entity_changed {
        // Drop focus + cancel any drag/stepper-hold so the next
        // force-rewrite isn't fighting in-progress state from the
        // previous entity.
        hero.store.set_focus(None);
        let _ = hero.store.end_number_input_drag();
        hero.store.end_number_stepper_hold();
        if let Some(info) = hero.inspector_transform {
            hero.store
                .set_number_value(ids::INSP_TRANSFORM_POS_X, info.translation[0] as f64);
            hero.store
                .set_number_value(ids::INSP_TRANSFORM_POS_Y, info.translation[1] as f64);
            hero.store.set_number_value(
                ids::INSP_TRANSFORM_ROT,
                info.rotation_rad.to_degrees() as f64,
            );
            hero.store
                .set_number_value(ids::INSP_TRANSFORM_SCALE_X, info.scale[0] as f64);
            hero.store
                .set_number_value(ids::INSP_TRANSFORM_SCALE_Y, info.scale[1] as f64);
        }
        // M14.E: force-rewrite the editable name TextInput buffer on
        // selection change so the previous entity's in-progress
        // typed-but-uncommitted edit can't leak onto the new entity.
        // Audit #2 fix: also flip `state` back to `Normal` — the
        // global `set_focus(None)` above only clears the focus_id;
        // the per-widget state field is what the painter consults to
        // draw caret + focus ring. Without this the painter keeps
        // the focused chrome on a field the user hasn't authored yet
        // (cosmetic but confusing — same pattern dispatch.rs:1189
        // uses on Blur).
        if let Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) = hero.store.get_mut(ids::INSP_ENTITY_NAME)
        {
            *state = crate::widget::TextInputState::Normal;
            text.clear();
            if let Some(info) = hero.inspector_name.as_ref() {
                text.push_str(&info.name);
            }
            *caret = text.len();
            *selection_anchor = None;
        }
        hero.last_inspector_entity = new_entity;
    } else {
        if let Some(info) = hero.inspector_transform {
            // Same entity — focus-guarded refresh (lets the user keep
            // typing while gizmo-driven mutations propagate to the
            // non-focused fields).
            hero.store
                .set_number_value(ids::INSP_TRANSFORM_POS_X, info.translation[0] as f64);
            hero.store
                .set_number_value(ids::INSP_TRANSFORM_POS_Y, info.translation[1] as f64);
            hero.store.set_number_value(
                ids::INSP_TRANSFORM_ROT,
                info.rotation_rad.to_degrees() as f64,
            );
            hero.store
                .set_number_value(ids::INSP_TRANSFORM_SCALE_X, info.scale[0] as f64);
            hero.store
                .set_number_value(ids::INSP_TRANSFORM_SCALE_Y, info.scale[1] as f64);
        }
        // Same-entity name refresh — propagates external renames
        // (e.g. via the Hierarchy panel's inline rename mode) into
        // the Inspector's name TextInput. Guards (mirror the
        // Visibility skip-when-pending pattern):
        //   - `pending_name_edit.is_some()` — user just typed in the
        //     Inspector field and the shell hasn't drained the edit
        //     yet; rewriting would clobber the in-progress commit.
        //   - `focus_id == Some(INSP_ENTITY_NAME)` — user is actively
        //     editing the Inspector field. Don't stomp their caret /
        //     selection mid-typing.
        //   - buffer already matches snapshot — no-op, avoid a
        //     spurious caret reset every frame.
        let focused = hero.store.focus_id() == Some(ids::INSP_ENTITY_NAME);
        if hero.pending_name_edit.is_none()
            && !focused
            && let Some(info) = hero.inspector_name.as_ref()
            && let Some(InteractiveState::TextInput {
                text,
                caret,
                selection_anchor,
                ..
            }) = hero.store.get_mut(ids::INSP_ENTITY_NAME)
            && text.as_str() != info.name.as_str()
        {
            text.clear();
            text.push_str(&info.name);
            *caret = text.len();
            *selection_anchor = None;
        }
    }
    // M14.D + audit fix #4: sync the Visibility checkbox value from
    // the snapshot UNLESS a `pending_visibility_edit` is already
    // queued — that means the user just clicked the checkbox AND
    // the shell hasn't drained yet. Without the skip we'd stomp the
    // just-toggled UI state back to the pre-click value for one
    // frame.
    if hero.pending_visibility_edit.is_none()
        && let Some(vis) = hero.inspector_visibility
        && let Some(InteractiveState::Checkbox { value, .. }) =
            hero.store.get_mut(ids::INSP_VISIBILITY_CHECK)
    {
        *value = if vis.visible {
            crate::widget::CheckboxValue::Checked
        } else {
            crate::widget::CheckboxValue::Unchecked
        };
    }
}
