//! Pointer + key dispatchers — translate raw shell events into
//! [`super::WidgetStore`] state mutations and [`super::WidgetEvent`]
//! emissions.
//!
//! Every dispatcher takes a `&'frame bumpalo::Bump` and returns a
//! `&'frame [WidgetEvent]` slice — the arena is the frame-local
//! event allocator. Caller drains the slice in the same frame and
//! resets the arena before the next frame.
//!
//! Phase A wires Button + Toggle. Slider/RadioGroup/Checkbox arrive
//! in Phase B; TextInput/NumberInput/Combobox in Phase C;
//! TreeView/ContextMenu/ColorPicker/Modal/Tabs in Phase D.

use super::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::widget::{
    ButtonState, CheckboxState, CheckboxValue, SliderOrientation, SliderState, ToggleState,
};
use crate::zones::Rect;
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;
use ph2d_host::{KeyEvent, KeyKind, PointerEvent, PointerKind};
use ph2d_text::TextSystem;

/// Keycodes the editor cares about. We don't pull in winit here —
/// the shell normalizes its keycodes to these constants before
/// forwarding to [`dispatch_key`]. Values mirror common
/// platform-independent keycodes (matches the shell's
/// `KeyEvent::keycode` field documentation).
pub const KEY_TAB: u32 = 0x09;
pub const KEY_ENTER: u32 = 0x0D;
pub const KEY_SPACE: u32 = 0x20;
pub const KEY_ESCAPE: u32 = 0x1B;
pub const KEY_BACKSPACE: u32 = 0x08;
pub const KEY_KEY_A: u32 = 0x41;
pub const KEY_KEY_C: u32 = 0x43;
pub const KEY_KEY_V: u32 = 0x56;
pub const KEY_KEY_X: u32 = 0x58;
pub const KEY_ARROW_UP: u32 = 0xF700;
pub const KEY_ARROW_DOWN: u32 = 0xF701;
pub const KEY_ARROW_LEFT: u32 = 0xF702;
pub const KEY_ARROW_RIGHT: u32 = 0xF703;

/// Entry point for pointer events. Updates [`WidgetStore`] hover /
/// active / focus cursors based on the hit-test, transitions the
/// per-widget interactive state, and emits widget events into the
/// caller's frame-local arena.
///
/// Returns the events emitted for this single dispatch call. Caller
/// drains synchronously; after the frame ends, caller resets the
/// arena (deallocates events for the next frame).
///
/// Approximate click→byte mapping (no real glyph measurement). For
/// pixel-accurate caret placement on text widgets, prefer
/// [`dispatch_pointer_with_text`] and pass a live [`TextSystem`].
pub fn dispatch_pointer<'frame>(
    store: &mut WidgetStore,
    hit_index: &HitIndex,
    event: PointerEvent,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent] {
    dispatch_pointer_with_text(store, hit_index, event, None, arena)
}

