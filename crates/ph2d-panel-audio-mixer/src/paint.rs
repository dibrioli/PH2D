//! Audio Mixer panel paint — **right-docked in the shared Inspector slot**
//! (mirror of the Sprite Inspector / Painter Layers dock pattern), NOT a
//! floating panel. Reads `ctx.layout.inspector` for its rect and registers the
//! shared `INSP_*` drag/resize handles so it moves/resizes with the dock slot.
//!
//! Vertical channel strips — one Master + one per sub-bus (Music, SFX) — each a
//! label + a standard [`ph2d_editor_core::widget::Slider`] fader + a live
//! [`ph2d_editor_core::widget::LevelMeter`] + a mute toggle. Every fader is the
//! canonical `Slider` widget (single source of truth for the slider look), so
//! the panel is built from gallery widgets, not bespoke chrome.

use crate::state::AudioMixerState;
use crate::{
    AMIX_CLOSE, AMIX_CUTOFF, AMIX_FADER, AMIX_MASTER_MUTE, AMIX_PAN, AMIX_PANEL, AudioMixerPanel,
    SUB_BUS_COUNT, SUB_BUS_LABELS, SUB_FADER, SUB_MUTE, SUB_PAN, snapshot,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, paint_text_centered, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface, paint_panel_title, panel_drag_handle_rect, panel_resize_handle_rect,
    panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    LevelMeter, Slider, SliderOrientation, paint_level_meter, paint_slider,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

const FADER_W: f32 = 22.0; // LITERAL-PX-OK: fader column width (chrome)
const METER_W: f32 = 14.0; // LITERAL-PX-OK: meter column width (chrome)
const STRIP_H: f32 = 150.0; // LITERAL-PX-OK: fader/meter height (chrome)
const MUTE_H: f32 = 24.0; // LITERAL-PX-OK: mute button height (chrome)

/// One channel strip's live data (Master or a sub-bus).
struct Strip {
    label: &'static str,
    fader_id: NodeId,
    pan_id: NodeId,
    mute_id: NodeId,
    gain: f32,
    /// Pan slider value in 0..1 (0.5 = center) — display only; the pan→shell
    /// remap to -1..1 happens in `event.rs`.
    pan: f32,
    muted: bool,
    levels: [f32; 2],
}

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

    // Gather the live snapshot (shell → panel) + the fader values (the store is
    // the source of truth — the shared slider dispatch writes them on drag).
    let master_levels = snapshot::levels();
    let master_muted = snapshot::muted();
    let sub_levels = snapshot::sub_levels();
    let sub_muted = snapshot::sub_muted();

    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let master_gain = store.slider(AMIX_FADER).map(|(_, v)| v).unwrap_or(1.0);
    let master_pan = store.slider(AMIX_PAN).map(|(_, v)| v).unwrap_or(0.5);
    let cutoff = store.slider(AMIX_CUTOFF).map(|(_, v)| v).unwrap_or(1.0);
    let sub_gain: [f32; SUB_BUS_COUNT] =
        std::array::from_fn(|i| store.slider(SUB_FADER[i]).map(|(_, v)| v).unwrap_or(1.0));
    let sub_pan: [f32; SUB_BUS_COUNT] =
        std::array::from_fn(|i| store.slider(SUB_PAN[i]).map(|(_, v)| v).unwrap_or(0.5));

    // Build the strips: Master first, then each sub-bus in canonical order.
    let mut strips = Vec::with_capacity(1 + SUB_BUS_COUNT);
    strips.push(Strip {
        label: "Master",
        fader_id: AMIX_FADER,
        pan_id: AMIX_PAN,
        mute_id: AMIX_MASTER_MUTE,
        gain: master_gain,
        pan: master_pan,
        muted: master_muted,
        levels: master_levels,
    });
    for i in 0..SUB_BUS_COUNT {
        strips.push(Strip {
            label: SUB_BUS_LABELS[i],
            fader_id: SUB_FADER[i],
            pan_id: SUB_PAN[i],
            mute_id: SUB_MUTE[i],
            gain: sub_gain[i],
            pan: sub_pan[i],
            muted: sub_muted[i],
            levels: sub_levels[i],
        });
    }

    // Responsive column layout: split the content width across the strips.
    let pad = Spacing::Lg.px();
    let content_x = rect.x + pad;
    let content_w = (rect.w - pad * 2.0).max(1.0);
    let gap = Spacing::Sm.px();
    let cols = strips.len() as f32;
    let col_w = ((content_w - gap * (cols - 1.0)) / cols).max(FADER_W);
    let strip_top = header_bottom;

    for (c, strip) in strips.iter().enumerate() {
        let col_x = content_x + c as f32 * (col_w + gap);
        paint_strip(
            strip,
            col_x,
            strip_top,
            col_w,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
        );
    }

    // Master low-pass cutoff — full-width standard horizontal Slider below the
    // strips (drag → filter sweep). Labelled, so its role reads at a glance.
    // The strip stack is: label · pan · fader/meter · mute (see `paint_strip`).
    let strips_bottom = strip_top
        + TypeToken::Sm.px()
        + Spacing::Sm.px()
        + Spacing::Md.px()
        + Spacing::Sm.px()
        + STRIP_H
        + Spacing::Sm.px()
        + MUTE_H;
    let mut y = strips_bottom + Spacing::Lg.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        "Cutoff",
        content_x,
        y,
        TypeToken::Sm.px(),
        content_w,
        resolve(ColorToken::Text2, theme),
    );
    y += TypeToken::Sm.px() + Spacing::Sm.px();
    let cutoff_rect = Rect::new(content_x, y, content_w, Spacing::Lg.px());
    let mut cutoff_slider = Slider::new(AMIX_CUTOFF, "Cutoff");
    cutoff_slider.set_value(cutoff.clamp(0.0, 1.0));
    paint_slider(&cutoff_slider, cutoff_rect, ctx.scene, theme);
    hit_index.register(AMIX_CUTOFF, cutoff_rect);
}

