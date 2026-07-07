//! Shared widget-row painters for the Audio Mixer panel — a labeled slider row
//! (master-fx params: EQ, reverb Size/Return, sends, ducking Depth) and a toggle
//! button (mute / solo / effect enables). Split out of `paint.rs` to keep the
//! paint orchestrator under the panel LOC cap; both are leaf helpers over the
//! canonical gallery widgets (no bespoke chrome).

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_centered, resolve};
use ph2d_editor_core::widget::{Slider, SliderOrientation, paint_slider};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

const FX_LABEL_W: f32 = 32.0; // LITERAL-PX-OK: master-fx label column width (chrome)

/// Paint a small left label + a full-width horizontal Slider on one row (the
/// master-fx parameter rows: EQ, reverb Size/Return, sends, ducking Depth).
/// Returns the next y.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_labeled_slider(
    y: f32,
    label: &str,
    id: NodeId,
    value: f32,
    content_x: f32,
    content_w: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) -> f32 {
    let label_rect = Rect::new(content_x, y, FX_LABEL_W, Spacing::Md.px());
    paint_text_centered(
        text_system,
        scene,
        label,
        label_rect,
        TypeToken::Xs.px(),
        resolve(ColorToken::Text2, theme),
    );
    let slider_x = content_x + FX_LABEL_W + Spacing::Sm.px();
    let slider_w = (content_w - FX_LABEL_W - Spacing::Sm.px()).max(1.0);
    let slider_rect = Rect::new(slider_x, y, slider_w, Spacing::Md.px());
    let mut slider = Slider::new(id, label).orientation(SliderOrientation::Horizontal);
    slider.set_value(value.clamp(0.0, 1.0));
    paint_slider(&slider, slider_rect, scene, theme);
    hit_index.register(id, slider_rect);
    y + Spacing::Md.px() + Spacing::Sm.px()
}

/// Paint one toggle button (mute / solo / effect enable): `active_bg` tint +
/// `AccentFg` text when engaged, else `Bg3` + `Text1`. Registers `id` as the hit
/// rect.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_toggle(
    rect: Rect,
    label: &str,
    active: bool,
    active_bg: ColorToken,
    id: NodeId,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) {
    let (bg, fg) = if active {
        (active_bg, ColorToken::AccentFg)
    } else {
        (ColorToken::Bg3, ColorToken::Text1)
    };
    fill_rounded_rect(scene, rect, Radius::Sm.px(), resolve(bg, theme));
    paint_text_centered(
        text_system,
        scene,
        label,
        rect,
        TypeToken::Sm.px(),
        resolve(fg, theme),
    );
    hit_index.register(id, rect);
}