/// Like [`dispatch_pointer`] but takes an optional `TextSystem`. When
/// `Some`, the click→byte mapping uses real glyph layout (binary
/// search the nearest glyph boundary) so the caret lands exactly
/// where the user clicked. When `None`, falls back to the
/// `font_size * APPROX_ADVANCE_RATIO` heuristic — adequate for
/// tests, but visibly off on long lines or proportional content.
pub fn dispatch_pointer_with_text<'frame>(
    store: &mut WidgetStore,
    hit_index: &HitIndex,
    event: PointerEvent,
    text_system: Option<&mut TextSystem>,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent] {
    // Borrow-checker tap dance: text_system is `&mut`; we need to
    // read it inside two separate branches (Move drag, Down place),
    // so we wrap in an `Option` we can `take()` and put back. In
    // practice each event is exactly one branch so the borrow only
    // crosses once.
    let mut ts = text_system;
    let mut events: BumpVec<'frame, WidgetEvent> = BumpVec::new_in(arena);

    match event.kind {
        PointerKind::Move => {
            // While a Slider is being dragged, every Move computes a
            // fresh value from the pointer position relative to the
            // active rect. Hover tracking is suppressed (the active
            // widget keeps its Pressed state regardless of where the
            // cursor went).
            // Picker drag — keep the picker stuck to the cursor.
            // Incremental model: anchor stores the *last cursor* pos,
            // not the down cursor. Each move applies a fresh delta to
            // the *currently stored* offset (which paint may have
            // clamped between frames), then re-anchors. This means a
            // reversed drag direction moves the panel immediately —
            // no "rubber band" of accumulated unbounded offset to
            // drain first.
            if let Some((parent, last_x, last_y, _off_x, _off_y)) = store.blender_drag_anchor() {
                let (cur_off_x, cur_off_y) = store.blender_picker_offset(parent);
                let new_off_x = cur_off_x + (event.x - last_x);
                let new_off_y = cur_off_y + (event.y - last_y);
                store.set_blender_picker_offset(parent, new_off_x, new_off_y);
                store.update_blender_drag_cursor(event.x, event.y);
            }
            // Panel manual resize — same incremental model. Each Move
            // applies (cursor − last_cursor) to the panel's stored
            // resize delta, then re-anchors so the next clamp happens
            // against current state. The painter clamps to (MIN_W,
            // MIN_H) and viewport bounds.
            if let Some((panel, last_x, last_y)) = store.panel_resize_anchor() {
                let (cur_dw, cur_dh) = store.panel_resize_delta(panel);
                let new_dw = cur_dw + (event.x - last_x);
                let new_dh = cur_dh + (event.y - last_y);
                store.set_panel_resize_delta(panel, new_dw, new_dh);
                store.update_panel_resize_cursor(event.x, event.y);
            }
            // Scrollbar drag — translate the cursor's y-delta into
            // a `panel_scroll` delta via `widget::scrollbar::
            // delta_for_drag`. Snapshot of metrics taken at Down so
            // the drag stays linear even if the painter republishes
            // mid-drag.
            if let Some(anchor) = store.scrollbar_drag() {
                let dy = event.y - anchor.cursor_y_at_down;
                let scroll_delta = crate::widget::scrollbar_delta_for_drag(
                    dy,
                    anchor.track_h,
                    anchor.content_h,
                    anchor.visible_h,
                );
                let max = (anchor.content_h - anchor.visible_h).max(0.0);
                let new_scroll = (anchor.scroll_at_down + scroll_delta).clamp(0.0, max);
                store.set_panel_scroll(anchor.panel, new_scroll);
            }
            // Hierarchy drag — keep cursor + active flag updated
            // each Move so the painter can render the drop indicator.
            if store.hierarchy_drag().is_some() {
                store.update_hierarchy_drag(event.x, event.y);
            }
            if let Some(active) = store.active_id() {
                if let Some(rect) = store.active_rect() {
                    // Text drag-to-select: extend the selection from
                    // the anchor (set on Down) to the new cursor x.
                    if matches!(
                        store.get(active),
                        Some(InteractiveState::TextInput { .. })
                            | Some(InteractiveState::NumberInput { .. })
                            | Some(InteractiveState::Combobox { .. })
                    ) {
                        let offset = byte_offset_from_click_xy(
                            store,
                            active,
                            rect,
                            event.x,
                            event.y,
                            ts.take(),
                        );
                        place_text_caret(store, active, offset, false);
                    }
                    // Plain slider drag.
                    if update_drag_value(store, active, rect, event.x, event.y) {
                        events.push(WidgetEvent::ValueChanged(active));
                    }
                    // BlenderColorPicker drag-relevant sub-controls —
                    // wheel, hue strip, channel sliders. Re-apply on
                    // every Move so the color tracks the cursor.
                    // Buttons / toggles / swatches / eyedropper are
                    // click-once: re-applying them on Move would, e.g.,
                    // append the current color N times when "+ swatch"
                    // is held with even the slightest cursor jitter.
                    // (See `docs/UI_Bugs/README.md` §2.1 for the
                    // multi-cor "+ swatch" bug.)
                    let drag_apply = matches!(
                        store.get(active),
                        Some(InteractiveState::BlenderHit {
                            kind: super::BlenderHitKind::Wheel
                                | super::BlenderHitKind::ValueSlider
                                | super::BlenderHitKind::ChannelSlider(_),
                            ..
                        })
                    );
                    if drag_apply
                        && let Some(parent) =
                            apply_blender_hit(store, active, rect, event.x, event.y, event.button)
                    {
                        events.push(WidgetEvent::ValueChanged(parent));
                    }
                }
            } else {
                let hit = hit_index.hit(event.x, event.y);
                update_hover(store, hit);
            }
        }
        PointerKind::Down => {
            let hit = hit_index.hit_with_rect(event.x, event.y);

            // Right-click → open a context menu. We dispatch in two
            // shapes:
            //   - Secondary on a registered widget id whose role is
            //     "section header" (the inspector marks these via
            //     `is_collapsible_section_id` — currently any id in
            //     the `INSP_SECTION_*` range) → `SectionOutline` menu.
            //   - Secondary anywhere inside a panel rect → "CreateNote"
            //     menu parented to that panel.
            // Primary clicks fall through to the regular focus/click
            // path below. A right-click on a non-panel area closes any
            // currently-open menu.
            if event.button == ph2d_host::PointerButton::Secondary {
                let panel_under = store.panel_at(event.x, event.y);
                let hit_id = hit.map(|(id, _)| id);
                let is_section = hit_id.map(is_section_header_id).unwrap_or(false);
                // Note slot hit (id range 800..811): right-click on a
                // painted note opens the NoteBackground menu for that
                // slot's index. The inspector painter publishes the
                // slot→note-index mapping by always painting note
                // `i` at `NOTE_SLOT_IDS[i]`, so slot id - 800 IS the
                // note index.
                let note_slot = hit_id.and_then(|id| {
                    let v = id.0;
                    if (800..=811).contains(&v) {
                        Some((v - 800) as u8)
                    } else {
                        None
                    }
                });
                if let Some(note_index) = note_slot
                    && let Some(panel) = panel_under
                {
                    store.open_context_menu(super::ContextMenuRequest {
                        x: event.x,
                        y: event.y,
                        kind: super::ContextMenuKind::NoteBackground { panel, note_index },
                    });
                } else if is_section {
                    let section_id = hit_id.unwrap();
                    store.open_context_menu(super::ContextMenuRequest {
                        x: event.x,
                        y: event.y,
                        kind: super::ContextMenuKind::SectionOutline {
                            section: section_id,
                        },
                    });
                } else if let Some(panel) = panel_under {
                    // `before_section` is filled in by apply_event
                    // — only the inspector knows the screen→body
                    // conversion + section y-ranges.
                    store.open_context_menu(super::ContextMenuRequest {
                        x: event.x,
                        y: event.y,
                        kind: super::ContextMenuKind::CreateNote {
                            panel,
                            before_section: None,
                        },
                    });
                } else {
                    store.close_context_menu();
                }
                return events.into_bump_slice();
            }
            // Primary click on the TopBar theme cluster opens the
            // ThemeSelector context menu (4 themes + 3 corner-radius
            // presets). Anchored just below the cluster's hit rect
            // so the popover doesn't overlap the cluster itself.
            // The `Plain` state check disambiguates from other
            // widgets that may happen to share the TOPBAR_THEME
            // NodeId numeric value in isolated unit tests (the
            // hero's real `populate` registers it as Plain).
            if event.button == ph2d_host::PointerButton::Primary
                && let Some((hit_id, hit_rect)) = hit
                && hit_id == crate::screens::hero::ids::TOPBAR_THEME
                && matches!(store.get(hit_id), Some(InteractiveState::Button { .. }))
            {
                store.open_context_menu(super::ContextMenuRequest {
                    x: hit_rect.x,
                    y: hit_rect.y + hit_rect.h + 4.0,
                    kind: super::ContextMenuKind::ThemeSelector,
                });
                return events.into_bump_slice();
            }
            // Primary click elsewhere closes any open menu before
            // running the regular focus/click path.
            if store.context_menu().is_some() {
                store.close_context_menu();
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
            // the picker dismissed itself — user reported "se eu
            // clicar dentro do painel sem acertar um controle ele
            // fecha". Fallback to the sub-control test when the
            // outer rect isn't published (e.g. first frame).
            if store.picker_target().is_some() {
                use crate::screens::hero::ids as hero_ids;
                let inside_outer = store
                    .panel_rect(hero_ids::INSP_BLENDER_PICKER)
                    .map(|r| r.contains(event.x, event.y))
                    .unwrap_or(false);
                let inside_sub = matches!(
                    hit.and_then(|(id, _)| store.get(id)),
                    Some(InteractiveState::BlenderHit { .. })
                );
                let is_color_target = hit.map(|(id, _)| is_color_target_id(id)).unwrap_or(false);
                if !inside_outer && !inside_sub && !is_color_target {
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
                        kind: super::BlenderHitKind::Eyedropper,
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
                    return events.into_bump_slice();
                }
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
            let prev_focus = store.focus_id();
            if let Some(old) = prev_focus
                && new_focus != Some(old)
            {
                commit_number_buffer(store, old, &mut events);
                commit_hex_buffer(store, old, &mut events);
                reset_focused_visual_state(store, old);
                events.push(WidgetEvent::Blur(old));
                store.set_focus(None);
            }

            // Detect double-click against the previous Down. Anything
            // landing on a TextInput / NumberInput within the
            // double-click window (and on the same id) selects all.
            let is_double_click = store.record_pointer_down(new_focus, event.timestamp_ns);

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
                // the caret or trigger select-all).
                let stepper_hit = !combo_cleared
                    && apply_number_stepper_if_hit(store, id, rect, event.x, event.y);
                if combo_cleared {
                    events.push(WidgetEvent::TextChanged(id));
                } else if stepper_hit {
                    events.push(WidgetEvent::ValueChanged(id));
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
                    let offset =
                        byte_offset_from_click_xy(store, id, rect, event.x, event.y, ts.take());
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
                if is_hierarchy_entity_id(id) {
                    store.begin_hierarchy_drag(id, event.x, event.y);
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
                    store.begin_scrollbar_drag(super::ScrollbarDragAnchor {
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
                if let Some(parent) =
                    apply_blender_hit(store, id, rect, event.x, event.y, event.button)
                {
                    events.push(WidgetEvent::ValueChanged(parent));
                }
            }
        }
        PointerKind::Up => {
            // Picker drag ends on Up — clear the anchor so a stray
            // Move after release doesn't drag the picker further.
            store.end_blender_drag();
            // Panel-resize drag also ends on Up.
            store.end_panel_resize();
            // Same for scrollbar drag.
            store.end_scrollbar_drag();
            // Hierarchy drag ends on Up. If the drag was active
            // (cursor moved past the threshold), find the drop
            // target by cursor y vs each row rect and reorder.
            // Otherwise treat as a regular click (selection,
            // handled by the Click event from `apply_click`).
            if let Some(drag) = store.end_hierarchy_drag()
                && drag.active
            {
                match find_hierarchy_drop(hit_index, event.x, event.y, drag.dragged) {
                    HierDrop::Before(t) => {
                        store.hierarchy_move(drag.dragged, Some(t));
                        // Inherit the target's parent so siblings
                        // stay siblings after a reorder.
                        let new_parent = store.hierarchy_parent_of(t);
                        let _ = store.hierarchy_set_parent(drag.dragged, new_parent);
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
                    }
                    HierDrop::End => {
                        store.hierarchy_move(drag.dragged, None);
                        let _ = store.hierarchy_set_parent(drag.dragged, None);
                    }
                }
            }
            if let Some(active) = store.active_id() {
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
                let is_drag_widget =
                    matches!(store.get(active), Some(InteractiveState::Slider { .. }));
                // Only Primary releases emit Click. Secondary clicks
                // are reserved for context-menu / right-click-deletes
                // (handled in the Down branch via the request side-
                // table + `apply_blender_hit`) — emitting Click for
                // them would, e.g., toggle section collapse on
                // right-click, which is exactly the bug the user
                // reported.
                if still_hot && !is_drag_widget && event.button == ph2d_host::PointerButton::Primary
                {
                    apply_click(store, active, &mut events);
                }
                set_widget_released(store, active, still_hot);
                store.set_active(None);
                store.set_active_rect(None);
            }
        }
    }

    events.into_bump_slice()
}

/// Recompute slider value from pointer position relative to its
/// active rect. Returns true iff the value actually changed (so
/// dispatcher can decide whether to emit `ValueChanged`). Mirrors
/// the new value into a linked NumberInput, if one is registered.
fn update_drag_value(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    rect: Rect,
    px: f32,
    py: f32,
) -> bool {
    let (changed, propagated) = {
        let Some(InteractiveState::Slider {
            state,
            value,
            orientation,
        }) = store.get_mut(id)
        else {
            return false;
        };
        let new_value = match *orientation {
            SliderOrientation::Horizontal => {
                if rect.w <= 0.0 {
                    0.0
                } else {
                    ((px - rect.x) / rect.w).clamp(0.0, 1.0)
                }
            }
            SliderOrientation::Vertical => {
                if rect.h <= 0.0 {
                    0.0
                } else {
                    (1.0 - (py - rect.y) / rect.h).clamp(0.0, 1.0)
                }
            }
        };
        let changed = (new_value - *value).abs() > f32::EPSILON;
        *value = new_value;
        *state = SliderState::Dragging;
        (changed, new_value as f64)
    };
    if changed && let Some(number_id) = store.linked_number(id) {
        store.set_number_value(number_id, propagated);
    }
    changed
}

/// Entry point for key events. Tab / Shift+Tab traverse the focus
/// chain; Enter / Space activate the focused widget; Escape blurs.
pub fn dispatch_key<'frame>(
    store: &mut WidgetStore,
    event: KeyEvent,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent] {
    let mut events: BumpVec<'frame, WidgetEvent> = BumpVec::new_in(arena);
    if event.kind == KeyKind::Up {
        return events.into_bump_slice();
    }
    match event.keycode {
        KEY_TAB => {
            if event.modifiers.shift {
                cycle_focus(store, false, &mut events);
            } else {
                cycle_focus(store, true, &mut events);
            }
        }
        KEY_KEY_A if event.modifiers.meta || event.modifiers.ctrl => {
            // Cmd/Ctrl+A on a focused TextInput / NumberInput selects
            // the whole buffer (same effect as double-click).
            if let Some(id) = store.focus_id() {
                select_all_in_text_widget(store, id);
            }
        }
        KEY_KEY_C if event.modifiers.meta || event.modifiers.ctrl => {
            if let Some(id) = store.focus_id()
                && let Some(text) = clipboard_extract_selection(store, id)
            {
                store.set_clipboard_copy(text);
            }
        }
        KEY_KEY_X if event.modifiers.meta || event.modifiers.ctrl => {
            if let Some(id) = store.focus_id()
                && let Some(text) = clipboard_extract_selection(store, id)
            {
                store.set_clipboard_copy(text);
                delete_selection_if_any(store, id);
                events.push(WidgetEvent::TextChanged(id));
            }
        }
        KEY_KEY_V if event.modifiers.meta || event.modifiers.ctrl => {
            if let Some(id) = store.focus_id()
                && matches!(
                    store.get(id),
                    Some(InteractiveState::TextInput { .. })
                        | Some(InteractiveState::Combobox { .. })
                        | Some(InteractiveState::NumberInput { .. })
                )
            {
                store.set_clipboard_paste_request(id);
            }
        }
        KEY_ENTER | KEY_SPACE => {
            if let Some(id) = store.focus_id() {
                // For NumberInput, Enter commits the buffer (parses)
                // and emits ValueChanged on success. Falls through to
                // apply_click for everything else.
                if matches!(store.get(id), Some(InteractiveState::NumberInput { .. }))
                    && event.keycode == KEY_ENTER
                {
                    commit_number_buffer(store, id, &mut events);
                    return events.into_bump_slice();
                }
                // Hex TextInput linked to a BlenderPicker: parse the
                // buffer and apply the resulting color to the parent,
                // then blur (Enter ends the edit, like a form field).
                if event.keycode == KEY_ENTER
                    && matches!(store.get(id), Some(InteractiveState::TextInput { .. }))
                    && store.blender_hex_parent(id).is_some()
                {
                    commit_hex_buffer(store, id, &mut events);
                    if let Some(InteractiveState::TextInput { state, .. }) = store.get_mut(id) {
                        *state = crate::widget::TextInputState::Normal;
                    }
                    store.set_focus(None);
                    events.push(WidgetEvent::Blur(id));
                    return events.into_bump_slice();
                }
                // SPACE while focus is on a text widget MUST insert a
                // space character (handled by dispatch_text_input) —
                // we used to also fire `apply_click` here, which
                // emitted a stray Click event and made the user press
                // SPACE twice before the char registered. Skip
                // apply_click for TextInput / Combobox / NumberInput.
                let is_text_widget = matches!(
                    store.get(id),
                    Some(InteractiveState::TextInput { .. })
                        | Some(InteractiveState::Combobox { .. })
                        | Some(InteractiveState::NumberInput { .. })
                );
                // SPACE on a text widget inserts a literal ' '
                // directly here, bypassing winit's text-input
                // pipeline. The shell's IME path delivers ' ' as a
                // text event AFTER the key event, but on macOS the
                // FIRST press's text-event sometimes arrives empty
                // (IME init) — so the user had to press SPACE twice.
                // Insert it ourselves on the key event; the shell
                // suppresses the matching text-event for KEY_SPACE
                // so we never double-insert.
                if event.keycode == KEY_SPACE
                    && matches!(
                        store.get(id),
                        Some(InteractiveState::TextInput { .. })
                            | Some(InteractiveState::Combobox { .. })
                    )
                {
                    delete_selection_if_any(store, id);
                    match store.get_mut(id) {
                        Some(InteractiveState::TextInput { text, caret, .. }) => {
                            text.insert(*caret, ' ');
                            *caret += 1;
                            events.push(WidgetEvent::TextChanged(id));
                        }
                        Some(InteractiveState::Combobox { query, caret, .. }) => {
                            query.insert(*caret, ' ');
                            *caret += 1;
                            events.push(WidgetEvent::TextChanged(id));
                        }
                        _ => {}
                    }
                    return events.into_bump_slice();
                }
                // Enter on a multi-line TextInput inserts a literal
                // newline so notes can have body paragraphs.
                if event.keycode == KEY_ENTER
                    && matches!(store.get(id), Some(InteractiveState::TextInput { .. }))
                {
                    delete_selection_if_any(store, id);
                    if let Some(InteractiveState::TextInput { text, caret, .. }) = store.get_mut(id)
                    {
                        text.insert(*caret, '\n');
                        *caret += 1;
                        events.push(WidgetEvent::TextChanged(id));
                    }
                    return events.into_bump_slice();
                }
                if !is_text_widget {
                    apply_click(store, id, &mut events);
                }
            }
        }
        KEY_ESCAPE => {
            if let Some(id) = store.focus_id() {
                // Dropdowns close on ESC instead of losing focus.
                if let Some(InteractiveState::Dropdown { open, .. }) = store.get_mut(id)
                    && *open
                {
                    *open = false;
                    return events.into_bump_slice();
                }
                // NumberInput: revert buffer to last committed value.
                if matches!(store.get(id), Some(InteractiveState::NumberInput { .. })) {
                    revert_number_buffer(store, id);
                }
                // Hex TextInput: revert buffer to canonical form of
                // the parent picker's current value.
                if matches!(store.get(id), Some(InteractiveState::TextInput { .. }))
                    && store.blender_hex_parent(id).is_some()
                {
                    write_hex_canonical(store, id);
                }
                store.set_focus(None);
                events.push(WidgetEvent::Blur(id));
            }
        }
        KEY_BACKSPACE => {
            if let Some(id) = store.focus_id() {
                if delete_selection_if_any(store, id) {
                    events.push(WidgetEvent::TextChanged(id));
                } else {
                    match store.get_mut(id) {
                        Some(InteractiveState::TextInput { text, caret, .. }) if *caret > 0 => {
                            let new_caret = prev_char_boundary(text, *caret);
                            text.replace_range(new_caret..*caret, "");
                            *caret = new_caret;
                            events.push(WidgetEvent::TextChanged(id));
                        }
                        Some(InteractiveState::NumberInput { buffer, caret, .. }) if *caret > 0 => {
                            let new_caret = prev_char_boundary(buffer, *caret);
                            buffer.replace_range(new_caret..*caret, "");
                            *caret = new_caret;
                            events.push(WidgetEvent::TextChanged(id));
                        }
                        Some(InteractiveState::Combobox { query, caret, .. }) if *caret > 0 => {
                            let new_caret = prev_char_boundary(query, *caret);
                            query.replace_range(new_caret..*caret, "");
                            *caret = new_caret;
                            events.push(WidgetEvent::TextChanged(id));
                        }
                        _ => {}
                    }
                }
            }
        }
        KEY_ARROW_LEFT => {
            if let Some(id) = store.focus_id() {
                // Selection collapse takes precedence over caret motion.
                if collapse_selection(store, id, false) {
                    return events.into_bump_slice();
                }
                match store.get_mut(id) {
                    Some(InteractiveState::TextInput { text, caret, .. }) if *caret > 0 => {
                        *caret = prev_char_boundary(text, *caret);
                    }
                    Some(InteractiveState::NumberInput { buffer, caret, .. }) if *caret > 0 => {
                        *caret = prev_char_boundary(buffer, *caret);
                    }
                    Some(InteractiveState::Combobox { query, caret, .. }) if *caret > 0 => {
                        *caret = prev_char_boundary(query, *caret);
                    }
                    _ => {}
                }
            }
        }
        KEY_ARROW_RIGHT => {
            if let Some(id) = store.focus_id() {
                if collapse_selection(store, id, true) {
                    return events.into_bump_slice();
                }
                match store.get_mut(id) {
                    Some(InteractiveState::TextInput { text, caret, .. })
                        if *caret < text.len() =>
                    {
                        *caret = next_char_boundary(text, *caret);
                    }
                    Some(InteractiveState::NumberInput { buffer, caret, .. })
                        if *caret < buffer.len() =>
                    {
                        *caret = next_char_boundary(buffer, *caret);
                    }
                    Some(InteractiveState::Combobox { query, caret, .. })
                        if *caret < query.len() =>
                    {
                        *caret = next_char_boundary(query, *caret);
                    }
                    _ => {}
                }
            }
        }
        KEY_ARROW_UP => {
            if let Some(id) = store.focus_id()
                && let Some(InteractiveState::NumberInput {
                    value,
                    buffer,
                    caret,
                    last_committed,
                    ..
                }) = store.get_mut(id)
            {
                *value += 1.0;
                *last_committed = *value;
                buffer.clear();
                use std::fmt::Write;
                let _ = write!(buffer, "{}", super::state::format_number(*value));
                *caret = buffer.len();
                events.push(WidgetEvent::ValueChanged(id));
            }
        }
        KEY_ARROW_DOWN => {
            if let Some(id) = store.focus_id()
                && let Some(InteractiveState::NumberInput {
                    value,
                    buffer,
                    caret,
                    last_committed,
                    ..
                }) = store.get_mut(id)
            {
                *value -= 1.0;
                *last_committed = *value;
                buffer.clear();
                use std::fmt::Write;
                let _ = write!(buffer, "{}", super::state::format_number(*value));
                *caret = buffer.len();
                events.push(WidgetEvent::ValueChanged(id));
            }
        }
        _ => {}
    }
    events.into_bump_slice()
}

fn prev_char_boundary(s: &str, mut i: usize) -> usize {
    if i == 0 {
        return 0;
    }
    i -= 1;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn next_char_boundary(s: &str, mut i: usize) -> usize {
    let len = s.len();
    if i >= len {
        return len;
    }
    i += 1;
    while i < len && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

/// Wheel / trackpad scroll. Finds the panel under `(x, y)` via
/// [`WidgetStore::panel_at`] and adjusts that panel's
/// `panel_scroll` by `delta_y`. Caller (painter) is responsible
/// for clamping the offset against the panel's `content_h` —
/// dispatch only deltas, doesn't know content height.
pub fn dispatch_wheel<'frame>(
    store: &mut WidgetStore,
    event: ph2d_host::WheelEvent,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent] {
    let events: BumpVec<'frame, WidgetEvent> = BumpVec::new_in(arena);
    if let Some(panel) = store.panel_at(event.x, event.y) {
        let cur = store.panel_scroll(panel);
        // delta_y > 0 from winit means "scroll forward" / content
        // moves up. We store offset as "how far down content
        // pretends to be" — so positive delta increments the
        // offset (showing content further down).
        let mut next = (cur - event.delta_y).max(0.0);
        // Clamp at the upper bound when the painter has published a
        // content_h for this panel. Without this, wheeling past the
        // last element pushes `next` arbitrarily high; the next
        // paint pass clamps it back, producing a 1-frame "jump"
        // (the user's "saltos indesejados se rodamos a roda no fim").
        if let Some(content_h) = store.panel_content_h(panel) {
            // Prefer the painter-published visible_h (exact body
            // height); fall back to `panel.h - 60` only when the
            // painter hasn't seeded one yet (first frame).
            let visible_h = store.panel_visible_h(panel).unwrap_or_else(|| {
                store
                    .panel_rect(panel)
                    .map(|r| (r.h - 60.0).max(0.0))
                    .unwrap_or(0.0)
            });
            let max_scroll = (content_h - visible_h).max(0.0);
            if next > max_scroll {
                next = max_scroll;
            }
        }
        store.set_panel_scroll(panel, next);
    }
    events.into_bump_slice()
}

/// Character input from the IME / keyboard. Inserts `ch` at the
/// caret of a focused [`InteractiveState::TextInput`] or appends to
/// a focused [`InteractiveState::Combobox::query`]. Other widget
/// kinds ignore the character.
pub fn dispatch_text_input<'frame>(
    store: &mut WidgetStore,
    ch: char,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent] {
    let mut events: BumpVec<'frame, WidgetEvent> = BumpVec::new_in(arena);
    // Filter control characters; only printable text gets inserted.
    if ch.is_control() {
        return events.into_bump_slice();
    }
    let Some(id) = store.focus_id() else {
        return events.into_bump_slice();
    };
    // If the focused widget has an active selection, replacing it
    // is the first half of "type to overwrite". For NumberInput we
    // additionally require the typed char to be a valid numeric
    // character — otherwise we drop the char without touching
    // selection state.
    let should_replace_selection = match store.get(id) {
        Some(InteractiveState::TextInput { .. }) | Some(InteractiveState::Combobox { .. }) => true,
        Some(InteractiveState::NumberInput { .. }) => is_numeric_input_char(ch),
        _ => false,
    };
    if should_replace_selection {
        delete_selection_if_any(store, id);
    }
    match store.get_mut(id) {
        Some(InteractiveState::TextInput { text, caret, .. }) => {
            text.insert(*caret, ch);
            *caret += ch.len_utf8();
            events.push(WidgetEvent::TextChanged(id));
        }
        Some(InteractiveState::Combobox { query, caret, .. }) => {
            let pos = (*caret).min(query.len());
            query.insert(pos, ch);
            *caret = pos + ch.len_utf8();
            events.push(WidgetEvent::TextChanged(id));
        }
        Some(InteractiveState::NumberInput { buffer, caret, .. }) if is_numeric_input_char(ch) => {
            buffer.insert(*caret, ch);
            *caret += ch.len_utf8();
            events.push(WidgetEvent::TextChanged(id));
        }
        _ => {}
    }
    events.into_bump_slice()
}

/// Filter for chars allowed in a NumberInput buffer: digits, sign,
/// decimal point, and scientific-notation `e`/`E`/`+`. Anything else
/// (letters, spaces, control chars) is dropped silently.
fn is_numeric_input_char(ch: char) -> bool {
    matches!(ch, '0'..='9' | '.' | '-' | '+' | 'e' | 'E')
}

/// On focus arrival into a NumberInput, sync `buffer` from `value`
/// using the same formatter the painter uses, place the caret at
/// the end, and mark state as Focused so the painter draws the
/// caret + focus ring (otherwise the user has no visual feedback
/// that the field accepted the click).
pub(super) fn init_number_buffer(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    // BlenderColorPicker channel chip: seed `value` from the parent
    // picker's current channel value (the chip's stored `value` is
    // stale; the painter renders the live derived value every
    // frame, but on focus the buffer needs to start from the
    // visible value, not the stale stored one).
    if let Some((parent, idx)) = store.blender_channel_chip(id) {
        let derived = derive_blender_channel_value(store, parent, idx);
        if let Some(InteractiveState::NumberInput { value, .. }) = store.get_mut(id) {
            *value = derived;
        }
    }
    if let Some(InteractiveState::NumberInput {
        state,
        value,
        buffer,
        caret,
        last_committed,
        selection_anchor,
    }) = store.get_mut(id)
    {
        *state = crate::widget::TextInputState::Focused;
        buffer.clear();
        use std::fmt::Write;
        let _ = write!(buffer, "{}", super::state::format_number(*value));
        *caret = buffer.len();
        *last_committed = *value;
        *selection_anchor = None;
    }
}

/// Approximate character advance per em — matches the painter's
/// caret-position formula. Used for drag-to-select byte-offset
/// computation without dragging text_system through dispatch.
const APPROX_ADVANCE_RATIO: f32 = 0.55;

/// True iff `id` belongs to the Inspector's collapsible section
/// header range. Used by the right-click dispatcher to decide
/// whether to open the section-outline menu vs the create-note
/// menu. Keeps the screen-specific knowledge in one place; if more
/// panels gain section headers, extend this match.
pub(super) fn is_section_header_id(id: ph2d_a11y::NodeId) -> bool {
    let v = id.0;
    (350..=359).contains(&v)
}

/// Color-target widgets — clicking these opens the global color
/// picker (and switches the target if it's already open). Includes
/// all section color circles (360..369) plus the standalone tint
/// ColorSwatch sample (328).
fn is_color_target_id(id: ph2d_a11y::NodeId) -> bool {
    let v = id.0;
    (360..=369).contains(&v) || v == 328
}

/// True iff `id` is one of the hierarchy entity rows (range
/// 400..411). Used by the drag-and-drop Down handler to detect
/// "the user is starting to drag a hierarchy row".
fn is_hierarchy_entity_id(id: ph2d_a11y::NodeId) -> bool {
    let v = id.0;
    (400..=411).contains(&v)
}

/// Drop kind resolved at the end of a hierarchy DnD: a sibling
/// insertion (above the given row, or at the very end of the list),
/// or a re-parent inside the given row.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum HierDrop {
    /// Drop dragged just before this row as a sibling.
    Before(ph2d_a11y::NodeId),
    /// Drop dragged as a child of this row.
    Inside(ph2d_a11y::NodeId),
    /// Drop at the very bottom (root level, end of list).
    End,
}

/// Resolve the drop position for a hierarchy DnD using cursor (x, y)
/// vs each row rect. Row y is split into three bands:
///   - top 30% → drop above this row (sibling)
///   - middle 40% → drop inside this row (child)
///   - bottom 30% → continue scanning (drop below this row)
///
/// `cursor_x` is checked against the row's horizontal extent. Without
/// this, a cursor far to the LEFT of an already-indented row was
/// still resolved as `Inside(that row)`, making the dragged entity a
/// grandchild instead of a root sibling — the user's "TAB fica tão
/// grande que parece neto" bug. The fallback is `End` (drop at root).
///
/// When no row is hit, returns `End`. Skips the dragged row itself.
fn find_hierarchy_drop(
    hit_index: &HitIndex,
    cursor_x: f32,
    cursor_y: f32,
    dragged: ph2d_a11y::NodeId,
) -> HierDrop {
    for (id, rect) in hit_index.iter_registrations() {
        if !is_hierarchy_entity_id(id) {
            continue;
        }
        if id == dragged {
            continue;
        }
        let top = rect.y;
        let bot = rect.y + rect.h;
        let inside_top = top + rect.h * 0.3;
        let inside_bot = top + rect.h * 0.7;
        // Cursor must overlap the row's horizontal extent to be a
        // candidate target. Otherwise drops from the left margin
        // accidentally re-parent the dragged entity into whatever
        // indented row happens to share its y band.
        let x_ok = cursor_x >= rect.x && cursor_x < rect.x + rect.w;
        if cursor_y < top || cursor_y >= bot {
            continue;
        }
        if !x_ok {
            // Cursor is in this row's y band but off to the left
            // (the indent gap). Treat as a sibling-Before for the
            // row that VISUALLY occupies this y, but at root depth —
            // the caller derives the new parent from the target so
            // a `Before` whose target has no parent lands at root.
            return HierDrop::Before(id);
        }
        if cursor_y < inside_top {
            return HierDrop::Before(id);
        } else if cursor_y < inside_bot {
            return HierDrop::Inside(id);
        } else {
            // Bottom band falls through to the next row's "before"
            // at the seam.
            continue;
        }
    }
    HierDrop::End
}

/// Maps a scrollbar thumb's hit id back to the panel it scrolls.
/// Returns `None` for non-scrollbar ids. Keeps the panel↔scrollbar
/// mapping in one place — hosts that add new scrollable panels
/// extend this match.
fn scrollbar_panel_for_id(id: ph2d_a11y::NodeId) -> Option<ph2d_a11y::NodeId> {
    use crate::screens::hero::ids;
    if id == crate::widget::INSPECTOR_SCROLLBAR_ID {
        Some(ids::INSP_PANEL)
    } else if id == crate::widget::HIERARCHY_SCROLLBAR_ID {
        Some(ids::HIER_PANEL)
    } else {
        None
    }
}

/// Detect a Down landing on a `NumberInput`'s up/down stepper and
/// apply +/- one step to the value + buffer. Returns true iff the
/// click landed on a stepper (caller emits `ValueChanged` and skips
/// the default caret-placement path so the click doesn't also move
/// the caret).
///
/// Step heuristic: `0.01` when the current buffer contains a `.`
/// (fractional value), `1.0` otherwise (integer). Dispatch has no
/// access to the widget's `step` field — that lives on the
/// `NumberInput` struct, not on the store's `InteractiveState`.
fn apply_number_stepper_if_hit(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    host: Rect,
    click_x: f32,
    click_y: f32,
) -> bool {
    use crate::widget::NumberInput;
    let (current_value, buffer) = match store.get(id) {
        Some(InteractiveState::NumberInput { value, buffer, .. }) => (*value, buffer.clone()),
        _ => return false,
    };
    let probe = NumberInput::new(id, "", current_value);
    let up = probe.up_rect(host);
    let down = probe.down_rect(host);
    let direction = if up.contains(click_x, click_y) {
        1.0_f64
    } else if down.contains(click_x, click_y) {
        -1.0_f64
    } else {
        return false;
    };
    let step = if buffer.contains('.') {
        0.01_f64
    } else {
        1.0_f64
    };
    let new_val = current_value + direction * step;
    if let Some(InteractiveState::NumberInput {
        value,
        buffer,
        last_committed,
        ..
    }) = store.get_mut(id)
    {
        *value = new_val;
        *buffer = super::format_number(new_val);
        *last_committed = new_val;
    }
    // Mirror to a linked slider if there is one. The store doesn't
    // currently expose a typed `set_slider_value`, so we mutate the
    // variant directly.
    if let Some(slider_id) = store.linked_slider(id)
        && let Some(InteractiveState::Slider { value, .. }) = store.get_mut(slider_id)
    {
        *value = (new_val as f32).clamp(0.0, 1.0);
    }
    true
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

/// Map a pointer (x, y) to a byte offset within the editable buffer
/// of the widget at `id`. Honors per-widget layout (hex field has a
/// 36 px label prefix; Combobox text sits after a 16 px search icon;
/// channel chips are centered; multi-line `TextInput` content split
/// on `\n` is line-aware via the y-coordinate).
///
/// When `text_system: Some(ts)`, walks the per-line glyph layout to
/// find the byte whose pixel position is **closest** to `click_x`
/// — pixel-perfect, "caret appears where you clicked". When `None`,
/// falls back to the `font_size * APPROX_ADVANCE_RATIO` heuristic
/// (acceptable for tests; visibly off on real fonts).
///
/// Always snaps to **end-of-line** when the click lands past the
/// last glyph on a multi-line widget's line (per
/// `docs/UI_Bugs/README.md` §3.3 lesson: don't let click→byte cross
/// visible line boundaries).
fn byte_offset_from_click_xy(
    store: &WidgetStore,
    id: ph2d_a11y::NodeId,
    rect: Rect,
    click_x: f32,
    click_y: f32,
    text_system: Option<&mut TextSystem>,
) -> usize {
    // Per-widget text + layout parameters. Font sizes MUST match
    // the painters' tokens exactly — a 1 px mismatch shifts every
    // measured prefix and the caret lands on the wrong byte. All
    // text widgets paint at `TypeToken::Base.px()` except the hex
    // field which uses `TypeToken::Sm.px()`.
    //
    // `multiline` = true when the painter is the `TextArea` 3+ row
    // layout (inferred from `\n` content since the dispatch has no
    // widget-kind discriminator).
    use ph2d_tokens::TypeToken;
    let font_base = TypeToken::Base.px();
    let font_sm = TypeToken::Sm.px();
    let (text, text_start_x, text_start_y, font_size, multiline) = match store.get(id) {
        Some(InteractiveState::TextInput { text, .. }) => {
            let is_hex = store.blender_hex_parent(id).is_some();
            if is_hex {
                // Hex field paints label "Hex" + value text at Sm.
                (text.as_str(), rect.x + 8.0 + 36.0, rect.y, font_sm, false)
            } else if text.contains('\n') {
                // TextArea: pad_x = Spacing::Lg, pad_y = Spacing::Md,
                // line_h = font_size + 4 (matches the painter).
                (text.as_str(), rect.x + 12.0, rect.y + 8.0, font_base, true)
            } else {
                (text.as_str(), rect.x + 12.0, rect.y, font_base, false)
            }
        }
        Some(InteractiveState::NumberInput { buffer, .. }) => {
            // Plain NumberInput uses Spacing::Lg pad. Channel chips
            // are centered — their click→byte offset depends on the
            // current text width which we don't measure here, so we
            // approximate by treating the chip as if text starts at
            // its left padding.
            (buffer.as_str(), rect.x + 12.0, rect.y, font_base, false)
        }
        Some(InteractiveState::Combobox { query, .. }) => {
            // Combobox text sits AFTER the search icon + gap, not at
            // the left edge of the pill. Mirrors the painter math
            // `inner_x = rect.x + pad_x + icon_size + Spacing::Md`.
            let icon_size = (rect.h * 0.5).clamp(14.0, 18.0);
            let inner_x = rect.x + 12.0 + icon_size + 8.0;
            (query.as_str(), inner_x, rect.y, font_base, false)
        }
        _ => return 0,
    };
    if multiline {
        // Determine which `\n`-separated line was clicked from the
        // y-coordinate relative to the text-area inner top. Then
        // snap to end-of-line if the click lands past the last
        // glyph — fixes the "clicking right of short line on line
        // 1 lands the caret at end of line 2" feel reported by the
        // user (TextArea bug log).
        let line_h = font_size + 4.0;
        let rel_y = (click_y - text_start_y).max(0.0);
        let mut line_idx = (rel_y / line_h).floor() as usize;
        let line_count = text.split('\n').count();
        if line_count > 0 && line_idx >= line_count {
            line_idx = line_count - 1;
        }

        let mut line_start: usize = 0;
        for (i, line) in text.split('\n').enumerate() {
            let line_end = line_start + line.len();
            if i == line_idx {
                let local =
                    nearest_byte_on_line(line, font_size, text_start_x, click_x, text_system);
                return line_start + local;
            }
            line_start = line_end + 1; // +1 for the '\n'
        }
        return text.len();
    }

    nearest_byte_on_line(text, font_size, text_start_x, click_x, text_system)
}

/// For a single line `text` rendered at `font_size` starting at
/// pixel `text_start_x`, return the byte offset whose glyph boundary
/// is closest to `click_x`. With a real `TextSystem`, this means
/// pixel-perfect "caret lands where you clicked" UX. Without one,
/// falls back to a `font_size * APPROX_ADVANCE_RATIO` heuristic
/// (off by 1–2 chars on proportional fonts but tolerable for tests).
///
/// Snaps to **end-of-line** when `click_x` is past the last glyph —
/// never returns a byte past `text.len()`.
fn nearest_byte_on_line(
    text: &str,
    font_size: f32,
    text_start_x: f32,
    click_x: f32,
    text_system: Option<&mut TextSystem>,
) -> usize {
    if let Some(ts) = text_system {
        // Walk every char boundary, layout the prefix, and pick the
        // boundary whose right edge is closest to click_x. O(n²)
        // for an n-char line but n is small (single-line content
        // for any reasonable input) and the parley LayoutContext
        // pools its allocations, so the actual cost per click is
        // microseconds.
        let target = (click_x - text_start_x).max(0.0);
        let mut best_byte: usize = 0;
        let mut best_dist = f32::INFINITY;
        for (idx, _) in text.char_indices() {
            // `prefix_width` includes trailing whitespace; the
            // naked `layout(...).width()` trimmed it, which broke
            // click→caret on lines that contained spaces.
            let w = ts.prefix_width(&text[..idx], font_size);
            let dist = (w - target).abs();
            if dist < best_dist {
                best_dist = dist;
                best_byte = idx;
            }
        }
        // Also consider the end-of-string boundary.
        let end_w = ts.prefix_width(text, font_size);
        let dist = (end_w - target).abs();
        if dist < best_dist {
            best_byte = text.len();
        }
        return best_byte;
    }
    // Fallback heuristic (no text_system).
    let advance = font_size * APPROX_ADVANCE_RATIO;
    if advance <= 0.0 {
        return 0;
    }
    let rel_x = (click_x - text_start_x).max(0.0);
    let approx_chars = (rel_x / advance).round() as usize;
    approx_chars.min(text.len())
}

/// Place the caret at byte offset `offset` on the TextInput /
/// NumberInput widget at `id`. When `seed_anchor` is true (single
/// Down event), the selection_anchor is reset to the new caret —
/// any prior selection collapses. When false (Move during drag),
/// the anchor is preserved so the selection extends from anchor →
/// new caret. No-op for non-text widgets.
fn place_text_caret(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    offset: usize,
    seed_anchor: bool,
) {
    let (text, caret, selection_anchor): (&str, &mut usize, &mut Option<usize>) =
        match store.get_mut(id) {
            Some(InteractiveState::TextInput {
                text,
                caret,
                selection_anchor,
                ..
            }) => (text.as_str(), caret, selection_anchor),
            Some(InteractiveState::NumberInput {
                buffer,
                caret,
                selection_anchor,
                ..
            }) => (buffer.as_str(), caret, selection_anchor),
            Some(InteractiveState::Combobox {
                query,
                caret,
                selection_anchor,
                ..
            }) => (query.as_str(), caret, selection_anchor),
            _ => return,
        };
    let bounded = offset.min(text.len());
    let snapped = nearest_char_boundary(text, bounded);
    *caret = snapped;
    if seed_anchor {
        *selection_anchor = Some(snapped);
    }
}

fn nearest_char_boundary(s: &str, mut i: usize) -> usize {
    let len = s.len();
    if i >= len {
        return len;
    }
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// Reset the focused visual state of a text-editing widget at `id`
/// to its `Normal` variant. Used on every blur path (Down handler,
/// `cycle_focus`, ESC, hex commit) so the painter stops drawing the
/// caret + focus border once the widget loses focus. Combobox uses
/// its own `ComboboxState` enum so it gets a separate match arm.
pub(super) fn reset_focused_visual_state(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    match store.get_mut(id) {
        Some(InteractiveState::NumberInput { state, .. })
        | Some(InteractiveState::TextInput { state, .. }) => {
            *state = crate::widget::TextInputState::Normal;
        }
        Some(InteractiveState::Combobox { state, .. }) => {
            *state = crate::widget::ComboboxState::Normal;
        }
        _ => {}
    }
}

/// Set `selection_anchor = Some(0)` and `caret = text.len()` on the
/// focused TextInput / NumberInput widget at `id`. Triggered by
/// double-click and by Cmd/Ctrl+A. No-op for any other widget kind.
/// Read the currently selected text from a focused TextInput /
/// NumberInput / Combobox. Returns `None` when the widget isn't a
/// text widget or has no active selection. Caret is treated as a
/// zero-length selection (returns `Some("")`) — caller decides
/// whether empty copies are interesting.
fn clipboard_extract_selection(store: &WidgetStore, id: ph2d_a11y::NodeId) -> Option<String> {
    let (text, caret, anchor) = match store.get(id) {
        Some(InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            ..
        }) => (text.as_str(), *caret, *selection_anchor),
        Some(InteractiveState::NumberInput {
            buffer,
            caret,
            selection_anchor,
            ..
        }) => (buffer.as_str(), *caret, *selection_anchor),
        Some(InteractiveState::Combobox {
            query,
            caret,
            selection_anchor,
            ..
        }) => (query.as_str(), *caret, *selection_anchor),
        _ => return None,
    };
    let anchor = anchor?;
    let (start, end) = if anchor < caret {
        (anchor, caret)
    } else {
        (caret, anchor)
    };
    if start == end {
        return None;
    }
    let start = start.min(text.len());
    let end = end.min(text.len());
    Some(text[start..end].to_string())
}

/// Insert `text` at the caret of a focused text widget, replacing
/// any active selection. Shell calls this after reading the OS
/// clipboard in response to a pending paste request. Returns true
/// when something was inserted.
pub fn apply_clipboard_paste(store: &mut WidgetStore, id: ph2d_a11y::NodeId, text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    let _ = delete_selection_if_any(store, id);
    match store.get_mut(id) {
        Some(InteractiveState::TextInput {
            text: buf, caret, ..
        }) => {
            buf.insert_str(*caret, text);
            *caret += text.len();
            true
        }
        Some(InteractiveState::Combobox { query, caret, .. }) => {
            query.insert_str(*caret, text);
            *caret += text.len();
            true
        }
        Some(InteractiveState::NumberInput { buffer, caret, .. }) => {
            // Filter non-numeric chars so paste can't put a NumberInput
            // into an unparsable state. Allowed: digits, '.', '-', '+'.
            let filtered: String = text
                .chars()
                .filter(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '+'))
                .collect();
            if filtered.is_empty() {
                return false;
            }
            buffer.insert_str(*caret, &filtered);
            *caret += filtered.len();
            true
        }
        _ => false,
    }
}

pub(super) fn select_all_in_text_widget(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    match store.get_mut(id) {
        Some(InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            ..
        }) => {
            *selection_anchor = Some(0);
            *caret = text.len();
        }
        Some(InteractiveState::NumberInput {
            buffer,
            caret,
            selection_anchor,
            ..
        }) => {
            *selection_anchor = Some(0);
            *caret = buffer.len();
        }
        Some(InteractiveState::Combobox {
            query,
            caret,
            selection_anchor,
            ..
        }) => {
            *selection_anchor = Some(0);
            *caret = query.len();
        }
        _ => {}
    }
}

/// If the focused TextInput / NumberInput has a non-empty selection,
/// delete the selected range (replacing it with an empty cut at
/// `caret = sel_start`) and return true. The caller is then expected
/// to insert any pending character at the new caret position.
fn delete_selection_if_any(store: &mut WidgetStore, id: ph2d_a11y::NodeId) -> bool {
    let (text_ref, caret_ref, anchor_ref): (&mut String, &mut usize, &mut Option<usize>) =
        match store.get_mut(id) {
            Some(InteractiveState::TextInput {
                text,
                caret,
                selection_anchor,
                ..
            }) => (text, caret, selection_anchor),
            Some(InteractiveState::NumberInput {
                buffer,
                caret,
                selection_anchor,
                ..
            }) => (buffer, caret, selection_anchor),
            Some(InteractiveState::Combobox {
                query,
                caret,
                selection_anchor,
                ..
            }) => (query, caret, selection_anchor),
            _ => return false,
        };
    let Some(anchor) = *anchor_ref else {
        return false;
    };
    let (start, end) = if anchor < *caret_ref {
        (anchor, *caret_ref)
    } else {
        (*caret_ref, anchor)
    };
    if start == end {
        *anchor_ref = None;
        return false;
    }
    let start = start.min(text_ref.len());
    let end = end.min(text_ref.len());
    text_ref.replace_range(start..end, "");
    *caret_ref = start;
    *anchor_ref = None;
    true
}

/// Collapse any active selection on the focused TextInput /
/// NumberInput, optionally moving the caret to the left or right
/// edge of the original selection (matching standard text-editor
/// behavior for non-shift Arrow keys with an active selection).
/// Returns true iff a selection was collapsed.
fn collapse_selection(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    move_to_right_edge: bool,
) -> bool {
    let (caret, selection_anchor) = match store.get_mut(id) {
        Some(InteractiveState::TextInput {
            caret,
            selection_anchor,
            ..
        })
        | Some(InteractiveState::NumberInput {
            caret,
            selection_anchor,
            ..
        })
        | Some(InteractiveState::Combobox {
            caret,
            selection_anchor,
            ..
        }) => (caret, selection_anchor),
        _ => return false,
    };
    let Some(anchor) = *selection_anchor else {
        return false;
    };
    let (lo, hi) = if anchor < *caret {
        (anchor, *caret)
    } else {
        (*caret, anchor)
    };
    *caret = if move_to_right_edge { hi } else { lo };
    *selection_anchor = None;
    true
}

/// Read a single channel value (0..=1) from the parent picker's
/// current `value` + `channel_mode`. Used when seeding a chip's
/// edit buffer on focus arrival.
fn derive_blender_channel_value(store: &WidgetStore, parent: ph2d_a11y::NodeId, idx: u8) -> f64 {
    use crate::widget::{ChannelMode, rgba_to_hsv};
    let Some((cur, mode, _, _)) = store.blender_picker(parent) else {
        return 0.0;
    };
    match mode {
        ChannelMode::Rgb => cur.rgba[idx as usize] as f64 / 255.0,
        ChannelMode::Hsv => {
            let (h, s, v, a) = rgba_to_hsv(cur.rgba);
            [h, s, v, a][idx as usize] as f64
        }
    }
}

/// On focus departure (Blur, Tab away, Enter commit) from a
/// NumberInput, parse `buffer.trim()`. On success → update `value` +
/// `last_committed` and emit `ValueChanged`. On failure → revert the
/// buffer to the formatted `last_committed`. After committing,
/// mirrors the new value into a linked Slider (clamped to [0..1]).
pub(super) fn commit_number_buffer<'a>(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    events: &mut BumpVec<'a, WidgetEvent>,
) {
    let mut new_value: Option<f64> = None;
    {
        let Some(InteractiveState::NumberInput {
            value,
            buffer,
            caret,
            last_committed,
            ..
        }) = store.get_mut(id)
        else {
            return;
        };
        match buffer.trim().parse::<f64>() {
            Ok(parsed) if parsed.is_finite() => {
                if (parsed - *value).abs() > f64::EPSILON {
                    *value = parsed;
                    *last_committed = parsed;
                    events.push(WidgetEvent::ValueChanged(id));
                    new_value = Some(parsed);
                }
                buffer.clear();
                use std::fmt::Write;
                let _ = write!(buffer, "{}", super::state::format_number(*value));
                *caret = buffer.len();
            }
            _ => {
                buffer.clear();
                use std::fmt::Write;
                let _ = write!(buffer, "{}", super::state::format_number(*last_committed));
                *value = *last_committed;
                *caret = buffer.len();
            }
        }
    }
    if let Some(v) = new_value
        && let Some(slider_id) = store.linked_slider(id)
        && let Some(InteractiveState::Slider { value, .. }) = store.get_mut(slider_id)
    {
        *value = (v as f32).clamp(0.0, 1.0);
        events.push(WidgetEvent::ValueChanged(slider_id));
    }
    // BlenderColorPicker channel chip: write the parsed value back
    // into the parent picker's RGBA / HSVA dimension at `idx`.
    if let Some(v) = new_value
        && let Some((parent, idx)) = store.blender_channel_chip(id)
    {
        apply_blender_channel_value(store, parent, idx, v as f32);
        events.push(WidgetEvent::ValueChanged(parent));
    }
}

/// Rewrite the parent BlenderPicker's color value with `new_norm`
/// (0..=1) at channel index `idx`. Honors the parent's current
/// `channel_mode`: in RGB mode `idx` maps to R/G/B/A, in HSV mode
/// `idx` maps to H/S/V/A (then converted to RGBA via
/// [`crate::widget::hsv_to_rgba8`]).
fn apply_blender_channel_value(
    store: &mut WidgetStore,
    parent: ph2d_a11y::NodeId,
    idx: u8,
    new_norm: f32,
) {
    use crate::widget::{ChannelMode, hsv_to_rgba8, rgba_to_hsv};
    use ph2d_tokens::ColorValue;
    let Some((cur, mode, _, _)) = store.blender_picker(parent) else {
        return;
    };
    let n = new_norm.clamp(0.0, 1.0);
    match mode {
        ChannelMode::Rgb => {
            let mut rgba = cur.rgba;
            rgba[idx as usize] = (n * 255.0).round() as u8;
            let new_value = ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]);
            store.set_blender_value(parent, new_value);
        }
        ChannelMode::Hsv => {
            // Use the retained (h, s) anchor as the canonical HSV
            // basis — RGBA→HSV would collapse H/S on V=0 / S=0
            // states and silently rotate the user's hue back to red
            // when they edit V or A. Only the channel being changed
            // gets overwritten with `n`; the others stay retained.
            let (retained_h, retained_s) = store.blender_hsv_anchor(parent).unwrap_or((0.0, 1.0));
            let (_, _, v_rgba, a_rgba) = rgba_to_hsv(cur.rgba);
            let mut h = retained_h;
            let mut s = retained_s;
            let mut v = v_rgba;
            let mut a = a_rgba;
            match idx {
                0 => h = n,
                1 => s = n,
                2 => v = n,
                3 => a = n,
                _ => {}
            }
            let rgba = hsv_to_rgba8(h, s, v, a);
            let new_value = ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]);
            store.set_blender_value_with_hsv(parent, new_value, h, s);
        }
    }
}

