//! Visibility — Inspector §8 section painter (Sprite Inspector v2 W3,
//! spec §3.8 / §6). Renders the optional-component controls BELOW the
//! always-on "Visible" toggle row (`paint_visibility_row`): the
//! `VisibilityLayer` 4×8 bitmask, the `ClipChildren` + `MaskInteraction`
//! segmented modes, the mask `alpha_cutoff` (when Mask != None), and the
//! `OnScreenEnabler` toggle + its Rect2 editor (when on). Snapshot-driven
//! like §9 Sampling; each control dispatches an `InspectorVisibilitySectionEdit`.

use super::*;
use ph2d_editor_core::screens::hero::InspectorVisibilitySectionInfo;

/// Label-above row with a single NumberInput. Returns the next `y`.
/// Mirrors §9 Sampling's `uv_pair_row` but for one value (cutoff, a rect
/// component).
#[allow(clippy::too_many_arguments)]
fn number_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    id: NodeId,
) -> f32 {
    let h = ROW_H_PX;
    let label_font = TypeToken::Sm.px();
    let label_h = label_font + Spacing::Xs.px();
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
    let row_y = y + label_h;
    let rect = Rect::new(x, row_y, w, h);
    hit_index.register(id, rect);
    let (state, value, buffer, caret, anchor) = read_number_input(store, id);
    let input = NumberInput::new(id, "", value).step(0.1).state(state); // LITERAL-PX-OK: cutoff/rect step
    paint_number_input_with_buffer(
        &input,
        Some(buffer),
        caret,
        anchor,
        rect,
        scene,
        text_system,
        theme,
    );
    row_y + h + Spacing::Sm.px()
}

/// Paint a 3-tab segmented control (label above), registering each tab's
/// hit rect. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
fn segmented_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    ids3: [NodeId; 3],
    labels3: [&str; 3],
    selected: usize,
) -> f32 {
    let h = ROW_H_PX;
    let label_font = TypeToken::Sm.px();
    let label_h = label_font + Spacing::Xs.px();
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
    let row_y = y + label_h;
    let rect = Rect::new(x, row_y, w, h);
    let tabs = Tabs::new(
        NodeId(0),
        "",
        vec![
            TabItem::new(ids3[0], labels3[0]),
            TabItem::new(ids3[1], labels3[1]),
            TabItem::new(ids3[2], labels3[2]),
        ],
    )
    .variant(TabsVariant::Segmented)
    .selected(selected.min(2));
    paint_tabs(&tabs, rect, scene, text_system, theme);
    for (i, item) in tabs.items.iter().enumerate() {
        hit_index.register(item.id, tabs.tab_rect(rect, i));
    }
    row_y + h + Spacing::Sm.px()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_visibility_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorVisibilitySectionInfo,
) -> f32 {
    let h = ROW_H_PX;
    let row_gap = Spacing::Sm.px();
    let label_font = TypeToken::Sm.px();
    let label_color = resolve(ColorToken::Text2, theme);
    let label_h = label_font + Spacing::Xs.px();
    let mut yy = y;

    // Visibility Layer — 32-bit mask as a 4-col × 8-row checkbox grid.
    // Bit `n` = layer `n+1`; absent component → all 32 set (ALL).
    paint_text(
        text_system,
        scene,
        "Visibility Layer",
        x,
        yy + (label_h - label_font) * 0.5,
        label_font,
        w,
        label_color,
    );
    yy += label_h;
    let cols = 4usize;
    let rows = 32 / cols;
    let cell_w = w / cols as f32;
    for bit in 0..32usize {
        let col = bit % cols;
        let row = bit / cols;
        let rect = Rect::new(x + cell_w * col as f32, yy + h * row as f32, cell_w, h);
        hit_index.register(ids::INSP_VIS_LAYER_BIT[bit], rect);
        let checked = (info.layer_mask >> bit as u32) & 1 == 1;
        let cb =
            Checkbox::new(ids::INSP_VIS_LAYER_BIT[bit], format!("{}", bit + 1)).value(if checked {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            });
        paint_checkbox(&cb, rect, scene, text_system, theme);
    }
    yy += h * rows as f32 + row_gap;

    // Clip Children — Disabled / ClipOnly / ClipAndDraw (tags 0/1/2).
    yy = segmented_row(
        scene,
        text_system,
        theme,
        hit_index,
        x,
        w,
        yy,
        "Clip Children",
        ids::INSP_VIS_CLIP,
        ["Disabled", "Clip", "Clip+Draw"],
        info.clip_mode as usize,
    );

    // Mask Interaction — None / VisibleInside / VisibleOutside (0/1/2).
    yy = segmented_row(
        scene,
        text_system,
        theme,
        hit_index,
        x,
        w,
        yy,
        "Mask Interaction",
        ids::INSP_VIS_MASK,
        ["None", "Inside", "Outside"],
        info.mask_mode as usize,
    );

    // Mask alpha cutoff — only meaningful when the sprite obeys a mask.
    if info.mask_mode != 0 {
        yy = number_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            "Mask Alpha Cutoff",
            ids::INSP_VIS_ALPHA_CUTOFF,
        );
    }

    // Mask Source toggle — makes this sprite a Mask2D source (its silhouette
    // masks sibling VisibleInside/Outside responders).
    let src_rect = Rect::new(x, yy, w, h);
    hit_index.register(ids::INSP_VIS_MASK_SOURCE, src_rect);
    let src_cb = Checkbox::new(ids::INSP_VIS_MASK_SOURCE, "Mask Source (Mask2D)").value(
        if info.mask_source {
            CheckboxValue::Checked
        } else {
            CheckboxValue::Unchecked
        },
    );
    paint_checkbox(&src_cb, src_rect, scene, text_system, theme);
    yy += h + row_gap;

    // On-Screen Enabler toggle (presence of the component).
    let on_rect = Rect::new(x, yy, w, h);
    hit_index.register(ids::INSP_VIS_ON_SCREEN, on_rect);
    let on_cb =
        Checkbox::new(ids::INSP_VIS_ON_SCREEN, "On-Screen Enabler").value(if info.on_screen {
            CheckboxValue::Checked
        } else {
            CheckboxValue::Unchecked
        });
    paint_checkbox(&on_cb, on_rect, scene, text_system, theme);
    yy += h + row_gap;

    // Rect2 editor (collapsible inner — only when the enabler is on).
    if info.on_screen {
        for (label, id) in [
            ("Enabler Rect X", ids::INSP_VIS_RECT_X),
            ("Enabler Rect Y", ids::INSP_VIS_RECT_Y),
            ("Enabler Rect W", ids::INSP_VIS_RECT_W),
            ("Enabler Rect H", ids::INSP_VIS_RECT_H),
        ] {
            yy = number_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                yy,
                label,
                id,
            );
        }
    }

    yy + SECTION_BOTTOM_PAD_PX
}
