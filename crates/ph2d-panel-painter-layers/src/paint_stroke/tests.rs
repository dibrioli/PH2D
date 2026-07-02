//! Paint-level proof that the per-method gate actually drops the irrelevant
//! rows. The existing `tests/seam.rs` checks bypass hit-testing (they inject a
//! `WidgetEvent` for a fixed id straight into `apply_event`), so they prove the
//! wire works *when shown* but cannot prove a row is *hidden*. Here we drive the
//! real `paint_stroke_section` and read the per-frame `HitIndex`: a row that is
//! not painted registers no hit rect, so no real pointer click can reach it
//! (the DIRETIVA §2 "no silent no-op" guarantee). The value still lives in the
//! WidgetStore, so switching methods round-trips it.

use super::*;
use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::{PaintCtx, PanelHostInternal};
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_ui_testkit::MockPanelHost;
use ph2d_vector::VectorScene;

/// Paint the Stroke section for `method` (all other fields at the Blender
/// defaults) and return every widget id that registered a hit rect this frame.
fn painted_hit_ids(method: StrokeMethod) -> Vec<NodeId> {
    let mut host = MockPanelHost::with_panel::<crate::PainterLayersPanel>();
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    // Tall viewport so no row is clipped out for a reason other than the gate.
    let viewport = Rect::new(0.0, 0.0, 360.0, 4000.0);
    let layout = HeroLayout::for_viewport(viewport);
    let brush = BrushSettings {
        stroke_method: method.to_u8(),
        ..crate::paint_brush::FALLBACK_BRUSH
    };
    {
        let mut ctx = PaintCtx {
            host: &mut host,
            layout: &layout,
            viewport,
            scene: &mut scene,
            text_system: &mut text,
        };
        paint_stroke_section(&mut ctx, Theme::default(), 0.0, 320.0, 0.0, brush);
    }
    host.hit_index_mut()
        .iter_registrations()
        .map(|(id, _)| id)
        .collect()
}

/// As [`painted_hit_ids`] but keep each id's registered hit rect, for geometry assertions.
fn painted_hit_rects(method: StrokeMethod, content_w: f32) -> Vec<(NodeId, Rect)> {
    let mut host = MockPanelHost::with_panel::<crate::PainterLayersPanel>();
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let viewport = Rect::new(0.0, 0.0, 360.0, 4000.0);
    let layout = HeroLayout::for_viewport(viewport);
    let brush = BrushSettings {
        stroke_method: method.to_u8(),
        ..crate::paint_brush::FALLBACK_BRUSH
    };
    {
        let mut ctx = PaintCtx {
            host: &mut host,
            layout: &layout,
            viewport,
            scene: &mut scene,
            text_system: &mut text,
        };
        paint_stroke_section(&mut ctx, Theme::default(), 0.0, content_w, 0.0, brush);
    }
    host.hit_index_mut().iter_registrations().collect()
}

/// Regression (Enio 2026-06-27): in the wide one-row layout the Apply & Keep button was painted at the
/// SAME x as the trailing E/✕ icon cluster, so its fill covered them and stole their clicks ("the X
/// disappeared and we still don't have the E button"). The bake buttons and the icon cluster must occupy
/// DISJOINT hit rects. Checked for Ellipse (E + ✕ present) at a wide content width that forces one row.
#[test]
fn apply_keep_does_not_overlap_the_edit_and_delete_icons() {
    let rects = painted_hit_rects(StrokeMethod::Ellipse, 320.0);
    let find = |id: NodeId| {
        rects
            .iter()
            .find(|(rid, _)| *rid == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("{id:?} registered no hit rect"))
    };
    let keep = find(core_ids::PAINTER_BRUSH_STROKE_APPLY_KEEP);
    let overlaps =
        |a: Rect, b: Rect| a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
    for icon in [
        core_ids::PAINTER_BRUSH_STROKE_EDIT,
        core_ids::PAINTER_BRUSH_STROKE_DELETE,
    ] {
        let r = find(icon);
        assert!(
            !overlaps(keep, r),
            "Apply & Keep {keep:?} overlaps {icon:?} {r:?} — the icon is covered/unclickable"
        );
    }
    // And Apply must not overlap Apply & Keep either.
    let apply = find(core_ids::PAINTER_BRUSH_STROKE_APPLY);
    assert!(
        !overlaps(apply, keep),
        "Apply {apply:?} overlaps Apply & Keep {keep:?}"
    );
}

