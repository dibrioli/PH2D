//! The **Selection** panel section (ADR-0103). Mode-exclusive: when the Painter is in Selection mode the
//! Brush body paints ONLY this section (nothing shared with the other tools — the Inpaint precedent).
//!
//! Layout (Widget-Gallery components, all reflow on a narrow panel via `SegmentedAdaptive`):
//! `SELECTION` title · **Mode** toggle group (Automatic / Freehand / Rectangle / Ellipse) · **Operation**
//! toggle group (New / Add / Remove) · **Threshold** slider (Automatic only) · **Feather** slider ·
//! **Edit Selection** toggle · **Actions** group (Invert / Clear). Every interactive id is registered in
//! `populate` and forwarded by `event.rs` over the frozen `PanelEvent` channel to `route_selection_event`.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::showcase::paint_section_separator;
use ph2d_editor_core::widget::{
    Card, SegmentedAdaptive, SegmentedOption, measure_segmented_adaptive, paint_card,
    paint_segmented_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};
use ph2d_tool_painter::BrushSettings;

use crate::paint_brush_top::{paint_checkbox_row, paint_slider_chip_row};

/// Paint the mode-exclusive Selection section, returning the next `y`.
pub(crate) fn paint_selection_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    // Operation — New / Add / Remove, in a card at the TOP of the section (above the other controls).
    let mut y = operation_card(ctx, theme, x, content_w, y, brush.selection_op as usize);

    // Mode picker — a segmented Toggle group; the selected segment mirrors `selection_mode`.
    y = seg_group(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_SEL_MODE,
        "Selection mode",
        &[
            (core_ids::PAINTER_SEL_MODE_AUTO, "Auto"),
            (core_ids::PAINTER_SEL_MODE_FREEHAND, "Free"),
            (core_ids::PAINTER_SEL_MODE_RECT, "Rect"),
            (core_ids::PAINTER_SEL_MODE_ELLIPSE, "Ellipse"),
        ],
        brush.selection_mode as usize,
    );

    // Automatic threshold (only meaningful in Automatic mode).
    if brush.selection_mode == 0 {
        y = paint_slider_chip_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Threshold",
            core_ids::PAINTER_SEL_THRESHOLD_SLIDER,
            core_ids::PAINTER_SEL_THRESHOLD_CHIP,
            brush.selection_threshold,
        );
    }

    // Stabilization — the lasso path smoothing (only meaningful in Freehand mode).
    if brush.selection_mode == 1 {
        y = paint_slider_chip_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Stabilization",
            core_ids::PAINTER_SEL_STABILIZE_SLIDER,
            core_ids::PAINTER_SEL_STABILIZE_CHIP,
            brush.selection_stabilizer,
        );
    }

    // Feather — softens the selection edge.
    y = paint_slider_chip_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Feather",
        core_ids::PAINTER_SEL_FEATHER_SLIDER,
        core_ids::PAINTER_SEL_FEATHER_CHIP,
        brush.selection_feather,
    );

    // Overlay opacity — how strongly the deselected-area hatching reads (a view preference).
    y = paint_slider_chip_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Overlay",
        core_ids::PAINTER_SEL_OPACITY_SLIDER,
        core_ids::PAINTER_SEL_OPACITY_CHIP,
        brush.selection_overlay_opacity,
    );

    // Offset — grow/shrink the whole selection; after Apply & Keep it sweeps concentric alternating
    // protected / paint bands (ADR-0103 Am.3). Slider + its two commit buttons, in a helper for the fn cap.
    y = offset_controls(ctx, theme, x, content_w, y, brush.selection_offset);

    y = paint_section_separator(ctx.scene, theme, x, content_w, y);

    // Edit Gizmos — reveal EVERY isolated shape gizmo at once, each manipulable (ADR-0103 Am.2 v2). It
    // does NOT bake a curve; the Convert button does that.
    y = paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_SEL_EDIT,
        "Edit Gizmos",
        brush.selection_edit,
    );

    // Gizmo-mode ops: Convert-to-Curve (merge all shapes → one editable curve) + Simplify (reduce its points).
    if brush.selection_edit {
        y = seg_group(
            ctx,
            theme,
            x,
            content_w,
            y,
            NodeId(0),
            "Convert / simplify selection curve",
            &[
                (core_ids::PAINTER_SEL_CONVERT, "Convert to Curve"),
                (core_ids::PAINTER_SEL_SIMPLIFY, "Simplify Curve"),
            ],
            usize::MAX,
        );
    }

    // Momentary action rows (Invert / Clear + Wave-5 content actions) — split out for the per-fn LOC cap.
    action_rows(ctx, theme, x, content_w, y)
}

