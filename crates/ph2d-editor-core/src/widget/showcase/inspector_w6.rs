//! Showcase `paint_inspector_w6_section` painter — the Widget Gallery
//! reference for the Sprite Inspector v2 W6 foundational widgets
//! (spec §15.7): [`Rect2Editor`], [`BitmaskGrid32`],
//! [`NumericInputWithUnit`], [`VariantEditor`], [`KeyValueList`],
//! [`SegmentedAdaptive`].
//!
//! The four scalar widgets (Rect2Editor, NumericInputWithUnit,
//! BitmaskGrid32, SegmentedAdaptive) are wired live against the store
//! samples seeded in `pre_populate`. The two composite editors
//! (VariantEditor, KeyValueList) are rendered as visual references —
//! their interaction model belongs to the consuming Inspector sections
//! (InstanceShaderParams in W4, NamedAnchor user_data in W5), not the
//! gallery.

use super::*;
use crate::widget::panel_chrome::{SECTION_BOTTOM_PAD_PX, SECTION_LABEL_TO_CONTROL_PX};
use crate::widget::{
    BitmaskGrid32, KeyValueEntry, KeyValueList, NumberInput, NumericInputWithUnit, Rect2Editor,
    Rect2Layout, SegmentedAdaptive, SegmentedOption, Unit, VariantEditor, VariantValue,
    paint_bitmask_grid32, paint_key_value_list, paint_numeric_input_with_unit,
    paint_rect2_editor_with_state, paint_segmented_adaptive, paint_variant_editor,
};
use ph2d_tokens::ColorToken;

const W6_FIELD_H: f32 = FIELD_H;