/// Restore a NumberInput's buffer to its last committed value
/// without emitting any event. Used by Escape.
pub(super) fn revert_number_buffer(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::NumberInput {
        value,
        buffer,
        caret,
        last_committed,
        ..
    }) = store.get_mut(id)
    {
        *value = *last_committed;
        buffer.clear();
        use std::fmt::Write;
        let _ = write!(buffer, "{}", super::state::format_number(*last_committed));
        *caret = buffer.len();
    }
}

/// Parse the hex `TextInput` buffer at `id` and apply the resulting
/// color to the linked parent BlenderPicker (via
/// [`WidgetStore::link_blender_hex`]). Whether the parse succeeds or
/// not, the buffer is normalised to the canonical `#RRGGBBAA` form
/// of the parent's resulting value, so the painter always shows a
/// consistent string after commit. No-op if `id` is not a TextInput
/// or has no linked parent.
pub(super) fn commit_hex_buffer<'a>(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    events: &mut BumpVec<'a, WidgetEvent>,
) {
    let Some(parent) = store.blender_hex_parent(id) else {
        return;
    };
    let buf_owned: String = match store.get(id) {
        Some(InteractiveState::TextInput { text, .. }) => text.clone(),
        _ => return,
    };
    if let Some(color) = crate::widget::parse_hex(&buf_owned) {
        store.set_blender_value(parent, color);
        events.push(WidgetEvent::ValueChanged(parent));
    }
    write_hex_canonical(store, id);
}

