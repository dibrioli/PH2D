//! Inspector panel — full apply_event handler (Wave 6+7 Phase 4
//! distribution). Owns every inspector-related event branch that
//! previously lived in `HeroScreen::apply_event`'s god-match.
//!
//! Distinct from the existing `super::apply_event(&mut WidgetStore,
//! event)` helper which handles section header toggle / radio
//! group pin / context-menu items — that runs on the store alone
//! and stays as a sub-helper called from this thunk at the end.

use crate::action_bus::EditorAction;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::screens::hero::{
    HeroScreen, InspectorNameInfo, InspectorSpriteSource, InspectorTransformInfo,
    InspectorVisibilityInfo, RequestedSpriteStrategy, ids,
};

/// Try to handle `ev` against the inspector panel. Returns true iff
/// claimed. Covers:
///
/// - `INSP_RENDER_SOURCE_REIMPORT` click → push Reimport.
/// - Transform editor commits (5 NumberInputs) → push
///   `InspectorTransformEdit` with display-unit→meters conversion.
/// - `INSP_TRANSFORM_RESET` click → push Identity transform.
/// - `INSP_VISIBILITY_CHECK` toggled → push
///   `InspectorVisibilityEdit`.
/// - `INSP_RENDER_STRATEGY_*` click → push
///   `InspectorSpriteSourceChange`.
/// - `INSP_ENTITY_NAME` TextChanged → push `InspectorNameEdit`.
/// - Falls back to the existing `super::apply_event(&mut store,
///   ev)` helper for section header toggles + context-menu items.
pub fn apply_event_full(hero: &mut HeroScreen, ev: WidgetEvent) -> bool {
    // M14.5 inspector phase (6.4) — Reimport button.
    if let WidgetEvent::Click(id) = ev
        && id == ids::INSP_RENDER_SOURCE_REIMPORT
        && let Some(info) = hero.inspector.sprite.as_ref()
        && info.can_reimport
    {
        hero.bus.push(EditorAction::Reimport {
            entity_bits: info.entity_bits,
        });
        return true;
    }
    // M14.A — Transform editor commits.
    if let WidgetEvent::ValueChanged(id) = ev
        && matches!(
            id,
            ids::INSP_TRANSFORM_POS_X
                | ids::INSP_TRANSFORM_POS_Y
                | ids::INSP_TRANSFORM_ROT
                | ids::INSP_TRANSFORM_SCALE_X
                | ids::INSP_TRANSFORM_SCALE_Y,
        )
        && let Some(info) = hero.inspector.transform
    {
        let unit = hero.project.display_unit;
        let ppm = hero.project.pixels_per_meter;
        let x_disp =
            hero.store
                .number_value(ids::INSP_TRANSFORM_POS_X)
                .unwrap_or(unit.from_meters(info.translation[0], ppm) as f64) as f32;
        let y_disp =
            hero.store
                .number_value(ids::INSP_TRANSFORM_POS_Y)
                .unwrap_or(unit.from_meters(info.translation[1], ppm) as f64) as f32;
        let x = unit.to_meters(x_disp, ppm);
        let y = unit.to_meters(y_disp, ppm);
        let rot_deg = hero
            .store
            .number_value(ids::INSP_TRANSFORM_ROT)
            .unwrap_or((info.rotation_rad as f64).to_degrees()) as f32;
        let sx = hero
            .store
            .number_value(ids::INSP_TRANSFORM_SCALE_X)
            .unwrap_or(info.scale[0] as f64) as f32;
        let sy = hero
            .store
            .number_value(ids::INSP_TRANSFORM_SCALE_Y)
            .unwrap_or(info.scale[1] as f64) as f32;
        hero.bus.push(EditorAction::InspectorTransformEdit(
            InspectorTransformInfo {
                entity_bits: info.entity_bits,
                translation: [x, y],
                rotation_rad: rot_deg.to_radians(),
                scale: [sx, sy],
            },
        ));
        return true;
    }
    // Reset-to-Identity button.
    if let WidgetEvent::Click(id) = ev
        && id == ids::INSP_TRANSFORM_RESET
        && let Some(info) = hero.inspector.transform
    {
        hero.bus.push(EditorAction::InspectorTransformEdit(
            InspectorTransformInfo {
                entity_bits: info.entity_bits,
                translation: [0.0, 0.0],
                rotation_rad: 0.0,
                scale: [1.0, 1.0],
            },
        ));
        return true;
    }
    // M14.D — Visibility checkbox toggled.
    if let WidgetEvent::Toggled(id) = ev
        && id == ids::INSP_VISIBILITY_CHECK
        && let Some(info) = hero.inspector.visibility
    {
        let visible = matches!(
            hero.store.checkbox(id).map(|(_, v)| v),
            Some(crate::widget::CheckboxValue::Checked),
        );
        hero.bus.push(EditorAction::InspectorVisibilityEdit(
            InspectorVisibilityInfo {
                entity_bits: info.entity_bits,
                visible,
            },
        ));
        return true;
    }
    // M14.C — Render Source Strategy switcher.
    if let WidgetEvent::Click(id) = ev
        && let Some(requested) = match id {
            ids::INSP_RENDER_STRATEGY_ATLAS => Some(RequestedSpriteStrategy::Atlas),
            ids::INSP_RENDER_STRATEGY_INDIVIDUAL => Some(RequestedSpriteStrategy::Individual),
            ids::INSP_RENDER_STRATEGY_HANDPACKED => Some(RequestedSpriteStrategy::HandPacked),
            _ => None,
        }
        && let Some(info) = hero.inspector.sprite.as_ref()
    {
        let current = match info.source_kind {
            InspectorSpriteSource::Atlas { .. } => RequestedSpriteStrategy::Atlas,
            InspectorSpriteSource::Individual { .. } => RequestedSpriteStrategy::Individual,
            InspectorSpriteSource::HandPacked => RequestedSpriteStrategy::HandPacked,
        };
        if requested != current {
            hero.bus.push(EditorAction::InspectorSpriteSourceChange {
                entity_bits: info.entity_bits,
                strategy: requested,
            });
        }
        // Audit fix #7 — reset clicked button state to Normal so the
        // painter's snapshot-driven pin stays the single source of
        // visual truth.
        if let Some(InteractiveState::Button { state }) = hero.store.get_mut(id) {
            *state = crate::widget::ButtonState::Normal;
        }
        return true;
    }
    // M14.E — entity-name TextInput edits (live commit on every
    // TextChanged).
    if let WidgetEvent::TextChanged(id) = ev
        && id == ids::INSP_ENTITY_NAME
        && let Some(info) = hero.inspector.name.as_ref()
    {
        let text = hero.store.text(id).unwrap_or("").to_string();
        hero.bus
            .push(EditorAction::InspectorNameEdit(InspectorNameInfo {
                entity_bits: info.entity_bits,
                name: text,
            }));
        return true;
    }
    // Fall back to the section-header / radio-pin / context-menu
    // helper that operates on the store alone.
    super::apply_event(&mut hero.store, ev)
}
