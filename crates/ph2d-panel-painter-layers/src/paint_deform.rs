//! The **Deform** (Liquify) panel section (Deform Wave 1). Mode-exclusive: when the Painter is in Deform
//! mode the Brush body paints ONLY this section (the Selection / Inpaint precedent).
//!
//! Layout (Widget-Gallery components, all reflow on a narrow panel via `SegmentedAdaptive` +
//! `paint_slider_chip_row`, which stacks label-over-slider when narrow): **Mode** card (Push / Twist /
//! Pinch / Wrinkle / Fold / Reconstruct) · **Size / Pressure / Distortion / Momentum / Strength** sliders
//! (Distortion + Momentum hidden in Reconstruct — they don't apply, never a silent no-op) · **Freeze**
//! toggle (+ Invert) · **Reset / Apply / Apply & Keep** actions. Every interactive id is registered in
//! `populate` and forwarded by `event.rs` over the frozen `PanelEvent` channel to `route_deform_event`.

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

use crate::paint_brush_top::paint_slider_chip_row;

/// The Reconstruct sub-mode index (Distortion + Momentum don't apply → hidden).
const MODE_RECONSTRUCT: u8 = 5;

/// Paint the mode-exclusive Deform section, returning the next `y`.
pub(crate) fn paint_deform_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    // Temperament — Reshape (brush dabs) vs Transform (gizmo). Opens UNSELECTED each time the panel is
    // entered: the artist must pick, and re-picking Transform re-lifts a fresh gizmo (Enio 2026-07-04). The
    // body + session actions only appear once a mode is chosen.
    let selected = match brush.deform_temperament {
        ph2d_tool_painter::DEFORM_TEMPERAMENT_RESHAPE => 0,
        ph2d_tool_painter::DEFORM_TEMPERAMENT_TRANSFORM => 1,
        _ => usize::MAX, // none picked
    };
    let mut y = seg_group(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_DEFORM_TEMPERAMENT,
        "Deform temperament",
        &[
            (core_ids::PAINTER_DEFORM_TEMPERAMENT_RESHAPE, "Reshape"),
            (core_ids::PAINTER_DEFORM_TEMPERAMENT_TRANSFORM, "Transform"),
        ],
        selected,
    );

    match brush.deform_temperament {
        ph2d_tool_painter::DEFORM_TEMPERAMENT_RESHAPE => {
            y = paint_reshape_body(ctx, theme, x, content_w, y, brush);
            y = paint_section_separator(ctx.scene, theme, x, content_w, y);
            y = paint_actions(ctx, theme, x, content_w, y);
        }
        ph2d_tool_painter::DEFORM_TEMPERAMENT_TRANSFORM => {
            y = paint_transform_body(
                ctx,
                theme,
                x,
                content_w,
                y,
                brush.deform_transform_mode as usize,
            );
            y = paint_section_separator(ctx.scene, theme, x, content_w, y);
            y = paint_actions(ctx, theme, x, content_w, y);
        }
        _ => {} // nothing picked yet — just the temperament toggle
    }
    y
}

/// The **Transform** body: the Uniform / Free sub-mode picker (the box handles live on the canvas overlay).
fn paint_transform_body(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    selected: usize,
) -> f32 {
    seg_group(
        ctx,
        theme,
        x,
        content_w,
        y,
        core_ids::PAINTER_DEFORM_TRANSFORM_MODE,
        "Transform mode",
        &[
            (core_ids::PAINTER_DEFORM_TRANSFORM_MODE_UNIFORM, "Uniform"),
            (core_ids::PAINTER_DEFORM_TRANSFORM_MODE_FREE, "Free"),
            (core_ids::PAINTER_DEFORM_TRANSFORM_MODE_DISTORT, "Distort"),
            (core_ids::PAINTER_DEFORM_TRANSFORM_MODE_WARP, "Warp"),
        ],
        selected,
    )
}