/// Rewrite the hex `TextInput` buffer at `id` with the canonical
/// `#RRGGBBAA` form of the linked parent BlenderPicker's current
/// value. Used by both commit (after parse + apply) and revert
/// (ESC) so the visible text always matches the parent state.
pub(super) fn write_hex_canonical(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    let Some(parent) = store.blender_hex_parent(id) else {
        return;
    };
    let Some((cv, ..)) = store.blender_picker(parent) else {
        return;
    };
    let [r, g, b, a] = cv.rgba;
    if let Some(InteractiveState::TextInput { text, caret, .. }) = store.get_mut(id) {
        text.clear();
        use std::fmt::Write;
        let _ = write!(text, "#{r:02X}{g:02X}{b:02X}{a:02X}");
        *caret = text.len();
    }
}

// ---------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------

fn is_focusable(store: &WidgetStore, id: ph2d_a11y::NodeId) -> bool {
    match store.get(id) {
        Some(InteractiveState::Button { state }) => *state != ButtonState::Disabled,
        Some(InteractiveState::Toggle { state, .. }) => *state != ToggleState::Disabled,
        Some(InteractiveState::Slider { state, .. }) => *state != SliderState::Disabled,
        Some(InteractiveState::Checkbox { state, .. }) => *state != CheckboxState::Disabled,
        // Plain rects (section headers without collapsibility, etc.)
        // are still focusable for keyboard nav purposes — they don't
        // emit click events but accept Tab focus.
        Some(InteractiveState::Plain) => true,
        // Phases C-D add per-kind focusability for the rest.
        Some(_) => true,
        None => false,
    }
}

