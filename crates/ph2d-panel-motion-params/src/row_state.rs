//! **O ESTADO DE UMA ROW no `WidgetStore`** — quem está a ser editado, a conversão entre
//! o curso do slider e o valor, e a semeadura dos campos.
//!
//! Separado do [`super`] pelo teto de LOC (HR-18, 600 para `ph2d-panel-*`), no corte que a
//! pergunta desenha: lá fica *o que o painel PINTA*, aqui *o que ele LEMBRA de cada row*.

use super::*;

/// True while any pooled param row is being interacted with — a slider dragged /
/// focused, a chip focused, or a standalone number box (Angle / Seed) focused.
/// The shell reads this as the undo-bracket edge (open on the false→true
/// transition, commit one step on true→false), so a whole slider drag — or a
/// whole type-into-the-angle-box — is a single undo step (M1.P1).
#[must_use]
pub fn any_param_editing(store: &WidgetStore) -> bool {
    (0..MAX_PARAM_ROWS).any(|slot| {
        matches!(
            store.slider(param_slider_id(slot)),
            Some((SliderState::Dragging | SliderState::Focused, _))
        ) || matches!(
            store.get(param_chip_id(slot)),
            Some(InteractiveState::NumberInput {
                state: TextInputState::Focused,
                ..
            })
        ) || number_is_typing(store, param_number_id(slot))
            || text_is_typing(store, param_text_id(slot))
    })
}

/// `[0,1]` slider track for a value in `[min, min+span]`.
pub(crate) fn normalized_track(value: f64, min: f64, span: f64) -> f32 {
    (((value - min) / span) as f32).clamp(0.0, 1.0)
}

/// The param value a slider `track` (`0..1`) maps to over `[min, max]`, rounded
/// to a whole number for integer params. Shared by `paint` (chip display) and
/// `apply_event` (the emitted `SetParam` value) so the knob and the doc agree.
pub(crate) fn row_value(track: f32, min: f64, max: f64, integer: bool) -> f64 {
    let span = (max - min).max(f64::EPSILON);
    let v = min + f64::from(track) * span;
    if integer { v.round() } else { v }
}

/// A `NumberInput` state seeded at `value` (buffer = its canonical formatting).
pub(crate) fn number_input(value: f64) -> InteractiveState {
    InteractiveState::NumberInput {
        state: TextInputState::Normal,
        value,
        buffer: format_number(value),
        caret: 0,
        last_committed: value,
        selection_anchor: None,
    }
}

/// Seed each pooled row from its doc value + refresh its range/link, EXCEPT for a
/// row currently being dragged (slider) or typed (chip) — so an in-progress
/// interaction owns the widget, but idle rows always mirror the doc (undo /
/// external edits live-update the knob).
pub(crate) fn seed_rows(store: &mut WidgetStore, rows: &[ParamRow]) {
    for (i, row) in rows.iter().enumerate().take(MAX_PARAM_ROWS) {
        // Toggle rows: sync the checkbox value to the doc while PRESERVING its
        // transient hover/press state (the dispatch owns that + flips the value
        // on click). Colour swatches seed from the bridge's `widget_color`; Enum
        // buttons are stateless (selection comes from the snapshot at paint).
        if let ParamRow::Toggle(t) = row {
            let cb_id = param_checkbox_id(i);
            let state = store
                .checkbox(cb_id)
                .map(|(s, _)| s)
                .unwrap_or(CheckboxState::Normal);
            store.register(
                cb_id,
                InteractiveState::Checkbox {
                    state,
                    value: if t.on {
                        CheckboxValue::Checked
                    } else {
                        CheckboxValue::Unchecked
                    },
                },
            );
            continue;
        }
        // Standalone number boxes (Angle in degrees, Seed as a whole number):
        // mirror the doc value when unfocused + register the range so the
        // drag-scrub is range-proportional and the stepper uses `step`.
        if let ParamRow::Angle(a) = row {
            let id = param_number_id(i);
            mirror_number(store, id, a.deg, ANGLE_DECIMALS);
            store.set_number_range(id, a.min_deg, a.max_deg, a.step_deg);
            continue;
        }
        if let ParamRow::Seed(s) = row {
            let id = param_number_id(i);
            mirror_number(store, id, s.value, SEED_DECIMALS);
            store.set_number_range(id, s.min, s.max, 1.0);
            continue;
        }
        // ⭐ O campo de texto PARTILHADO do slot: espelha o valor vivo do documento quando não
        // tem foco. ⚠️ **As quatro rows que o usam**, não só a `Text` — a lista é a de
        // [`crate::shared_field`], que é a mesma que o `on_text_commit` lê do outro lado.
        // Até 2026-08-30 esta arma era `ParamRow::Text` e a `Channels`/`Source`/`File` abriam
        // sempre em branco; como o `Blur` comita, tocar no campo GRAVAVA o vazio.
        if let Some(v) = crate::shared_field::shared_text_value(row) {
            mirror_text(store, param_text_id(i), v);
            continue;
        }
        let ParamRow::Scalar(row) = row else { continue };
        let slider_id = param_slider_id(i);
        let chip_id = param_chip_id(i);
        let span = (row.max - row.min).max(f64::EPSILON);

        let dragging = matches!(
            store.slider(slider_id),
            Some((SliderState::Dragging | SliderState::Focused, _))
        );
        let typing = matches!(
            store.get(chip_id),
            Some(InteractiveState::NumberInput {
                state: TextInputState::Focused,
                ..
            })
        );
        if !dragging && !typing {
            // Value from the doc, STATE from the dispatch — `number_rows` owns both
            // halves of that split now, for the reason written there.
            mirror_slider(store, slider_id, normalized_track(row.value, row.min, span));
            mirror_chip(store, chip_id, row.value);
        }
        // Range (chip typed-value clamp/step) + slider↔chip affine (track 0..1 →
        // value): display = track * span + min. Integer rows snap the chip.
        //
        // The CHIP gets the HARD range and the slider keeps the soft one: the
        // drag range and the legal range are different questions (Blender's soft
        // vs hard limits). Outside `[row.min, row.max]` the affine below saturates
        // the track at 0.0 / 1.0, so such a value cannot come back through the
        // slider — which is exactly why `on_value_changed` lets the chip speak for
        // itself out there. Both ENDS, since doc 88: the ceiling shipped alone and
        // the floor was pinned to the slider's `min`, so a param whose useful drag
        // starts at `0.01` could not be typed to `0.001`.
        store.set_number_range(chip_id, row.hard_min, row.hard_max, row.step);
        if row.integer {
            store.link_slider_number_mapped_integer(
                slider_id,
                chip_id,
                span as f32,
                row.min as f32,
            );
        } else {
            store.link_slider_number_mapped(slider_id, chip_id, span as f32, row.min as f32);
        }
    }
}
