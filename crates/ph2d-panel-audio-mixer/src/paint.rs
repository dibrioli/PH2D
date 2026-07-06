//! Audio Mixer panel paint — **right-docked in the shared Inspector slot**
//! (mirror of the Sprite Inspector / Painter Layers dock pattern), NOT a
//! floating panel. Reads `ctx.layout.inspector` for its rect and registers the
//! shared `INSP_*` drag/resize handles so it moves/resizes with the dock slot.

use crate::state::AudioMixerState;
use crate::{
    AMIX_CLOSE, AMIX_CUTOFF, AMIX_FADER, AMIX_MASTER_MUTE, AMIX_PANEL, AudioMixerPanel, snapshot,
};
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, paint_text_centered, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface, paint_panel_title, panel_drag_handle_rect, panel_resize_handle_rect,
    panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    LevelMeter, SliderOrientation, paint_level_meter, paint_slider_track,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

const FADER_W: f32 = 24.0; // LITERAL-PX-OK: master fader width (chrome)
const METER_W: f32 = 18.0; // LITERAL-PX-OK: master meter width (chrome)
const STRIP_H: f32 = 160.0; // LITERAL-PX-OK: master fader/meter height (chrome)
const MUTE_H: f32 = 28.0; // LITERAL-PX-OK: mute button height (chrome)

pub(crate) fn paint(_state: &mut AudioMixerState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(AudioMixerPanel::ID) {
        ctx.host.store_mut().clear_panel_rect(AMIX_PANEL);
        return;
    }
    // Right-docked in the shared Inspector slot (already clamped + drag/resize
    // applied by the orchestrator into `layout.inspector`).
    let rect: Rect = ctx.layout.inspector;
    let theme = ctx.host.theme();
    ctx.host.store_mut().set_panel_rect(AMIX_PANEL, rect);

    // Opaque backing: `paint_panel_surface` is 0.96-alpha "glass", so without
    // this the Inspector still painting behind the shared dock slot bleeds
    // through. The mixer takes over the slot → fully cover it.
    fill_rounded_rect(
        ctx.scene,
        rect,
        Radius::Sm.px(),
        resolve(ColorToken::BgElev, theme),
    );
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);

    // Shared dock drag/resize handles (Inspector right-dock canon) — reuse the
    // `INSP_*` ids so the dock slot moves/resizes as one.
    {
        let drag_rect =
            panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
        let resize_rect = panel_resize_handle_rect(rect);
        let resize_bl_rect = panel_resize_handle_rect_bl(rect);
        let hit_index = ctx.host.hit_index_mut();
        hit_index.register(core_ids::INSP_DRAG_HANDLE, drag_rect);
        hit_index.register(core_ids::INSP_RESIZE_HANDLE, resize_rect);
        hit_index.register(core_ids::INSP_RESIZE_HANDLE_BL, resize_bl_rect);
    }

    let title_size = paint_panel_title(
        rect,
        "Audio Mixer",
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(rect, AMIX_CLOSE, ctx.host.hit_index_mut(), ctx.scene, theme);
    let header_bottom = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();

    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    // The fader's live value is the source of truth (the shared slider dispatch
    // writes it on drag); the shell reads the published gain to drive the engine.
    let fader_gain = store.slider(AMIX_FADER).map(|(_, v)| v).unwrap_or(1.0);
    let cutoff = store.slider(AMIX_CUTOFF).map(|(_, v)| v).unwrap_or(1.0);
    paint_master_strip(
        rect,
        header_bottom,
        fader_gain,
        cutoff,
        ctx.scene,
        ctx.text_system,
        theme,
        hit_index,
    );
}

/// Paint the Master channel strip below the header.
#[allow(clippy::too_many_arguments)]
fn paint_master_strip(
    panel: Rect,
    top: f32,
    gain: f32,
    cutoff: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut ph2d_editor_core::interaction::HitIndex,
) {
    let levels = snapshot::levels();
    let muted = snapshot::muted();

    let pad = Spacing::Lg.px();
    let content_x = panel.x + pad;
    let content_w = (panel.w - pad * 2.0).max(1.0);
    let mut y = top;

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
    hit_index.register(AMIX_FADER, fader);
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
    y += MUTE_H + Spacing::Lg.px();

    // Master low-pass cutoff — label + horizontal slider (drag → filter sweep).
    paint_text(
        text_system,
        scene,
        "Cutoff",
        content_x,
        y,
        TypeToken::Base.px(),
        content_w,
        resolve(ColorToken::Text2, theme),
    );
    y += TypeToken::Base.px() + Spacing::Sm.px();
    let cutoff_rect = Rect::new(content_x, y, content_w, Spacing::Lg.px());
    paint_slider_track(
        cutoff_rect,
        cutoff.clamp(0.0, 1.0),
        SliderOrientation::Horizontal,
        scene,
        theme,
    );
    hit_index.register(AMIX_CUTOFF, cutoff_rect);
}