fn update_hover(store: &mut WidgetStore, hit: Option<ph2d_a11y::NodeId>) {
    let prev = store.hot_id();
    if prev == hit {
        return;
    }
    if let Some(old) = prev {
        // Revert previous widget's state from Hovered → Normal
        // (unless it's currently Pressed/Disabled, which we leave
        // alone).
        leave_hover(store, old);
    }
    if let Some(new) = hit {
        // Skip hover state on the active (dragging) widget — its
        // state stays Pressed.
        if store.active_id() != Some(new) {
            enter_hover(store, new);
        }
    }
    store.set_hot(hit);
}

fn enter_hover(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    match store.get_mut(id) {
        Some(InteractiveState::Button { state }) if *state == ButtonState::Normal => {
            *state = ButtonState::Hovered
        }
        Some(InteractiveState::Toggle { state, .. }) if *state == ToggleState::Normal => {
            *state = ToggleState::Hovered
        }
        Some(InteractiveState::Slider { state, .. }) if *state == SliderState::Normal => {
            *state = SliderState::Hovered
        }
        Some(InteractiveState::Checkbox { state, .. }) if *state == CheckboxState::Normal => {
            *state = CheckboxState::Hovered
        }
        _ => {}
    }
}

fn leave_hover(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    match store.get_mut(id) {
        Some(InteractiveState::Button { state }) if *state == ButtonState::Hovered => {
            *state = ButtonState::Normal
        }
        Some(InteractiveState::Toggle { state, .. }) if *state == ToggleState::Hovered => {
            *state = ToggleState::Normal
        }
        Some(InteractiveState::Slider { state, .. }) if *state == SliderState::Hovered => {
            *state = SliderState::Normal
        }
        Some(InteractiveState::Checkbox { state, .. }) if *state == CheckboxState::Hovered => {
            *state = CheckboxState::Normal
        }
        _ => {}
    }
}

fn set_widget_pressed(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    match store.get_mut(id) {
        Some(InteractiveState::Button { state }) => *state = ButtonState::Pressed,
        Some(InteractiveState::Toggle { state, .. }) => *state = ToggleState::Pressed,
        Some(InteractiveState::Slider { state, .. }) => *state = SliderState::Dragging,
        Some(InteractiveState::Checkbox { state, .. }) => *state = CheckboxState::Pressed,
        _ => {}
    }
}

fn set_widget_released(store: &mut WidgetStore, id: ph2d_a11y::NodeId, still_hot: bool) {
    match store.get_mut(id) {
        Some(InteractiveState::Button { state }) => {
            *state = if still_hot {
                ButtonState::Hovered
            } else {
                ButtonState::Normal
            };
        }
        Some(InteractiveState::Toggle { state, .. }) => {
            *state = if still_hot {
                ToggleState::Hovered
            } else {
                ToggleState::Normal
            };
        }
        Some(InteractiveState::Slider { state, .. }) => {
            *state = if still_hot {
                SliderState::Hovered
            } else {
                SliderState::Normal
            };
        }
        Some(InteractiveState::Checkbox { state, .. }) => {
            *state = if still_hot {
                CheckboxState::Hovered
            } else {
                CheckboxState::Normal
            };
        }
        _ => {}
    }
}

fn apply_click<'a>(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    events: &mut BumpVec<'a, WidgetEvent>,
) {
    match store.get_mut(id) {
        Some(InteractiveState::Toggle { on, .. }) => {
            *on = !*on;
            events.push(WidgetEvent::Toggled(id));
        }
        Some(InteractiveState::Checkbox { value, .. }) => {
            *value = match *value {
                CheckboxValue::Unchecked | CheckboxValue::Indeterminate => CheckboxValue::Checked,
                CheckboxValue::Checked => CheckboxValue::Unchecked,
            };
            events.push(WidgetEvent::Toggled(id));
        }
        Some(InteractiveState::Dropdown { open, .. }) => {
            *open = !*open;
            // No event — caller observes via store.get(id).
        }
        Some(InteractiveState::Combobox { open, .. }) => {
            *open = !*open;
        }
        Some(InteractiveState::Button { .. }) | Some(InteractiveState::Plain) => {
            events.push(WidgetEvent::Click(id));
        }
        // Phase D adds per-kind click semantics (Tabs select,
        // Modal dismiss, TreeView select, ContextMenu item, etc.).
        _ => {
            events.push(WidgetEvent::Click(id));
        }
    }
}

fn cycle_focus<'a>(store: &mut WidgetStore, forward: bool, events: &mut BumpVec<'a, WidgetEvent>) {
    let order = store.focus_order();
    if order.is_empty() {
        return;
    }
    let current_pos = match store.focus_id() {
        Some(id) => order.iter().position(|x| *x == id),
        None => None,
    };
    let len = order.len();
    let start = match current_pos {
        Some(p) => {
            if forward {
                (p + 1) % len
            } else {
                (p + len - 1) % len
            }
        }
        None => {
            if forward {
                0
            } else {
                len - 1
            }
        }
    };
    // Walk forward until we find a focusable widget. Stop after one
    // full cycle to avoid infinite loop if nothing is focusable.
    let mut idx = start;
    for _ in 0..len {
        let id = order[idx];
        if is_focusable(store, id) {
            if let Some(old) = store.focus_id()
                && old != id
            {
                commit_number_buffer(store, old, events);
                commit_hex_buffer(store, old, events);
                reset_focused_visual_state(store, old);
                events.push(WidgetEvent::Blur(old));
            }
            if store.focus_id() != Some(id) {
                store.set_focus(Some(id));
                init_number_buffer(store, id);
                events.push(WidgetEvent::Focus(id));
            }
            return;
        }
        idx = if forward {
            (idx + 1) % len
        } else {
            (idx + len - 1) % len
        };
    }
}