/// Group ids for the SegmentedAdaptive demo — clicking one pins the
/// selection (handled by `apply_showcase_event`).
pub const W6_SEG_IDS: [NodeId; 4] = ids::INSP_SAMPLE_W6_SEG;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_inspector_w6_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (mut y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_W6,
        ids::INSP_SECTION_W6_COLOR,
        "Inspector v2 (W6)",
        6,
    );
    if !open {
        return y;
    }
    let label_font = TypeToken::Sm.px();
    let label_h = label_font + Spacing::Xs.px();

    // ── Rect2Editor ─────────────────────────────────────────────────
    y = mini_label(scene, text_system, theme, x, w, y, "Rect2Editor", label_h, label_font);
    let (sx, vx, bx, cx, ax) = read_number_input(store, ids::INSP_SAMPLE_W6_RECT[0]);
    let (sy, vy, by, cy, ay) = read_number_input(store, ids::INSP_SAMPLE_W6_RECT[1]);
    let (sw, vw, bw, cw, aw) = read_number_input(store, ids::INSP_SAMPLE_W6_RECT[2]);
    let (sh, vh, bh, ch, ah) = read_number_input(store, ids::INSP_SAMPLE_W6_RECT[3]);
    let rect2 = Rect2Editor::new(
        ids::INSP_SECTION_W6,
        "Rect",
        NumberInput::new(ids::INSP_SAMPLE_W6_RECT[0], "", vx).state(sx),
        NumberInput::new(ids::INSP_SAMPLE_W6_RECT[1], "", vy).state(sy),
        NumberInput::new(ids::INSP_SAMPLE_W6_RECT[2], "", vw).state(sw),
        NumberInput::new(ids::INSP_SAMPLE_W6_RECT[3], "", vh).state(sh),
    )
    .layout(Rect2Layout::Grid2x2);
    let rect2_h = Rect2Editor::preferred_height(Rect2Layout::Grid2x2, W6_FIELD_H);
    let rect2_host = Rect::new(x, y, w, rect2_h);
    for (i, fr) in rect2.field_rects(rect2_host).iter().enumerate() {
        hit_index.register(ids::INSP_SAMPLE_W6_RECT[i], *fr);
    }
    paint_rect2_editor_with_state(
        &rect2,
        [Some(bx), Some(by), Some(bw), Some(bh)],
        [cx, cy, cw, ch],
        [ax, ay, aw, ah],
        rect2_host,
        scene,
        text_system,
        theme,
    );
    y += rect2_h + ROW_GAP;

    // ── NumericInputWithUnit ────────────────────────────────────────
    y = mini_label(scene, text_system, theme, x, w, y, "NumericInputWithUnit (deg)", label_h, label_font);
    let (us, uv, ub, uc, ua) = read_number_input(store, ids::INSP_SAMPLE_W6_UNIT);
    let unit_widget = NumericInputWithUnit::new(
        NumberInput::new(ids::INSP_SAMPLE_W6_UNIT, "", uv).state(us),
        Unit::Degrees,
    );
    let unit_host = Rect::new(x, y, w, W6_FIELD_H);
    hit_index.register(ids::INSP_SAMPLE_W6_UNIT, unit_widget.input_rect(unit_host));
    paint_numeric_input_with_unit(
        &unit_widget,
        Some(ub),
        uc,
        ua,
        unit_host,
        scene,
        text_system,
        theme,
    );
    y += W6_FIELD_H + ROW_GAP;

    // ── BitmaskGrid32 ───────────────────────────────────────────────
    y = mini_label(scene, text_system, theme, x, w, y, "BitmaskGrid32", label_h, label_font);
    let mut mask = 0u32;
    for (bit, id) in ids::INSP_SAMPLE_W6_MASK.iter().enumerate() {
        let (_state, value) = store
            .checkbox(*id)
            .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
        if value == CheckboxValue::Checked {
            mask |= 1 << bit as u32;
        }
    }
    let cell_h = Density::Compact.row_h_px();
    let grid = BitmaskGrid32::new(ids::INSP_SECTION_W6, "Visibility Layer", ids::INSP_SAMPLE_W6_MASK, mask);
    for (bit, id) in ids::INSP_SAMPLE_W6_MASK.iter().enumerate() {
        hit_index.register(*id, BitmaskGrid32::cell_rect(x, y, w, cell_h, bit));
    }
    paint_bitmask_grid32(&grid, x, y, w, cell_h, scene, text_system, theme);
    y += BitmaskGrid32::grid_height(cell_h) + ROW_GAP;

    // ── SegmentedAdaptive ───────────────────────────────────────────
    y = mini_label(scene, text_system, theme, x, w, y, "SegmentedAdaptive (9-slice modes)", label_h, label_font);
    let selected = active_index(store, &W6_SEG_IDS).unwrap_or(0);
    let seg = SegmentedAdaptive::new(
        ids::INSP_SECTION_W6,
        "Draw Mode",
        vec![
            SegmentedOption::new(W6_SEG_IDS[0], "Simple"),
            SegmentedOption::new(W6_SEG_IDS[1], "Sliced"),
            SegmentedOption::new(W6_SEG_IDS[2], "Tiled"),
            SegmentedOption::new(W6_SEG_IDS[3], "Tiled Fit"),
        ],
    )
    .selected(selected);
    let seg_h = paint_segmented_adaptive(
        &seg,
        Rect::new(x, y, w, W6_FIELD_H),
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += seg_h + ROW_GAP;

    // ── VariantEditor (visual reference) ────────────────────────────
    y = mini_label(scene, text_system, theme, x, w, y, "VariantEditor (recursive, depth \u{2264}4)", label_h, label_font);
    let variant = VariantEditor::new(
        ids::INSP_SAMPLE_W6_VARIANT,
        "user_data",
        VariantValue::Dict(vec![
            ("hue_shift".to_string(), VariantValue::Float(0.25)), // LITERAL-PX-OK: showcase demo seed value
            ("label".to_string(), VariantValue::Str("muzzle".to_string())),
            ("tint".to_string(), VariantValue::Color([200, 30, 30, 255])),
        ]),
    );
    let variant_rows = variant.rows().len();
    paint_variant_editor(
        &variant,
        Rect::new(x, y, w, W6_FIELD_H * variant_rows as f32),
        W6_FIELD_H,
        scene,
        text_system,
        theme,
    );
    y += W6_FIELD_H * variant_rows as f32 + ROW_GAP;

    // ── KeyValueList (visual reference) ─────────────────────────────
    y = mini_label(scene, text_system, theme, x, w, y, "KeyValueList", label_h, label_font);
    let kv = KeyValueList::new(
        ids::INSP_SECTION_W6,
        "Shader Params",
        vec![KeyValueEntry::new(
            ids::INSP_SAMPLE_W6_KV_KEY,
            ids::INSP_SAMPLE_W6_KV_VAL,
            ids::INSP_SAMPLE_W6_KV_REMOVE,
            "hue_shift",
            "0.25", // LITERAL-PX-OK: showcase demo value preview string

        )],
        ids::INSP_SAMPLE_W6_KV_ADD,
    );
    let kv_host = Rect::new(x, y, w, kv.total_height(W6_FIELD_H));
    paint_key_value_list(&kv, kv_host, W6_FIELD_H, scene, text_system, theme);
    y += kv.total_height(W6_FIELD_H);

    y + SECTION_BOTTOM_PAD_PX
}

/// Short label row above a control, returning the next `y`.
#[allow(clippy::too_many_arguments)]
fn mini_label(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    label_h: f32,
    label_font: f32,
) -> f32 {
    paint_text(
        text_system,
        scene,
        label,
        x,
        y + (label_h - label_font) * 0.5,
        label_font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    y + label_h + SECTION_LABEL_TO_CONTROL_PX
}