/// The **Reshape** body: the sub-mode card (Push … Reconstruct) + the brush sliders.
fn paint_reshape_body(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    // Mode picker — a segmented group in a titled card at the top (reflows on a narrow panel).
    let mut y = mode_card(ctx, theme, x, content_w, y, brush.deform_mode as usize);

    let is_reconstruct = brush.deform_mode == MODE_RECONSTRUCT;

    y = paint_slider_chip_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Size",
        core_ids::PAINTER_DEFORM_SIZE_SLIDER,
        core_ids::PAINTER_DEFORM_SIZE_CHIP,
        brush.deform_size_norm,
    );
    y = paint_slider_chip_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "Pressure",
        core_ids::PAINTER_DEFORM_PRESSURE_SLIDER,
        core_ids::PAINTER_DEFORM_PRESSURE_CHIP,
        brush.deform_pressure,
    );
    // Distortion + Momentum shape the field; Reconstruct only resamples, so they're hidden there (never a
    // control that silently does nothing — DIRETIVA §2).
    if !is_reconstruct {
        y = paint_slider_chip_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Distortion",
            core_ids::PAINTER_DEFORM_DISTORTION_SLIDER,
            core_ids::PAINTER_DEFORM_DISTORTION_CHIP,
            brush.deform_distortion,
        );
        y = paint_slider_chip_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Momentum",
            core_ids::PAINTER_DEFORM_MOMENTUM_SLIDER,
            core_ids::PAINTER_DEFORM_MOMENTUM_CHIP,
            brush.deform_momentum,
        );
        // Strength — bipolar (0.5 = neutral; Pinch −suck / +bulge, Twist CW/CCW). Neutral in Reconstruct.
        y = paint_slider_chip_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            "Strength",
            core_ids::PAINTER_DEFORM_STRENGTH_SLIDER,
            core_ids::PAINTER_DEFORM_STRENGTH_CHIP,
            brush.deform_strength,
        );
    }
    y
}

/// The **Reset / Apply / Apply & Keep** session actions — shared by both temperaments (they act on the
/// whole session, not a specific brush/gizmo mode). Deform is confined to the active selection (or the whole
/// sprite when nothing is selected) automatically, so there's no Freeze toggle. Returns next `y`.
fn paint_actions(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
) -> f32 {
    // Session actions — Reset / Apply / Apply & Keep (momentary buttons, none selected).
    seg_group(
        ctx,
        theme,
        x,
        content_w,
        y,
        NodeId(0),
        "Deform actions",
        &[
            (core_ids::PAINTER_DEFORM_RESET, "Reset"),
            (core_ids::PAINTER_DEFORM_APPLY, "Apply"),
            (core_ids::PAINTER_DEFORM_APPLY_KEEP, "Apply & Keep"),
        ],
        usize::MAX,
    )
}

/// The **Mode** card (Push / Twist / Pinch / Wrinkle / Fold / Reconstruct) — a titled surface whose
/// segmented group reflows onto extra rows on a narrow panel; the card grows to fit. Returns next `y`.
fn mode_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    selected: usize,
) -> f32 {
    let header_h = Spacing::Xl3.px();
    let pad = Spacing::Lg.px();
    let card = Card::new(core_ids::PAINTER_DEFORM_MODE_CARD).title("MODE");
    let seg = SegmentedAdaptive::new(
        core_ids::PAINTER_DEFORM_MODE,
        "Deform mode",
        vec![
            SegmentedOption::new(core_ids::PAINTER_DEFORM_MODE_PUSH, "Push"),
            SegmentedOption::new(core_ids::PAINTER_DEFORM_MODE_TWIST, "Twist"),
            SegmentedOption::new(core_ids::PAINTER_DEFORM_MODE_PINCH, "Pinch"),
            SegmentedOption::new(core_ids::PAINTER_DEFORM_MODE_WRINKLE, "Wrinkle"),
            SegmentedOption::new(core_ids::PAINTER_DEFORM_MODE_FOLD, "Fold"),
            SegmentedOption::new(core_ids::PAINTER_DEFORM_MODE_RECONSTRUCT, "Reconstruct"),
        ],
    )
    .selected(selected);
    // Body width is height-independent — probe it, then measure the reflowed segmented height so the card
    // grows to fit all six segments when they wrap on a narrow panel (mirrors `paint_selection::operation_card`).
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
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        paint_segmented_adaptive(
            &seg,
            Rect::new(body.x, body.y, body.w, ROW_H_PX),
            scene,
            text_system,
            theme,
            store,
            hit_index,
        );
    }
    y + card_h + Spacing::Sm.px()
}