/// Dots: per-event method — Spacing and Dash/Length are no-ops, so they stay hidden; Jitter,
/// Input Samples and the Stabilizer (Blender enables smooth-stroke for Dots) show.
#[test]
fn dots_hides_spacing_and_dash_but_shows_jitter_samples_and_stabilizer() {
    let ids = painted_hit_ids(StrokeMethod::Dots);
    for hidden in [
        core_ids::PAINTER_BRUSH_SPACING,
        core_ids::PAINTER_BRUSH_SPACE_ATTEN,
        core_ids::PAINTER_BRUSH_DASH_RATIO,
        core_ids::PAINTER_BRUSH_DASH_LENGTH,
        core_ids::PAINTER_BRUSH_RATE,
    ] {
        assert!(
            !ids.contains(&hidden),
            "Dots painted a hit rect for {hidden:?} — Spacing/Dash must be hidden \
             (silent no-op). painted = {ids:?}"
        );
    }
    for shown in [
        core_ids::PAINTER_BRUSH_STROKE_METHOD,
        core_ids::PAINTER_BRUSH_JITTER,
        core_ids::PAINTER_BRUSH_JITTER_UNIT,
        core_ids::PAINTER_BRUSH_INPUT_SAMPLES,
        core_ids::PAINTER_BRUSH_STABILIZE,
    ] {
        assert!(
            ids.contains(&shown),
            "Dots dropped a row it should show ({shown:?}). painted = {ids:?}"
        );
    }
}

/// Space (the default): the only positive control — proves the gate is not
/// hiding everything; every spacing-driven row is present.
#[test]
fn space_shows_spacing_dash_and_the_rest() {
    let ids = painted_hit_ids(StrokeMethod::Space);
    for shown in [
        core_ids::PAINTER_BRUSH_STROKE_METHOD,
        core_ids::PAINTER_BRUSH_SPACING,
        core_ids::PAINTER_BRUSH_SPACE_ATTEN,
        core_ids::PAINTER_BRUSH_JITTER,
        core_ids::PAINTER_BRUSH_JITTER_UNIT,
        core_ids::PAINTER_BRUSH_DASH_RATIO,
        core_ids::PAINTER_BRUSH_DASH_LENGTH,
        core_ids::PAINTER_BRUSH_INPUT_SAMPLES,
        core_ids::PAINTER_BRUSH_STABILIZE,
    ] {
        assert!(
            ids.contains(&shown),
            "Space dropped {shown:?} — the spacing-driven default must show every \
             Stroke row. painted = {ids:?}"
        );
    }
}

