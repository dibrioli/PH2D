//! M14 live Inspector section painters (Name / Visibility / Transform
//! / Render Source).
//!
//! Extracted from [`super`] (Track C3). These are the **active**
//! Inspector body sections — i.e. the ones that read live snapshots
//! the host publishes via [`super::state::CURRENT_INSPECTOR_*`] and
//! paint editable controls bound back through `pending_*_edit` slots
//! on [`crate::screens::hero::HeroScreen`].
//!
//! The dead-code Widget Gallery showcase painters stay in [`super`]
//! / `inspector::showcase` (later Track C5 extract).

use super::super::{InspectorSpriteInfo, InspectorSpriteSource};
use super::ids;
use super::state::current_display_unit;
use super::{paint_section_separator, read_number_input};
use crate::interaction::{HitIndex, InteractiveState, WidgetStore};
use crate::paint::{paint_text, paint_text_title, resolve};
use crate::widget::{
    Button, ButtonKind, ButtonState, Checkbox, CheckboxState, CheckboxValue, NumberInput,
    TextInput, TextInputState, paint_button, paint_checkbox, paint_number_input_with_buffer,
    paint_text_input_with_buffer,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// M14.E: paint the editable entity-name TextInput row at the top of
/// the Inspector body. The host seeds the store buffer with the
/// selected entity's `Name` on the selection-change frame; thereafter
/// the painter just reads the live buffer, so in-progress edits stay
/// alive across snapshot republishes.
///
/// Returns the y-coordinate of the bottom of the painted row.
#[allow(clippy::too_many_arguments)]
pub(in crate::screens::hero) fn paint_entity_name_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let row_h = ROW_H_PX;
    let host = Rect::new(x, y, w, row_h);
    hit_index.register(ids::INSP_ENTITY_NAME, host);
    let (state, text, caret, anchor) = match store.get(ids::INSP_ENTITY_NAME) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, Some(text.as_str()), *caret, *selection_anchor),
        _ => (TextInputState::Normal, None, 0, None),
    };
    let input = TextInput::new(ids::INSP_ENTITY_NAME, "")
        .placeholder("Name\u{2026}")
        .state(state);
    paint_text_input_with_buffer(
        &input,
        text,
        Some(caret),
        anchor,
        host,
        scene,
        text_system,
        theme,
    );
    y + row_h + Spacing::Sm.px()
}

/// M14.D: paint the live Visibility checkbox row above the Transform
/// section. Mirrors the eye toggle in the Hierarchy panel.
#[allow(clippy::too_many_arguments)]
pub(in crate::screens::hero) fn paint_visibility_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let row_h = 24.0_f32; // LITERAL-PX-OK: compact checkbox row (shorter than ROW_H_PX=28; checkbox is mini-control)
    let (state, value) = match store.checkbox(ids::INSP_VISIBILITY_CHECK) {
        Some(pair) => pair,
        // Fallback: render Checked as a sensible default if the
        // store hasn't been populated (e.g. early-paint smoke tests).
        None => (CheckboxState::Normal, CheckboxValue::Checked),
    };
    let host = Rect::new(x, y, w, row_h);
    hit_index.register(ids::INSP_VISIBILITY_CHECK, host);
    let checkbox = Checkbox::new(ids::INSP_VISIBILITY_CHECK, "Visible")
        .state(state)
        .value(value);
    paint_checkbox(&checkbox, host, scene, text_system, theme);
    y + row_h + Spacing::Sm.px()
}

