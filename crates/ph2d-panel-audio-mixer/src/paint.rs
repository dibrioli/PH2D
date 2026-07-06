//! Audio Mixer panel paint — the Master channel strip.
//!
//! Renders a fixed floating panel: surface + title + Master strip (name +
//! vertical fader visual + live stereo [`LevelMeter`] + mute button). The fader
//! is a readout of the published master gain for now; drag→gain lands with
//! orientation-aware slider dispatch in a follow-up.

use crate::state::AudioMixerState;
use crate::{AMIX_MASTER_MUTE, AMIX_PANEL, AudioMixerPanel, snapshot};
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, paint_text_centered, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEADER_H_DEFAULT, paint_panel_surface, paint_panel_title,
};
use ph2d_editor_core::widget::{
    LevelMeter, SliderOrientation, paint_level_meter, paint_slider_track,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

const PANEL_W: f32 = 190.0; // LITERAL-PX-OK: mixer panel width (chrome)
const PANEL_H: f32 = 300.0; // LITERAL-PX-OK: mixer panel height (chrome)
const PANEL_MARGIN: f32 = 16.0; // LITERAL-PX-OK: viewport inset (chrome)
const PANEL_TOP: f32 = 96.0; // LITERAL-PX-OK: below TopBar (chrome)
const FADER_W: f32 = 24.0; // LITERAL-PX-OK: master fader width (chrome)
const METER_W: f32 = 18.0; // LITERAL-PX-OK: master meter width (chrome)
const STRIP_H: f32 = 150.0; // LITERAL-PX-OK: master fader/meter height (chrome)
const MUTE_H: f32 = 26.0; // LITERAL-PX-OK: mute button height (chrome)

/// Fixed default geometry (top-left, below the TopBar).
pub fn default_rect(_viewport_w: f32, _viewport_h: f32) -> Rect {
    Rect::new(PANEL_MARGIN, PANEL_TOP, PANEL_W, PANEL_H)
}

pub(crate) fn paint(state: &mut AudioMixerState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(AudioMixerPanel::ID) {
        ctx.host.store_mut().clear_panel_rect(AMIX_PANEL);
        return;
    }
    let rect = *state
        .rect
        .get_or_insert_with(|| default_rect(ctx.layout.viewport.w, ctx.layout.viewport.h));
    ctx.host.store_mut().set_panel_rect(AMIX_PANEL, rect);

    let theme = ctx.host.theme();
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_title(rect, "Audio Mixer", 0.0, ctx.scene, ctx.text_system, theme);

    let (_store, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_master_strip(rect, ctx.scene, ctx.text_system, theme, hit_index);
}

/// Paint the Master channel strip inside the panel.
fn paint_master_strip(
    panel: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) {
    let levels = snapshot::levels();
    let gain = snapshot::master_gain();
    let muted = snapshot::muted();

    let pad = Spacing::Lg.px();
    let content_x = panel.x + pad;
    let content_w = (panel.w - pad * 2.0).max(1.0);
    let mut y = panel.y + PANEL_HEADER_H_DEFAULT;

    // Strip name.
    paint_text(
        text_system,
        scene,
        "Master",
        content_x,
        y,
        TypeToken::Base.px(),
        content_w,
        resolve(ColorToken::Text1, theme),
    );
    y += TypeToken::Base.px() + Spacing::Sm.px();

    // Vertical fader (visual readout of master gain) + stereo level meter.
    let fader = Rect::new(content_x, y, FADER_W, STRIP_H);
    paint_slider_track(
        fader,
        gain.clamp(0.0, 1.0),
        SliderOrientation::Vertical,
        scene,
        theme,
    );
    let meter = Rect::new(content_x + FADER_W + Spacing::Sm.px(), y, METER_W, STRIP_H);
    let m = LevelMeter::new(AMIX_PANEL, "Master").levels(levels[0], levels[1]);
    paint_level_meter(&m, meter, scene, theme);
    y += STRIP_H + Spacing::Lg.px();

    // Mute button — panel-owned state (Danger tint when engaged).
    let mute_rect = Rect::new(content_x, y, content_w, MUTE_H);
    let bg = if muted {
        ColorToken::Danger
    } else {
        ColorToken::Bg3
    };
    let fg = if muted {
        ColorToken::AccentFg
    } else {
        ColorToken::Text1
    };
    fill_rounded_rect(scene, mute_rect, Radius::Sm.px(), resolve(bg, theme));
    paint_text_centered(
        text_system,
        scene,
        "Mute",
        mute_rect,
        TypeToken::Base.px(),
        resolve(fg, theme),
    );
    hit_index.register(AMIX_MASTER_MUTE, mute_rect);
}