/// Airbrush: the one timer-driven method — the **Rate** row shows (its defining param), while
/// the spacing/dash rows stay hidden (airbrush isn't spacing-driven). Jitter, Samples and the
/// Stabilizer also show (Blender enables smooth-stroke for Airbrush). Locks "Rate is
/// airbrush-only" so the slider can't become a silent no-op on another method.
#[test]
fn airbrush_shows_rate_and_hides_spacing_dash() {
    let ids = painted_hit_ids(StrokeMethod::Airbrush);
    for shown in [
        core_ids::PAINTER_BRUSH_STROKE_METHOD,
        core_ids::PAINTER_BRUSH_RATE,
        core_ids::PAINTER_BRUSH_JITTER,
        core_ids::PAINTER_BRUSH_JITTER_UNIT,
        core_ids::PAINTER_BRUSH_INPUT_SAMPLES,
        core_ids::PAINTER_BRUSH_STABILIZE,
    ] {
        assert!(
            ids.contains(&shown),
            "Airbrush dropped a row it should show ({shown:?}). painted = {ids:?}"
        );
    }
    for hidden in [
        core_ids::PAINTER_BRUSH_SPACING,
        core_ids::PAINTER_BRUSH_SPACE_ATTEN,
        core_ids::PAINTER_BRUSH_DASH_RATIO,
        core_ids::PAINTER_BRUSH_DASH_LENGTH,
    ] {
        assert!(
            !ids.contains(&hidden),
            "Airbrush painted a hit rect for {hidden:?} — it's not spacing-driven. painted = {ids:?}"
        );
    }
}

/// Anchored: shows **Edge to Edge** (its one real control) + Method + Input Samples. Jitter and
/// the Stabilizer stay HIDDEN even though Blender's panel shows them — both are no-ops for
/// Anchored in Blender's own code (`paint_stroke_use_jitter` and `paint_supports_smooth_stroke`
/// reject ANCHORED), so PH2D hides them rather than paint a silent no-op (DIRETIVA §2).
#[test]
fn anchored_shows_edge_to_edge_method_and_samples_hides_the_no_op_rows() {
    let ids = painted_hit_ids(StrokeMethod::Anchored);
    for shown in [
        core_ids::PAINTER_BRUSH_STROKE_METHOD,
        core_ids::PAINTER_BRUSH_EDGE_TO_EDGE,
        core_ids::PAINTER_BRUSH_INPUT_SAMPLES,
    ] {
        assert!(
            ids.contains(&shown),
            "Anchored dropped a row it should show ({shown:?}). painted = {ids:?}"
        );
    }
    for hidden in [
        core_ids::PAINTER_BRUSH_SPACING,
        core_ids::PAINTER_BRUSH_SPACE_ATTEN,
        core_ids::PAINTER_BRUSH_DASH_RATIO,
        core_ids::PAINTER_BRUSH_DASH_LENGTH,
        core_ids::PAINTER_BRUSH_RATE,
        core_ids::PAINTER_BRUSH_JITTER,
        core_ids::PAINTER_BRUSH_JITTER_UNIT,
        core_ids::PAINTER_BRUSH_STABILIZE,
    ] {
        assert!(
            !ids.contains(&hidden),
            "Anchored painted a hit rect for {hidden:?} — it's a Blender no-op for Anchored. \
             painted = {ids:?}"
        );
    }
}

/// Line: a spacing-driven method (Spacing + Adjust-Strength + Dash + Jitter + Samples), but
/// NOT smooth-stroke (Blender rejects LINE in `paint_supports_smooth_stroke`), so the Stabilizer
/// stays hidden. Rate/Edge-to-Edge are other methods' controls — hidden.
#[test]
fn line_shows_spacing_dash_jitter_samples_hides_stabilize_rate_edge() {
    let ids = painted_hit_ids(StrokeMethod::Line);
    for shown in [
        core_ids::PAINTER_BRUSH_STROKE_METHOD,
        core_ids::PAINTER_BRUSH_SPACING,
        core_ids::PAINTER_BRUSH_SPACE_ATTEN,
        core_ids::PAINTER_BRUSH_DASH_RATIO,
        core_ids::PAINTER_BRUSH_DASH_LENGTH,
        core_ids::PAINTER_BRUSH_JITTER,
        core_ids::PAINTER_BRUSH_JITTER_UNIT,
        core_ids::PAINTER_BRUSH_INPUT_SAMPLES,
    ] {
        assert!(
            ids.contains(&shown),
            "Line dropped a spacing-driven row it should show ({shown:?}). painted = {ids:?}"
        );
    }
    for hidden in [
        core_ids::PAINTER_BRUSH_STABILIZE,
        core_ids::PAINTER_BRUSH_RATE,
        core_ids::PAINTER_BRUSH_EDGE_TO_EDGE,
    ] {
        assert!(
            !ids.contains(&hidden),
            "Line painted a hit rect for {hidden:?} — not a Line control. painted = {ids:?}"
        );
    }
}

