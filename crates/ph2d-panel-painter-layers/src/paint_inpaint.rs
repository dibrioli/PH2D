//! The **Inpaint** card (shown only in Inpaint mode): three canonical slider-with-chip rows tuning the
//! heal reconstruction — Patch Size (`2..6`), Quality (`3..12`) and Search (`50..300`%). The fixed-id
//! sliders + chips are registered in `crate::populate` (mapped-integer chips) and their `ValueChanged`
//! forwarded by `crate::event`'s brush-slider whitelist. Split from `paint_brush` for the LOC cap.

use crate::paint_brush_top::paint_slider_chip_row;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_tokens::{ColorToken, ROW_H_PX, TypeToken};
use ph2d_tool_painter::BrushSettings;

/// Paint the Inpaint card and return the next `y`. Only called in Inpaint mode (`brush.is_inpaint`).
pub(crate) fn paint_inpaint_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    // One-line hint (the mode's gesture); the sliders below tune the reconstruction.
    let font = TypeToken::Sm.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        "Brush over a defect · release to heal.",
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        content_w,
        resolve(ColorToken::Text2, theme),
    );
    let mut y = y + ROW_H_PX;

    y = paint_slider_chip_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Patch Size",
        core_ids::PAINTER_INPAINT_PATCH_SLIDER,
        core_ids::PAINTER_INPAINT_PATCH_CHIP,
        brush.inpaint_patch,
    );
    y = paint_slider_chip_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Quality",
        core_ids::PAINTER_INPAINT_QUALITY_SLIDER,
        core_ids::PAINTER_INPAINT_QUALITY_CHIP,
        brush.inpaint_quality,
    );
    y = paint_slider_chip_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Search",
        core_ids::PAINTER_INPAINT_SEARCH_SLIDER,
        core_ids::PAINTER_INPAINT_SEARCH_CHIP,
        brush.inpaint_search,
    );
    y
}

#[cfg(test)]
mod tests {
    //! Paint-level proof (mirrors `paint_stroke/tests::painted_hit_ids`) that the Inpaint card's three
    //! sliders register hit rects only in Inpaint mode — a row that isn't painted registers no hit rect,
    //! so no real pointer click can reach it (the DIRETIVA §2 "no silent no-op" guarantee).
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
    /// widget id that registered a hit rect.
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

    const INPAINT_SLIDERS: [NodeId; 3] = [
        core_ids::PAINTER_INPAINT_PATCH_SLIDER,
        core_ids::PAINTER_INPAINT_QUALITY_SLIDER,
        core_ids::PAINTER_INPAINT_SEARCH_SLIDER,
    ];

    #[test]
    fn inpaint_card_sliders_paint_only_in_inpaint_mode() {
        let inpaint = ph2d_tool_painter::BrushSettings {
            is_inpaint: true,
            ..crate::paint_brush::FALLBACK_BRUSH
        };
        let ids = body_hit_ids(inpaint);
        for sid in INPAINT_SLIDERS {
            assert!(
                ids.contains(&sid),
                "inpaint slider {sid:?} not hit-registered in Inpaint mode"
            );
        }
        // A plain Brush (the fallback default) must NOT paint the Inpaint card.
        let plain = body_hit_ids(crate::paint_brush::FALLBACK_BRUSH);
        for sid in INPAINT_SLIDERS {
            assert!(
                !plain.contains(&sid),
                "inpaint slider {sid:?} leaked into non-Inpaint mode"
            );
        }
    }
}