/// M14.A: paint the live `Transform` editor section. Shows Position
/// X/Y (meters), Rotation (degrees, rad ↔ deg conversion at the
/// paint/commit boundary), Scale X/Y (unitless), and a Reset-to-
/// Identity button in the section header. Z is intentionally absent
/// — `Transform` is 2D by design (SKILL §3 + ADR-0025).
///
/// Wiring: the section paints the canonical
/// [`crate::widget::paint_number_input_with_buffer`] (Widget Gallery
/// reference) for each of the 5 editable fields. Live values come
/// from the [`WidgetStore`]'s number-value cache; the host seeds
/// those via `set_number_value` whenever a new
/// [`super::super::InspectorTransformInfo`] snapshot lands (selection
/// change, gizmo drag, script mutation). Per
/// [`crate::interaction::WidgetStore::set_number_value`], focused
/// fields skip the rewrite so an in-progress edit isn't clobbered.
///
/// Commits flow through `WidgetEvent::ValueChanged` (Enter / blur)
/// in [`super::super::HeroScreen::apply_event`], which assembles a
/// fresh [`super::super::InspectorTransformInfo`] from the 5 store
/// values and publishes it via `pending_transform_edit` for the
/// shell to push to its `EditorCommandQueue` as
/// [`ph2d_ecs::scene::commands::EditorCommand::SetComponent`].
///
/// Returns the y-coordinate of the bottom of the painted section so
/// the caller can advance the body cursor.
#[allow(clippy::too_many_arguments)]
pub(in crate::screens::hero) fn paint_transform_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let label_font = TypeToken::Sm.px();
    let field_h = ROW_H_PX;
    let row_gap = Spacing::Sm.px();
    let label_color = resolve(ColorToken::Text2, theme);

    // ── Section header: "Transform" title + Reset (Identity) button ──
    paint_text_title(
        text_system,
        scene,
        "Transform",
        x,
        y,
        TypeToken::Md.px(),
        w - 90.0, // LITERAL-PX-OK: width budget for title, reserves 90px (≈reset_w 80 + padding) for the Reset button
        resolve(ColorToken::Text1, theme),
    );
    let reset_w = 80.0_f32; // LITERAL-PX-OK: Reset button explicit width (chrome dim)
    let reset_h = 24.0_f32; // LITERAL-PX-OK: Reset button explicit height (compact, shorter than ROW_H_PX)
    let reset_rect = Rect::new(x + w - reset_w, y - Spacing::Xxs.px(), reset_w, reset_h);
    let reset_state = store
        .button_state(ids::INSP_TRANSFORM_RESET)
        .unwrap_or(ButtonState::Normal);
    hit_index.register(ids::INSP_TRANSFORM_RESET, reset_rect);
    let reset_btn = Button::new(ids::INSP_TRANSFORM_RESET, "Reset")
        .kind(ButtonKind::Default)
        .state(reset_state);
    paint_button(&reset_btn, reset_rect, scene, text_system, theme);
    let mut cur_y = y + TypeToken::Md.px() + 10.0; // LITERAL-PX-OK: title baseline + breathing gap (composite of font + spacing)
    cur_y = paint_section_separator(scene, theme, x, w, cur_y);

    // ── 5-column grid geometry ──────────────────────────────────────
    // | col 1: row label | col 2: X tag | col 3: X box | col 4: Y tag | col 5: Y box |
    // Col 2 and col 4 are a single-letter wide so the axis tags hug
    // their boxes. Col 1 fixed at the widest row label ("Rotation
    // (°)"); cols 3 + 5 split the remaining width evenly.
    //
    // Two gap sizes: the gap BEFORE each axis tag (col 1→2, col 3→4)
    // is the standard `col_gap` so columns breathe, but the gap
    // BETWEEN the tag and its own box (col 2→3, col 4→5) is tighter
    // (`tag_box_gap`) so the eye reads tag+box as a single unit.
    let col_gap = Spacing::Md.px();
    let tag_box_gap = Spacing::Xxs.px();
    let label_col_w = 78.0_f32; // LITERAL-PX-OK: row-label column width sized for "Rotation (°)" widest case
    let axis_col_w = Spacing::Lg.px();
    // Width consumed by the non-box columns: label + 2×(col_gap before
    // tag) + 2×(tag) + 2×(tag→box gap). The boxes share what's left.
    let non_box_w = label_col_w + col_gap * 2.0 + (axis_col_w + tag_box_gap) * 2.0;
    let box_col_w = ((w - non_box_w) * 0.5).max(40.0); // LITERAL-PX-OK: minimum field box width (chrome-specific min)
    let axis_label_font = TypeToken::Base.px();

    // ── Helper: paint one row of the grid ──────────────────────────
    // `right_id == None` means "row has only an X field" (Rotation
    // case) — col 4 + col 5 stay empty so the grid still aligns.
    let paint_row = |scene: &mut VectorScene,
                     text_system: &mut TextSystem,
                     hit_index: &mut HitIndex,
                     row_y: f32,
                     row_label: &str,
                     left_id: NodeId,
                     left_tag: &str,
                     left_color: ColorToken,
                     left_step: f64,
                     right: Option<(NodeId, &str, ColorToken, f64)>| {
        // Col 1: row label, vertically centered in the field row.
        paint_text(
            text_system,
            scene,
            row_label,
            x,
            row_y + (field_h - label_font) * 0.5,
            label_font,
            label_col_w,
            label_color,
        );
        // Col 2: X / left-axis tag.
        let left_tag_x = x + label_col_w + col_gap;
        paint_text(
            text_system,
            scene,
            left_tag,
            left_tag_x,
            row_y + (field_h - axis_label_font) * 0.5,
            axis_label_font,
            axis_col_w,
            resolve(left_color, theme),
        );
        // Col 3: left field, hugging the X tag (`tag_box_gap`, not
        // `col_gap` — see grid geometry comment). Reads full state
        // from the store so the canonical focus-guard semantics in
        // [`WidgetStore::set_number_value`] take effect — host
        // snapshot refreshes never clobber an in-progress edit.
        let left_box_x = left_tag_x + axis_col_w + tag_box_gap;
        let left_rect = Rect::new(left_box_x, row_y, box_col_w, field_h);
        hit_index.register(left_id, left_rect);
        let (state, value, buffer, caret, anchor) = read_number_input(store, left_id);
        let input = NumberInput::new(left_id, "", value)
            .step(left_step)
            .state(state);
        paint_number_input_with_buffer(
            &input,
            Some(buffer),
            caret,
            anchor,
            left_rect,
            scene,
            text_system,
            theme,
        );
        // Col 4 + 5: right-axis tag + box, when present.
        if let Some((right_id, right_tag, right_color, right_step)) = right {
            let right_tag_x = left_box_x + box_col_w + col_gap;
            paint_text(
                text_system,
                scene,
                right_tag,
                right_tag_x,
                row_y + (field_h - axis_label_font) * 0.5,
                axis_label_font,
                axis_col_w,
                resolve(right_color, theme),
            );
            let right_box_x = right_tag_x + axis_col_w + tag_box_gap;
            let right_rect = Rect::new(right_box_x, row_y, box_col_w, field_h);
            hit_index.register(right_id, right_rect);
            let (r_state, r_value, r_buffer, r_caret, r_anchor) =
                read_number_input(store, right_id);
            let r_input = NumberInput::new(right_id, "", r_value)
                .step(right_step)
                .state(r_state);
            paint_number_input_with_buffer(
                &r_input,
                Some(r_buffer),
                r_caret,
                r_anchor,
                right_rect,
                scene,
                text_system,
                theme,
            );
        }
    };

    // ── Three rows, same grid ──
    // Position label reflects the active display unit ("Position (m)"
    // or "Position (px)") so the user reads the right magnitude. Step
    // also scales — 0.01 m ≈ 1 px @ default 100 px/m, so we want the
    // arrow-key step to feel similar across units (1 px ≈ 1 px).
    let unit = current_display_unit();
    let (pos_label, pos_step) = match unit {
        crate::project::DisplayUnit::Meters => ("Position (m)", 0.01_f64), // LITERAL-PX-OK: NumberInput step (slider precision, not a dimension)
        crate::project::DisplayUnit::Pixels => ("Position (px)", 1.0_f64),
    };
    paint_row(
        scene,
        text_system,
        hit_index,
        cur_y,
        pos_label,
        ids::INSP_TRANSFORM_POS_X,
        "X",
        ColorToken::Danger,
        pos_step,
        Some((
            ids::INSP_TRANSFORM_POS_Y,
            "Y",
            ColorToken::Success,
            pos_step,
        )),
    );
    cur_y += field_h + row_gap;
    paint_row(
        scene,
        text_system,
        hit_index,
        cur_y,
        "Rotation (°)",
        ids::INSP_TRANSFORM_ROT,
        "",
        ColorToken::Text3,
        1.0,
        None,
    );
    cur_y += field_h + row_gap;
    paint_row(
        scene,
        text_system,
        hit_index,
        cur_y,
        "Scale",
        ids::INSP_TRANSFORM_SCALE_X,
        "X",
        ColorToken::Danger,
        0.1, // LITERAL-PX-OK: scale NumberInput step (slider precision)
        Some((ids::INSP_TRANSFORM_SCALE_Y, "Y", ColorToken::Success, 0.1)), // LITERAL-PX-OK: scale NumberInput step
    );
    cur_y += field_h + Spacing::Xs.px();

    cur_y
}