/// Curve: PH2D's point-editor (author control points, auto-smooth between them). Spacing-driven
/// like Line (Spacing + Adjust-Strength + Dash + Jitter + Samples), but NOT freehand — there is
/// no shaky path to filter, so the Stabilizer stays hidden (same visible set as Line).
#[test]
fn curve_shows_spacing_dash_jitter_samples_hides_stabilize_rate_edge() {
    let ids = painted_hit_ids(StrokeMethod::Curve);
    for shown in [
        core_ids::PAINTER_BRUSH_STROKE_METHOD,
        core_ids::PAINTER_BRUSH_SPACING,
        core_ids::PAINTER_BRUSH_SPACE_ATTEN,
        core_ids::PAINTER_BRUSH_DASH_RATIO,
        core_ids::PAINTER_BRUSH_DASH_LENGTH,
        core_ids::PAINTER_BRUSH_JITTER,
        core_ids::PAINTER_BRUSH_JITTER_UNIT,
        core_ids::PAINTER_BRUSH_INPUT_SAMPLES,
    ] {
        assert!(
            ids.contains(&shown),
            "Curve dropped a spacing-driven row it should show ({shown:?}). painted = {ids:?}"
        );
    }
    for hidden in [
        core_ids::PAINTER_BRUSH_STABILIZE,
        core_ids::PAINTER_BRUSH_RATE,
        core_ids::PAINTER_BRUSH_EDGE_TO_EDGE,
    ] {
        assert!(
            !ids.contains(&hidden),
            "Curve painted a hit rect for {hidden:?} — not a Curve control (point-editor, no \
             freehand to stabilize). painted = {ids:?}"
        );
    }
}

/// Ellipse: the PH2D ellipse shape — spacing-driven like Line/Curve (Spacing + Adjust-Strength +
/// Dash + Jitter + Samples), NOT freehand, so the Stabilizer stays hidden. Same visible set as
/// Line/Curve.
#[test]
fn circle_shows_spacing_dash_jitter_samples_hides_stabilize_rate_edge() {
    let ids = painted_hit_ids(StrokeMethod::Ellipse);
    for shown in [
        core_ids::PAINTER_BRUSH_STROKE_METHOD,
        core_ids::PAINTER_BRUSH_SPACING,
        core_ids::PAINTER_BRUSH_SPACE_ATTEN,
        core_ids::PAINTER_BRUSH_DASH_RATIO,
        core_ids::PAINTER_BRUSH_DASH_LENGTH,
        core_ids::PAINTER_BRUSH_JITTER,
        core_ids::PAINTER_BRUSH_JITTER_UNIT,
        core_ids::PAINTER_BRUSH_INPUT_SAMPLES,
    ] {
        assert!(
            ids.contains(&shown),
            "Ellipse dropped a spacing-driven row it should show ({shown:?}). painted = {ids:?}"
        );
    }
    for hidden in [
        core_ids::PAINTER_BRUSH_STABILIZE,
        core_ids::PAINTER_BRUSH_RATE,
        core_ids::PAINTER_BRUSH_EDGE_TO_EDGE,
    ] {
        assert!(
            !ids.contains(&hidden),
            "Ellipse painted a hit rect for {hidden:?} — not a Ellipse control (shape, no freehand \
             to stabilize). painted = {ids:?}"
        );
    }
}

