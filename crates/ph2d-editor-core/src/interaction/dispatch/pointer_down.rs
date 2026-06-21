//! Pointer **Down** dispatch arm. Extracted from the
//! `dispatch_pointer_with_text` god-function (blindagem Fase 3.2) — pure move,
//! same `super::` paths, same behaviour (covered by `dispatch::tests`). The
//! right-click / TopBar menu openers live in `pointer_down_menus`.

use super::blender::apply_blender_hit;
use super::curve::apply_curve_point_drag;
use super::focus::is_focusable;
use super::hover::set_widget_pressed;
use super::number_input::{apply_number_stepper_if_hit, update_drag_value};
use super::scroll::scrollbar_panel_for_id;
use super::text_ops::{byte_offset_from_click_xy, place_text_caret};
use super::{
    commit_hex_buffer, commit_number_buffer, init_number_buffer, reset_focused_visual_state,
    select_all_in_text_widget,
};
use crate::interaction::types::{BlenderHitKind, ContextMenuKind};
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore, drag};
use crate::zones::Rect;
use bumpalo::collections::Vec as BumpVec;
use ph2d_host::PointerEvent;
use ph2d_text::TextSystem;

/// Handle a pointer-`Down` event: opens context menus (delegated), updates
/// focus/active, seeds drags (slider/hierarchy/painter-layer/scrollbar/number-
/// input) and emits Focus/ValueChanged/etc. for the hit widget.
pub(super) fn dispatch_down<'frame>(
    store: &mut WidgetStore,
    hit_index: &HitIndex,
    event: PointerEvent,
    mut ts: Option<&mut TextSystem>,
    events: &mut BumpVec<'frame, WidgetEvent>,
) {
    let hit = hit_index.hit_with_rect(event.x, event.y);

    // Right-click context menus + TopBar/chip popovers. On a handled event the
    // original arm returned immediately — preserve that here.
    if super::pointer_down_menus::handle_down_menus(store, hit, event) {
        return;
    }
    // Primary click elsewhere closes any open menu before
    // running the regular focus/click path. The SceneList
    // popover hosts a TextInput (its search field) — clicks
    // on that input must NOT close the menu so the user can
    // type into it. Same for any scene row (the click is
    // routed via apply_event which closes the menu after
    // updating the chip).
    if store.context_menu().is_some() {
        let hit_id = hit.map(|(id, _)| id);
        let inside_scene_list = matches!(
            store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::SceneList)
        ) && matches!(
            hit_id,
            Some(id) if id == crate::ids::CTX_SCENE_SEARCH
                || crate::ids::CTX_SCENE_ROWS.contains(&id)
        );
        // Same rule for the API-key popover (P4): clicking its TextInput
        // (to focus + type/paste) or its Save row must NOT close the menu.
        let inside_api_key = matches!(
            store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::SettingsApiKeySubmenu)
        ) && matches!(
            hit_id,
            Some(id) if id == crate::ids::CTX_MENU_API_KEY_INPUT
                || id == crate::ids::CTX_MENU_API_KEY_SAVE
        );
        // And the LLM vector-prompt dialog (P4): its input + Generate button.
        let inside_prompt = matches!(
            store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::VectorPromptDialog)
        ) && matches!(
            hit_id,
            Some(id) if id == crate::ids::CTX_MENU_VECTOR_PROMPT_INPUT
                || id == crate::ids::CTX_MENU_VECTOR_PROMPT_GENERATE
        );
        // Only an empty-space click dismisses the menu on the Down. A click on a
        // REGISTERED widget — a menu item, or the special-menu inputs above —
        // keeps the menu open: a menu item's handler closes it on the Up (via
        // `apply_event`). Closing on the Down broke item clicks whenever a frame
        // repainted between Down and Up (e.g. the Painter's continuous preview):
        // the closed menu un-registered its items, so the Up landed on the widget
        // underneath and never produced the item `Click`.
        if hit_id.is_none() && !inside_scene_list && !inside_api_key && !inside_prompt {
            store.close_context_menu();
        }
    }

    // Close the global color picker when the click lands
    // OUTSIDE the picker's outer rect AND outside another
    // color-target widget. Color-target ids re-open the
    // picker via `apply_event`; clicking anywhere else
    // dismisses it.
    //
    // Test the OUTER RECT (published each frame to
    // `panel_rect(INSP_BLENDER_PICKER)`) rather than only
    // the BlenderHit sub-controls. Without this, dead space
    // INSIDE the picker (gaps between controls, padding
    // bands, the drag bar's left/right ends) had no hit and
    // the picker dismissed itself — user reported "if I
    // click inside the panel but not on any control, it
    // closes". Fallback to the sub-control test when the
    // outer rect isn't published (e.g. first frame).
    if store.picker_target().is_some() {
        use crate::ids as hero_ids;
        let inside_outer = store
            .panel_rect(hero_ids::INSP_BLENDER_PICKER)
            .map(|r| r.contains(event.x, event.y))
            .unwrap_or(false);
        let inside_sub = matches!(
            hit.and_then(|(id, _)| store.get(id)),
            Some(InteractiveState::BlenderHit { .. })
        );
        let is_color_target = hit.map(|(id, _)| is_color_target_id(id)).unwrap_or(false);
        // Don't dismiss while an eyedropper pick is armed: a click
        // OUTSIDE the picker is the user sampling a canvas pixel, not
        // dismissing. Without this guard the dismiss (here) fires
        // before the eyedropper interception below, closing the
        // popover before the sample registers (Enio smoke W2.T2.4).
        let eyedropper_armed = store.eyedropper_pending().is_some();
        if !inside_outer && !inside_sub && !is_color_target && !eyedropper_armed {
            store.set_picker_target(None);
        }
    }

    // Eyedropper interception: while a pick is pending and
    // the click isn't on the eyedropper button itself, emit
    // `EyedropperPick` for the host to read back the
    // rendered pixel. Skip the rest of the Down logic so
    // we don't focus / drag whatever's under the cursor.
    if let Some(parent) = store.eyedropper_pending() {
        let is_eyedropper_btn = matches!(
            hit.and_then(|(id, _)| store.get(id)),
            Some(InteractiveState::BlenderHit {
                kind: BlenderHitKind::Eyedropper,
                ..
            })
        );
        if !is_eyedropper_btn {
            events.push(WidgetEvent::EyedropperPick {
                parent,
                px: event.x.max(0.0) as u32,
                py: event.y.max(0.0) as u32,
            });
            store.set_eyedropper_pending(None);
            return;
        }
    }

    // Picker swatch → open the shared Blender picker seeded with the
    // swatch's last published color. GENERALIZED (was a per-id
    // `PAINTER_COLOR_THUMB` special-case): any panel that paints a
    // `ColorSwatch` and calls `store.register_picker_swatch(id)` gets
    // this for free (Painter brush color, Vector fill, …). The bridge /
    // panel keeps `widget_color(id)` synced to the live color. Short-
    // circuit the rest of Down so the click doesn't focus/drag the canvas.
    if let Some((id, _)) = hit
        && store.is_picker_swatch(id)
    {
        let seed = store.widget_color(id).unwrap_or([0x88, 0x88, 0x88, 0xFF]);
        store.set_widget_color(id, seed);
        store.set_picker_target(Some(id));
        store.set_blender_value(
            crate::ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(seed[0], seed[1], seed[2], seed[3]),
        );
        return;
    }

    // Compute the new focus target (if the click landed on a
    // focusable widget). Blur+commit the previous focus
    // whenever it isn't the same target — including the case
    // where the click landed in dead space (canvas, panel
    // chrome, etc.) so the user's typed buffer always
    // commits when the field loses focus.
    let new_focus = match hit {
        Some((id, _)) if is_focusable(store, id) => Some(id),
        _ => None,
    };

    // Bump the clicked panel to the top of the z-order so it
    // paints over its siblings. Iterate the canonical panel
    // ids and pick whichever one's published rect contains
    // the click. The picker takes precedence because it's the
    // only floating panel and visually overlaps the others
    // when displayed.
    {
        use crate::ids as hero_ids;
        const PANEL_IDS: [ph2d_a11y::NodeId; 3] = [
            hero_ids::INSP_BLENDER_PICKER,
            hero_ids::INSP_PANEL,
            hero_ids::HIER_PANEL,
        ];
        for panel_id in PANEL_IDS {
            if let Some(r) = store.panel_rect(panel_id)
                && r.contains(event.x, event.y)
            {
                store.bump_panel_z(panel_id);
                break;
            }
        }
    }
    let prev_focus = store.focus_id();
    if let Some(old) = prev_focus
        && new_focus != Some(old)
    {
        commit_number_buffer(store, old, events);
        commit_hex_buffer(store, old, events);
        reset_focused_visual_state(store, old);
        events.push(WidgetEvent::Blur(old));
        store.set_focus(None);
    }

    // Detect double-click against the previous Down. Use the
    // raw hit id (not `new_focus`) so hierarchy companion ids
    // — which are hit-registered but absent from `WidgetStore`,
    // so `is_focusable` rejects them and `new_focus` is None —
    // can still arm `pending_double_click`. Without this,
    // double-click on the hierarchy entity-icon never upgrades
    // to `WidgetEvent::DoubleClick(icon_companion_id)` and the
    // panel's "focus camera on row" gesture never fires
    // (Enio 2026-05-26: "O duplo clique no ícone deveria focar
    // o objeto no canvas, mas não funciona").
    let down_id = hit.map(|(id, _)| id);
    let is_double_click = store.record_pointer_down(down_id, event.timestamp_ns);

    // Hierarchy chrome companions (eye toggle, chevron toggle,
    // lock toggle, group toggle, entity-icon focus) are
    // registered in HitIndex but NOT in WidgetStore — the
    // painter has no &mut WidgetStore. The `is_focusable` gate
    // below would reject them. Capture them as ephemeral
    // buttons: set active+rect on Down so the Up branch fires
    // `apply_click`, whose `_` fallthrough pushes a generic
    // `WidgetEvent::Click(id)` for unregistered ids. Hero's
    // `apply_event` then routes by companion bit pattern.
    // Enio 2026-05-26: lock + group + icon companions adicionados
    // — sem isso clicks neles iam pra is_focusable, eram rejei-
    // tados como não-registrados, e nunca emitiam Click.
    if let Some((id, rect)) = hit
        && (crate::ids::hier_eye_companion_to_row(id).is_some()
            || crate::ids::hier_expand_companion_to_row(id).is_some()
            || crate::ids::hier_lock_companion_to_row(id).is_some()
            || crate::ids::hier_group_companion_to_row(id).is_some()
            || crate::ids::hier_icon_companion_to_row(id).is_some())
    {
        store.set_active(Some(id));
        store.set_active_rect(Some(rect));
        return;
    }

    // W4 §3 — Down on a curve control point: make it active (so Move
    // drags it) and apply the initial position now (click-to-move).
    // Handled before `is_focusable` so it skips the focus/number-buffer
    // machinery — the curve lives in the painter tool, not the store.
    if let Some((id, rect)) = hit
        && matches!(store.get(id), Some(InteractiveState::CurvePoint { .. }))
    {
        store.set_active(Some(id));
        // Set active_rect too: the Move-drag block is gated on
        // `active_rect.is_some()`. The VALUE is irrelevant to the mapping
        // (`apply_curve_point_drag` normalizes against the variant's
        // `canvas`, not this rect) — only its PRESENCE unlocks the drag.
        store.set_active_rect(Some(rect));
        if let Some(parent) = apply_curve_point_drag(store, id, event.x, event.y) {
            events.push(WidgetEvent::ValueChanged(parent));
        }
        return;
    }

    if let Some((id, rect)) = hit
        && is_focusable(store, id)
    {
        store.set_active(Some(id));
        store.set_active_rect(Some(rect));
        if store.focus_id() != Some(id) {
            store.set_focus(Some(id));
            init_number_buffer(store, id);
            match store.get_mut(id) {
                Some(InteractiveState::TextInput { state, .. }) => {
                    *state = crate::widget::TextInputState::Focused;
                }
                Some(InteractiveState::Combobox { state, .. }) => {
                    *state = crate::widget::ComboboxState::Focused;
                }
                _ => {}
            }
            events.push(WidgetEvent::Focus(id));
        }
        // Inline clear-✕ on the Combobox right edge. Takes
        // precedence over both caret placement AND
        // double-click select-all: clicking the X is the
        // unambiguous "wipe the query" gesture.
        let combo_cleared = clear_combobox_if_button_hit(store, id, rect, event.x, event.y);
        // NumberInput up/down steppers — same precedence
        // (click on a stepper bumps the value; doesn't move
        // the caret or trigger select-all). M14.A also seeds
        // a continuous-hold state so dispatch_tick can repeat
        // while the arrow stays pressed.
        //
        // Skip the stepper hit-test entirely for chips painted
        // as bare pills (`paint_number_chip`) — those don't
        // draw arrows but the dispatch carves the right column
        // out of every NumberInput's hit rect by default,
        // producing a phantom continuous-hold that keeps
        // climbing while the pointer is still.
        let stepper_hit = !combo_cleared
            && !store.is_chip_no_stepper(id)
            && apply_number_stepper_if_hit(store, id, rect, event.x, event.y, event.timestamp_ns);
        // M14.A drag-or-edit: when the Down lands on a
        // NumberInput body (NOT on the stepper, NOT a
        // double-click that triggers select-all), record a
        // drag candidate. Move events past the threshold flip
        // it into slider mode; Up before then leaves edit
        // mode active (the focus / `init_number_buffer` calls
        // above already entered edit state). HR-3-safe — the
        // capture is a single `Copy`-struct, no allocation.
        if !combo_cleared
            && !stepper_hit
            && !is_double_click
            && let Some(InteractiveState::NumberInput { value, buffer, .. }) = store.get(id)
        {
            let step = if buffer.contains('.') { 0.01 } else { 1.0 };
            let drag = drag::NumberInputDragState {
                id,
                start_x: event.x,
                start_y: event.y,
                start_value: *value,
                // Seed `last_x` / `last_y` at the Down position
                // so the FIRST post-threshold Move's incremental
                // delta is measured against Down (matching the
                // old absolute-from-Down behaviour for the
                // initial Move). Subsequent Moves advance the
                // anchor.
                last_x: event.x,
                last_y: event.y,
                step,
                crossed_threshold: false,
                // Axis is decided at the moment the threshold
                // flips (in the Move handler); the field's
                // default before that is irrelevant since
                // `crossed_threshold == false` short-circuits
                // the slider math.
                axis_horizontal: false,
                // Caret offset is already placed below via
                // the regular `place_text_caret` call; we
                // duplicate the field for completeness so a
                // future "defer caret-place" refactor has the
                // data on hand.
                caret_offset_at_down: 0,
            };
            store.begin_number_input_drag(drag);
        }
        if combo_cleared {
            events.push(WidgetEvent::TextChanged(id));
        } else if stepper_hit {
            events.push(WidgetEvent::ValueChanged(id));
            // `apply_number_stepper_if_hit` also writes the
            // linked slider via `apply_chip_value_with_mirror`;
            // emit its ValueChanged so panel handlers keyed off
            // the slider id (canonical pattern post-mapped-link)
            // see the change in lockstep with the chip event.
            if let Some(slider_id) = store.linked_slider(id)
                && matches!(store.get(slider_id), Some(InteractiveState::Slider { .. }))
            {
                events.push(WidgetEvent::ValueChanged(slider_id));
            }
        } else if is_double_click {
            let is_text_widget = matches!(
                store.get(id),
                Some(InteractiveState::TextInput { .. })
                    | Some(InteractiveState::NumberInput { .. })
                    | Some(InteractiveState::Combobox { .. })
            );
            select_all_in_text_widget(store, id);
            if is_text_widget {
                // Clear active_rect so the next Move event
                // (almost always present from mouse jitter on
                // release) doesn't re-enter the text drag-to-
                // select branch and shrink the selection back
                // to (0..clicked_byte). The Up handler still
                // uses active_id for release/click cleanup.
                // Guarded to text widgets — clearing for
                // every widget breaks click-toggle handlers
                // that read active_rect on Up.
                store.set_active_rect(None);
            }
        } else if matches!(
            store.get(id),
            Some(InteractiveState::TextInput { .. })
                | Some(InteractiveState::NumberInput { .. })
                | Some(InteractiveState::Combobox { .. })
        ) {
            // Single Down on a text widget: place caret at
            // the clicked byte position and seed the
            // selection anchor there. Subsequent Move events
            // extend the selection from anchor → new caret.
            let offset = byte_offset_from_click_xy(store, id, rect, event.x, event.y, ts.take());
            place_text_caret(store, id, offset, true);
        }
        set_widget_pressed(store, id);
        // For sliders, the initial Down also sets value
        // (jump-to-clicked-position behavior).
        if matches!(store.get(id), Some(InteractiveState::Slider { .. }))
            && update_drag_value(store, id, rect, event.x, event.y)
        {
            events.push(WidgetEvent::ValueChanged(id));
        }
        // Hierarchy row Down → seed a drag candidate. Up
        // without movement is treated as a click (selection);
        // Up after the threshold is exceeded reorders the
        // entity. The threshold check is in
        // `update_hierarchy_drag` (Move handler).
        //
        // Pre-M14.6B: the criterion was `is_hierarchy_entity_id`
        // (fixture range 400..=411), which silently dropped
        // every live ECS-bridge row (ids start at 100_000+).
        // Now we ask the store, which the hierarchy painter
        // updates per-frame in both modes.
        if store.is_hierarchy_row(id) {
            store.begin_hierarchy_drag(id, event.x, event.y, event.timestamp_ns);
        }
        // Painter layers-panel row drag (W3 T3.8) — same anchor as the
        // hierarchy; the Up handler resolves the drop into a
        // `PainterLayerReparent` for the painter tool to apply.
        if store.is_painter_layer_row(id) {
            store.begin_painter_layer_drag(id, event.x, event.y, event.timestamp_ns);
        }
        // Scrollbar thumb drag — snapshot the panel
        // metrics so subsequent Move events can compute a
        // proportional `panel_scroll` delta. The
        // scrollbar id encodes its panel (see helper
        // below); the metrics come from the side-tables
        // the painters publish each frame.
        if let Some(panel) = scrollbar_panel_for_id(id)
            && let (Some(content_h), Some(visible_h)) =
                (store.panel_content_h(panel), store.panel_visible_h(panel))
        {
            store.begin_scrollbar_drag(drag::ScrollbarDragAnchor {
                panel,
                cursor_y_at_down: event.y,
                scroll_at_down: store.panel_scroll(panel),
                track_h: rect.h,
                content_h,
                visible_h,
            });
        }
        // BlenderColorPicker sub-control hits route into the
        // parent's stored state mutation. Right-click on a
        // palette swatch removes it instead of picking it.
        if let Some(parent) = apply_blender_hit(store, id, rect, event.x, event.y, event.button) {
            events.push(WidgetEvent::ValueChanged(parent));
        }
    }
}

