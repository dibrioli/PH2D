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

mod blender;
pub mod clipboard;
mod focus;
pub mod hierarchy;
mod hover;
pub mod keymap;
mod number_input;
pub mod scroll;
mod text_ops;

use blender::{apply_blender_channel_value, apply_blender_hit, derive_blender_channel_value};
pub use clipboard::apply_clipboard_paste;
use clipboard::{clipboard_extract_selection, collapse_selection, delete_selection_if_any};
use focus::{apply_click, cycle_focus, is_focusable};
pub(crate) use hierarchy::HierDrop;
use hierarchy::find_hierarchy_drop;
use hover::{set_widget_pressed, set_widget_released, update_hover};
pub use keymap::{
    KEY_ARROW_DOWN, KEY_ARROW_LEFT, KEY_ARROW_RIGHT, KEY_ARROW_UP, KEY_BACKSPACE, KEY_ENTER,
    KEY_ESCAPE, KEY_KEY_A, KEY_KEY_C, KEY_KEY_V, KEY_KEY_X, KEY_SPACE, KEY_TAB,
};
use number_input::{apply_number_stepper_if_hit, is_numeric_input_char, update_drag_value};
pub use scroll::dispatch_wheel;
use scroll::scrollbar_panel_for_id;
use text_ops::{
    byte_offset_from_click_xy, next_char_boundary, place_text_caret, prev_char_boundary,
};