/// Polygon: the PH2D regular-N-gon shape — spacing-driven like Ellipse (Spacing + Adjust + Dash +
/// Jitter + Samples), NOT freehand, so the Stabilizer stays hidden. Same visible set as Ellipse.
#[test]
fn polygon_shows_spacing_dash_jitter_samples_hides_stabilize_rate_edge() {
    let ids = painted_hit_ids(StrokeMethod::Polygon);
    for shown in [
        core_ids::PAINTER_BRUSH_STROKE_METHOD,
        core_ids::PAINTER_BRUSH_SPACING,
        core_ids::PAINTER_BRUSH_SPACE_ATTEN,
        core_ids::PAINTER_BRUSH_DASH_RATIO,
        core_ids::PAINTER_BRUSH_DASH_LENGTH,
        core_ids::PAINTER_BRUSH_JITTER,
        core_ids::PAINTER_BRUSH_JITTER_UNIT,
        core_ids::PAINTER_BRUSH_INPUT_SAMPLES,
    ] {
        assert!(
            ids.contains(&shown),
            "Polygon dropped a spacing-driven row it should show ({shown:?}). painted = {ids:?}"
        );
    }
    for hidden in [
        core_ids::PAINTER_BRUSH_STABILIZE,
        core_ids::PAINTER_BRUSH_RATE,
        core_ids::PAINTER_BRUSH_EDGE_TO_EDGE,
    ] {
        assert!(
            !ids.contains(&hidden),
            "Polygon painted a hit rect for {hidden:?} — not a Polygon control (shape, no freehand \
             to stabilize). painted = {ids:?}"
        );
    }
}

/// Offset slider: the perpendicular path offset applies to the shape editors (Curve / Free Hand / Ellipse
/// / Polygon), so its hit rect registers for those and stays hidden for the finalise-on-up methods.
#[test]
fn offset_slider_shows_only_for_the_shape_editor_methods() {
    for m in [
        StrokeMethod::Curve,
        StrokeMethod::FreeHand,
        StrokeMethod::Ellipse,
        StrokeMethod::Polygon,
    ] {
        assert!(
            painted_hit_ids(m).contains(&core_ids::PAINTER_BRUSH_OFFSET),
            "{m:?} must paint the Offset slider"
        );
    }
    for m in [StrokeMethod::Line, StrokeMethod::Space] {
        assert!(
            !painted_hit_ids(m).contains(&core_ids::PAINTER_BRUSH_OFFSET),
            "{m:?} has no on-canvas shape to offset — Offset must be hidden"
        );
    }
}

/// Apply / Apply & Keep: the two bake buttons register hit rects ONLY for the methods with a
/// persistent on-canvas shape editor (Curve/Free Hand/Ellipse/Polygon/Line) — so a real click can reach
/// them — and stay hidden for the finalise-on-up methods (Space). A regression guard for the
/// "Apply does nothing" report: no hit rect ⇒ no Click ⇒ dead button (the populate-register gotcha).
#[test]
fn apply_buttons_register_hit_rects_for_the_editor_methods_only() {
    for m in [
        StrokeMethod::Curve,
        StrokeMethod::FreeHand,
        StrokeMethod::Ellipse,
        StrokeMethod::Polygon,
        StrokeMethod::Line,
    ] {
        let ids = painted_hit_ids(m);
        for b in [
            core_ids::PAINTER_BRUSH_STROKE_APPLY,
            core_ids::PAINTER_BRUSH_STROKE_APPLY_KEEP,
            core_ids::PAINTER_BRUSH_STROKE_DELETE,
        ] {
            assert!(
                ids.contains(&b),
                "{m:?} must paint a hit rect for {b:?} (else the button is dead). painted = {ids:?}"
            );
        }
    }
    // Space finalises on pen-up — no Apply row (Line now has the polyline editor, so it DOES get one).
    let ids = painted_hit_ids(StrokeMethod::Space);
    assert!(
        !ids.contains(&core_ids::PAINTER_BRUSH_STROKE_APPLY),
        "Space finalises on pen-up — no Apply button. painted = {ids:?}"
    );
}

