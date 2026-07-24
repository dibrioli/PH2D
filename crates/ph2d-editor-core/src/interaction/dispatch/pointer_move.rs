//! Pointer **Move** dispatch arm. Extracted verbatim from the
//! `dispatch_pointer_with_text` god-function (blindagem Fase 3.2) — pure move,
//! same `super::` paths, same behaviour (covered by `dispatch::tests`).

use super::blender::{apply_blender_channel_value, apply_blender_hit};
use super::curve::apply_curve_point_drag;
use super::hover::update_hover;
use super::number_input::update_drag_value;
use super::text_ops::{byte_offset_from_click_xy, place_text_caret};
use crate::interaction::flip_strip::FlipStripGesture;
use crate::interaction::types::{BlenderHitKind, GesturePhase, GraphGesture, TimelineGesture};
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore, drag};
use bumpalo::collections::Vec as BumpVec;
use ph2d_host::PointerEvent;
use ph2d_text::TextSystem;

/// Handle a pointer-`Move` event: advances any active drag (slider/picker/
/// panel-resize/scrollbar/hierarchy/painter-layer/number-input-scrub), or
/// updates hover when nothing is active.
pub(super) fn dispatch_move<'frame>(
    store: &mut WidgetStore,
    hit_index: &HitIndex,
    event: PointerEvent,
    mut ts: Option<&mut TextSystem>,
    events: &mut BumpVec<'frame, WidgetEvent>,
) {
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
    // Painter layers-panel row drag (W3 T3.8) — advance the anchor so
    // the panel can render the drop indicator + flip `active`.
    if store.painter_layer_drag().is_some() {
        store.update_painter_layer_drag(event.x, event.y);
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
            // Range-proportional scrub when the box registered a `number_range`: a fixed drag spans the
            // WHOLE `[min,max]` range (horizontal fast, vertical precise) regardless of magnitude — else
            // the legacy step-based rate for unbounded boxes (e.g. pixel position). Enio 2026-06-25.
            let range = store.number_range(d.id);
            let (rate_x, rate_y) = if let Some(rate) = store.number_drag_rate(d.id) {
                // Calibrated UNBOUNDED scrub: `rate` value-units per horizontal
                // cursor pixel, 10× finer on the vertical (precise) axis. No range
                // ⇒ no clamp (see the `bounds` computation below).
                (rate, rate / 10.0)
            } else {
                match range {
                    Some((min, max, _)) => {
                        let r = (max - min).abs();
                        (r / drag::DRAG_RANGE_PX_H, r / drag::DRAG_RANGE_PX_V)
                    }
                    None => (drag::DRAG_RATE_X * d.step, drag::DRAG_RATE_Y * d.step),
                }
            };
            let delta = (dom_dx as f64 * rate_x - dom_dy as f64 * rate_y) * shift_mul;
            // Apply the per-Move delta on top of the chip's
            // CURRENT value (not `start_value`). Read it back
            // out before mutating so the clamp logic below can
            // operate on the same number we wrote.
            let current_value = match store.get(d.id) {
                Some(InteractiveState::NumberInput { value, .. }) => *value,
                _ => d.start_value,
            };
            // ⚠️ Numa caixa que ARREDONDA, a base do Move seguinte não pode ser o valor já
            // arredondado: o resíduo seria descartado a cada Move e o scrub travava
            // (`round(round(v) + d) == round(v)` para todo `d < 0.5`). No eixo VERTICAL, que é
            // o preciso (`DRAG_RANGE_PX_V` = 2500), um Move típico carrega ~0.15 de uma
            // contagem — nunca meia unidade. Elas acumulam num contínuo à parte; as caixas
            // contínuas continuam a ler o valor de volta, byte por byte como antes.
            let base = if store.linked_slider_snap_integer(d.id) {
                d.accum
            } else {
                current_value
            };
            let raw_value = base + delta;
            // When the chip is bounded by a slider, the valid
            // DISPLAY range is the affine projection of the
            // slider's `0..1` storage — for a mapped link with
            // `display = storage*scale + offset` that's the
            // interval `[offset, scale+offset]` (or its reverse
            // when `scale` is negative). Without this, dragging
            // Grow (display ±1) silently clamped at 0..1 and
            // never reached the negative half.
            let bounds = if store.number_drag_rate(d.id).is_some() {
                // A registered drag rate means "unbounded scrub" (Vector transform
                // chips): the rate calibrates px→value directly, no clamp — world
                // coords span any magnitude. Wins over any range/slider mapping.
                None
            } else if let Some((min, max, _)) = range {
                Some((min.min(max), min.max(max)))
            } else if store.linked_slider(d.id).is_some() {
                let (scale, offset) = store.linked_slider_mapping(d.id);
                let a = offset as f64;
                let b = (scale + offset) as f64;
                Some((a.min(b), a.max(b)))
            } else if store.blender_channel_chip(d.id).is_some() {
                Some((0.0_f64, 1.0_f64))
            } else {
                None
            };
            let new_value = if let Some((lo, hi)) = bounds {
                raw_value.clamp(lo, hi) // CLAMP-OK: (lo,hi) pre-swapped via a.min/a.max above; (scale,offset) registered via link_slider_number_mapped (debug_assert scale!=0).
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
            // O acumulador guarda o valor CLAMPADO, não o cru: assim uma inversão depois de
            // bater no limite move o valor já no Move seguinte — a propriedade que o modelo
            // incremental existe para garantir.
            store.set_number_input_drag_accum(new_value);
            // Audit follow-up #7 (MED, 2026-05-28): converge
            // on the shared `apply_chip_value_with_mirror`
            // helper — single source of truth for chip+slider
            // writes across commit / stepper / tick / drag-
            // scrub. The helper no longer writes
            // `last_committed` (split out 2026-05-28), so the
            // audit fix #2 CRITICAL invariant (drag preserves
            // pre-drag rollback anchor) survives. Bonus: drag-
            // scrub now also inherits integer snap via
            // `link_slider_number_mapped_integer`.
            let (_final_val, _was_clamped) =
                super::apply_chip_value_with_mirror(store, d.id, new_value);
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
            // Drag-scrub also wrote the linked slider above
            // (the inverse-projected, clamped storage). Emit
            // its ValueChanged so panel handlers keyed off the
            // slider id (post-2026-05-27 canonical pattern in
            // padding/upscale/color-eq) see the live drag —
            // without this, swallow-the-chip-event handlers
            // dropped per-frame drag-scrub mutations on the
            // floor (audit finding #1, lens B).
            if let Some(slider_id) = store.linked_slider(d.id)
                && matches!(store.get(slider_id), Some(InteractiveState::Slider { .. }))
            {
                events.push(WidgetEvent::ValueChanged(slider_id));
            }
            number_input_drag_consumed = Some(d.id);
        }
    }
    if let Some(active) = store.active_id() {
        // Motion Nodes M0.T3 — a captured graph surface streams an Update on
        // every move (even once the pointer has left its rect: a node drag
        // continues past the panel edge). The panel drains + interprets it.
        if let Some((surface, kind)) = store.graph_surface_at_id(active) {
            store.set_graph_moved(true);
            let mods = store.gesture_mods();
            store.push_graph_gesture(GraphGesture {
                surface,
                kind,
                phase: GesturePhase::Update,
                x: event.x,
                y: event.y,
                button: event.button,
                mods,
            });
            return;
        }
        // W2.E5b — a captured timeline surface streams an Update on every move
        // (even once the pointer has left its rect: a key drag continues past the
        // panel edge). Mirror of the graph-surface Update above.
        if let Some((surface, kind)) = store.timeline_surface_at_id(active) {
            store.note_timeline_pointer(event.x, event.y);
            let mods = store.gesture_mods();
            store.push_timeline_gesture(TimelineGesture {
                surface,
                kind,
                phase: GesturePhase::Update,
                x: event.x,
                y: event.y,
                button: event.button,
                mods,
            });
            return;
        }
        // A tira do Flip, pela mesma lei: o Update continua mesmo com o ponteiro
        // FORA do retângulo da célula — arrastar uma chave três células adiante é o
        // gesto, não um escape.
        if let Some((surface, kind)) = store.flip_strip_surface_at_id(active) {
            store.note_flip_strip_pointer(event.x, event.y);
            let mods = store.gesture_mods();
            store.push_flip_strip_gesture(FlipStripGesture {
                surface,
                kind,
                phase: GesturePhase::Update,
                x: event.x,
                y: event.y,
                button: event.button,
                mods,
            });
            return;
        }
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
                let offset =
                    byte_offset_from_click_xy(store, active, rect, event.x, event.y, ts.take());
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
            // W4 §3 — drag the active curve control point (normalizes
            // against the editor's plotting canvas carried in the variant).
            let is_curve = matches!(store.get(active), Some(InteractiveState::CurvePoint { .. }));
            if is_curve
                && let Some(parent) = apply_curve_point_drag(store, active, event.x, event.y)
            {
                events.push(WidgetEvent::ValueChanged(parent));
            }
        }
    } else {
        let hit = hit_index.hit(event.x, event.y);
        update_hover(store, hit);
    }
}