/// M14.5 inspector phase (6.4/§9): paint the "Render Source" section
/// when a sprite entity is selected. Shows the entity name, world
/// size, source kind (Atlas / Hand-packed / Individual), source-image
/// pixels, and a "Reimport at current px/m" button that re-decodes
/// the source asset at the project's current `pixels_per_meter`.
///
/// Read-only display except for the Reimport button — the strategy
/// switcher is a later milestone (M14.5 follow-up); the picker shows
/// the current strategy without offering a swap so callers can already
/// see which storage backs each sprite.
#[allow(clippy::too_many_arguments)]
pub(in crate::screens::hero) fn paint_render_source_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorSpriteInfo,
) -> f32 {
    let line_font = TypeToken::Sm.px();
    let label_font = TypeToken::Xs.px();
    let row_gap = Spacing::Xs.px();
    let row_h = line_font + row_gap;
    // Section title.
    paint_text_title(
        text_system,
        scene,
        "Render Source",
        x,
        y,
        TypeToken::Md.px(),
        w,
        resolve(ColorToken::Text1, theme),
    );
    let mut cur_y = y + TypeToken::Md.px() + Spacing::Md.px();
    // Separator under the title.
    cur_y = paint_section_separator(scene, theme, x, w, cur_y);

    // Helper: paint "label · value" two-line row.
    let paint_pair = |scene: &mut VectorScene,
                      text_system: &mut TextSystem,
                      label: &str,
                      value: &str,
                      mut yy: f32|
     -> f32 {
        paint_text(
            text_system,
            scene,
            label,
            x,
            yy,
            label_font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        yy += label_font + 2.0;
        paint_text(
            text_system,
            scene,
            value,
            x,
            yy,
            line_font,
            w,
            resolve(ColorToken::Text1, theme),
        );
        yy + row_h + row_gap
    };

    // M14.E: "Name" and "World size" rows previously lived here.
    // They moved to the editable name TextInput at the top of the
    // Inspector body and the header subtitle, respectively. Render
    // Source now focuses on the actual storage strategy + identifier.
    // M14.C: 3-segment Strategy switcher. Each button is `Pressed`
    // when its strategy matches the current source_kind; the painter
    // computes this from the snapshot each frame so the buttons
    // always agree with the underlying ECS (no in-progress edit
    // state to worry about — click → host swap → snapshot
    // republishes → painter re-pins). HandPacked stays clickable
    // but the host shows a toast and skips the swap in v1.
    paint_text(
        text_system,
        scene,
        "Strategy",
        x,
        cur_y,
        label_font,
        w,
        resolve(ColorToken::Text3, theme),
    );
    cur_y += label_font + Spacing::Xs.px();
    let strategy_btn_h = ROW_H_PX;
    let strategy_gap = Spacing::Sm.px();
    let strategy_btn_w = ((w - strategy_gap * 2.0) / 3.0).max(40.0); // LITERAL-PX-OK: 3-segment strategy button minimum width (chrome dim) — divisor 3 is a count
    let strategy_buttons = [
        (
            ids::INSP_RENDER_STRATEGY_ATLAS,
            "Atlas",
            matches!(info.source_kind, InspectorSpriteSource::Atlas { .. }),
        ),
        (
            ids::INSP_RENDER_STRATEGY_INDIVIDUAL,
            "Individual",
            matches!(info.source_kind, InspectorSpriteSource::Individual { .. }),
        ),
        (
            ids::INSP_RENDER_STRATEGY_HANDPACKED,
            "Hand-packed",
            matches!(info.source_kind, InspectorSpriteSource::HandPacked),
        ),
    ];
    for (i, (id, label, pressed)) in strategy_buttons.into_iter().enumerate() {
        let bx = x + (strategy_btn_w + strategy_gap) * i as f32;
        let r = Rect::new(bx, cur_y, strategy_btn_w, strategy_btn_h);
        hit_index.register(id, r);
        // Driven from the snapshot — hover/normal/pressed otherwise
        // mirror the canonical button states.
        let state = if pressed {
            ButtonState::Pressed
        } else {
            store.button_state(id).unwrap_or(ButtonState::Normal)
        };
        let btn = Button::new(id, label)
            .kind(ButtonKind::Default)
            .state(state);
        paint_button(&btn, r, scene, text_system, theme);
    }
    cur_y += strategy_btn_h + Spacing::Md.px();
    // Storage detail (atlas key / texture id) — kept as a small
    // line under the switcher so the user can still see the
    // identifier without it cluttering the buttons.
    let storage_detail = match info.source_kind {
        InspectorSpriteSource::Atlas { key } => format!("Atlas key: {}", key),
        InspectorSpriteSource::Individual { texture_id } => {
            format!("Texture id: {}", texture_id)
        }
        InspectorSpriteSource::HandPacked => "Hand-packed (atlas asset)".to_string(),
    };
    cur_y = paint_pair(scene, text_system, "Storage", &storage_detail, cur_y);
    if let Some((pw, ph)) = info.source_pixels {
        let px_str = format!("{} × {} px", pw, ph);
        cur_y = paint_pair(scene, text_system, "Source", &px_str, cur_y);
    }

    // Pixel-format segmented picker — RGBA8 (default, supported) +
    // RGBA16 (disabled until the asset layer grows 16-bit storage).
    // Pressed = current choice; clicking the alternative flips the
    // pin via `pin_button_selection` in `apply_event`. Reimport
    // reads the pressed id at drain time.
    paint_text(
        text_system,
        scene,
        "Pixel format",
        x,
        cur_y,
        label_font,
        w,
        resolve(ColorToken::Text3, theme),
    );
    cur_y += label_font + Spacing::Xs.px();
    let btn_h = ROW_H_PX;
    let gap = Spacing::Sm.px();
    let half_w = (w - gap) * 0.5;
    let rgba8_rect = Rect::new(x, cur_y, half_w, btn_h);
    let rgba16_rect = Rect::new(x + half_w + gap, cur_y, half_w, btn_h);
    let rgba8_state = store
        .button_state(ids::INSP_RENDER_FORMAT_RGBA8)
        .unwrap_or(ButtonState::Pressed);
    // RGBA16 stays Disabled regardless of stored state until the
    // asset crate adds half-float decode — the click handler skips
    // pinning Disabled buttons, so the user can't even land on it.
    let rgba16_state = ButtonState::Disabled;
    hit_index.register(ids::INSP_RENDER_FORMAT_RGBA8, rgba8_rect);
    hit_index.register(ids::INSP_RENDER_FORMAT_RGBA16, rgba16_rect);
    let rgba8_btn = Button::new(ids::INSP_RENDER_FORMAT_RGBA8, "RGBA8")
        .kind(ButtonKind::Default)
        .state(rgba8_state);
    let rgba16_btn = Button::new(ids::INSP_RENDER_FORMAT_RGBA16, "RGBA16")
        .kind(ButtonKind::Default)
        .state(rgba16_state);
    paint_button(&rgba8_btn, rgba8_rect, scene, text_system, theme);
    paint_button(&rgba16_btn, rgba16_rect, scene, text_system, theme);
    cur_y += btn_h + Spacing::Md.px();

    // Reimport button — disabled when the snapshot says the source
    // doesn't resolve to a re-decodable asset (procedural / lost).
    let reimport_h = 30.0_f32; // LITERAL-PX-OK: Reimport button height (compact, distinct from ROW_H_PX)
    let btn_rect = Rect::new(x, cur_y, w, reimport_h);
    let id = ids::INSP_RENDER_SOURCE_REIMPORT;
    let state = if !info.can_reimport {
        ButtonState::Disabled
    } else {
        store.button_state(id).unwrap_or(ButtonState::Normal)
    };
    hit_index.register(id, btn_rect);
    let btn = Button::new(id, "Reimport at current px/m")
        .kind(ButtonKind::Default)
        .state(state);
    paint_button(&btn, btn_rect, scene, text_system, theme);
    cur_y + reimport_h + Spacing::Xs.px()
}