/// Build + paint one adaptive segmented group and register its per-segment hits; returns next `y`. Reflows
/// on narrow widths (tablet-safe). Shared shape with `paint_selection::seg_group`.
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
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let used = paint_segmented_adaptive(
        &seg,
        Rect::new(x, y, content_w, ROW_H_PX),
        scene,
        text_system,
        theme,
        store,
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

    /// Drive the real `paint_brush_body` with `brush` published and return every hit-registered id
    /// (the observable seam: painted ⟹ hit-indexed, guarded by `architecture_panel_wiring_parity`).
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

    /// A Deform brush with the **Reshape** temperament picked (the panel opens on NONE; these body tests
    /// assume a mode has been chosen).
    fn deform_brush() -> ph2d_tool_painter::BrushSettings {
        ph2d_tool_painter::BrushSettings {
            is_deform: true,
            deform_temperament: ph2d_tool_painter::DEFORM_TEMPERAMENT_RESHAPE,
            ..crate::paint_brush::FALLBACK_BRUSH
        }
    }

    #[test]
    fn deform_mode_paints_only_deform_controls() {
        let ids = body_hit_ids(deform_brush());
        for did in core_ids::PAINTER_DEFORM_MODE_IDS
            .iter()
            .chain(core_ids::PAINTER_DEFORM_ACTION_IDS.iter())
            .chain(
                [
                    core_ids::PAINTER_DEFORM_SIZE_SLIDER,
                    core_ids::PAINTER_DEFORM_PRESSURE_SLIDER,
                    core_ids::PAINTER_DEFORM_STRENGTH_SLIDER,
                ]
                .iter(),
            )
        {
            assert!(
                ids.contains(did),
                "deform control {did:?} not painted in Deform mode"
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
                "brush control {bid:?} leaked into Deform mode"
            );
        }
    }

    #[test]
    fn distortion_and_momentum_hide_in_reconstruct() {
        // Push (default mode 0) shows Distortion + Momentum; Reconstruct (mode 5) hides them (they don't
        // apply → never a silent no-op).
        let push = body_hit_ids(deform_brush());
        assert!(push.contains(&core_ids::PAINTER_DEFORM_DISTORTION_SLIDER));
        assert!(push.contains(&core_ids::PAINTER_DEFORM_MOMENTUM_SLIDER));
        let reconstruct = ph2d_tool_painter::BrushSettings {
            deform_mode: 5,
            ..deform_brush()
        };
        let ids = body_hit_ids(reconstruct);
        assert!(!ids.contains(&core_ids::PAINTER_DEFORM_DISTORTION_SLIDER));
        assert!(!ids.contains(&core_ids::PAINTER_DEFORM_MOMENTUM_SLIDER));
    }

    #[test]
    fn temperament_toggle_is_always_painted() {
        // Both temperament segments (Reshape / Transform) are hit-registered in either temperament, so the
        // artist can always switch back.
        for id in core_ids::PAINTER_DEFORM_TEMPERAMENT_IDS {
            assert!(
                body_hit_ids(deform_brush()).contains(&id),
                "temperament segment {id:?} not painted in Reshape"
            );
        }
        let transform = ph2d_tool_painter::BrushSettings {
            deform_temperament: ph2d_tool_painter::DEFORM_TEMPERAMENT_TRANSFORM,
            ..deform_brush()
        };
        for id in core_ids::PAINTER_DEFORM_TEMPERAMENT_IDS {
            assert!(
                body_hit_ids(transform).contains(&id),
                "temperament segment {id:?} not painted in Transform"
            );
        }
    }

    #[test]
    fn transform_body_replaces_reshape_body() {
        // Transform temperament shows the Uniform/Free picker and HIDES the Reshape mode card + brush
        // sliders (mode-exclusive body swap — never both at once).
        let transform = ph2d_tool_painter::BrushSettings {
            deform_temperament: ph2d_tool_painter::DEFORM_TEMPERAMENT_TRANSFORM,
            ..deform_brush()
        };
        let ids = body_hit_ids(transform);
        for id in core_ids::PAINTER_DEFORM_TRANSFORM_MODE_IDS {
            assert!(
                ids.contains(&id),
                "transform sub-mode {id:?} shown in Transform"
            );
        }
        // The Reshape bodies are gone.
        assert!(
            !ids.contains(&core_ids::PAINTER_DEFORM_MODE_PUSH),
            "Reshape mode card hidden in Transform"
        );
        assert!(
            !ids.contains(&core_ids::PAINTER_DEFORM_SIZE_SLIDER),
            "Reshape brush sliders hidden in Transform"
        );
        // But the shared session actions survive.
        assert!(ids.contains(&core_ids::PAINTER_DEFORM_APPLY));
        // And Reshape keeps the transform sub-mode hidden.
        let reshape = body_hit_ids(deform_brush());
        assert!(
            !reshape.contains(&core_ids::PAINTER_DEFORM_TRANSFORM_MODE_UNIFORM),
            "transform sub-mode hidden in Reshape"
        );
    }
}
