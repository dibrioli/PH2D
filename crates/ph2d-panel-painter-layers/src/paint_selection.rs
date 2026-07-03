//! The **Selection** panel section (ADR-0103). Mode-exclusive: when the Painter is in Selection mode the
//! Brush body paints ONLY this section (nothing shared with the other tools — the Inpaint precedent).
//!
//! Layout (Widget-Gallery components, all reflow on a narrow panel via `SegmentedAdaptive`):
//! `SELECTION` title · **Mode** toggle group (Automatic / Freehand / Rectangle / Ellipse) · **Operation**
//! toggle group (New / Add / Remove) · **Threshold** slider (Automatic only) · **Feather** slider ·
//! **Edit Selection** toggle · **Actions** group (Invert / Clear). Every interactive id is registered in
//! `populate` and forwarded by `event.rs` over the frozen `PanelEvent` channel to `route_selection_event`.

use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::showcase::paint_section_separator;
use ph2d_editor_core::widget::{
    SectionHeader, SegmentedAdaptive, SegmentedOption, paint_section_header, paint_segmented_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::{ROW_H_PX, Spacing, TypeToken};
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
    let mut y = title(ctx, theme, x, content_w, y, "SELECTION");

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

    // Boolean operator — New / Add / Remove.
    y = seg_group(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_SEL_OP,
        "Boolean operation",
        &[
            (core_ids::PAINTER_SEL_OP_NEW, "New"),
            (core_ids::PAINTER_SEL_OP_ADD, "Add"),
            (core_ids::PAINTER_SEL_OP_REMOVE, "Remove"),
        ],
        brush.selection_op as usize,
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

    y = paint_section_separator(ctx.scene, theme, x, content_w, y);

    // Edit Selection — enter the editable-Shape boundary (handles / gizmos / offset — Wave EDIT).
    y = paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_SEL_EDIT,
        "Edit Selection (handles)",
        brush.selection_edit,
    );

    // Actions — momentary Invert / Clear (a button group; no persistent selection).
    y = seg_group(
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

    y
}

/// A non-interactive ALL-CAPS section title (visual only; not hit-indexed).
fn title(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme, x: f32, content_w: f32, y: f32, label: &str) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px();
    let header = SectionHeader::new(core_ids::PAINTER_SEL_MODE, label).collapsible(false);
    let rect = Rect::new(x, y, content_w, header_h);
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    paint_section_header(&header, rect, scene, text_system, theme);
    y + header_h + Spacing::Xs.px()
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
            .chain([core_ids::PAINTER_SEL_FEATHER_SLIDER, core_ids::PAINTER_SEL_EDIT].iter())
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
}
