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
use crate::interaction::ContextMenuKind;
use crate::interaction::flip_strip::FlipStripGesture;
use crate::interaction::types::{BlenderHitKind, GesturePhase, GraphGesture, TimelineGesture};
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

    // With a context menu open, a non-Secondary Down on a graph / timeline
    // surface is a DISMISSAL, not a gesture: the menus float OVER those
    // surfaces, so "clicking outside the menu" usually lands on one — and the
    // captures below would both start a gesture under the menu and `return`
    // before the close-on-outside block ever ran (an easing submenu that never
    // closed, Enio 2026-07-11). Fully consumed: the dismissing click must not
    // box-select / drag what it happened to land on. Secondary falls through —
    // right-click relocates the menu via `handle_down_menus` / the graph's own
    // add-menu, exactly as with no menu open.
    if store.context_menu().is_some()
        && event.button != ph2d_host::PointerButton::Secondary
        && let Some((id, _)) = hit
        && (store.graph_surface_at_id(id).is_some()
            || store.timeline_surface_at_id(id).is_some()
            || store.flip_strip_surface_at_id(id).is_some())
    {
        store.close_context_menu();
        return;
    }

    // Motion Nodes M0.T3 — a graph surface captures the pointer for ALL buttons
    // (incl. Secondary/Middle), BEFORE the context-menu delegation, so a
    // right-click on the graph opens the graph's own add-menu (panel-side)
    // instead of the global note menu, and a middle-drag reaches the graph pan.
    if let Some((id, rect)) = hit
        && let Some((surface, kind)) = store.graph_surface_at_id(id)
    {
        store.set_active(Some(id));
        store.set_active_rect(Some(rect));
        store.set_graph_moved(false);
        // Record the Down for double-click detection HERE: this capture returns early,
        // past the general path at the bottom, so the graph would otherwise never see a
        // double-click (the Up reads the flag back to upgrade `Click` → `DoubleClick`).
        let is_double = store.record_pointer_down(Some(id), event.timestamp_ns);
        store.set_graph_double(is_double);
        let mods = store.gesture_mods();
        store.push_graph_gesture(GraphGesture {
            surface,
            kind,
            phase: GesturePhase::Begin,
            x: event.x,
            y: event.y,
            button: event.button,
            mods,
        });
        return;
    }

    // W2.E5b — the timeline dope-sheet captures the pointer the same way (a key
    // diamond or an empty lane); the panel drives select / drag-move / clear off
    // the gesture stream. Mirror of the graph-surface capture above.
    //
    // Secondary is NOT captured: right-click over the dope sheet belongs to the
    // context menu (`handle_down_menus`, W3.E4), and capturing it here would
    // return before that ever ran — a menu that compiles, tests green, and never
    // opens. The panel's gesture loop reserves Secondary for exactly this reason.
    if event.button != ph2d_host::PointerButton::Secondary
        && let Some((id, rect)) = hit
        && let Some((surface, kind)) = store.timeline_surface_at_id(id)
    {
        store.set_active(Some(id));
        store.set_active_rect(Some(rect));
        // Record the down for double-click detection here (the general path at
        // the bottom returns early past this capture, so it never runs for a
        // timeline surface). The Up reads the flag back to open a marker rename.
        let is_double = store.record_pointer_down(Some(id), event.timestamp_ns);
        store.begin_timeline_press(event.x, event.y, is_double);
        let mods = store.gesture_mods();
        store.push_timeline_gesture(TimelineGesture {
            surface,
            kind,
            phase: GesturePhase::Begin,
            x: event.x,
            y: event.y,
            button: event.button,
            mods,
        });
        return;
    }

    // A tira de frames do Flip captura pelo MESMO caminho (uma célula ou a borda
    // que define o hold); o painel decide selecionar / mover a chave / esticar a
    // exposição a partir do fluxo de gestos.
    //
    // Secondary NÃO é capturado, pela mesma razão dos irmãos acima: o botão direito
    // sobre a tira pertence ao menu de contexto (`handle_down_menus`), e capturá-lo
    // aqui retornaria antes de ele existir.
    if event.button != ph2d_host::PointerButton::Secondary
        && let Some((id, rect)) = hit
        && let Some((surface, kind)) = store.flip_strip_surface_at_id(id)
    {
        store.set_active(Some(id));
        store.set_active_rect(Some(rect));
        store.begin_flip_strip_press(event.x, event.y);
        let mods = store.gesture_mods();
        store.push_flip_strip_gesture(FlipStripGesture {
            surface,
            kind,
            phase: GesturePhase::Begin,
            x: event.x,
            y: event.y,
            button: event.button,
            mods,
        });
        return;
    }

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
        // And the palette-rename modal: its shared name field + Rename button.
        let inside_palette_rename = matches!(
            store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::RenamePaletteDialog)
        ) && matches!(
            hit_id,
            Some(id) if id == crate::ids::BLENDER_PALETTE_NAME
                || id == crate::ids::CTX_MENU_PALETTE_RENAME
        );
        // And the New-image modal: clicking a Size / Background radio or Create must NOT dismiss it
        // (the user picks size, then background, then Create — all in one open).
        let inside_new_image = matches!(
            store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::NewImageDialog)
        ) && hit_id.is_some_and(|id| {
            id == crate::ids::CTX_MENU_NEW_IMAGE_CREATE
                || crate::ids::CTX_MENU_NEW_IMAGE_SIZES
                    .iter()
                    .any(|(_, b)| *b == id)
                || crate::ids::CTX_MENU_NEW_IMAGE_BGS
                    .iter()
                    .any(|(_, b)| *b == id)
        });
        if !inside_scene_list && !inside_palette_rename && !inside_new_image {
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
        // The rename modal is a centered context menu OUTSIDE the picker rect, but it belongs to the
        // picker — clicking its field / Rename button must NOT dismiss the picker underneath it.
        let rename_dialog_open = matches!(
            store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::RenamePaletteDialog)
        );
        if !inside_outer
            && !inside_sub
            && !is_color_target
            && !eyedropper_armed
            && !rename_dialog_open
        {
            store.set_picker_target(None);
        }
    }

    // An OPEN generic dropdown light-dismisses when the Down lands OUTSIDE its popover AND off its
    // chip (Enio 2026-06-24: "se o usuário clicar fora do dropdown ele deve se fechar"). The popover
    // rect + owner id are republished each frame the popover paints (`set_dropdown_popover`); the
    // `open: true` re-check guards against a stale publish after the dropdown already closed. We close
    // (not `return`) so — exactly like the context-menu light-dismiss above — the same click still
    // drives whatever widget it lands on, so clicking straight onto another chip swaps dropdowns. A
    // click on the chip itself (`on_chip`) is left to `apply_click`'s toggle; a click on an option
    // (`inside_popover`) is left to the panel's select-then-close.
    if let Some((dd_id, popover)) = store.dropdown_popover()
        && matches!(
            store.get(dd_id),
            Some(InteractiveState::Dropdown { open: true, .. })
        )
    {
        let on_chip = hit.map(|(id, _)| id) == Some(dd_id);
        let inside_popover = popover.contains(event.x, event.y);
        if !on_chip
            && !inside_popover
            && let Some(InteractiveState::Dropdown { open, .. }) = store.get_mut(dd_id)
        {
            *open = false;
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
        commit_number_buffer(store, old, events, false);
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
        let focus_gained = store.focus_id() != Some(id);
        if focus_gained {
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
                // O acumulador contínuo nasce onde o valor está (ver `NumberInputDragState::accum`).
                accum: *value,
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
            //
            // EXCEPT the Down that just FOCUSED a NumberInput: focus selected
            // all (`init_number_buffer`) so typing REPLACES the readout, and
            // placing the caret here would collapse that selection back to the
            // clicked byte — the exact re-collapse that made typing "2" into a
            // chip showing "2" author 22 (Dur(s), 2026-07-23). A second click
            // on the already-focused chip still places the caret for surgical
            // edits (Blender's number-field model).
            let kept_focus_selection =
                focus_gained && matches!(store.get(id), Some(InteractiveState::NumberInput { .. }));
            if !kept_focus_selection {
                let offset =
                    byte_offset_from_click_xy(store, id, rect, event.x, event.y, ts.take());
                place_text_caret(store, id, offset, true);
            }
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
        // BlenderColorPicker sub-control hits route into the
        // parent's stored state mutation. Right-click on a
        // palette swatch removes it instead of picking it.
        if let Some(parent) = apply_blender_hit(store, id, rect, event.x, event.y, event.button) {
            events.push(WidgetEvent::ValueChanged(parent));
        }
    }

    // Scrollbar drag — NOT gated on `is_focusable`. The bar isn't a registered `InteractiveState`, so
    // it is never focusable: gating the begin behind the focus block above meant the drag never
    // started (the real reason no scrollbar could be dragged). Snapshot the panel metrics so Move
    // computes a proportional `panel_scroll` delta. The single dropdown scrollbar id maps to whichever
    // dropdown is open; the rest are fixed (see `scrollbar_panel_for_id`).
    if let Some((id, rect)) = hit {
        let scroll_panel = if id == crate::widget::DROPDOWN_SCROLLBAR_ID {
            store.dropdown_popover().map(|(dd, _)| dd)
        } else {
            scrollbar_panel_for_id(id)
        };
        if let Some(panel) = scroll_panel
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