/// Paint the momentary action button rows (Invert / Clear, then Layer / Fill / Copy / Paste); returns `y`.
fn action_rows(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
) -> f32 {
    let y = seg_group(
        ctx,
        theme,
        x,
        content_w,
        y,
        NodeId(0), // group a11y id unused for a momentary action row
        "Selection actions",
        &[
            (core_ids::PAINTER_SEL_INVERT, "Invert"),
            (core_ids::PAINTER_SEL_CLEAR, "Clear"),
        ],
        usize::MAX, // none selected — these are actions, not a radio
    );
    seg_group(
        ctx,
        theme,
        x,
        content_w,
        y,
        NodeId(0),
        "Selection content actions",
        &[
            (core_ids::PAINTER_SEL_LAYER_CONTENTS, "Layer"),
            (core_ids::PAINTER_SEL_FILL, "Fill"),
            (core_ids::PAINTER_SEL_COPY, "Copy"),
            (core_ids::PAINTER_SEL_PASTE, "Paste"),
        ],
        usize::MAX,
    )
}

/// Paint the **Operation** card (New / Add / Remove) — a framed surface with a title, at the top of the
/// section. Returns the next `y`. The segmented group inside reflows on narrow panels.
fn operation_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    selected: usize,
) -> f32 {
    let header_h = Spacing::Xl3.px();
    let pad = Spacing::Lg.px();
    let card = Card::new(core_ids::PAINTER_SEL_OP_CARD).title("OPERATION");
    let seg = SegmentedAdaptive::new(
        core_ids::PAINTER_SEL_OP,
        "Boolean operation",
        vec![
            SegmentedOption::new(core_ids::PAINTER_SEL_OP_NEW, "New"),
            SegmentedOption::new(core_ids::PAINTER_SEL_OP_ADD, "Add"),
            SegmentedOption::new(core_ids::PAINTER_SEL_OP_REMOVE, "Remove"),
        ],
    )
    .selected(selected);
    // Card body width is height-INDEPENDENT — probe it, then MEASURE the segmented group's reflowed height
    // so the card grows to fit New/Add/Remove when they wrap onto extra rows on a narrow panel (the card
    // used to clip "Remove"). The card height = header + padding + the measured segmented height.
    let probe_body = card.body_rect(Rect::new(x, y, content_w, header_h + pad * 2.0 + ROW_H_PX));
    let seg_h = measure_segmented_adaptive(&seg, probe_body.w, ROW_H_PX, ctx.text_system);
    let card_h = header_h + pad * 2.0 + seg_h;
    let card_rect = Rect::new(x, y, content_w, card_h);
    {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        paint_card(&card, card_rect, scene, text_system, theme);
    }
    let body = card.body_rect(card_rect);
    {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        let hit_index = ctx.host.hit_index_mut();
        paint_segmented_adaptive(
            &seg,
            Rect::new(body.x, body.y, body.w, ROW_H_PX),
            scene,
            text_system,
            theme,
            hit_index,
        );
    }
    y + card_h + Spacing::Sm.px()
}

/// Paint the **Offset** row (slider + Apply / Apply & Keep buttons); returns next `y`. Split out of
/// `paint_selection_section` for the per-fn LOC cap (ADR-0103 Am.3).
fn offset_controls(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    offset: f32,
) -> f32 {
    let y = paint_slider_chip_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Offset",
        core_ids::PAINTER_SEL_OFFSET_SLIDER,
        core_ids::PAINTER_SEL_OFFSET_CHIP,
        offset,
    );
    seg_group(
        ctx,
        theme,
        x,
        content_w,
        y,
        NodeId(0),
        "Selection offset commit",
        &[
            (core_ids::PAINTER_SEL_OFFSET_APPLY, "Apply"),
            (core_ids::PAINTER_SEL_OFFSET_APPLY_KEEP, "Apply & Keep"),
        ],
        usize::MAX,
    )
}

/// Build + paint one adaptive segmented Toggle group and register its per-segment hits; returns next `y`.
/// Reflows onto extra rows at narrow widths (tablet-safe).
#[allow(clippy::too_many_arguments)]
fn seg_group(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    group_id: NodeId,
    a11y: &str,
    options: &[(NodeId, &str)],
    selected: usize,
) -> f32 {
    let opts: Vec<SegmentedOption> = options
        .iter()
        .map(|(id, label)| SegmentedOption::new(*id, *label))
        .collect();
    let seg = SegmentedAdaptive::new(group_id, a11y, opts).selected(selected);
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let hit_index = ctx.host.hit_index_mut();
    let used = paint_segmented_adaptive(
        &seg,
        Rect::new(x, y, content_w, ROW_H_PX),
        scene,
        text_system,
        theme,
        hit_index,
    );
    y + used + Spacing::Xs.px()
}