/// The Edit (E) button — convert to an editable curve — paints a hit rect ONLY for the convertible
/// parametric shapes (Ellipse / Polygon), not for Curve / Free Hand (already curves) or Line.
#[test]
fn edit_button_registers_only_for_circle_and_polygon() {
    for m in [StrokeMethod::Ellipse, StrokeMethod::Polygon] {
        assert!(
            painted_hit_ids(m).contains(&core_ids::PAINTER_BRUSH_STROKE_EDIT),
            "{m:?} must paint the Edit (E) button"
        );
    }
    for m in [
        StrokeMethod::Curve,
        StrokeMethod::FreeHand,
        StrokeMethod::Line,
    ] {
        assert!(
            !painted_hit_ids(m).contains(&core_ids::PAINTER_BRUSH_STROKE_EDIT),
            "{m:?} must NOT paint the Edit button (not a convertible parametric shape)"
        );
    }
}

/// Save As Object: the floppy button beside the Method dropdown registers a hit rect ONLY while a curve
/// with points is drawn (`has_drawn_curve`), and stays hidden otherwise — so it appears exactly when the
/// curve can be saved.
#[test]
fn save_as_object_button_shows_only_when_a_curve_is_drawn() {
    let painted = |has_drawn_curve: bool| -> Vec<NodeId> {
        let mut host = MockPanelHost::with_panel::<crate::PainterLayersPanel>();
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let viewport = Rect::new(0.0, 0.0, 360.0, 4000.0);
        let layout = HeroLayout::for_viewport(viewport);
        let brush = BrushSettings {
            stroke_method: StrokeMethod::Curve.to_u8(),
            has_drawn_curve,
            ..crate::paint_brush::FALLBACK_BRUSH
        };
        {
            let mut ctx = PaintCtx {
                host: &mut host,
                layout: &layout,
                viewport,
                scene: &mut scene,
                text_system: &mut text,
            };
            paint_stroke_section(&mut ctx, Theme::default(), 0.0, 320.0, 0.0, brush);
        }
        host.hit_index_mut()
            .iter_registrations()
            .map(|(id, _)| id)
            .collect()
    };
    assert!(
        painted(true).contains(&core_ids::PAINTER_BRUSH_STROKE_SAVE_OBJECT),
        "the Save As Object button shows when a curve with points is drawn"
    );
    assert!(
        !painted(false).contains(&core_ids::PAINTER_BRUSH_STROKE_SAVE_OBJECT),
        "the Save As Object button stays hidden when no curve is drawn"
    );
}

/// Drag Dot: the most-restricted method (no jitter, no spacing, no stabilizer — the dot sits
/// raw under the cursor) — only Method + Input Samples survive. UI-honesty for "Drag Dot wrong".
#[test]
fn dragdot_shows_only_method_and_samples() {
    let ids = painted_hit_ids(StrokeMethod::DragDot);
    for hidden in [
        core_ids::PAINTER_BRUSH_SPACING,
        core_ids::PAINTER_BRUSH_SPACE_ATTEN,
        core_ids::PAINTER_BRUSH_DASH_RATIO,
        core_ids::PAINTER_BRUSH_DASH_LENGTH,
        core_ids::PAINTER_BRUSH_JITTER,
        core_ids::PAINTER_BRUSH_JITTER_UNIT,
        core_ids::PAINTER_BRUSH_STABILIZE,
    ] {
        assert!(
            !ids.contains(&hidden),
            "Drag Dot painted a hit rect for {hidden:?} — it forces pressure 1, no jitter, \
             raw positioning. painted = {ids:?}"
        );
    }
    for shown in [
        core_ids::PAINTER_BRUSH_STROKE_METHOD,
        core_ids::PAINTER_BRUSH_INPUT_SAMPLES,
    ] {
        assert!(
            ids.contains(&shown),
            "Drag Dot dropped {shown:?} (always shown). painted = {ids:?}"
        );
    }
}