use super::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::zones::Rect;
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;
use ph2d_host::{KeyEvent, KeyKind, PointerEvent, PointerKind};
use ph2d_text::TextSystem;

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
            // M14.A: NumberInput drag-or-slider. When a Down on the
            // NumberInput body seeded `number_input_drag`, every Move
            // first checks distance against the threshold; once
            // crossed, the field switches to slider mode and the
            // delta is computed Blender-style with **axis lock**:
            //   - At the moment the threshold flips, compare
            //     `|total_dx|` vs `|total_dy|` and lock the dominant
            //     axis on the drag state. The lock STAYS for the
            //     rest of the drag — a new click (fresh Down) is the
            //     only way to reset the axis. This stops late-drag
            //     wobble on the off-axis from contaminating the
            //     scrub when the user committed to one direction.
            //   - Horizontal locked: 50 step-units / px (fast).
            //   - Vertical locked (up = +, down = -): 5 step-units / px (slow).
            //   - Shift held: multiply delta by 0.001 (fine).
            // The painter reads `value` + `buffer` from the store —
            // we mutate both directly here so the focused field's
            // displayed text refreshes in real time during the drag.
            // (Using `set_number_value` would skip the buffer rewrite
            // because the field IS focused: Down → focus + buffer
            // seed → drag begins; the focus-guard would keep the
            // pre-drag buffer visible.)
            let mut number_input_drag_consumed = None;
            if let Some(drag) = store.number_input_drag() {
                let dx_total = event.x - drag.start_x;
                let dy_total = event.y - drag.start_y;
                if !drag.crossed_threshold {
                    let dist_sq = dx_total * dx_total + dy_total * dy_total;
                    let thr = super::drag::NUMBER_INPUT_DRAG_THRESHOLD_PX;
                    if dist_sq >= thr * thr {
                        // Decide the locked axis at THIS Move based
                        // on which delta is larger. `>=` so a perfect
                        // 45° diagonal defaults to horizontal (the
                        // primary scrub axis).
                        let horizontal = dx_total.abs() >= dy_total.abs();
                        store.promote_number_input_drag_to_slider(horizontal);
                    }
                }
                // Re-read after the potential promotion.
                if let Some(d) = store.number_input_drag()
                    && d.crossed_threshold
                {
                    // Use the LOCKED axis (decided at promotion). The
                    // other axis is zeroed unconditionally — its
                    // delta is not consulted again until the drag
                    // ends.
                    let (dom_dx, dom_dy) = if d.axis_horizontal {
                        (dx_total, 0.0)
                    } else {
                        (0.0, dy_total)
                    };
                    let shift_mul = if store.shift_held() {
                        super::drag::DRAG_SHIFT_MUL
                    } else {
                        1.0
                    };
                    let delta = (dom_dx as f64 * super::drag::DRAG_RATE_X
                        - dom_dy as f64 * super::drag::DRAG_RATE_Y)
                        * shift_mul;
                    let new_value = d.start_value + delta * d.step;
                    // Audit fix #2 (CRITICAL): mutate `value` and
                    // `buffer` for live display, but DO NOT touch
                    // `last_committed` — that anchor must keep
                    // pointing at the pre-drag value so Esc can
                    // revert. The Up handler commits
                    // `last_committed = new_value` on a successful
                    // drag release.
                    if let Some(InteractiveState::NumberInput { value, buffer, .. }) =
                        store.get_mut(d.id)
                    {
                        *value = new_value;
                        *buffer = super::format_number(new_value);
                    }
                    // Mirror to a linked slider if any (same pattern
                    // as `apply_number_stepper_if_hit`).
                    if let Some(slider_id) = store.linked_slider(d.id)
                        && let Some(InteractiveState::Slider { value, .. }) =
                            store.get_mut(slider_id)
                    {
                        *value = (new_value as f32).clamp(0.0, 1.0);
                    }
                    events.push(WidgetEvent::ValueChanged(d.id));
                    number_input_drag_consumed = Some(d.id);
                }
            }
            if let Some(active) = store.active_id() {
                if let Some(rect) = store.active_rect() {
                    // Text drag-to-select: extend the selection from
                    // the anchor (set on Down) to the new cursor x.
                    // Skipped when this widget is in NumberInput
                    // slider mode (drag past threshold) — the slider
                    // owns the gesture; falling through to text-drag-
                    // select would also extend the selection while
                    // the user is scrubbing the value.
                    if matches!(
                        store.get(active),
                        Some(InteractiveState::TextInput { .. })
                            | Some(InteractiveState::NumberInput { .. })
                            | Some(InteractiveState::Combobox { .. })
                    ) && number_input_drag_consumed != Some(active)
                    {
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
                // M14.6 F: right-click on a hierarchy row opens the
                // per-entity actions menu. Resolved BEFORE the broader
                // panel-under fallback because the row lives inside
                // the hierarchy panel — the CreateNote menu must not
                // win over this more specific kind. Eye/chevron
                // companion ids are stripped first so a right-click on
                // those toggles still reaches the parent row.
                let hier_row_id = hit_id.and_then(|id| {
                    if let Some(row) = crate::screens::hero::ids::hier_eye_companion_to_row(id) {
                        Some(row)
                    } else if let Some(row) =
                        crate::screens::hero::ids::hier_expand_companion_to_row(id)
                    {
                        Some(row)
                    } else {
                        Some(id)
                    }
                    .filter(|row| store.is_hierarchy_row(*row))
                });
                if let Some(row) = hier_row_id {
                    store.open_context_menu(super::ContextMenuRequest {
                        x: event.x,
                        y: event.y,
                        kind: super::ContextMenuKind::HierarchyRow { row },
                    });
                } else if let Some(note_index) = note_slot
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
            // Same pattern for the Save chip — Primary opens the
            // Save / Save As menu anchored below the chip.
            if event.button == ph2d_host::PointerButton::Primary
                && let Some((hit_id, hit_rect)) = hit
                && hit_id == crate::screens::hero::ids::TOPBAR_SAVE
                && matches!(store.get(hit_id), Some(InteractiveState::Button { .. }))
            {
                store.open_context_menu(super::ContextMenuRequest {
                    x: hit_rect.x,
                    y: hit_rect.y + hit_rect.h + 4.0,
                    kind: super::ContextMenuKind::SaveMenu,
                });
                return events.into_bump_slice();
            }
            // Open chip — same anchor logic.
            if event.button == ph2d_host::PointerButton::Primary
                && let Some((hit_id, hit_rect)) = hit
                && hit_id == crate::screens::hero::ids::TOPBAR_OPEN
                && matches!(store.get(hit_id), Some(InteractiveState::Button { .. }))
            {
                store.open_context_menu(super::ContextMenuRequest {
                    x: hit_rect.x,
                    y: hit_rect.y + hit_rect.h + 4.0,
                    kind: super::ContextMenuKind::OpenMenu,
                });
                return events.into_bump_slice();
            }
            // Settings cluster (gear) — opens the SettingsMenu with
            // px/m presets. Same anchor convention as Save/Open.
            if event.button == ph2d_host::PointerButton::Primary
                && let Some((hit_id, hit_rect)) = hit
                && hit_id == crate::screens::hero::ids::TOPBAR_SETTINGS
                && matches!(store.get(hit_id), Some(InteractiveState::Button { .. }))
            {
                store.open_context_menu(super::ContextMenuRequest {
                    x: hit_rect.x,
                    y: hit_rect.y + hit_rect.h + 4.0,
                    kind: super::ContextMenuKind::SettingsMenu,
                });
                return events.into_bump_slice();
            }
            // Project chip → SceneList popover (search + scenes).
            if event.button == ph2d_host::PointerButton::Primary
                && let Some((hit_id, hit_rect)) = hit
                && hit_id == crate::screens::hero::ids::TOPBAR_PROJECT
                && matches!(store.get(hit_id), Some(InteractiveState::Button { .. }))
            {
                store.open_context_menu(super::ContextMenuRequest {
                    x: hit_rect.x,
                    y: hit_rect.y + hit_rect.h + 4.0,
                    kind: super::ContextMenuKind::SceneList,
                });
                return events.into_bump_slice();
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
                    Some(super::ContextMenuKind::SceneList)
                ) && matches!(
                    hit_id,
                    Some(id) if id == crate::screens::hero::ids::CTX_SCENE_SEARCH
                        || crate::screens::hero::ids::CTX_SCENE_ROWS.contains(&id)
                );
                if !inside_scene_list {
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

            // Bump the clicked panel to the top of the z-order so it
            // paints over its siblings. Iterate the canonical panel
            // ids and pick whichever one's published rect contains
            // the click. The picker takes precedence because it's the
            // only floating panel and visually overlaps the others
            // when displayed.
            {
                use crate::screens::hero::ids as hero_ids;
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

            // Hierarchy chrome companions (eye toggle, chevron toggle)
            // are registered in HitIndex but NOT in WidgetStore — the
            // painter has no &mut WidgetStore. The `is_focusable` gate
            // below would reject them. Capture them as ephemeral
            // buttons: set active+rect on Down so the Up branch fires
            // `apply_click`, whose `_` fallthrough pushes a generic
            // `WidgetEvent::Click(id)` for unregistered ids. Hero's
            // `apply_event` then routes by companion bit pattern.
            if let Some((id, rect)) = hit
                && (crate::screens::hero::ids::hier_eye_companion_to_row(id).is_some()
                    || crate::screens::hero::ids::hier_expand_companion_to_row(id).is_some())
            {
                store.set_active(Some(id));
                store.set_active_rect(Some(rect));
                return events.into_bump_slice();
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
                let stepper_hit = !combo_cleared
                    && apply_number_stepper_if_hit(
                        store,
                        id,
                        rect,
                        event.x,
                        event.y,
                        event.timestamp_ns,
                    );
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
                    let drag = super::drag::NumberInputDragState {
                        id,
                        start_x: event.x,
                        start_y: event.y,
                        start_value: *value,
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
                //
                // Pre-M14.6B: the criterion was `is_hierarchy_entity_id`
                // (fixture range 400..=411), which silently dropped
                // every live ECS-bridge row (ids start at 100_000+).
                // Now we ask the store, which the hierarchy painter
                // updates per-frame in both modes.
                if store.is_hierarchy_row(id) {
                    store.begin_hierarchy_drag(id, event.x, event.y, event.timestamp_ns);
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
                    >= super::LONG_PRESS_THRESHOLD_NS
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
                        event.y >= r.y
                            && event.y < r.y + r.h
                            && event.x >= r.x
                            && event.x < r.x + r.w
                    })
                    .unwrap_or(false);
                if over_self {
                    return events.into_bump_slice();
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
                if still_hot
                    && !is_drag_widget
                    && !suppress_click
                    && event.button == ph2d_host::PointerButton::Primary
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
                // M14.7 polish: Enter on the rename TextInput
                // commits via `WidgetEvent::Submit` instead of
                // inserting a newline. Caller (hero apply_event)
                // reads the buffer and applies the rename.
                if event.keycode == KEY_ENTER
                    && id == crate::screens::hero::ids::HIER_RENAME_INPUT
                    && matches!(store.get(id), Some(InteractiveState::TextInput { .. }))
                {
                    if let Some(InteractiveState::TextInput { state, .. }) = store.get_mut(id) {
                        *state = crate::widget::TextInputState::Normal;
                    }
                    store.set_focus(None);
                    events.push(WidgetEvent::Submit(id));
                    events.push(WidgetEvent::Blur(id));
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
            // Audit fix #1 (CRITICAL): Esc must also abort any
            // in-flight NumberInput drag-slider OR stepper-hold,
            // regardless of focus. Without this the drag state stays
            // armed; the next Move would continue advancing
            // `last_committed` from a stale `start_value`, and Esc
            // (which is supposed to revert) would no longer work.
            // Cleared unconditionally — these `end_*` calls are no-ops
            // when nothing is in flight.
            let _ = store.end_number_input_drag();
            store.end_number_stepper_hold();
            if let Some(id) = store.focus_id() {
                // Dropdowns close on ESC instead of losing focus.
                if let Some(InteractiveState::Dropdown { open, .. }) = store.get_mut(id)
                    && *open
                {
                    *open = false;
                    return events.into_bump_slice();
                }
                // M14.7 polish: Esc on the rename TextInput emits
                // `Cancel` so hero can drop the rename mode without
                // committing.
                if id == crate::screens::hero::ids::HIER_RENAME_INPUT
                    && matches!(store.get(id), Some(InteractiveState::TextInput { .. }))
                {
                    if let Some(InteractiveState::TextInput { state, .. }) = store.get_mut(id) {
                        *state = crate::widget::TextInputState::Normal;
                    }
                    store.set_focus(None);
                    events.push(WidgetEvent::Cancel(id));
                    events.push(WidgetEvent::Blur(id));
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

/// M14.A: drive the continuous-hold repeat on a NumberInput stepper
/// arrow. The shell calls this once per frame with the current host
/// timestamp. After the initial 250 ms delay since Down, the function
/// fires one increment / decrement every 30 ms while the hold stays
/// active. Returns the slice of `WidgetEvent::ValueChanged` events
/// that fired this tick (zero-allocation via the bumpalo arena).
///
/// The Down event itself counts as the first tick (`apply_number_stepper_if_hit`
/// already applied the increment); `dispatch_tick` only handles the
/// repeats after the initial delay. The hold is cleared on Up
/// (see `PointerKind::Up` in `dispatch_pointer`).
pub fn dispatch_tick<'frame>(
    arena: &'frame Bump,
    store: &mut WidgetStore,
    now_ns: u128,
) -> &'frame [WidgetEvent] {
    let mut events = BumpVec::new_in(arena);
    let hold = match store.number_stepper_hold() {
        Some(h) => h,
        None => return events.into_bump_slice(),
    };
    // Initial delay: wait `STEPPER_HOLD_INITIAL_DELAY_NS` after the
    // press before the first repeat tick fires (matches macOS Aqua).
    if now_ns.saturating_sub(hold.press_ns) < super::drag::STEPPER_HOLD_INITIAL_DELAY_NS {
        return events.into_bump_slice();
    }
    // After the initial delay, gate by the repeat interval.
    if now_ns.saturating_sub(hold.last_tick_ns) < super::drag::STEPPER_REPEAT_INTERVAL_NS {
        return events.into_bump_slice();
    }
    let new_value = match store.get(hold.id) {
        Some(InteractiveState::NumberInput { value, .. }) => *value + hold.direction * hold.step,
        _ => {
            // Widget vanished mid-hold (e.g. selection switched and
            // the field was force-rewritten). Clear the hold so we
            // stop ticking against a non-existent target.
            store.end_number_stepper_hold();
            return events.into_bump_slice();
        }
    };
    if let Some(InteractiveState::NumberInput {
        value,
        buffer,
        last_committed,
        ..
    }) = store.get_mut(hold.id)
    {
        *value = new_value;
        *buffer = super::format_number(new_value);
        *last_committed = new_value;
    }
    if let Some(slider_id) = store.linked_slider(hold.id)
        && let Some(InteractiveState::Slider { value, .. }) = store.get_mut(slider_id)
    {
        *value = (new_value as f32).clamp(0.0, 1.0);
    }
    store.record_number_stepper_tick(now_ns);
    events.push(WidgetEvent::ValueChanged(hold.id));
    events.into_bump_slice()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::InteractiveState;
    use crate::widget::{
        ButtonState, CheckboxState, CheckboxValue, SliderOrientation, SliderState, ToggleState,
    };
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
    fn hierarchy_eye_companion_click_emits_click_event() {
        // Regression: companion NodeIds for the hierarchy eye-toggle
        // (and chevron) are registered in HitIndex only — never in
        // WidgetStore (the painter has no &mut store). Before the
        // M14.6A bugfix, the `is_focusable` gate in PointerKind::Down
        // rejected unregistered ids, so no `active` was captured and
        // Up emitted nothing. Now the dispatcher special-cases these
        // companions and routes them through the regular Up→Click
        // path; this test pins that behavior.
        use crate::screens::hero::ids;
        let mut store = WidgetStore::with_capacity(4);
        // Simulate a live hierarchy row (registered as Plain by
        // `hierarchy::populate_live`); only the companion is missing
        // from the store — which is the realistic scenario.
        let row_id = ph2d_a11y::NodeId(412);
        store.register(row_id, InteractiveState::Plain);
        let eye_id = ids::hier_eye_companion(row_id);
        let mut hits = HitIndex::new();
        hits.register(row_id, Rect::new(0.0, 0.0, 200.0, 20.0));
        hits.register(eye_id, Rect::new(170.0, 0.0, 24.0, 20.0));
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 182.0, 10.0),
            &arena,
        );
        // Active must be set even though the companion isn't in store.
        assert_eq!(store.active_id(), Some(eye_id));
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 182.0, 10.0),
            &arena,
        );
        assert_eq!(evts, &[WidgetEvent::Click(eye_id)]);
        assert_eq!(store.active_id(), None);
    }

    #[test]
    fn hierarchy_expand_companion_click_emits_click_event() {
        // Same contract as the eye test above, for the chevron
        // companion (collapse/expand). Lives separately so a
        // regression on one toggle bit doesn't silently break both.
        use crate::screens::hero::ids;
        let mut store = WidgetStore::with_capacity(4);
        let row_id = ph2d_a11y::NodeId(413);
        store.register(row_id, InteractiveState::Plain);
        let chev_id = ids::hier_expand_companion(row_id);
        let mut hits = HitIndex::new();
        hits.register(row_id, Rect::new(0.0, 0.0, 200.0, 20.0));
        hits.register(chev_id, Rect::new(4.0, 4.0, 12.0, 12.0));
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 10.0, 10.0),
            &arena,
        );
        assert_eq!(store.active_id(), Some(chev_id));
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 10.0, 10.0),
            &arena,
        );
        assert_eq!(evts, &[WidgetEvent::Click(chev_id)]);
    }

    #[test]
    fn hierarchy_drag_in_live_mode_emits_reparent_intent() {
        // Pre-M14.6B regression: dragging a live (ECS-bridge) row
        // used `is_hierarchy_entity_id` which only matched the
        // fixture range 400..=411 — so live rows (NodeIds in the
        // 100_000+ range) never became drag candidates and Up
        // emitted no `HierReparent`. This test pins the new
        // contract: the row set published via
        // `set_hierarchy_row_ids` is the single source of truth.
        let mut store = WidgetStore::with_capacity(8);
        // Two "live" rows from the bridge — far outside the
        // fixture range that the old code looked at.
        let parent_id = ph2d_a11y::NodeId(100_000);
        let dragged_id = ph2d_a11y::NodeId(100_001);
        store.register(parent_id, InteractiveState::Plain);
        store.register(dragged_id, InteractiveState::Plain);
        let mut row_set = std::collections::BTreeSet::new();
        row_set.insert(parent_id);
        row_set.insert(dragged_id);
        store.set_hierarchy_row_ids(row_set);
        store.set_hierarchy_order(vec![parent_id, dragged_id]);
        let mut hits = HitIndex::new();
        // Parent row at y=0..20, dragged row at y=30..50.
        hits.register(parent_id, Rect::new(0.0, 0.0, 200.0, 20.0));
        hits.register(dragged_id, Rect::new(0.0, 30.0, 200.0, 20.0));
        let arena = Bump::new();
        // Down on dragged row.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, 100.0, 40.0),
            &arena,
        );
        // Move enough to cross the drag threshold (8 px in any axis;
        // bumping the cursor 50 px up clears it comfortably).
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, 100.0, 10.0),
            &arena,
        );
        // Up over the middle of the parent row → HierDrop::Inside,
        // which emits HierReparent { dragged, new_parent: Some(parent), before: None }.
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, 100.0, 10.0),
            &arena,
        );
        assert!(
            evts.iter().any(|e| matches!(
                e,
                WidgetEvent::HierReparent {
                    dragged,
                    new_parent: Some(np),
                    before: None,
                    after: _,
                } if *dragged == dragged_id && *np == parent_id
            )),
            "expected HierReparent Inside({parent_id:?}); got {evts:?}"
        );
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

    /// Read NumberInput value via the store accessor — avoids
    /// boilerplate in the M14.A drag tests below.
    fn read_value(store: &WidgetStore, id: NodeId) -> f64 {
        store.number_value(id).expect("NumberInput value")
    }

    /// M14.A: Down on the body (NOT the stepper) seeds a drag
    /// candidate. Move right past the threshold flips into slider
    /// mode with the horizontal rate (50× step / px) — fast.
    #[test]
    fn number_input_body_drag_horizontal_uses_fast_rate() {
        let (mut store, hits, rect) = number_input_setup(5.0);
        let arena = Bump::new();
        // Body click — anywhere left of the up/down rects on the right
        // edge. (rect.x + 10) puts us comfortably inside the body.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, rect.x + 10.0, rect.y + rect.h * 0.5),
            &arena,
        );
        // Move right 10 px → dx=10, dy=0 → delta = 10 * 50 * step = 500.
        // (Step is 1.0 for buffer "5" — no decimal.)
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, rect.x + 20.0, rect.y + rect.h * 0.5),
            &arena,
        );
        let v = read_value(&store, NodeId(77));
        assert!(
            (v - 505.0).abs() < 1e-6,
            "expected 505.0 (5 + 10*50*1) got {v}"
        );
    }

    /// M14.A: vertical drag uses the slow rate (5× step / px) and
    /// inverts dy so cursor-up = positive delta (screen coords have
    /// y growing down).
    #[test]
    fn number_input_body_drag_vertical_uses_slow_rate_and_inverts() {
        let (mut store, hits, rect) = number_input_setup(0.0);
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, rect.x + 10.0, rect.y + rect.h * 0.5),
            &arena,
        );
        // Move up 10 px → dx=0, dy=-10 → delta = (0 - (-10) * 5) * 1 = 50.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(
                PointerKind::Move,
                rect.x + 10.0,
                rect.y + rect.h * 0.5 - 10.0,
            ),
            &arena,
        );
        let v = read_value(&store, NodeId(77));
        assert!((v - 50.0).abs() < 1e-6, "expected 50 (0 + 10*5*1) got {v}");
    }

    /// M14.A: holding Shift multiplies the delta by 0.001 — Blender-
    /// style fine adjustment. With horizontal 50× × 0.001 = 0.05 / px,
    /// a 10 px drag yields 0.5 step-units of change.
    #[test]
    fn number_input_body_drag_with_shift_uses_fine_rate() {
        let (mut store, hits, rect) = number_input_setup(0.0);
        let arena = Bump::new();
        store.set_shift_held(true);
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, rect.x + 10.0, rect.y + rect.h * 0.5),
            &arena,
        );
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, rect.x + 20.0, rect.y + rect.h * 0.5),
            &arena,
        );
        let v = read_value(&store, NodeId(77));
        assert!(
            (v - 0.5).abs() < 1e-6,
            "expected 0.5 (10*50*0.001*1) got {v}"
        );
    }

    /// M14.A: the axis lock survives off-axis wobble after the
    /// threshold cross. User crosses with dx > dy → horizontal locks
    /// → subsequent drift into the vertical direction is ignored,
    /// because the only way to release the axis is a fresh Down.
    #[test]
    fn number_input_body_drag_locked_axis_persists_through_off_axis_wobble() {
        let (mut store, hits, rect) = number_input_setup(0.0);
        let arena = Bump::new();
        let down_x = rect.x + 10.0;
        let down_y = rect.y + rect.h * 0.5;
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, down_x, down_y),
            &arena,
        );
        // Move horizontally past the 4 px threshold → horizontal
        // axis locks. dx=5 → delta = 5*50 = 250.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, down_x + 5.0, down_y),
            &arena,
        );
        assert!((read_value(&store, NodeId(77)) - 250.0).abs() < 1e-6);
        // Now drift vertically a lot (dy=86 >> dx=5). Without the
        // lock, vertical would dominate → delta = -(-86)*5 = 430 → value 430.
        // With the lock, horizontal stays active → value still 250.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, down_x + 5.0, down_y + 86.0),
            &arena,
        );
        let v = read_value(&store, NodeId(77));
        assert!(
            (v - 250.0).abs() < 1e-6,
            "horizontal axis lock leaked: expected 250.0 got {v}"
        );
        // Up clears the drag (and the lock). A new Down + drag would
        // pick a fresh axis based on its own first-move dominance.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, down_x + 5.0, down_y + 86.0),
            &arena,
        );
        assert!(store.number_input_drag().is_none());
    }

    /// M14.A: at the moment the threshold flips, the dominant axis
    /// is decided and locked on the drag state. A drag that's
    /// predominantly horizontal (dx 20, dy 5) ignores the dy
    /// contribution; the formula uses dx only.
    #[test]
    fn number_input_body_drag_locks_to_dominant_axis() {
        let (mut store, hits, rect) = number_input_setup(0.0);
        let arena = Bump::new();
        let down_x = rect.x + 10.0;
        let down_y = rect.y + rect.h * 0.5;
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, down_x, down_y),
            &arena,
        );
        // Horizontal-dominant: dx=20, dy=5. Without axis-lock:
        //   delta = (20*50 - 5*5) * 1 = 1000 - 25 = 975
        // With axis-lock: dy is zeroed, delta = 20*50 = 1000.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, down_x + 20.0, down_y + 5.0),
            &arena,
        );
        let v = read_value(&store, NodeId(77));
        assert!(
            (v - 1000.0).abs() < 1e-6,
            "horizontal-dominant axis lock failed: expected 1000.0 got {v}"
        );
    }

    /// M14.A: during a drag-slider the displayed text in the field
    /// MUST refresh every Move — not just `value`, but the `buffer`
    /// that the focused-state painter renders. (Bypass the
    /// `set_number_value` focus-guard via direct mutation.)
    #[test]
    fn number_input_body_drag_refreshes_buffer_in_realtime() {
        let (mut store, hits, rect) = number_input_setup(0.0);
        let arena = Bump::new();
        let down_x = rect.x + 10.0;
        let down_y = rect.y + rect.h * 0.5;
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, down_x, down_y),
            &arena,
        );
        // Move right past threshold.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, down_x + 10.0, down_y),
            &arena,
        );
        // Buffer must mirror the new value, not the start value's
        // formatted form ("0").
        let buffer = store.text(NodeId(77)).unwrap_or("").to_string();
        assert_eq!(
            buffer, "500",
            "buffer must refresh during drag-slider; got {buffer:?}"
        );
    }

    /// M14.A: Down + Up at (almost) the same position never crosses
    /// the threshold → no ValueChanged, drag state cleared, focus is
    /// retained from Down (edit mode = click→type behavior preserved).
    #[test]
    fn number_input_body_click_without_drag_preserves_edit_mode() {
        let (mut store, hits, rect) = number_input_setup(3.0);
        let arena = Bump::new();
        let down_x = rect.x + 10.0;
        let down_y = rect.y + rect.h * 0.5;
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, down_x, down_y),
            &arena,
        );
        // Move 2 px (< 4 px threshold) → drag stays pending.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, down_x + 2.0, down_y),
            &arena,
        );
        // Up — drag never crossed; edit mode stays active.
        let evts = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, down_x + 2.0, down_y),
            &arena,
        );
        assert!(
            !evts
                .iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(_))),
            "no-drag click must not emit ValueChanged"
        );
        // Focus remained on the field (placed at Down by the existing
        // text-widget pathway). Drag candidate cleared.
        assert_eq!(store.focus_id(), Some(NodeId(77)));
        assert!(store.number_input_drag().is_none());
        // Value unchanged.
        assert!((read_value(&store, NodeId(77)) - 3.0).abs() < 1e-6);
    }

    /// M14.A: continuous-hold on the up arrow. Down fires the first
    /// increment (already covered by `number_input_up_arrow_increments_integer`).
    /// `dispatch_tick` skips while inside the initial 250 ms delay,
    /// then fires repeats every 30 ms.
    #[test]
    fn number_stepper_hold_repeats_after_initial_delay() {
        use crate::interaction::drag::{STEPPER_HOLD_INITIAL_DELAY_NS, STEPPER_REPEAT_INTERVAL_NS};
        let (mut store, hits, rect) = number_input_setup(10.0);
        let arena = Bump::new();
        let probe = crate::widget::NumberInput::new(NodeId(77), "", 10.0);
        let up = probe.up_rect(rect);
        // Down at t=0 ns — first tick fires (10 → 11) via apply_number_stepper_if_hit.
        let mut down_evt = pointer(PointerKind::Down, up.x + up.w * 0.5, up.y + up.h * 0.5);
        down_evt.timestamp_ns = 0;
        let _ = dispatch_pointer(&mut store, &hits, down_evt, &arena);
        assert!((read_value(&store, NodeId(77)) - 11.0).abs() < f64::EPSILON);
        // Tick at 100 ms — still inside the initial delay; nothing.
        let evts = dispatch_tick(&arena, &mut store, 100_000_000);
        assert!(evts.is_empty(), "no repeat inside initial delay");
        // Tick at 300 ms — past the delay → one repeat fires (11 → 12).
        let evts = dispatch_tick(
            &arena,
            &mut store,
            STEPPER_HOLD_INITIAL_DELAY_NS + 50_000_000,
        );
        assert_eq!(evts.len(), 1);
        assert!((read_value(&store, NodeId(77)) - 12.0).abs() < f64::EPSILON);
        // Another tick 50 ms later (> 30 ms repeat) → second repeat (12 → 13).
        let evts = dispatch_tick(
            &arena,
            &mut store,
            STEPPER_HOLD_INITIAL_DELAY_NS + 50_000_000 + STEPPER_REPEAT_INTERVAL_NS + 5_000_000,
        );
        assert_eq!(evts.len(), 1);
        assert!((read_value(&store, NodeId(77)) - 13.0).abs() < f64::EPSILON);
    }

    /// M14.A audit fix #1 (CRITICAL): Esc mid-drag must abort the
    /// in-flight `number_input_drag` and `number_stepper_hold`. Old
    /// behavior: Esc reverted the buffer but the drag stayed armed,
    /// so the next Move would continue overwriting the value from a
    /// stale `start_value`. This regression test pins the new
    /// invariant.
    #[test]
    fn esc_clears_in_flight_number_input_drag() {
        let (mut store, hits, rect) = number_input_setup(0.0);
        let arena = Bump::new();
        // Down on body — drag candidate seeded (focus also lands).
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, rect.x + 10.0, rect.y + rect.h * 0.5),
            &arena,
        );
        // Move past 4 px threshold — drag promoted to slider.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, rect.x + 30.0, rect.y + rect.h * 0.5),
            &arena,
        );
        assert!(store.number_input_drag().is_some(), "drag armed before Esc");
        // Esc clears it.
        let _ = dispatch_key(&mut store, key(KEY_ESCAPE, false), &arena);
        assert!(
            store.number_input_drag().is_none(),
            "Esc must clear in-flight drag"
        );
        assert!(
            store.number_stepper_hold().is_none(),
            "Esc must also clear any stepper hold"
        );
    }

    /// M14.A audit fix #2 (CRITICAL): while the drag-slider is
    /// scrubbing, `last_committed` must stay anchored on the
    /// pre-Down value so Esc rollback works. Only the Up commit
    /// updates `last_committed`. Old behavior overwrote it on every
    /// Move and silently destroyed the rollback target.
    #[test]
    fn drag_slider_last_committed_anchors_until_up_commits() {
        let (mut store, hits, rect) = number_input_setup(7.0);
        let arena = Bump::new();
        // Down + Move past threshold → drag in flight.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, rect.x + 10.0, rect.y + rect.h * 0.5),
            &arena,
        );
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, rect.x + 30.0, rect.y + rect.h * 0.5),
            &arena,
        );
        match store.get(NodeId(77)) {
            Some(InteractiveState::NumberInput {
                value,
                last_committed,
                ..
            }) => {
                assert!(
                    (*last_committed - 7.0).abs() < f64::EPSILON,
                    "last_committed must stay at the pre-drag value during Move, got {last_committed}"
                );
                assert!(
                    (*value - 7.0).abs() > f64::EPSILON,
                    "value should already have moved during drag"
                );
            }
            _ => panic!("expected NumberInput state"),
        }
        // Up commits — last_committed now matches value.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, rect.x + 30.0, rect.y + rect.h * 0.5),
            &arena,
        );
        match store.get(NodeId(77)) {
            Some(InteractiveState::NumberInput {
                value,
                last_committed,
                ..
            }) => {
                assert!(
                    (*last_committed - *value).abs() < f64::EPSILON,
                    "Up must commit last_committed = value"
                );
            }
            _ => panic!("expected NumberInput state"),
        }
    }

    /// Re-audit fix: `set_number_value` must NOT overwrite
    /// `last_committed` when a drag-slider is actively scrubbing the
    /// same field. Without this guard, the per-frame snapshot
    /// republish (host path) silently moved the rollback anchor to
    /// the latest dragged value, defeating audit fix #2.
    #[test]
    fn set_number_value_preserves_last_committed_during_drag() {
        let (mut store, hits, rect) = number_input_setup(7.0);
        let arena = Bump::new();
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, rect.x + 10.0, rect.y + rect.h * 0.5),
            &arena,
        );
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, rect.x + 30.0, rect.y + rect.h * 0.5),
            &arena,
        );
        // Host-side snapshot republish: writes a value via
        // `set_number_value` while drag is active. Must NOT clobber
        // `last_committed` (still anchored at 7.0 = pre-drag).
        store.set_number_value(NodeId(77), 999.0);
        match store.get(NodeId(77)) {
            Some(InteractiveState::NumberInput { last_committed, .. }) => {
                assert!(
                    (*last_committed - 7.0).abs() < f64::EPSILON,
                    "set_number_value mid-drag must not move last_committed; got {last_committed}"
                );
            }
            _ => panic!("expected NumberInput state"),
        }
    }

    /// M14.A: pointer-Up clears the continuous-hold so subsequent
    /// ticks (even at a time past the delay) do nothing — release
    /// stops the repeat. Verified against the same fixture as the
    /// previous test minus the trailing ticks.
    #[test]
    fn number_stepper_hold_ends_on_pointer_up() {
        use crate::interaction::drag::STEPPER_HOLD_INITIAL_DELAY_NS;
        let (mut store, hits, rect) = number_input_setup(0.0);
        let arena = Bump::new();
        let probe = crate::widget::NumberInput::new(NodeId(77), "", 0.0);
        let up = probe.up_rect(rect);
        let mut down_evt = pointer(PointerKind::Down, up.x + up.w * 0.5, up.y + up.h * 0.5);
        down_evt.timestamp_ns = 0;
        let _ = dispatch_pointer(&mut store, &hits, down_evt, &arena);
        // Up at t=10 ms — hold cleared.
        let mut up_evt = pointer(PointerKind::Up, up.x + up.w * 0.5, up.y + up.h * 0.5);
        up_evt.timestamp_ns = 10_000_000;
        let _ = dispatch_pointer(&mut store, &hits, up_evt, &arena);
        assert!(store.number_stepper_hold().is_none());
        // Tick at 500 ms (well past delay) — nothing fires.
        let evts = dispatch_tick(
            &arena,
            &mut store,
            STEPPER_HOLD_INITIAL_DELAY_NS + 250_000_000,
        );
        assert!(
            evts.is_empty(),
            "no repeat after pointer-Up cleared the hold"
        );
        // Value remained at the single Down-increment.
        assert!((read_value(&store, NodeId(77)) - 1.0).abs() < f64::EPSILON);
    }

    /// Audit fix #1 (CRITICAL): Esc clears any in-flight
    /// `number_input_drag` AND `number_stepper_hold` regardless of
    /// focus. Without this, the drag state stays armed and the next
    /// Move would continue advancing `last_committed` from a stale
    /// `start_value`.
    #[test]
    fn esc_clears_in_flight_drag_and_stepper_hold() {
        let (mut store, hits, rect) = number_input_setup(7.0);
        let arena = Bump::new();
        // 1) Start a drag-slider mid-scrub.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, rect.x + 10.0, rect.y + rect.h * 0.5),
            &arena,
        );
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, rect.x + 30.0, rect.y + rect.h * 0.5),
            &arena,
        );
        assert!(store.number_input_drag().is_some());
        // 2) Esc cancels.
        let evts = dispatch_key(&mut store, key(KEY_ESCAPE, false), &arena);
        let _ = evts;
        assert!(
            store.number_input_drag().is_none(),
            "Esc must clear number_input_drag"
        );

        // Same coverage for stepper hold.
        let (mut store, hits, rect) = number_input_setup(0.0);
        let arena = Bump::new();
        let probe = crate::widget::NumberInput::new(NodeId(77), "", 0.0);
        let up = probe.up_rect(rect);
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, up.x + up.w * 0.5, up.y + up.h * 0.5),
            &arena,
        );
        assert!(store.number_stepper_hold().is_some());
        let _ = dispatch_key(&mut store, key(KEY_ESCAPE, false), &arena);
        assert!(
            store.number_stepper_hold().is_none(),
            "Esc must clear number_stepper_hold"
        );
    }

    /// Audit fix #2 (CRITICAL): per-Move drag updates `value` +
    /// `buffer` but leaves `last_committed` untouched until Up.
    /// Otherwise Esc-revert would only roll back to the most recent
    /// scrubbed value, not to the pre-Down anchor.
    #[test]
    fn drag_move_does_not_advance_last_committed() {
        let (mut store, hits, rect) = number_input_setup(42.0);
        let arena = Bump::new();
        // Pre-Down anchor is `last_committed = 42.0`.
        let initial_last_committed = match store.get(NodeId(77)) {
            Some(InteractiveState::NumberInput { last_committed, .. }) => *last_committed,
            _ => -1.0,
        };
        assert_eq!(initial_last_committed, 42.0);
        // Down → Move past threshold → value advances, last_committed
        // must NOT.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Down, rect.x + 10.0, rect.y + rect.h * 0.5),
            &arena,
        );
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Move, rect.x + 30.0, rect.y + rect.h * 0.5),
            &arena,
        );
        if let Some(InteractiveState::NumberInput {
            value,
            last_committed,
            ..
        }) = store.get(NodeId(77))
        {
            assert!(
                (*value - 42.0).abs() > 1e-3,
                "value should have advanced during drag"
            );
            assert_eq!(
                *last_committed, 42.0,
                "last_committed must remain pre-Down anchor until Up"
            );
        }
        // Up commits last_committed to the scrubbed value.
        let _ = dispatch_pointer(
            &mut store,
            &hits,
            pointer(PointerKind::Up, rect.x + 30.0, rect.y + rect.h * 0.5),
            &arena,
        );
        if let Some(InteractiveState::NumberInput {
            value,
            last_committed,
            ..
        }) = store.get(NodeId(77))
        {
            assert_eq!(*value, *last_committed);
        }
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