/// True iff a `Down` on `id` should RE-OPEN the colour picker (a colour-target
/// widget) rather than dismiss it. Mix of legacy numeric ranges + the hashed
/// Painter colour thumb.
fn is_color_target_id(id: ph2d_a11y::NodeId) -> bool {
    let v = id.0;
    // W2.T2.3: the Painter color thumb is a hashed id (outside the raw
    // legacy range), so it can't be matched arithmetically — clicking it
    // re-opens the picker rather than dismissing it.
    (360..=369).contains(&v) || v == 328 || id == crate::ids::PAINTER_COLOR_THUMB
}

/// Detect a Down landing on the Combobox's inline clear-✕ icon. When
/// the widget at `id` is a Combobox with a non-empty query and the
/// click coordinates fall inside `Combobox::clear_button_rect(host)`,
/// wipe the query/caret/selection and return true. Returns false in
/// every other case (non-Combobox, empty query, click outside the X).
/// The caller emits `WidgetEvent::TextChanged(id)` on a true return.
fn clear_combobox_if_button_hit(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    host: Rect,
    click_x: f32,
    click_y: f32,
) -> bool {
    use crate::widget::{Combobox, ComboboxOption};
    // Build a throw-away Combobox snapshot just to reuse the
    // widget-side `clear_button_rect` math — keeps geometry in
    // exactly one place (the widget). Cost: one empty Vec alloc per
    // Down event, dwarfed by the bumpalo event arena cost.
    let (query, state) = match store.get(id) {
        Some(InteractiveState::Combobox { query, state, .. }) => (query.clone(), *state),
        _ => return false,
    };
    let probe = Combobox::new(id, "", Vec::<ComboboxOption>::new())
        .query(query)
        .state(state);
    let Some(btn_rect) = probe.clear_button_rect(host) else {
        return false;
    };
    if !btn_rect.contains(click_x, click_y) {
        return false;
    }
    if let Some(InteractiveState::Combobox {
        query,
        caret,
        selection_anchor,
        ..
    }) = store.get_mut(id)
    {
        query.clear();
        *caret = 0;
        *selection_anchor = None;
        return true;
    }
    false
}
