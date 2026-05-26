//! Pointer-event dispatcher.
//!
//! Extracted from `dispatch/mod.rs` in Wave 6+7 Phase 1.A. The
//! 893-LOC `dispatch_pointer_with_text` god-function plus its
//! file-private helpers (`is_color_target_id`,
//! `clear_combobox_if_button_hit`) live here.

use super::blender::{apply_blender_channel_value, apply_blender_hit};
use super::focus::{apply_click, is_focusable};
use super::hierarchy::{HierDrop, find_hierarchy_drop};
use super::hover::{set_widget_pressed, set_widget_released, update_hover};
use super::number_input::{apply_number_stepper_if_hit, update_drag_value};
use super::scroll::scrollbar_panel_for_id;
use super::text_ops::{byte_offset_from_click_xy, place_text_caret};
use super::{
    commit_hex_buffer, commit_number_buffer, init_number_buffer, is_section_header_id,
    reset_focused_visual_state, select_all_in_text_widget,
};
use crate::interaction::types::{BlenderHitKind, ContextMenuKind, ContextMenuRequest};
use crate::interaction::util::format_number;
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore, drag};
use crate::zones::Rect;
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;
use ph2d_host::{PointerEvent, PointerKind};
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
            // Bottom-LEFT resize — mirror of the BR path above but
            // also shifts the panel offset by the same x-delta so the
            // RIGHT edge stays anchored while the LEFT edge follows
            // the cursor. Width adjusts in the opposite direction
            // (cursor right → width shrinks, cursor left → width
            // grows). Height (cy) is identical to BR.
            if let Some((panel, last_x, last_y)) = store.panel_resize_anchor_bl() {
                let dx = event.x - last_x;
                let dy = event.y - last_y;
                let (cur_dw, cur_dh) = store.panel_resize_delta(panel);
                let new_dw = cur_dw - dx;
                let new_dh = cur_dh + dy;
                store.set_panel_resize_delta(panel, new_dw, new_dh);
                let (cur_off_x, cur_off_y) = store.blender_picker_offset(panel);
                store.set_blender_picker_offset(panel, cur_off_x + dx, cur_off_y);
                store.update_panel_resize_cursor_bl(event.x, event.y);
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
                let dx_total = event.x - drag.start_x; // DRAG-ABS-OK: total distance from press (threshold-crossing test only — actual value uses step_dx)
                let dy_total = event.y - drag.start_y; // DRAG-ABS-OK: total distance from press (threshold-crossing test only — actual value uses step_dy)
                if !drag.crossed_threshold {
                    let dist_sq = dx_total * dx_total + dy_total * dy_total;
                    let thr = drag::NUMBER_INPUT_DRAG_THRESHOLD_PX;
                    if dist_sq >= thr * thr {
                        // Decide the locked axis at THIS Move based
                        // on which delta is larger. `>=` so a perfect
                        // 45° diagonal defaults to horizontal (the
                        // primary scrub axis).
                        let horizontal = dx_total.abs() >= dy_total.abs();
                        // Pass cursor position so the promotion re-anchors
                        // `last_x`/`last_y` here — otherwise the same Move
                        // that crossed the threshold would apply the
                        // entire ~5 px Down→here delta as a value JUMP.
                        store.promote_number_input_drag_to_slider(horizontal, event.x, event.y);
                    }
                }
                // Re-read after the potential promotion.
                if let Some(d) = store.number_input_drag()
                    && d.crossed_threshold
                {
                    // Incremental delta from the LAST Move (not from
                    // Down). The previous absolute-delta model paired
                    // with the clamp pegged the value at the bound: a
                    // reversal after going past the cap kept the chip
                    // stuck at the cap until the cursor returned all
                    // the way to `start_x`. Standard Blender/AE scrub
                    // is incremental: each Move adds its own dx to the
                    // current value, so a reversal IMMEDIATELY moves
                    // the value the other way.
                    let step_dx = event.x - d.last_x;
                    let step_dy = event.y - d.last_y;
                    let (dom_dx, dom_dy) = if d.axis_horizontal {
                        (step_dx, 0.0)
                    } else {
                        (0.0, step_dy)
                    };
                    let shift_mul = if store.shift_held() {
                        drag::DRAG_SHIFT_MUL
                    } else {
                        1.0
                    };
                    let delta = (dom_dx as f64 * drag::DRAG_RATE_X
                        - dom_dy as f64 * drag::DRAG_RATE_Y)
                        * shift_mul
                        * d.step;
                    // Apply the per-Move delta on top of the chip's
                    // CURRENT value (not `start_value`). Read it back
                    // out before mutating so the clamp logic below can
                    // operate on the same number we wrote.
                    let current_value = match store.get(d.id) {
                        Some(InteractiveState::NumberInput { value, .. }) => *value,
                        _ => d.start_value,
                    };
                    let raw_value = current_value + delta;
                    let is_bounded = store.linked_slider(d.id).is_some()
                        || store.blender_channel_chip(d.id).is_some();
                    let new_value = if is_bounded {
                        raw_value.clamp(0.0, 1.0)
                    } else {
                        raw_value
                    };
                    // Advance the per-Move anchor BEFORE writing back —
                    // the next Move computes its delta from this new
                    // `last`. The anchor advances unconditionally
                    // (even when the value clamped at a bound) so a
                    // reversal still produces a non-zero step_dx on
                    // the very next Move.
                    store.advance_number_input_drag_anchor(event.x, event.y);
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
                        *buffer = format_number(new_value);
                    }
                    // Mirror to a linked slider if any (same pattern
                    // as `apply_number_stepper_if_hit`).
                    if let Some(slider_id) = store.linked_slider(d.id)
                        && let Some(InteractiveState::Slider { value, .. }) =
                            store.get_mut(slider_id)
                    {
                        *value = (new_value as f32).clamp(0.0, 1.0);
                    }
                    // BlenderColorPicker channel chip drag: push the
                    // scrubbed value back into the parent picker's
                    // RGBA / HSVA dimension so the swatch + wheel +
                    // sibling channels re-render live. Mirrors the
                    // commit path in `commit_number_buffer`.
                    if let Some((parent, idx)) = store.blender_channel_chip(d.id) {
                        apply_blender_channel_value(store, parent, idx, new_value as f32);
                        events.push(WidgetEvent::ValueChanged(parent));
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
                            kind: BlenderHitKind::Wheel
                                | BlenderHitKind::ValueSlider
                                | BlenderHitKind::ChannelSlider(_),
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
                    if let Some(row) = crate::ids::hier_eye_companion_to_row(id) {
                        Some(row)
                    } else if let Some(row) = crate::ids::hier_expand_companion_to_row(id) {
                        Some(row)
                    } else {
                        Some(id)
                    }
                    .filter(|row| store.is_hierarchy_row(*row))
                });
                if let Some(row) = hier_row_id {
                    store.open_context_menu(ContextMenuRequest {
                        x: event.x,
                        y: event.y,
                        kind: ContextMenuKind::HierarchyRow { row },
                    });
                } else if let Some(note_index) = note_slot
                    && let Some(panel) = panel_under
                {
                    store.open_context_menu(ContextMenuRequest {
                        x: event.x,
                        y: event.y,
                        kind: ContextMenuKind::NoteBackground { panel, note_index },
                    });
                } else if is_section {
                    let section_id = hit_id.unwrap();
                    store.open_context_menu(ContextMenuRequest {
                        x: event.x,
                        y: event.y,
                        kind: ContextMenuKind::SectionOutline {
                            section: section_id,
                        },
                    });
                } else if let Some(panel) = panel_under.filter(|p| {
                    *p != crate::ids::HIER_PANEL
                        && *p != crate::ids::PAD_PANEL
                        && *p != crate::ids::BGR_PANEL
                        && *p != crate::ids::CEQ_PANEL
                        && *p != crate::ids::UPS_PANEL
                        && *p != crate::ids::EQS_PANEL
                        && *p != crate::grid_snap::ids::GS_PANEL
                }) {
                    // `before_section` is filled in by apply_event
                    // — only the inspector knows the screen→body
                    // conversion + section y-ranges.
                    //
                    // Hierarchy + image-tool panels (PAD/BGR/CEQ/UPS/EQS)
                    // are excluded by design — these are transient
                    // operation surfaces, not annotation surfaces. UI
                    // canon post-2026-05-24: notes + outlines live in
                    // Inspector + Widget Gallery only.
                    store.open_context_menu(ContextMenuRequest {
                        x: event.x,
                        y: event.y,
                        kind: ContextMenuKind::CreateNote {
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
                && hit_id == crate::ids::TOPBAR_THEME
                && matches!(store.get(hit_id), Some(InteractiveState::Button { .. }))
            {
                store.open_context_menu(ContextMenuRequest {
                    x: hit_rect.x,
                    y: hit_rect.y + hit_rect.h + 4.0,
                    kind: ContextMenuKind::ThemeSelector,
                });
                return events.into_bump_slice();
            }
            // Same pattern for the Save chip — Primary opens the
            // Save / Save As menu anchored below the chip.
            if event.button == ph2d_host::PointerButton::Primary
                && let Some((hit_id, hit_rect)) = hit
                && hit_id == crate::ids::TOPBAR_SAVE
                && matches!(store.get(hit_id), Some(InteractiveState::Button { .. }))
            {
                store.open_context_menu(ContextMenuRequest {
                    x: hit_rect.x,
                    y: hit_rect.y + hit_rect.h + 4.0,
                    kind: ContextMenuKind::SaveMenu,
                });
                return events.into_bump_slice();
            }
            // Open chip — same anchor logic.
            if event.button == ph2d_host::PointerButton::Primary
                && let Some((hit_id, hit_rect)) = hit
                && hit_id == crate::ids::TOPBAR_OPEN
                && matches!(store.get(hit_id), Some(InteractiveState::Button { .. }))
            {
                store.open_context_menu(ContextMenuRequest {
                    x: hit_rect.x,
                    y: hit_rect.y + hit_rect.h + 4.0,
                    kind: ContextMenuKind::OpenMenu,
                });
                return events.into_bump_slice();
            }
            // Settings cluster (gear) — opens the SettingsMenu with
            // px/m presets. Same anchor convention as Save/Open.
            if event.button == ph2d_host::PointerButton::Primary
                && let Some((hit_id, hit_rect)) = hit
                && hit_id == crate::ids::TOPBAR_SETTINGS
                && matches!(store.get(hit_id), Some(InteractiveState::Button { .. }))
            {
                store.open_context_menu(ContextMenuRequest {
                    x: hit_rect.x,
                    y: hit_rect.y + hit_rect.h + 4.0,
                    kind: ContextMenuKind::SettingsMenu,
                });
                return events.into_bump_slice();
            }
            // Project chip → SceneList popover (search + scenes).
            if event.button == ph2d_host::PointerButton::Primary
                && let Some((hit_id, hit_rect)) = hit
                && hit_id == crate::ids::TOPBAR_PROJECT
                && matches!(store.get(hit_id), Some(InteractiveState::Button { .. }))
            {
                store.open_context_menu(ContextMenuRequest {
                    x: hit_rect.x,
                    y: hit_rect.y + hit_rect.h + 4.0,
                    kind: ContextMenuKind::SceneList,
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
                    Some(ContextMenuKind::SceneList)
                ) && matches!(
                    hit_id,
                    Some(id) if id == crate::ids::CTX_SCENE_SEARCH
                        || crate::ids::CTX_SCENE_ROWS.contains(&id)
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
                && (crate::ids::hier_eye_companion_to_row(id).is_some()
                    || crate::ids::hier_expand_companion_to_row(id).is_some())
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
                //
                // Skip the stepper hit-test entirely for chips painted
                // as bare pills (`paint_number_chip`) — those don't
                // draw arrows but the dispatch carves the right column
                // out of every NumberInput's hit rect by default,
                // producing a phantom continuous-hold that keeps
                // climbing while the pointer is still.
                let stepper_hit = !combo_cleared
                    && !store.is_chip_no_stepper(id)
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

/// On focus arrival into a NumberInput, sync `buffer` from `value`
/// using the same formatter the painter uses, place the caret at
/// the end, and mark state as Focused so the painter draws the
/// caret + focus ring (otherwise the user has no visual feedback
/// that the field accepted the click).
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