/// If `id` is a [`InteractiveState::BlenderHit`] sub-control, apply
/// the click to its parent [`InteractiveState::BlenderPicker`] and
/// return the parent's id (so the caller can emit `ValueChanged`).
/// Returns `None` for non-picker hits.
fn apply_blender_hit(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    rect: Rect,
    px: f32,
    py: f32,
    button: ph2d_host::PointerButton,
) -> Option<ph2d_a11y::NodeId> {
    use crate::interaction::BlenderHitKind;
    use crate::widget::{
        ChannelMode, InterpolationMode, apply_blender_value_pick, apply_blender_wheel_pick,
    };
    let (parent, kind) = match store.get(id)? {
        InteractiveState::BlenderHit { parent, kind } => (*parent, *kind),
        _ => return None,
    };
    match kind {
        BlenderHitKind::Wheel => {
            apply_blender_wheel_pick(store, parent, rect, px, py).then_some(parent)
        }
        BlenderHitKind::ValueSlider => {
            apply_blender_value_pick(store, parent, rect, px, py).then_some(parent)
        }
        BlenderHitKind::InterpolationLinear => {
            store.set_blender_interpolation(parent, InterpolationMode::Linear);
            Some(parent)
        }
        BlenderHitKind::InterpolationPerceptual => {
            store.set_blender_interpolation(parent, InterpolationMode::Perceptual);
            Some(parent)
        }
        BlenderHitKind::ChannelRgb => {
            store.set_blender_channel_mode(parent, ChannelMode::Rgb);
            Some(parent)
        }
        BlenderHitKind::ChannelHsv => {
            store.set_blender_channel_mode(parent, ChannelMode::Hsv);
            Some(parent)
        }
        BlenderHitKind::ChannelSlider(idx) => {
            // The hit rect is now the slider track itself (the
            // painter registers only the inner track region, not
            // the full row). Direct normalisation against rect.x
            // and rect.w gives the value cleanly.
            let norm = if rect.w > 0.0 {
                ((px - rect.x) / rect.w).clamp(0.0, 1.0)
            } else {
                0.0
            };
            store.set_blender_channel(parent, idx, norm);
            Some(parent)
        }
        BlenderHitKind::Hex => {
            // `Hex` hits redirect focus to the hex TextInput sibling
            // registered with the same parent id hierarchy. The
            // BlenderHit NodeId IS the hex TextInput id in our
            // registration scheme (BLENDER_HEX = NodeId 604 which is
            // registered as TextInput, not as BlenderHit). This arm
            // is a no-op: focus was already set by the Down handler.
            // Return None so no spurious ValueChanged fires.
            None
        }
        BlenderHitKind::PaletteSwatch(swatch_idx) => {
            // Right-click: remove the swatch from the picker's
            // palette. Left/middle: pick its color.
            if button == ph2d_host::PointerButton::Secondary {
                if store.blender_palette_remove(parent, swatch_idx as usize) {
                    Some(parent)
                } else {
                    None
                }
            } else {
                let color = store
                    .blender_palette(parent)
                    .and_then(|p| p.get(swatch_idx as usize).copied());
                if let Some(color) = color {
                    store.set_blender_value(parent, color);
                    Some(parent)
                } else {
                    None
                }
            }
        }
        BlenderHitKind::AddSwatch => {
            // Append the picker's current value to the palette.
            let (cur, _, _, _) = store.blender_picker(parent)?;
            store.blender_palette_push(parent, cur);
            Some(parent)
        }
        BlenderHitKind::Eyedropper => {
            // Toggle eyedropper "pending" mode. While pending, the
            // next pointer Down anywhere except this button is
            // intercepted by the Down handler and emitted as
            // `WidgetEvent::EyedropperPick` for the host to perform
            // the GPU pixel readback. Clicking the button a second
            // time (still pending → same parent) cancels the mode.
            let already_pending = store.eyedropper_pending() == Some(parent);
            store.set_eyedropper_pending(if already_pending { None } else { Some(parent) });
            None
        }
        BlenderHitKind::DragHandle => {
            // Down on the drag bar — snapshot the cursor + current
            // offset so Move events can apply (cursor − down_cursor)
            // as the new delta. Up clears the anchor.
            store.begin_blender_drag(parent, px, py);
            None
        }
        BlenderHitKind::ResizeHandle => {
            // Down on the bottom-right gripper — anchor the cursor so
            // subsequent Moves apply incremental delta to the panel's
            // stored (dw, dh). Up clears the anchor.
            store.begin_panel_resize(parent, px, py);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::InteractiveState;
    use crate::widget::ButtonState;
    use crate::zones::Rect;
    use ph2d_a11y::NodeId;
    use ph2d_host::{Modifiers, PointerSource};

    fn pointer(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
        PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: ph2d_host::PointerButton::Primary,
            timestamp_ns: 0,
        }
    }

    fn key(kc: u32, shift: bool) -> KeyEvent {
        KeyEvent {
            keycode: kc,
            modifiers: Modifiers {
                shift,
                ctrl: false,
                alt: false,
                meta: false,
            },
            kind: KeyKind::Down,
            timestamp_ns: 0,
        }
    }

    fn one_button_setup() -> (WidgetStore, HitIndex) {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 100.0, 50.0));
        (store, hits)
    }

    #[test]
    fn pointer_move_into_widget_sets_hot_id_and_hover_state() {
        let (mut store, hits) = one_button_setup();
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, 50.0, 25.0),
            &arena,
        );
        assert_eq!(store.hot_id(), Some(NodeId(7)));
        assert_eq!(store.button_state(NodeId(7)), Some(ButtonState::Hovered));
    }

    #[test]
    fn pointer_move_out_clears_hot_and_reverts_state() {
        let (mut store, hits) = one_button_setup();
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, 50.0, 25.0),
            &arena,
        );
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, 500.0, 500.0),
            &arena,
        );
        assert_eq!(store.hot_id(), None);
        assert_eq!(store.button_state(NodeId(7)), Some(ButtonState::Normal));
    }

    #[test]
    fn button_down_sets_pressed_and_emits_focus() {
        let (mut store, hits) = one_button_setup();
        let arena = Bump::new();
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 25.0),
            &arena,
        );
        assert_eq!(store.button_state(NodeId(7)), Some(ButtonState::Pressed));
        assert_eq!(store.active_id(), Some(NodeId(7)));
        assert_eq!(evts, &[WidgetEvent::Focus(NodeId(7))]);
    }

    #[test]
    fn button_down_then_up_emits_click_and_clears_active() {
        let (mut store, hits) = one_button_setup();
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 25.0),
            &arena,
        );
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 50.0, 25.0),
            &arena,
        );
        assert_eq!(evts, &[WidgetEvent::Click(NodeId(7))]);
        assert_eq!(store.active_id(), None);
    }

    #[test]
    fn button_down_then_drag_out_then_up_does_not_click() {
        let (mut store, hits) = one_button_setup();
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 25.0),
            &arena,
        );
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 500.0, 500.0),
            &arena,
        );
        assert_eq!(evts, &[]);
        assert_eq!(store.active_id(), None);
    }

    #[test]
    fn disabled_button_does_not_focus_or_press_on_down() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Button {
                state: ButtonState::Disabled,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 100.0, 50.0));
        let arena = Bump::new();
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 25.0),
            &arena,
        );
        assert_eq!(evts, &[]);
        assert_eq!(store.active_id(), None);
        assert_eq!(store.button_state(NodeId(7)), Some(ButtonState::Disabled));
    }

    #[test]
    fn toggle_click_flips_on_and_emits_toggled() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Toggle {
                state: ToggleState::Normal,
                on: false,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 100.0, 50.0));
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 25.0),
            &arena,
        );
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 50.0, 25.0),
            &arena,
        );
        assert_eq!(evts, &[WidgetEvent::Toggled(NodeId(7))]);
        let (_, on) = store.toggle(NodeId(7)).unwrap();
        assert!(on);
    }

    #[test]
    fn tab_cycles_focus_forward() {
        let mut store = WidgetStore::with_capacity(4);
        for id in [1, 2, 3] {
            store.register(
                NodeId(id),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
        let arena = Bump::new();
        let evts = dispatch_key(&mut store, key(KEY_TAB, false), &arena);
        assert_eq!(evts, &[WidgetEvent::Focus(NodeId(1))]);
        let _ = dispatch_key(&mut store, key(KEY_TAB, false), &arena);
        assert_eq!(store.focus_id(), Some(NodeId(2)));
        let _ = dispatch_key(&mut store, key(KEY_TAB, false), &arena);
        assert_eq!(store.focus_id(), Some(NodeId(3)));
        let _ = dispatch_key(&mut store, key(KEY_TAB, false), &arena);
        assert_eq!(store.focus_id(), Some(NodeId(1)), "wraps around");
    }

    #[test]
    fn shift_tab_cycles_focus_backward() {
        let mut store = WidgetStore::with_capacity(4);
        for id in [1, 2, 3] {
            store.register(
                NodeId(id),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
        let arena = Bump::new();
        let _ = dispatch_key(&mut store, key(KEY_TAB, true), &arena);
        assert_eq!(store.focus_id(), Some(NodeId(3)));
        let _ = dispatch_key(&mut store, key(KEY_TAB, true), &arena);
        assert_eq!(store.focus_id(), Some(NodeId(2)));
    }

    #[test]
    fn enter_on_focused_button_emits_click() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(1),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        store.set_focus(Some(NodeId(1)));
        let arena = Bump::new();
        let evts = dispatch_key(&mut store, key(KEY_ENTER, false), &arena);
        assert_eq!(evts, &[WidgetEvent::Click(NodeId(1))]);
    }

    #[test]
    fn escape_blurs_focus() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(1),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        store.set_focus(Some(NodeId(1)));
        let arena = Bump::new();
        let evts = dispatch_key(&mut store, key(KEY_ESCAPE, false), &arena);
        assert_eq!(evts, &[WidgetEvent::Blur(NodeId(1))]);
        assert_eq!(store.focus_id(), None);
    }

    #[test]
    fn slider_down_jumps_to_pointer_and_emits_value_changed() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.5,
                orientation: SliderOrientation::Horizontal,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 100.0, 20.0));
        let arena = Bump::new();
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 75.0, 10.0),
            &arena,
        );
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(id) if *id == NodeId(7)))
        );
        let (state, v) = store.slider(NodeId(7)).unwrap();
        assert_eq!(state, SliderState::Dragging);
        assert!((v - 0.75).abs() < 0.01, "expected 0.75, got {v}");
    }

    #[test]
    fn slider_drag_emits_value_changed_per_move() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 100.0, 20.0));
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 25.0, 10.0),
            &arena,
        );
        // Drag the cursor outside the rect — value still updates,
        // because active drag persists.
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, 90.0, 200.0),
            &arena,
        );
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(_)))
        );
        let (_, v) = store.slider(NodeId(7)).unwrap();
        assert!((v - 0.90).abs() < 0.01);
    }

    #[test]
    fn slider_release_clears_active_and_does_not_emit_click() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 100.0, 20.0));
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 10.0),
            &arena,
        );
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 50.0, 10.0),
            &arena,
        );
        assert!(
            !evts.iter().any(|e| matches!(e, WidgetEvent::Click(_))),
            "Slider should not emit Click on release"
        );
        assert_eq!(store.active_id(), None);
    }

    #[test]
    fn vertical_slider_inverts_y_to_value() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Vertical,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 20.0, 100.0));
        let arena = Bump::new();
        // Down at the top of the rect → value should be near 1.0.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 10.0, 5.0),
            &arena,
        );
        let (_, v) = store.slider(NodeId(7)).unwrap();
        assert!((v - 0.95).abs() < 0.01, "expected ~0.95 at top, got {v}");
    }

    #[test]
    fn checkbox_click_cycles_unchecked_to_checked() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Unchecked,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 18.0, 18.0));
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 9.0, 9.0),
            &arena,
        );
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 9.0, 9.0),
            &arena,
        );
        assert!(evts.iter().any(|e| matches!(e, WidgetEvent::Toggled(_))));
        let (_, v) = store.checkbox(NodeId(7)).unwrap();
        assert_eq!(v, CheckboxValue::Checked);
    }

    #[test]
    fn checkbox_indeterminate_then_click_yields_checked() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(7),
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Indeterminate,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(7), Rect::new(0.0, 0.0, 18.0, 18.0));
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 9.0, 9.0),
            &arena,
        );
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 9.0, 9.0),
            &arena,
        );
        let (_, v) = store.checkbox(NodeId(7)).unwrap();
        assert_eq!(v, CheckboxValue::Checked);
    }

    #[test]
    fn key_up_event_is_ignored() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(NodeId(1), InteractiveState::Plain);
        let arena = Bump::new();
        let evts = dispatch_key(
            &mut store,
            KeyEvent {
                keycode: KEY_TAB,
                modifiers: Modifiers::default(),
                kind: KeyKind::Up,
                timestamp_ns: 0,
            },
            &arena,
        );
        assert_eq!(evts, &[]);
    }

    // -----------------------------------------------------------------
    // Phase C — TextInput / NumberInput / Combobox / Dropdown
    // -----------------------------------------------------------------

    use crate::widget::TextInputState;

    fn text_input(text: &str) -> InteractiveState {
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: text.into(),
            caret: text.len(),
            selection_anchor: None,
        }
    }

    #[test]
    fn text_input_char_insert_advances_caret() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(NodeId(1), text_input(""));
        store.set_focus(Some(NodeId(1)));
        let arena = Bump::new();
        let evts = dispatch_text_input(&mut store, 'a', &arena);
        assert!(matches!(evts, [WidgetEvent::TextChanged(_)]));
        let evts2 = dispatch_text_input(&mut store, 'b', &arena);
        assert!(matches!(evts2, [WidgetEvent::TextChanged(_)]));
        assert_eq!(store.text(NodeId(1)), Some("ab"));
    }

    #[test]
    fn text_input_backspace_at_caret() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(NodeId(1), text_input("hello"));
        store.set_focus(Some(NodeId(1)));
        let arena = Bump::new();
        let evts = dispatch_key(&mut store, key(KEY_BACKSPACE, false), &arena);
        assert!(matches!(evts, [WidgetEvent::TextChanged(_)]));
        assert_eq!(store.text(NodeId(1)), Some("hell"));
    }

    #[test]
    fn text_input_arrow_left_moves_caret() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(NodeId(1), text_input("xyz"));
        store.set_focus(Some(NodeId(1)));
        let arena = Bump::new();
        let _ = dispatch_key(&mut store, key(KEY_ARROW_LEFT, false), &arena);
        // The caret moved (no text changed). Reading caret directly:
        if let Some(InteractiveState::TextInput { caret, .. }) = store.get(NodeId(1)) {
            assert_eq!(*caret, 2);
        } else {
            panic!("expected TextInput");
        }
    }

    #[test]
    fn text_input_unfocused_ignores_input() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(NodeId(1), text_input(""));
        let arena = Bump::new();
        let evts = dispatch_text_input(&mut store, 'x', &arena);
        assert_eq!(evts, &[]);
        assert_eq!(store.text(NodeId(1)), Some(""));
    }

    #[test]
    fn number_input_arrow_up_increments() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(1),
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 5.0,
                buffer: "5".into(),
                caret: 1,
                last_committed: 5.0,
                selection_anchor: None,
            },
        );
        store.set_focus(Some(NodeId(1)));
        let arena = Bump::new();
        let evts = dispatch_key(&mut store, key(KEY_ARROW_UP, false), &arena);
        assert!(matches!(evts, [WidgetEvent::ValueChanged(_)]));
        if let Some(InteractiveState::NumberInput { value, .. }) = store.get(NodeId(1)) {
            assert!((value - 6.0).abs() < f64::EPSILON);
        } else {
            panic!("expected NumberInput");
        }
    }

    fn make_number_store(value: f64) -> WidgetStore {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(1),
            InteractiveState::NumberInput {
                state: TextInputState::Focused,
                value,
                buffer: super::super::state::format_number(value),
                caret: super::super::state::format_number(value).len(),
                last_committed: value,
                selection_anchor: None,
            },
        );
        store.set_focus(Some(NodeId(1)));
        store
    }

    #[test]
    fn number_input_typing_replaces_buffer_and_commits_on_enter() {
        let mut store = make_number_store(5.0);
        let arena = Bump::new();
        // Erase '5' then type "1.25".
        let _ = dispatch_key(&mut store, key(KEY_BACKSPACE, false), &arena);
        for ch in ['1', '.', '2', '5'] {
            let _ = dispatch_text_input(&mut store, ch, &arena);
        }
        // Buffer reflects edits but value has not yet committed.
        let (_, value, buf, _, _) = store.number_input(NodeId(1)).unwrap();
        assert_eq!(buf, "1.25");
        assert!((value - 5.0).abs() < f64::EPSILON);
        // Enter commits.
        let evts = dispatch_key(&mut store, key(KEY_ENTER, false), &arena);
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(_)))
        );
        let (_, value, _, _, _) = store.number_input(NodeId(1)).unwrap();
        assert!((value - 1.25).abs() < 1e-9);
    }

    #[test]
    fn number_input_escape_reverts_to_last_committed() {
        let mut store = make_number_store(7.0);
        let arena = Bump::new();
        for ch in ['9', '9'] {
            let _ = dispatch_text_input(&mut store, ch, &arena);
        }
        let (_, _, buf, _, _) = store.number_input(NodeId(1)).unwrap();
        assert_eq!(buf, "799");
        let evts = dispatch_key(&mut store, key(KEY_ESCAPE, false), &arena);
        assert!(evts.iter().any(|e| matches!(e, WidgetEvent::Blur(_))));
        let (_, value, buf, _, _) = store.number_input(NodeId(1)).unwrap();
        assert!((value - 7.0).abs() < f64::EPSILON);
        assert_eq!(buf, "7");
    }

    #[test]
    fn number_input_unparsable_buffer_reverts_on_commit() {
        let mut store = make_number_store(3.0);
        let arena = Bump::new();
        // Replace the existing single digit with garbage.
        let _ = dispatch_key(&mut store, key(KEY_BACKSPACE, false), &arena);
        for ch in ['e', 'e', 'e'] {
            let _ = dispatch_text_input(&mut store, ch, &arena);
        }
        let _ = dispatch_key(&mut store, key(KEY_ENTER, false), &arena);
        let (_, value, buf, _, _) = store.number_input(NodeId(1)).unwrap();
        assert!((value - 3.0).abs() < f64::EPSILON);
        assert_eq!(buf, "3");
    }

    #[test]
    fn number_input_filters_non_numeric_chars() {
        let mut store = make_number_store(0.0);
        let arena = Bump::new();
        let _ = dispatch_key(&mut store, key(KEY_BACKSPACE, false), &arena);
        // Typing letters should be filtered.
        for ch in ['a', 'b', 'X', '!', ' '] {
            let _ = dispatch_text_input(&mut store, ch, &arena);
        }
        let (_, _, buf, _, _) = store.number_input(NodeId(1)).unwrap();
        assert_eq!(buf, "");
    }

    #[test]
    fn number_input_set_value_syncs_buffer_when_unfocused() {
        let mut store = make_number_store(0.0);
        store.set_focus(None); // simulate unfocused
        store.set_number_value(NodeId(1), 0.42);
        let (_, value, buf, _, _) = store.number_input(NodeId(1)).unwrap();
        assert!((value - 0.42).abs() < 1e-9);
        assert_eq!(buf, "0.420");
    }

    #[test]
    fn number_input_set_value_preserves_buffer_when_focused() {
        let mut store = make_number_store(0.0);
        // Type a partial edit.
        let arena = Bump::new();
        let _ = dispatch_key(&mut store, key(KEY_BACKSPACE, false), &arena);
        for ch in ['1', '.', '2'] {
            let _ = dispatch_text_input(&mut store, ch, &arena);
        }
        // While focused, programmatic set_number_value should NOT
        // clobber the in-progress buffer.
        store.set_number_value(NodeId(1), 9.99);
        let (_, value, buf, _, _) = store.number_input(NodeId(1)).unwrap();
        assert!((value - 9.99).abs() < 1e-9);
        assert_eq!(buf, "1.2");
    }

    #[test]
    fn slider_drag_propagates_to_linked_number_input() {
        let mut store = WidgetStore::with_capacity(8);
        store.register(
            NodeId(1),
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            NodeId(2),
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: "0".into(),
                caret: 1,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.link_slider_number(NodeId(1), NodeId(2));
        let mut hits = HitIndex::new();
        hits.register(NodeId(1), Rect::new(0.0, 0.0, 100.0, 30.0));
        let arena = Bump::new();
        // Down at x=50 → value 0.5 → number value 0.5.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 15.0),
            &arena,
        );
        let (_, num_value, num_buf, _, _) = store.number_input(NodeId(2)).unwrap();
        assert!((num_value - 0.5).abs() < 1e-6);
        assert_eq!(num_buf, "0.500");
    }

    #[test]
    fn number_commit_propagates_to_linked_slider() {
        let mut store = WidgetStore::with_capacity(8);
        store.register(
            NodeId(1),
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            NodeId(2),
            InteractiveState::NumberInput {
                state: TextInputState::Focused,
                value: 0.0,
                buffer: "0".into(),
                caret: 1,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.link_slider_number(NodeId(1), NodeId(2));
        store.set_focus(Some(NodeId(2)));
        let arena = Bump::new();
        // Erase '0' then type "0.75".
        let _ = dispatch_key(&mut store, key(KEY_BACKSPACE, false), &arena);
        for ch in ['0', '.', '7', '5'] {
            let _ = dispatch_text_input(&mut store, ch, &arena);
        }
        let _ = dispatch_key(&mut store, key(KEY_ENTER, false), &arena);
        let (_, sv) = store.slider(NodeId(1)).unwrap();
        assert!((sv - 0.75).abs() < 1e-5);
    }

    #[test]
    fn blender_wheel_click_mutates_picker_value() {
        use crate::interaction::BlenderHitKind;
        use crate::widget::{ChannelMode, InterpolationMode};
        use ph2d_tokens::ColorValue;
        let mut store = WidgetStore::with_capacity(8);
        store.register(
            NodeId(100),
            InteractiveState::BlenderPicker {
                value: ColorValue::from_rgba8(231, 231, 231, 255),
                channel_mode: ChannelMode::Rgb,
                interpolation: InterpolationMode::Perceptual,
                active_palette: 0,
                hsv_h: 0.0,
                hsv_s: 1.0,
            },
        );
        store.register(
            NodeId(101),
            InteractiveState::BlenderHit {
                parent: NodeId(100),
                kind: BlenderHitKind::Wheel,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(101), Rect::new(0.0, 0.0, 100.0, 100.0));
        let arena = Bump::new();
        // Click right-edge → hue ≈ 0°, sat ≈ 1.0 → red-leaning value.
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 95.0, 50.0),
            &arena,
        );
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(_))),
            "expected a ValueChanged event from wheel click"
        );
        let (new_value, _, _, _) = store.blender_picker(NodeId(100)).unwrap();
        // Value should have rotated away from neutral grey.
        assert!(
            new_value.rgba != [231, 231, 231, 255],
            "picker value should change after wheel click"
        );
    }

    #[test]
    fn linked_number_value_clamps_into_slider_range() {
        // NumberInput accepts arbitrary f64; the slider snapshot
        // clamps to [0..1] without panicking on out-of-range commits.
        let mut store = WidgetStore::with_capacity(8);
        store.register(
            NodeId(1),
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.5,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            NodeId(2),
            InteractiveState::NumberInput {
                state: TextInputState::Focused,
                value: 0.5,
                buffer: "0.5".into(),
                caret: 3,
                last_committed: 0.5,
                selection_anchor: None,
            },
        );
        store.link_slider_number(NodeId(1), NodeId(2));
        store.set_focus(Some(NodeId(2)));
        let arena = Bump::new();
        for _ in 0..3 {
            let _ = dispatch_key(&mut store, key(KEY_BACKSPACE, false), &arena);
        }
        for ch in ['9', '9'] {
            let _ = dispatch_text_input(&mut store, ch, &arena);
        }
        let _ = dispatch_key(&mut store, key(KEY_ENTER, false), &arena);
        let (_, sv) = store.slider(NodeId(1)).unwrap();
        assert!((sv - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn dropdown_click_toggles_open() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(1),
            InteractiveState::Dropdown {
                state: crate::widget::DropdownState::Normal,
                open: false,
                selected_index: None,
            },
        );
        let mut hits = HitIndex::new();
        hits.register(NodeId(1), Rect::new(0.0, 0.0, 100.0, 30.0));
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 15.0),
            &arena,
        );
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 50.0, 15.0),
            &arena,
        );
        let open_after_first = matches!(
            store.get(NodeId(1)),
            Some(InteractiveState::Dropdown { open: true, .. })
        );
        assert!(open_after_first);
        // Second click closes it.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 15.0),
            &arena,
        );
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 50.0, 15.0),
            &arena,
        );
        let open_after_second = matches!(
            store.get(NodeId(1)),
            Some(InteractiveState::Dropdown { open: true, .. })
        );
        assert!(!open_after_second);
    }

    #[test]
    fn escape_closes_open_dropdown_without_blur() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(1),
            InteractiveState::Dropdown {
                state: crate::widget::DropdownState::Normal,
                open: true,
                selected_index: None,
            },
        );
        store.set_focus(Some(NodeId(1)));
        let arena = Bump::new();
        let evts = dispatch_key(&mut store, key(KEY_ESCAPE, false), &arena);
        assert_eq!(evts, &[]); // closing the dropdown does not blur
        assert_eq!(store.focus_id(), Some(NodeId(1)));
        assert!(matches!(
            store.get(NodeId(1)),
            Some(InteractiveState::Dropdown { open: false, .. })
        ));
    }

    #[test]
    fn combobox_text_input_appends_to_query() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(1),
            InteractiveState::Combobox {
                state: crate::widget::ComboboxState::Normal,
                open: false,
                query: String::new(),
                caret: 0,
                selection_anchor: None,
            },
        );
        store.set_focus(Some(NodeId(1)));
        let arena = Bump::new();
        let _ = dispatch_text_input(&mut store, 's', &arena);
        let _ = dispatch_text_input(&mut store, 'p', &arena);
        assert_eq!(store.text(NodeId(1)), Some("sp"));
    }

    // -----------------------------------------------------------------
    // BlenderColorPicker sub-control dispatch (B4 fix)
    // -----------------------------------------------------------------

    fn blender_picker_setup() -> (WidgetStore, HitIndex) {
        use crate::interaction::BlenderHitKind;
        use crate::widget::{ChannelMode, InterpolationMode};
        use ph2d_tokens::ColorValue;

        let mut store = WidgetStore::with_capacity(32);
        store.register(
            NodeId(100),
            InteractiveState::BlenderPicker {
                value: ColorValue::from_rgba8(128, 64, 32, 255),
                channel_mode: ChannelMode::Rgb,
                interpolation: InterpolationMode::Perceptual,
                active_palette: 0,
                hsv_h: 0.07,
                hsv_s: 0.75,
            },
        );
        // Seed the picker's palette so swatch clicks have something
        // to read (the default 12 colors from `default_palette`).
        store.init_blender_palette(
            NodeId(100),
            crate::widget::default_palette().swatches.clone(),
        );
        // Channel slider shims (0..3 = R, G, B, A).
        for idx in 0u8..4 {
            store.register(
                NodeId(200 + idx as u64),
                InteractiveState::BlenderHit {
                    parent: NodeId(100),
                    kind: BlenderHitKind::ChannelSlider(idx),
                },
            );
        }
        // Interpolation toggle shims.
        store.register(
            NodeId(210),
            InteractiveState::BlenderHit {
                parent: NodeId(100),
                kind: BlenderHitKind::InterpolationLinear,
            },
        );
        store.register(
            NodeId(211),
            InteractiveState::BlenderHit {
                parent: NodeId(100),
                kind: BlenderHitKind::InterpolationPerceptual,
            },
        );
        // Channel mode toggle shims.
        store.register(
            NodeId(212),
            InteractiveState::BlenderHit {
                parent: NodeId(100),
                kind: BlenderHitKind::ChannelRgb,
            },
        );
        store.register(
            NodeId(213),
            InteractiveState::BlenderHit {
                parent: NodeId(100),
                kind: BlenderHitKind::ChannelHsv,
            },
        );
        // Palette swatch shims.
        for swatch in 0u8..4 {
            store.register(
                NodeId(220 + swatch as u64),
                InteractiveState::BlenderHit {
                    parent: NodeId(100),
                    kind: BlenderHitKind::PaletteSwatch(swatch),
                },
            );
        }
        let mut hits = HitIndex::new();
        // Channel slider track rects — painter now registers only the
        // inner track (no label/value chip), so x=0..110 covers the
        // interactive region directly.
        for idx in 0u8..4 {
            hits.register(
                NodeId(200 + idx as u64),
                Rect::new(0.0, idx as f32 * 30.0, 110.0, 22.0),
            );
        }
        // Toggle half-rects.
        hits.register(NodeId(210), Rect::new(0.0, 200.0, 100.0, 28.0));
        hits.register(NodeId(211), Rect::new(100.0, 200.0, 100.0, 28.0));
        hits.register(NodeId(212), Rect::new(0.0, 240.0, 100.0, 28.0));
        hits.register(NodeId(213), Rect::new(100.0, 240.0, 100.0, 28.0));
        // Swatch rects.
        for swatch in 0u8..4 {
            hits.register(
                NodeId(220 + swatch as u64),
                Rect::new(swatch as f32 * 30.0, 300.0, 24.0, 24.0),
            );
        }
        (store, hits)
    }

    #[test]
    fn channel_slider_down_mutates_red_channel() {
        let (mut store, hits) = blender_picker_setup();
        let arena = Bump::new();
        // Red slider track (NodeId 200) is x: 0..110. Click at x=55
        // (midpoint) → R ≈ 128 (0.5 * 255).
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 55.0, 11.0),
            &arena,
        );
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(id) if *id == NodeId(100))),
            "expected ValueChanged(100) from channel slider hit"
        );
        let (v, _, _, _) = store.blender_picker(NodeId(100)).unwrap();
        assert!(
            (v.rgba[0] as f32 / 255.0 - 0.5).abs() < 0.01,
            "red channel should be ≈128 (0.5 * 255), got {}",
            v.rgba[0]
        );
        // Other channels should be unchanged.
        assert_eq!(v.rgba[1], 64, "green channel unchanged");
        assert_eq!(v.rgba[2], 32, "blue channel unchanged");
    }

    #[test]
    fn channel_slider_down_mutates_alpha_channel() {
        let (mut store, hits) = blender_picker_setup();
        let arena = Bump::new();
        // Alpha slider (NodeId 203) rect is y offset at 90. Click at x=0 → A = 0.
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 0.0, 101.0),
            &arena,
        );
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(id) if *id == NodeId(100))),
            "expected ValueChanged(100) from alpha channel slider"
        );
        let (v, _, _, _) = store.blender_picker(NodeId(100)).unwrap();
        assert_eq!(v.rgba[3], 0, "alpha channel should be 0 after click at x=0");
    }

    #[test]
    fn interp_toggle_linear_switches_mode() {
        use crate::widget::InterpolationMode;
        let (mut store, hits) = blender_picker_setup();
        let arena = Bump::new();
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 50.0, 214.0),
            &arena,
        );
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(id) if *id == NodeId(100)))
        );
        let (_, _, interp, _) = store.blender_picker(NodeId(100)).unwrap();
        assert_eq!(interp, InterpolationMode::Linear);
    }

    #[test]
    fn channel_mode_toggle_hsv_switches_mode() {
        use crate::widget::ChannelMode;
        let (mut store, hits) = blender_picker_setup();
        let arena = Bump::new();
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 150.0, 254.0),
            &arena,
        );
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(id) if *id == NodeId(100)))
        );
        let (_, mode, _, _) = store.blender_picker(NodeId(100)).unwrap();
        assert_eq!(mode, ChannelMode::Hsv);
    }

    #[test]
    fn palette_swatch_click_changes_picker_value() {
        let (mut store, hits) = blender_picker_setup();
        let arena = Bump::new();
        // Click swatch 2 (NodeId 222), which maps to default_palette().swatches[2].
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 60.0 + 12.0, 312.0),
            &arena,
        );
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(id) if *id == NodeId(100))),
            "expected ValueChanged(100) from swatch click"
        );
        let (new_val, _, _, _) = store.blender_picker(NodeId(100)).unwrap();
        let expected = crate::widget::default_palette().swatches[2];
        assert_eq!(
            new_val.rgba, expected.rgba,
            "picker value should match swatch 2 of default palette"
        );
    }

    // ── Multi-line click mapping (TextArea) ────────────────────────────────

    fn textarea_setup(initial: &str) -> (WidgetStore, HitIndex, Rect) {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(42),
            InteractiveState::TextInput {
                state: crate::widget::TextInputState::Normal,
                text: initial.to_string(),
                caret: 0,
                selection_anchor: None,
            },
        );
        let rect = Rect::new(100.0, 200.0, 240.0, 60.0);
        let mut hits = HitIndex::new();
        hits.register(NodeId(42), rect);
        (store, hits, rect)
    }

    #[test]
    fn textarea_click_line2_places_caret_on_line2() {
        // Two lines: "abc" (3 bytes) + '\n' + "defgh" (5 bytes). Total 9.
        let (mut store, hits, rect) = textarea_setup("abc\ndefgh");
        let arena = Bump::new();
        // Click well into line 2's y range (line_h ~ 18, padding 8,
        // so y ≈ rect.y + 8 + 18 + 4 = rect.y + 30 hits line 2).
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, rect.x + 12.0 + 1.0, rect.y + 32.0),
            &arena,
        );
        let caret = match store.get(NodeId(42)) {
            Some(InteractiveState::TextInput { caret, .. }) => *caret,
            _ => 0,
        };
        // Line 2 starts at byte 4 (`abc` + '\n'). Caret at byte 4 means
        // start of line 2 — exactly what the user wants when clicking
        // near the left of line 2.
        assert!(
            (4..=9).contains(&caret),
            "expected caret on line 2 (>= byte 4), got {caret}"
        );
    }

    #[test]
    fn textarea_click_far_right_snaps_to_end_of_line() {
        // Line 1 is short ("abc"); clicking far right of line 1 must
        // not jump into line 2 — caret should land at byte 3 (end of
        // line 1).
        let (mut store, hits, rect) = textarea_setup("abc\ndefghijklmnop");
        let arena = Bump::new();
        // Click on line 1 (y ≈ rect.y + 12, inside first line band)
        // at the far right of the rect.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, rect.x + rect.w - 4.0, rect.y + 12.0),
            &arena,
        );
        let caret = match store.get(NodeId(42)) {
            Some(InteractiveState::TextInput { caret, .. }) => *caret,
            _ => 99,
        };
        assert_eq!(
            caret, 3,
            "click past end of line 1 must snap to end-of-line (byte 3), got {caret}"
        );
    }

    // ── Combobox clear-✕ button ────────────────────────────────────────────

    fn combobox_setup(initial_query: &str) -> (WidgetStore, HitIndex, Rect) {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(55),
            InteractiveState::Combobox {
                state: crate::widget::ComboboxState::Normal,
                open: false,
                query: initial_query.to_string(),
                caret: initial_query.len(),
                selection_anchor: None,
            },
        );
        let rect = Rect::new(50.0, 100.0, 240.0, 32.0);
        let mut hits = HitIndex::new();
        hits.register(NodeId(55), rect);
        (store, hits, rect)
    }

    #[test]
    fn combobox_clear_x_wipes_query_and_emits_text_changed() {
        let (mut store, hits, rect) = combobox_setup("spike");
        let arena = Bump::new();
        let probe = crate::widget::Combobox::new(NodeId(55), "", vec![]).query("spike");
        let xr = probe
            .clear_button_rect(rect)
            .expect("clear rect must exist");
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, xr.x + xr.w * 0.5, xr.y + xr.h * 0.5),
            &arena,
        );
        let q = match store.get(NodeId(55)) {
            Some(InteractiveState::Combobox { query, .. }) => query.clone(),
            _ => "<missing>".to_string(),
        };
        assert!(
            q.is_empty(),
            "expected empty query after X click, got {q:?}"
        );
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::TextChanged(id) if *id == NodeId(55))),
            "expected TextChanged(55) after clear, got {evts:?}"
        );
    }

    #[test]
    fn combobox_no_clear_x_when_query_empty() {
        // Clicking on the right side of an empty Combobox should not
        // mutate any state (no clear, no error). It just focuses +
        // places caret at 0.
        let (mut store, hits, rect) = combobox_setup("");
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(
                PointerKind::Down,
                rect.x + rect.w - 8.0,
                rect.y + rect.h * 0.5,
            ),
            &arena,
        );
        // Still empty.
        let q = match store.get(NodeId(55)) {
            Some(InteractiveState::Combobox { query, .. }) => query.clone(),
            _ => "<missing>".to_string(),
        };
        assert!(q.is_empty());
    }

    // ── NumberInput stepper buttons ────────────────────────────────────────

    fn number_input_setup(initial: f64) -> (WidgetStore, HitIndex, Rect) {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(77),
            InteractiveState::NumberInput {
                state: crate::widget::TextInputState::Normal,
                value: initial,
                buffer: super::super::format_number(initial),
                caret: 0,
                last_committed: initial,
                selection_anchor: None,
            },
        );
        let rect = Rect::new(0.0, 0.0, 80.0, 28.0);
        let mut hits = HitIndex::new();
        hits.register(NodeId(77), rect);
        (store, hits, rect)
    }

    #[test]
    fn number_input_up_arrow_increments_integer() {
        let (mut store, hits, rect) = number_input_setup(5.0);
        let arena = Bump::new();
        let probe = crate::widget::NumberInput::new(NodeId(77), "", 5.0);
        let up = probe.up_rect(rect);
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, up.x + up.w * 0.5, up.y + up.h * 0.5),
            &arena,
        );
        let v = match store.get(NodeId(77)) {
            Some(InteractiveState::NumberInput { value, .. }) => *value,
            _ => -1.0,
        };
        assert!((v - 6.0).abs() < f64::EPSILON, "expected 6.0 got {v}");
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(id) if *id == NodeId(77)))
        );
    }

    #[test]
    fn number_input_down_arrow_decrements_fractional_by_001() {
        // Buffer "0.50" contains '.', so the step heuristic picks 0.01.
        let (mut store, hits, rect) = number_input_setup(0.5);
        let arena = Bump::new();
        let probe = crate::widget::NumberInput::new(NodeId(77), "", 0.5);
        let down = probe.down_rect(rect);
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(
                PointerKind::Down,
                down.x + down.w * 0.5,
                down.y + down.h * 0.5,
            ),
            &arena,
        );
        let v = match store.get(NodeId(77)) {
            Some(InteractiveState::NumberInput { value, .. }) => *value,
            _ => -1.0,
        };
        assert!((v - 0.49).abs() < 1e-6, "expected 0.49 got {v}");
    }

    fn meta_key(kc: u32) -> KeyEvent {
        KeyEvent {
            keycode: kc,
            modifiers: Modifiers {
                shift: false,
                ctrl: false,
                alt: false,
                meta: true,
            },
            kind: KeyKind::Down,
            timestamp_ns: 0,
        }
    }

    fn focused_text_input(text: &str, caret: usize, anchor: Option<usize>) -> WidgetStore {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(50),
            InteractiveState::TextInput {
                state: crate::widget::TextInputState::Focused,
                text: text.to_string(),
                caret,
                selection_anchor: anchor,
            },
        );
        store.set_focus(Some(NodeId(50)));
        store
    }

    #[test]
    fn cmd_c_copies_selection_to_outbox() {
        let mut store = focused_text_input("hello world", 5, Some(0));
        let arena = Bump::new();
        let _ = dispatch_key(&mut store, meta_key(KEY_KEY_C), &arena);
        assert_eq!(store.take_clipboard_copy().as_deref(), Some("hello"));
    }

    #[test]
    fn cmd_c_without_selection_emits_nothing() {
        let mut store = focused_text_input("hello", 3, None);
        let arena = Bump::new();
        let _ = dispatch_key(&mut store, meta_key(KEY_KEY_C), &arena);
        assert!(store.take_clipboard_copy().is_none());
    }

    #[test]
    fn cmd_x_cuts_selection_and_emits_text_changed() {
        let mut store = focused_text_input("hello world", 11, Some(5));
        let arena = Bump::new();
        let evts = dispatch_key(&mut store, meta_key(KEY_KEY_X), &arena);
        assert_eq!(store.take_clipboard_copy().as_deref(), Some(" world"));
        match store.get(NodeId(50)) {
            Some(InteractiveState::TextInput { text, caret, .. }) => {
                assert_eq!(text, "hello");
                assert_eq!(*caret, 5);
            }
            _ => panic!("expected TextInput"),
        }
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::TextChanged(_)))
        );
    }

    #[test]
    fn cmd_v_sets_paste_request() {
        let mut store = focused_text_input("abc", 3, None);
        let arena = Bump::new();
        let _ = dispatch_key(&mut store, meta_key(KEY_KEY_V), &arena);
        assert_eq!(store.take_clipboard_paste_request(), Some(NodeId(50)));
    }

    #[test]
    fn apply_clipboard_paste_inserts_at_caret() {
        let mut store = focused_text_input("abxy", 2, None);
        let ok = apply_clipboard_paste(&mut store, NodeId(50), "cd");
        assert!(ok);
        match store.get(NodeId(50)) {
            Some(InteractiveState::TextInput { text, caret, .. }) => {
                assert_eq!(text, "abcdxy");
                assert_eq!(*caret, 4);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn apply_clipboard_paste_replaces_selection() {
        let mut store = focused_text_input("hello world", 5, Some(0));
        apply_clipboard_paste(&mut store, NodeId(50), "Hi");
        match store.get(NodeId(50)) {
            Some(InteractiveState::TextInput { text, caret, .. }) => {
                assert_eq!(text, "Hi world");
                assert_eq!(*caret, 2);
            }
            _ => panic!(),
        }
    }
}
