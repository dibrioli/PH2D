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

    // Visibility Layer — 32-bit mask as a 4-col × 8-row checkbox grid
    // (canonical BitmaskGrid32 widget). Bit `n` = layer `n+1`; absent
    // component → all 32 set (ALL).
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
    let grid = BitmaskGrid32::new(
        ids::INSP_LIVE_VISIBILITY_SECTION,
        "Visibility Layer",
        ids::INSP_VIS_LAYER_BIT,
        info.layer_mask,
    );
    for (bit, id) in ids::INSP_VIS_LAYER_BIT.iter().enumerate() {
        hit_index.register(*id, BitmaskGrid32::cell_rect(x, yy, w, h, bit));
    }
    paint_bitmask_grid32(&grid, x, yy, w, h, scene, text_system, theme);
    yy += BitmaskGrid32::grid_height(h) + row_gap;

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

    // Enabler Rect — canonical Rect2Editor (X/Y/W/H in one row), only
    // when the enabler is on.
    if info.on_screen {
        paint_text(
            text_system,
            scene,
            "Enabler Rect",
            x,
            yy + (label_h - label_font) * 0.5,
            label_font,
            w,
            label_color,
        );
        yy += label_h;
        let (sx, vx, bx, cx, ax) = read_number_input(store, ids::INSP_VIS_RECT_X);
        let (sy, vy, by, cy, ay) = read_number_input(store, ids::INSP_VIS_RECT_Y);
        let (sw, vw, bw, cw, aw) = read_number_input(store, ids::INSP_VIS_RECT_W);
        let (sh, vh, bh, ch, ah) = read_number_input(store, ids::INSP_VIS_RECT_H);
        let editor = Rect2Editor::new(
            ids::INSP_LIVE_VISIBILITY_SECTION,
            "Enabler Rect",
            NumberInput::new(ids::INSP_VIS_RECT_X, "", vx)
                .step(0.1)
                .state(sx), // LITERAL-PX-OK: rect editor step
            NumberInput::new(ids::INSP_VIS_RECT_Y, "", vy)
                .step(0.1)
                .state(sy), // LITERAL-PX-OK: rect editor step
            NumberInput::new(ids::INSP_VIS_RECT_W, "", vw)
                .step(0.1)
                .state(sw), // LITERAL-PX-OK: rect editor step
            NumberInput::new(ids::INSP_VIS_RECT_H, "", vh)
                .step(0.1)
                .state(sh), // LITERAL-PX-OK: rect editor step
        )
        // 2×2 grid: the Inspector column is too narrow for four number
        // inputs in one row (each would fall below NumberInput's usable
        // minimum width).
        .layout(Rect2Layout::Grid2x2);
        let editor_h = Rect2Editor::preferred_height(Rect2Layout::Grid2x2, h);
        let host = Rect::new(x, yy, w, editor_h);
        let field_rects = editor.field_rects(host);
        for (fr, id) in field_rects.iter().zip([
            ids::INSP_VIS_RECT_X,
            ids::INSP_VIS_RECT_Y,
            ids::INSP_VIS_RECT_W,
            ids::INSP_VIS_RECT_H,
        ]) {
            hit_index.register(id, *fr);
        }
        paint_rect2_editor_with_state(
            &editor,
            [Some(bx), Some(by), Some(bw), Some(bh)],
            [cx, cy, cw, ch],
            [ax, ay, aw, ah],
            host,
            scene,
            text_system,
            theme,
        );
        yy += editor_h + row_gap;
    }

    yy + SECTION_BOTTOM_PAD_PX
}
