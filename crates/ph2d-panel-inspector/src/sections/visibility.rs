//! Visibility — Inspector §8 section painter (Sprite Inspector v2 W3,
//! spec §3.8 / §6). Renders the optional-component controls BELOW the
//! always-on "Visible" toggle row (`paint_visibility_row`): the
//! `VisibilityLayer` 4×8 bitmask, the `ClipChildren` + `MaskInteraction`
//! segmented modes, the mask `alpha_cutoff` (when Mask != None), and the
//! `OnScreenEnabler` toggle + its Rect2 editor (when on). Snapshot-driven
//! like §9 Sampling; each control dispatches an `InspectorVisibilitySectionEdit`.

use super::*;
use ph2d_editor_core::screens::hero::InspectorVisibilitySectionInfo;
use ph2d_editor_core::widget::SectionFold;
use ph2d_editor_core::widget::{SegmentedAdaptive, SegmentedOption, paint_segmented_adaptive};

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
    let (control_w, dot) = ph2d_editor_core::widget::form_row_columns(x, w, row_y, h);
    let rect = Rect::new(x, row_y, control_w, h);
    hit_index.register(id, rect);
    let (state, value, buffer, caret, anchor) = read_number_input(store, id);
    let input = NumberInput::new(id, "", value)
        .step(0.1) // LITERAL-PX-OK: cutoff/rect step
        .visual((state, store.hover_live(id)));
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
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
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
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    ids3: [NodeId; 3],
    labels3: [&str; 3],
    // `None` = a seleção diverge; nenhum segmento acende.
    selected: Option<usize>,
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
    let (control_w, dot) = ph2d_editor_core::widget::form_row_columns(x, w, row_y, h);
    let rect = Rect::new(x, row_y, control_w, h);
    // ⚠️ **`SegmentedAdaptive` e não `Tabs`, para poder dizer «misto».** O `Tabs::selected()`
    // clampa (`idx.min(len-1)`), por isso é **incapaz** de renderizar «nenhum aceso» — e era isso
    // que fazia estas duas rows acenderem o valor da primária como se toda a seleção concordasse,
    // enquanto o host já calculava a divergência e a deitava fora
    // (auditoria `docs/Sprite_projeto/20` §3.3).
    let seg = SegmentedAdaptive::new(
        NodeId(0),
        label,
        ids3.iter()
            .zip(labels3)
            .map(|(&id, l)| SegmentedOption::new(id, l))
            .collect(),
    )
    .selected(selected.unwrap_or(usize::MAX));
    let seg_h = paint_segmented_adaptive(&seg, rect, scene, text_system, theme, store, hit_index);
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, dot);
    row_y + seg_h + Spacing::Sm.px()
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

    // Visibility Layer — collapsible sub-section using the CANONICAL
    // section header (a divider above + UPPERCASE title + chevron), so it
    // matches every other section. The 32-bit 4×8 grid is tall + advanced
    // (camera cull mask, not z-order), so it defaults COLLAPSED
    // (`set_collapsed` in `pre_populate`). Clicking the header toggles
    // `is_collapsed(INSP_VIS_LAYER_HEADER)` via `apply_click` (the id is
    // marked collapsible). Bit `n` = layer `n+1`; absent component → ALL.
    yy = paint_section_separator(scene, theme, x, w, yy);
    let layer_header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let layer_header = section_header(store, ids::INSP_VIS_LAYER_HEADER, "Visibility Layer")
        .open_t(store.section_open_live(ids::INSP_VIS_LAYER_HEADER));
    let layer_header_rect = Rect::new(x, yy, w, layer_header_h);
    paint_section_header(&layer_header, layer_header_rect, scene, text_system, theme);
    hit_index.register(ids::INSP_VIS_LAYER_HEADER, layer_header_rect);
    yy += layer_header_h;
    // ⚠️ A grade de 32 bits e' ALTA, entao ela e' a sub-seccao onde a dobra do corpo mais se ve'.
    // O `begin` devolve `None` so' quando a seccao esta' fechada **e parada** — e' ai' que o corpo
    // nao e' percorrido de todo, exactamente o `if layer_collapsed` de sempre.
    match SectionFold::begin(
        store,
        ids::INSP_VIS_LAYER_HEADER,
        x,
        w,
        yy,
        scene,
        hit_index,
    ) {
        None => yy += row_gap,
        Some(fold) => {
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
            let inner = yy + BitmaskGrid32::grid_height(h) + row_gap;
            yy = fold.finish(store, scene, hit_index, inner);
        }
    }

    // Clip Children — Disabled / ClipOnly / ClipAndDraw (tags 0/1/2).
    yy = segmented_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Clip Children",
        ids::INSP_VIS_CLIP,
        ["Disabled", "Clip", "Clip+Draw"],
        (!info.mixed.clip_mode).then_some(usize::from(info.clip_mode)),
    );

    // Mask Interaction — None / VisibleInside / VisibleOutside (0/1/2).
    yy = segmented_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Mask Interaction",
        ids::INSP_VIS_MASK,
        ["None", "Inside", "Outside"],
        (!info.mixed.mask_mode).then_some(usize::from(info.mask_mode)),
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
    let src_cb = Checkbox::new(ids::INSP_VIS_MASK_SOURCE, "Mask Source (Mask2D)")
        .visual(store.checkbox_visual(ids::INSP_VIS_MASK_SOURCE))
        .value(if info.mask_source {
            CheckboxValue::Checked
        } else {
            CheckboxValue::Unchecked
        });
    paint_checkbox(&src_cb, src_rect, scene, text_system, theme);
    yy += h + row_gap;

    // On-Screen Enabler toggle (presence of the component).
    let on_rect = Rect::new(x, yy, w, h);
    hit_index.register(ids::INSP_VIS_ON_SCREEN, on_rect);
    let on_cb = Checkbox::new(ids::INSP_VIS_ON_SCREEN, "On-Screen Enabler")
        .visual(store.checkbox_visual(ids::INSP_VIS_ON_SCREEN))
        .value(if info.on_screen {
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
        const RECT_STEP: f64 = 0.1; // LITERAL-PX-OK: enabler-rect editor nudge step
        let editor = Rect2Editor::new(
            ids::INSP_LIVE_VISIBILITY_SECTION,
            "Enabler Rect",
            NumberInput::new(ids::INSP_VIS_RECT_X, "", vx)
                .step(RECT_STEP)
                .visual((sx, store.hover_live(ids::INSP_VIS_RECT_X))),
            NumberInput::new(ids::INSP_VIS_RECT_Y, "", vy)
                .step(RECT_STEP)
                .visual((sy, store.hover_live(ids::INSP_VIS_RECT_Y))),
            NumberInput::new(ids::INSP_VIS_RECT_W, "", vw)
                .step(RECT_STEP)
                .visual((sw, store.hover_live(ids::INSP_VIS_RECT_W))),
            NumberInput::new(ids::INSP_VIS_RECT_H, "", vh)
                .step(RECT_STEP)
                .visual((sh, store.hover_live(ids::INSP_VIS_RECT_H))),
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
