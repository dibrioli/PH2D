//! Pointer **Up** dispatch arm. Extracted verbatim from the
//! `dispatch_pointer_with_text` god-function (blindagem Fase 3.2) — pure move,
//! same `super::` paths, same behaviour (covered by `dispatch::tests`).

use super::focus::apply_click;
use super::hierarchy::{HierDrop, find_hierarchy_drop, find_painter_layer_drop};
use super::hover::set_widget_released;
use crate::interaction::types::{GesturePhase, GraphGesture, TimelineGesture};
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore, drag};
use bumpalo::collections::Vec as BumpVec;
use ph2d_host::PointerEvent;

/// Handle a pointer-`Up` event: ends drags (picker/resize/scrollbar/number/
/// hierarchy/painter-layer), resolves drops, and emits the release-click.
pub(super) fn dispatch_up<'frame>(
    store: &mut WidgetStore,
    hit_index: &HitIndex,
    event: PointerEvent,
    events: &mut BumpVec<'frame, WidgetEvent>,
) {
    // Picker drag ends on Up — clear the anchor so a stray
    // Move after release doesn't drag the picker further.
    store.end_blender_drag();
    // Panel-resize drag also ends on Up.
    store.end_panel_resize();
    // Same for scrollbar drag.
    store.end_scrollbar_drag();
    // M14.A: NumberInput drag-or-edit Up cleanup. If the
    // threshold was crossed, this Up *commits* the drag-slider
    // delta (already applied during Move) and clears the
    // focused edit state — the user was scrubbing, not
    // editing. If the threshold was NOT crossed, the Down
    // already entered edit mode (focus + caret), so we just
    // forget the drag candidate. Continuous-hold on the
    // stepper arrow always ends on Up.
    if let Some(drag) = store.end_number_input_drag()
        && drag.crossed_threshold
    {
        // Audit fix #2 (CRITICAL): commit `last_committed` to
        // the final scrubbed value here, on Up. The per-Move
        // path mutates `value` + `buffer` only; the rollback
        // anchor lives until this release point so Esc
        // mid-drag can restore the pre-Down value.
        if let Some(InteractiveState::NumberInput {
            state,
            value,
            last_committed,
            ..
        }) = store.get_mut(drag.id)
        {
            *last_committed = *value;
            // Clear the focused state — drag-completed = no edit.
            *state = crate::widget::TextInputState::Normal;
        }
        if store.focus_id() == Some(drag.id) {
            store.set_focus(None);
            events.push(WidgetEvent::Blur(drag.id));
        }
        events.push(WidgetEvent::ValueChanged(drag.id));
        // Drag commit also wrote the linked slider during the
        // Move path — emit its ValueChanged so panel handlers
        // (padding/upscale swallow the chip event) still see
        // the final scrubbed value. Symmetric with the Move
        // emission above + stepper/tick/commit_number_buffer.
        if let Some(slider_id) = store.linked_slider(drag.id)
            && matches!(store.get(slider_id), Some(InteractiveState::Slider { .. }))
        {
            events.push(WidgetEvent::ValueChanged(slider_id));
        }
    }
    store.end_number_stepper_hold();
    // Hierarchy drag ends on Up. If the drag was active
    // (cursor moved past the threshold), find the drop
    // target by cursor y vs each row rect and reorder.
    // Otherwise treat as a regular click (selection,
    // handled by the Click event from `apply_click`), or,
    // when the Down→Up hold exceeded `LONG_PRESS_THRESHOLD_NS`
    // without movement, emit `LongPress` so the hierarchy
    // row enters inline rename.
    let drag_end = store.end_hierarchy_drag();
    // Long-press detection: still pointer-Down (`!active`) for
    // ≥ 600 ms emits LongPress. The Click that `apply_click`
    // would otherwise also emit on this same Up is suppressed
    // below (`suppress_click`) so the long-press doesn't
    // silently mutate hierarchy selection on top of the rename
    // mode it just opened.
    let mut suppress_click = false;
    if let Some(drag) = drag_end
        && !drag.active
        && event.timestamp_ns.saturating_sub(drag.down_timestamp_ns)
            >= drag::LONG_PRESS_THRESHOLD_NS
    {
        events.push(WidgetEvent::LongPress(drag.dragged));
        suppress_click = true;
    }
    if let Some(drag) = drag_end
        && drag.active
    {
        // Drop-on-self short-circuit: if the cursor at Up is
        // back inside the dragged row's own rect (user almost
        // dragged, then drifted back), don't fire a
        // HierReparent. Pre-fix: the drag-active path
        // unconditionally resolved to End and silently
        // root-promoted the entity.
        let over_self = hit_index
            .iter_registrations()
            .find(|(id, _)| *id == drag.dragged)
            .map(|(_, r)| {
                event.y >= r.y && event.y < r.y + r.h && event.x >= r.x && event.x < r.x + r.w
            })
            .unwrap_or(false);
        if over_self {
            return;
        }
        let drop = find_hierarchy_drop(hit_index, store, event.y, drag.dragged);
        match drop {
            HierDrop::Before(t) => {
                store.hierarchy_move(drag.dragged, Some(t));
                // Inherit the target's parent so siblings
                // stay siblings after a reorder.
                let new_parent = store.hierarchy_parent_of(t);
                let _ = store.hierarchy_set_parent(drag.dragged, new_parent);
                events.push(WidgetEvent::HierReparent {
                    dragged: drag.dragged,
                    new_parent,
                    before: Some(t),
                    after: None,
                });
            }
            HierDrop::After(t) => {
                // Fixture-mode store move: insert just after
                // target. Live mode delegates to the host
                // which resolves the after target into the
                // matching parent + before-next-sibling slot.
                let order = store.hierarchy_order();
                let next_id = order
                    .iter()
                    .position(|i| *i == t)
                    .and_then(|idx| order.get(idx + 1).copied());
                store.hierarchy_move(drag.dragged, next_id);
                let new_parent = store.hierarchy_parent_of(t);
                let _ = store.hierarchy_set_parent(drag.dragged, new_parent);
                events.push(WidgetEvent::HierReparent {
                    dragged: drag.dragged,
                    new_parent,
                    before: None,
                    after: Some(t),
                });
            }
            HierDrop::Inside(t) => {
                if store.hierarchy_set_parent(drag.dragged, Some(t)) {
                    // Move dragged immediately after target
                    // in the order list so the child sits
                    // visually next to its new parent.
                    let order = store.hierarchy_order();
                    let after_idx = order.iter().position(|i| *i == t);
                    if let Some(idx) = after_idx {
                        let next_id = order.get(idx + 1).copied();
                        store.hierarchy_move(drag.dragged, next_id);
                    }
                }
                events.push(WidgetEvent::HierReparent {
                    dragged: drag.dragged,
                    new_parent: Some(t),
                    before: None,
                    after: None,
                });
            }
            HierDrop::End => {
                store.hierarchy_move(drag.dragged, None);
                let _ = store.hierarchy_set_parent(drag.dragged, None);
                events.push(WidgetEvent::HierReparent {
                    dragged: drag.dragged,
                    new_parent: None,
                    before: None,
                    after: None,
                });
            }
        }
    }
    // Painter layers-panel row drag (W3 T3.8): on Up of an ACTIVE drag,
    // resolve the drop band → emit `PainterLayerReparent` for the
    // painter tool to apply. NO store-side mutation (the tool owns the
    // LayerStack). Drop-on-self (drifted back onto the dragged row) is a
    // no-op. Only one row drag can be active per frame, so the hierarchy
    // block above already no-op'd when this one is `Some`.
    if let Some(drag) = store.end_painter_layer_drag()
        && drag.active
    {
        let over_self = hit_index
            .iter_registrations()
            .find(|(id, _)| *id == drag.dragged)
            .map(|(_, r)| {
                event.y >= r.y && event.y < r.y + r.h && event.x >= r.x && event.x < r.x + r.w
            })
            .unwrap_or(false);
        if !over_self {
            let drop = find_painter_layer_drop(hit_index, store, event.y, drag.dragged);
            events.push(WidgetEvent::PainterLayerReparent {
                dragged: drag.dragged,
                drop,
            });
        }
    }
    if let Some(active) = store.active_id() {
        // Motion Nodes M0.T3 — end a graph-surface capture: End if it dragged,
        // else a Click (a tap). No apply_click / focus side effects — the panel
        // owns all graph semantics. Runs before the generic release logic.
        if let Some((surface, kind)) = store.graph_surface_at_id(active) {
            let phase = if store.take_graph_moved() {
                GesturePhase::End
            } else {
                GesturePhase::Click
            };
            let mods = store.gesture_mods();
            store.push_graph_gesture(GraphGesture {
                surface,
                kind,
                phase,
                x: event.x,
                y: event.y,
                button: event.button,
                mods,
            });
            store.set_active(None);
            store.set_active_rect(None);
            return;
        }
        // W2.E5b — end a timeline-surface capture: End if it dragged, else Click
        // (a tap). No apply_click/focus side effects — the panel owns semantics.
        if let Some((surface, kind)) = store.timeline_surface_at_id(active) {
            let dragged = store.take_timeline_moved();
            // Consume the double-click flag regardless (keeps it from leaking to
            // the next tap), but only a MARKER tap upgrades to `DoubleClick` —
            // every other surface treats a second tap as a plain Click, so this
            // stays a no-op behaviour change for keys / lanes / braces.
            let double = store.take_timeline_double();
            let phase = if dragged {
                GesturePhase::End
            } else if double
                && matches!(
                    kind,
                    crate::interaction::types::TimelineHitKind::Marker { .. }
                )
            {
                GesturePhase::DoubleClick
            } else {
                GesturePhase::Click
            };
            let mods = store.gesture_mods();
            store.push_timeline_gesture(TimelineGesture {
                surface,
                kind,
                phase,
                x: event.x,
                y: event.y,
                button: event.button,
                mods,
            });
            store.set_active(None);
            store.set_active_rect(None);
            return;
        }
        // "still_hot" is whether the pointer is still inside
        // the widget's rect captured on Down. Using the live
        // `hit_index` for this check breaks transient widgets
        // (e.g. context-menu items): Down opens, hit_index
        // updates, by Up the menu is gone and the hit
        // resolves to whatever sits behind it — `still_hot`
        // would be false and `apply_click` would never fire.
        // The `active_rect` snapshot was taken at Down and
        // doesn't depend on the current frame's hit_index.
        let still_hot = store
            .active_rect()
            .map(|r| r.contains(event.x, event.y))
            .unwrap_or(false);
        // Some downstream branches still need to know the
        // current hit (e.g. for drag-release routing).
        let _ = hit_index.hit(event.x, event.y);
        // Sliders emit no Click on release — they emitted
        // ValueChanged events throughout the drag. Buttons,
        // Toggles, and Checkboxes only count Click if the
        // pointer ended inside the original widget.
        let is_drag_widget = matches!(store.get(active), Some(InteractiveState::Slider { .. }));
        // Only Primary releases emit Click. Secondary clicks
        // are reserved for context-menu / right-click-deletes
        // (handled in the Down branch via the request side-
        // table + `apply_blender_hit`) — emitting Click for
        // them would, e.g., toggle section collapse on
        // right-click, which is exactly the bug the user
        // reported.
        if still_hot
            && !is_drag_widget
            && !suppress_click
            && event.button == ph2d_host::PointerButton::Primary
        {
            apply_click(store, active, events);
        }
        set_widget_released(store, active, still_hot);
        store.set_active(None);
        store.set_active_rect(None);
    }
}
