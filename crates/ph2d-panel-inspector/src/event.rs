//! Inspector panel `apply_event` — ADR-0029 Phase C.1 port.
//!
//! Migrated from `ph2d_editor_core::screens::hero::inspector::{mod,
//! apply_event_full}` to the panel crate. The signature changed from
//! `(hero: &mut HeroScreen, ev: WidgetEvent)` to
//! `(state: &mut InspectorState, host: &mut dyn PanelHostInternal,
//! ev: WidgetEvent)`. All `hero.<field>` accesses route through
//! [`PanelHostInternal`] trait methods.

use crate::state;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::screens::hero::{
    InspectorNameInfo, InspectorSpriteSource, InspectorTransformInfo, InspectorVisibilityInfo,
    RequestedSpriteStrategy,
};
use ph2d_editor_core::widget::{ButtonState, CheckboxValue};

pub(crate) fn apply_event(
    _state: &mut state::InspectorState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    EventOutcome::from_bool(apply_event_impl(host, ev))
}

fn apply_event_impl(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    // Section color-dot click — seed the canonical BlenderPicker
    // pointing at the section's color id, exact same flow Widget
    // Gallery uses for its `SECTION_COLOR_IDS`. The picker writes
    // chosen rgba back via `set_widget_color(<color_id>, rgba)`
    // (drained in hero.rs:760), and the next `paint_section_header`
    // call paints the dot in that color. UI canon 2026-05-24:
    // every section can carry a per-user accent color.
    if let WidgetEvent::Click(id) = ev
        && matches!(
            id,
            ids::INSP_LIVE_TRANSFORM_COLOR | ids::INSP_LIVE_RENDER_COLOR
        )
    {
        let seed = host
            .store()
            .widget_color(id)
            .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral seed
        host.store_mut().set_widget_color(id, seed);
        host.store_mut().set_picker_target(Some(id));
        host.store_mut().set_blender_value(
            ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(seed[0], seed[1], seed[2], seed[3]),
        );
        return true;
    }

    // Close (X) — hide the Inspector. Same effect as toggling the
    // left-rail Inspector pill (vide `chrome/rail_panels.rs`). UI canon
    // post-2026-05-24: every floating panel except Hierarchy has X.
    //
    // Sync the left-rail RAIL_SHOW_INSPECTOR button state so its
    // Pressed/Normal visual tracks the panel's actual visibility —
    // without this, hiding via X leaves the rail toggle stuck
    // Pressed (bug reported 2026-05-24).
    if let WidgetEvent::Click(id) = ev
        && id == ids::INSP_CLOSE
    {
        let next = !host.panel_visible("inspector");
        host.set_panel_visible("inspector", next);
        if let Some(InteractiveState::Button { state }) =
            host.store_mut().get_mut(ids::RAIL_SHOW_INSPECTOR)
        {
            *state = if next {
                ButtonState::Pressed
            } else {
                ButtonState::Normal
            };
        }
        return true;
    }
    // M14.5 inspector phase (6.4) — Reimport button.
    if let WidgetEvent::Click(id) = ev
        && id == ids::INSP_RENDER_SOURCE_REIMPORT
        && let Some(info) = state::current_inspector_sprite()
        && info.can_reimport
    {
        host.bus_mut().push(EditorAction::Reimport {
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
                | ids::INSP_TRANSFORM_SCALE_Y
                | ids::INSP_TRANSFORM_SKEW_X
                | ids::INSP_TRANSFORM_SKEW_Y,
        )
        && let Some(info) = state::current_inspector_transform()
    {
        let unit = host.project().display_unit;
        let ppm = host.project().pixels_per_meter;
        let x_disp =
            host.store()
                .number_value(ids::INSP_TRANSFORM_POS_X)
                .unwrap_or(unit.from_meters(info.translation[0], ppm) as f64) as f32;
        let y_disp =
            host.store()
                .number_value(ids::INSP_TRANSFORM_POS_Y)
                .unwrap_or(unit.from_meters(info.translation[1], ppm) as f64) as f32;
        let x = unit.to_meters(x_disp, ppm);
        let y = unit.to_meters(y_disp, ppm);
        let rot_deg = host
            .store()
            .number_value(ids::INSP_TRANSFORM_ROT)
            .unwrap_or((info.rotation_rad as f64).to_degrees()) as f32;
        let sx = host
            .store()
            .number_value(ids::INSP_TRANSFORM_SCALE_X)
            .unwrap_or(info.scale[0] as f64) as f32;
        let sy = host
            .store()
            .number_value(ids::INSP_TRANSFORM_SCALE_Y)
            .unwrap_or(info.scale[1] as f64) as f32;
        // Skew authored in degrees for UX parity with Rotation; the
        // ECS-commit boundary converts to radians and clamps to
        // Transform::SKEW_LIMIT (ADR-0025-amendment-1 §2.5).
        let skew_x_deg = host
            .store()
            .number_value(ids::INSP_TRANSFORM_SKEW_X)
            .unwrap_or((info.skew_rad[0] as f64).to_degrees()) as f32;
        let skew_y_deg = host
            .store()
            .number_value(ids::INSP_TRANSFORM_SKEW_Y)
            .unwrap_or((info.skew_rad[1] as f64).to_degrees()) as f32;
        host.bus_mut().push(EditorAction::InspectorTransformEdit(
            InspectorTransformInfo {
                entity_bits: info.entity_bits,
                translation: [x, y],
                rotation_rad: rot_deg.to_radians(),
                scale: [sx, sy],
                skew_rad: [skew_x_deg.to_radians(), skew_y_deg.to_radians()],
            },
        ));
        return true;
    }
    if let WidgetEvent::Click(id) = ev
        && id == ids::INSP_TRANSFORM_RESET
        && let Some(info) = state::current_inspector_transform()
    {
        host.bus_mut().push(EditorAction::InspectorTransformEdit(
            InspectorTransformInfo {
                entity_bits: info.entity_bits,
                translation: [0.0, 0.0],
                rotation_rad: 0.0,
                scale: [1.0, 1.0],
                skew_rad: [0.0, 0.0],
            },
        ));
        return true;
    }
    // M14.D — Visibility checkbox toggled.
    if let WidgetEvent::Toggled(id) = ev
        && id == ids::INSP_VISIBILITY_CHECK
        && let Some(info) = state::current_inspector_visibility()
    {
        let visible = matches!(
            host.store().checkbox(id).map(|(_, v)| v),
            Some(CheckboxValue::Checked),
        );
        host.bus_mut().push(EditorAction::InspectorVisibilityEdit(
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
        && let Some(info) = state::current_inspector_sprite()
    {
        let current = match info.source_kind {
            InspectorSpriteSource::Atlas { .. } => RequestedSpriteStrategy::Atlas,
            InspectorSpriteSource::Individual { .. } => RequestedSpriteStrategy::Individual,
            InspectorSpriteSource::HandPacked => RequestedSpriteStrategy::HandPacked,
        };
        if requested != current {
            host.bus_mut()
                .push(EditorAction::InspectorSpriteSourceChange {
                    entity_bits: info.entity_bits,
                    strategy: requested,
                });
        }
        if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
            *state = ButtonState::Normal;
        }
        return true;
    }
    // M14.E — entity-name TextInput edits.
    if let WidgetEvent::TextChanged(id) = ev
        && id == ids::INSP_ENTITY_NAME
        && let Some(info) = state::current_inspector_name()
    {
        let text = host.store().text(id).unwrap_or("").to_string();
        host.bus_mut()
            .push(EditorAction::InspectorNameEdit(InspectorNameInfo {
                entity_bits: info.entity_bits,
                name: text,
            }));
        return true;
    }
    // ADR-0029 Phase C.1: showcase-shared events
    // (`CTX_MENU_OUTLINE_*`, `CTX_MENU_CREATE_NOTE`, `SECTION_IDS`,
    // radio/tab/tree pinning) are now routed at host level via
    // `widget::showcase::apply_showcase_event`. The Inspector panel
    // returns `Ignored` for those — host picks them up after the
    // registry walk.
    false
}
