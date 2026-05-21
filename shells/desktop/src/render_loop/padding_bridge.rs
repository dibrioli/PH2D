//! Padding panel ⟷ tool bridge + live on-canvas preview.
//!
//! Run once per frame BEFORE `paint_hero_screen` (sibling of
//! `bgremoval_preview.rs`). Does, in order:
//!
//! 1. Drives the panel's visibility (shown iff `padding` is the active
//!    tool, keyed "padding" to match `PaddingPanel::ID`).
//! 2. Drains the panel's `PaddingUiEdit`s into the active `PaddingTool`.
//! 3. Publishes the per-frame snapshot the panel paints next frame.
//! 4. Draws a NON-DESTRUCTIVE live preview of the new canvas bounds
//!    (an accent outline) so dragging a slider shows the padding change
//!    on the canvas in real time. Crucially the preview never mutates
//!    the sprite Transform — so in either pivot mode the existing
//!    content pixels and the pivot stay put while editing; the actual
//!    resize + pivot reproject happen only on Apply (the bake).
//! 5. Returns the selection + spec + pivot mode to bake on Apply.

use ph2d_ecs::SimWorld;
use ph2d_editor::HeroScreen;
use ph2d_editor::ToolRegistry;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, Sprite};
use ph2d_tokens::ColorToken;
use ph2d_vector::{Affine, Brush, Color, Rect, Stroke, VectorScene};

/// Returns `Some((entity_bits, spec, recenter_pivot))` iff Apply fired
/// this frame — the caller runs the full-resolution bake against that
/// selection with the captured per-edge spec + pivot mode and then tears
/// the tool down (deactivate + restore Inspector), exactly like the
/// Bg-Removal apply teardown. The spec + pivot flag are captured here
/// (while the tool is borrowed) so the bake site doesn't have to
/// re-borrow `tools`.
#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch(
    hero: &mut HeroScreen,
    tools: &mut ToolRegistry,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    padding_ui_edits: Vec<ph2d_editor::tools::padding::PaddingUiEdit>,
) -> Option<(u64, ph2d_tool_padding::PaddingSpec, bool)> {
    let padding_is_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor::ToolId::new("padding"))
        .unwrap_or(false);
    // Visibility: shown iff padding is the active tool.
    hero.panel_visibility.insert("padding", padding_is_active);

    let mut apply: Option<(u64, ph2d_tool_padding::PaddingSpec, bool)> = None;
    // Captured while the tool is borrowed; used for the on-canvas preview
    // after the borrow ends (the preview needs `sim` + `camera`, not the
    // tool).
    let mut preview_spec: Option<(i32, i32, i32, i32, bool)> = None;
    if let Some(tool) = tools.active_mut()
        && let Some(pad) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_editor::tools::padding::PaddingTool>()
    {
        for edit in padding_ui_edits {
            pad.apply_ui_edit(edit);
        }
        if pad.take_pending_apply()
            && let Some(bits) = hero.gizmo.selection
        {
            let (top, right, bottom, left) = pad.spec();
            apply = Some((
                bits,
                ph2d_tool_padding::PaddingSpec {
                    top,
                    right,
                    bottom,
                    left,
                },
                pad.recenter_pivot(),
            ));
        }
        #[cfg(feature = "panel-padding")]
        ph2d_panel_padding::set_current_padding_snapshot(if padding_is_active {
            Some(pad.ui_snapshot())
        } else {
            None
        });
        if padding_is_active {
            let (top, right, bottom, left) = pad.spec();
            preview_spec = Some((top, right, bottom, left, pad.recenter_pivot()));
        }
    }

    // ── Live preview ────────────────────────────────────────────────
    // Draw the new canvas bounds as an accent outline over the selected
    // sprite's footprint. Non-destructive: nothing about the sprite or
    // its Transform moves while editing — only this overlay grows /
    // shrinks as the sliders change.
    if let Some((top, right, bottom, left, recenter)) = preview_spec
        && (top != 0 || right != 0 || bottom != 0 || left != 0)
        && let Some(bits) = hero.gizmo.selection
    {
        draw_canvas_outline(
            hero,
            sim,
            camera,
            window_size,
            vector_scene,
            bits,
            [top, right, bottom, left],
            recenter,
        );
    }

    apply
}

/// Stroke the new-canvas world rect for `entity_bits` onto the canvas.
///
/// `spec = [top, right, bottom, left]` signed px. In `recenter` mode the
/// rect grows outward from the CONTENT edges (the content stays put on
/// Apply, so the preview anchors to it); in keep mode it is centred on
/// the current pivot (which stays put on Apply). Either way the existing
/// sprite + pivot are untouched — this only paints an overlay.
#[allow(clippy::too_many_arguments)]
fn draw_canvas_outline(
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    entity_bits: u64,
    spec: [i32; 4],
    recenter: bool,
) {
    let entity = ph2d_ecs::Entity::from_bits(entity_bits);
    let Some(sprite) = sim.world().get::<Sprite>(entity) else {
        return;
    };
    let Some(tr) = sim.world().get::<ph2d_ecs::Transform>(entity) else {
        return;
    };
    let ppm = hero.project.pixels_per_meter.max(1.0e-3);
    let (sx, sy) = (sprite.size[0], sprite.size[1]);
    let (cx, cy) = (tr.translation.x, tr.translation.y);
    // Per-edge padding in world units (positive = expand, negative = crop).
    let (t, r, b, l) = (
        spec[0] as f32 / ppm,
        spec[1] as f32 / ppm,
        spec[2] as f32 / ppm,
        spec[3] as f32 / ppm,
    );
    // World rect edges (Y-up). Content rect = sprite footprint centred at
    // the translation.
    let (c_left, c_right) = (cx - sx * 0.5, cx + sx * 0.5);
    let (c_top, c_bottom) = (cy + sy * 0.5, cy - sy * 0.5);
    let (left_w, right_w, top_w, bottom_w) = if recenter {
        // Grow outward from the content edges (content stays world-fixed
        // on Apply).
        (c_left - l, c_right + r, c_top + t, c_bottom - b)
    } else {
        // Canvas centred on the (unchanged) pivot; total per-axis growth
        // split symmetrically around it.
        let new_w = (sx + l + r).max(1.0e-3);
        let new_h = (sy + t + b).max(1.0e-3);
        (
            cx - new_w * 0.5,
            cx + new_w * 0.5,
            cy + new_h * 0.5,
            cy - new_h * 0.5,
        )
    };
    // World → screen (top-left + bottom-right corners).
    let (x0, y0) = camera.world_to_screen([left_w, top_w], window_size);
    let (x1, y1) = camera.world_to_screen([right_w, bottom_w], window_size);
    let rect = Rect::new(x0 as f64, y0 as f64, x1 as f64, y1 as f64);
    let accent = ColorToken::Accent.resolve(hero.theme);
    let color = Color::from_rgba8(accent.r, accent.g, accent.b, 235); // LITERAL-OK: overlay outline alpha
    vector_scene.inner_mut().stroke(
        &Stroke::new(2.0), // LITERAL-OK: preview outline width px
        Affine::IDENTITY,
        &Brush::Solid(color),
        None,
        &rect,
    );
}