/// Paint one channel strip in its column: label · fader (standard `Slider`) +
/// meter · mute button. Registers the fader + mute hit rects.
#[allow(clippy::too_many_arguments)]
fn paint_strip(
    strip: &Strip,
    col_x: f32,
    top: f32,
    col_w: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) {
    // Column title.
    let label_rect = Rect::new(col_x, top, col_w, TypeToken::Sm.px());
    paint_text_centered(
        text_system,
        scene,
        strip.label,
        label_rect,
        TypeToken::Sm.px(),
        resolve(ColorToken::Text1, theme),
    );
    let mut y = top + TypeToken::Sm.px() + Spacing::Sm.px();

    // Pan / balance — a thin full-column horizontal Slider (center = 0.5).
    let pan_rect = Rect::new(col_x, y, col_w, Spacing::Md.px());
    let mut pan = Slider::new(strip.pan_id, strip.label).orientation(SliderOrientation::Horizontal);
    pan.set_value(strip.pan.clamp(0.0, 1.0));
    paint_slider(&pan, pan_rect, scene, theme);
    hit_index.register(strip.pan_id, pan_rect);
    y += Spacing::Md.px() + Spacing::Sm.px();

    // Fader + meter cluster, centered in the column.
    let cluster_w = FADER_W + Spacing::Sm.px() + METER_W;
    let cluster_x = col_x + ((col_w - cluster_w) * 0.5).max(0.0);

    let fader_rect = Rect::new(cluster_x, y, FADER_W, STRIP_H);
    let mut fader = Slider::new(strip.fader_id, strip.label).orientation(SliderOrientation::Vertical);
    fader.set_value(strip.gain.clamp(0.0, 1.0));
    paint_slider(&fader, fader_rect, scene, theme);
    hit_index.register(strip.fader_id, fader_rect);

    let meter_rect = Rect::new(cluster_x + FADER_W + Spacing::Sm.px(), y, METER_W, STRIP_H);
    let m = LevelMeter::new(strip.fader_id, strip.label).levels(strip.levels[0], strip.levels[1]);
    paint_level_meter(&m, meter_rect, scene, theme);
    y += STRIP_H + Spacing::Sm.px();

    // Mute button — Danger tint when engaged.
    let mute_rect = Rect::new(col_x, y, col_w, MUTE_H);
    let (bg, fg) = if strip.muted {
        (ColorToken::Danger, ColorToken::AccentFg)
    } else {
        (ColorToken::Bg3, ColorToken::Text1)
    };
    fill_rounded_rect(scene, mute_rect, Radius::Sm.px(), resolve(bg, theme));
    paint_text_centered(
        text_system,
        scene,
        "Mute",
        mute_rect,
        TypeToken::Sm.px(),
        resolve(fg, theme),
    );
    hit_index.register(strip.mute_id, mute_rect);
}