#[cfg(test)]
mod tests {
    use ph2d_a11y::NodeId;
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::panel::{PaintCtx, PanelHostInternal};
    use ph2d_editor_core::screens::HeroLayout;
    use ph2d_editor_core::zones::Rect;
    use ph2d_text::TextSystem;
    use ph2d_tokens::Theme;
    use ph2d_ui_testkit::MockPanelHost;
    use ph2d_vector::VectorScene;

    /// Drive the real `paint_brush_body` with `brush` published as the frame snapshot and return every
    /// widget id that registered a hit rect (the observable seam: painted ⟹ hit-indexed).
    fn body_hit_ids(brush: ph2d_tool_painter::BrushSettings) -> Vec<NodeId> {
        crate::state::set_current_brush(Some(brush));
        let mut host = MockPanelHost::with_panel::<crate::PainterLayersPanel>();
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let viewport = Rect::new(0.0, 0.0, 360.0, 8000.0);
        let layout = HeroLayout::for_viewport(viewport);
        let rect = Rect::new(0.0, 0.0, 320.0, 8000.0);
        {
            let mut ctx = PaintCtx {
                host: &mut host,
                layout: &layout,
                viewport,
                scene: &mut scene,
                text_system: &mut text,
            };
            crate::paint_brush::paint_brush_body(&mut ctx, Theme::default(), rect, 0.0);
        }
        crate::state::set_current_brush(None);
        host.hit_index_mut()
            .iter_registrations()
            .map(|(id, _)| id)
            .collect()
    }

    fn selection_brush() -> ph2d_tool_painter::BrushSettings {
        ph2d_tool_painter::BrushSettings {
            is_selection: true,
            ..crate::paint_brush::FALLBACK_BRUSH
        }
    }

    #[test]
    fn selection_mode_paints_only_selection_controls() {
        let ids = body_hit_ids(selection_brush());
        for sid in core_ids::PAINTER_SEL_MODE_IDS
            .iter()
            .chain(core_ids::PAINTER_SEL_OP_IDS.iter())
            .chain(core_ids::PAINTER_SEL_ACTION_IDS.iter())
            .chain(
                [
                    core_ids::PAINTER_SEL_FEATHER_SLIDER,
                    core_ids::PAINTER_SEL_EDIT,
                ]
                .iter(),
            )
        {
            assert!(
                ids.contains(sid),
                "selection control {sid:?} not painted in Selection mode"
            );
        }
        // Mode-exclusive: no shared brush control leaks in.
        for bid in [
            core_ids::PAINTER_BRUSH_SIZE_SLIDER,
            core_ids::PAINTER_BRUSH_STRENGTH_SLIDER,
            core_ids::PAINTER_BRUSH_SYNC,
        ] {
            assert!(
                !ids.contains(&bid),
                "brush control {bid:?} leaked into Selection mode"
            );
        }
    }

    #[test]
    fn threshold_shows_in_automatic_and_hides_in_rectangle() {
        assert!(
            body_hit_ids(selection_brush()).contains(&core_ids::PAINTER_SEL_THRESHOLD_SLIDER),
            "Threshold shows in Automatic (default mode 0)"
        );
        let rect_mode = ph2d_tool_painter::BrushSettings {
            selection_mode: 2,
            ..selection_brush()
        };
        assert!(
            !body_hit_ids(rect_mode).contains(&core_ids::PAINTER_SEL_THRESHOLD_SLIDER),
            "Threshold is hidden outside Automatic"
        );
    }

    #[test]
    fn offset_slider_and_commit_buttons_paint_in_selection_mode() {
        let ids = body_hit_ids(selection_brush());
        for oid in [
            core_ids::PAINTER_SEL_OFFSET_SLIDER,
            core_ids::PAINTER_SEL_OFFSET_APPLY,
            core_ids::PAINTER_SEL_OFFSET_APPLY_KEEP,
        ] {
            assert!(
                ids.contains(&oid),
                "offset control {oid:?} not painted in Selection mode"
            );
        }
    }

    #[test]
    fn stabilization_shows_in_freehand_only() {
        let free = ph2d_tool_painter::BrushSettings {
            selection_mode: 1, // Freehand
            ..selection_brush()
        };
        assert!(
            body_hit_ids(free).contains(&core_ids::PAINTER_SEL_STABILIZE_SLIDER),
            "Stabilization shows in Freehand mode"
        );
        assert!(
            !body_hit_ids(selection_brush()).contains(&core_ids::PAINTER_SEL_STABILIZE_SLIDER),
            "Stabilization is hidden outside Freehand (default mode 0 = Automatic)"
        );
    }
}
