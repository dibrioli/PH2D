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

use crate::fader::{FADER_UNITY_POS, fader_db};
use crate::state::AudioMixerState;
use crate::{
    AMIX_CLOSE, AMIX_CUTOFF, AMIX_FADER, AMIX_LIMITER, AMIX_MASTER_METER, AMIX_MASTER_MUTE, AMIX_PAN,
    AMIX_PANEL, AMIX_PLAY, AMIX_REVERB, AMIX_REVERB_MIX, AMIX_REVERB_SIZE, AudioMixerPanel,
    SUB_BUS_COUNT, SUB_BUS_LABELS, SUB_FADER, SUB_METER, SUB_MUTE, SUB_PAN, SUB_SOLO, SUB_TONE,
    snapshot,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_centered, resolve};
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
const REVERB_LABEL_W: f32 = 32.0; // LITERAL-PX-OK: reverb Size/Mix label column width (chrome)

/// One channel strip's live data (Master or a sub-bus).
struct Strip {
    label: &'static str,
    fader_id: NodeId,
    pan_id: NodeId,
    mute_id: NodeId,
    /// Meter id — clicking the meter clears its latched clip cap.
    meter_id: NodeId,
    /// Tone (low-pass cutoff) slider id — master's is `AMIX_CUTOFF`.
    tone_id: NodeId,
    /// Solo id, or `None` for the Master strip (solo applies to sub-buses only).
    solo_id: Option<NodeId>,
    /// Fader slider position in 0..1 — the dB taper (`fader.rs`) maps it to gain
    /// (in `event.rs`) and to the dB readout (here).
    gain_pos: f32,
    /// Pan slider value in 0..1 (0.5 = center) — display only; the pan→shell
    /// remap to -1..1 happens in `event.rs`.
    pan: f32,
    /// Tone slider value in 0..1 (1.0 = open) — display only; the →Hz map is in
    /// `event.rs`.
    tone: f32,
    muted: bool,
    soloed: bool,
    /// RMS fill `[L, R]` for the meter bar.
    rms: [f32; 2],
    /// Peak-hold marker `[L, R]` (decayed panel-side).
    peak_hold: [f32; 2],
    /// Latched clip flags `[L, R]` (a red cap until the meter is clicked).
    clipped: [bool; 2],
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
    let master_rms = snapshot::master_rms();
    let master_hold = snapshot::tick_master_hold();
    let master_muted = snapshot::muted();
    let master_clipped = snapshot::master_clipped();
    let sub_rms = snapshot::sub_rms();
    let sub_muted = snapshot::sub_muted();
    let sub_soloed = snapshot::sub_soloed();
    let sub_clipped = snapshot::sub_clipped();

    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    // Fader values are slider *positions* (0..1); the dB taper maps them to gain.
    let master_gain = store
        .slider(AMIX_FADER)
        .map(|(_, v)| v)
        .unwrap_or(FADER_UNITY_POS);
    let master_pan = store.slider(AMIX_PAN).map(|(_, v)| v).unwrap_or(0.5);
    let master_tone = store.slider(AMIX_CUTOFF).map(|(_, v)| v).unwrap_or(1.0);
    let sub_gain: [f32; SUB_BUS_COUNT] = std::array::from_fn(|i| {
        store
            .slider(SUB_FADER[i])
            .map(|(_, v)| v)
            .unwrap_or(FADER_UNITY_POS)
    });
    let sub_pan: [f32; SUB_BUS_COUNT] =
        std::array::from_fn(|i| store.slider(SUB_PAN[i]).map(|(_, v)| v).unwrap_or(0.5));
    let sub_tone: [f32; SUB_BUS_COUNT] =
        std::array::from_fn(|i| store.slider(SUB_TONE[i]).map(|(_, v)| v).unwrap_or(1.0));

    // Build the strips: Master first, then each sub-bus in canonical order.
    let mut strips = Vec::with_capacity(1 + SUB_BUS_COUNT);
    strips.push(Strip {
        label: "Master",
        fader_id: AMIX_FADER,
        pan_id: AMIX_PAN,
        mute_id: AMIX_MASTER_MUTE,
        meter_id: AMIX_MASTER_METER,
        tone_id: AMIX_CUTOFF,
        solo_id: None, // solo applies to sub-buses only
        gain_pos: master_gain,
        pan: master_pan,
        tone: master_tone,
        muted: master_muted,
        soloed: false,
        rms: master_rms,
        peak_hold: master_hold,
        clipped: master_clipped,
    });
    for i in 0..SUB_BUS_COUNT {
        strips.push(Strip {
            label: SUB_BUS_LABELS[i],
            fader_id: SUB_FADER[i],
            pan_id: SUB_PAN[i],
            mute_id: SUB_MUTE[i],
            meter_id: SUB_METER[i],
            tone_id: SUB_TONE[i],
            solo_id: Some(SUB_SOLO[i]),
            gain_pos: sub_gain[i],
            pan: sub_pan[i],
            tone: sub_tone[i],
            muted: sub_muted[i],
            soloed: sub_soloed[i],
            rms: sub_rms[i],
            peak_hold: snapshot::tick_sub_hold(i),
            clipped: sub_clipped[i],
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

    // Master section below the strips. Strip stack (see `paint_strip`):
    // label · pan · tone · fader/meter · dB readout · M/S.
    let strips_bottom = strip_top
        + TypeToken::Sm.px()
        + Spacing::Sm.px()
        + Spacing::Md.px()
        + Spacing::Sm.px()
        + Spacing::Md.px()
        + Spacing::Sm.px()
        + STRIP_H
        + Spacing::Sm.px()
        + TypeToken::Xs.px()
        + Spacing::Sm.px()
        + MUTE_H;
    let footer_y = strips_bottom + Spacing::Lg.px();

    // Master section footer: limiter · reverb · Play Test.
    paint_master_section(
        footer_y,
        content_x,
        content_w,
        ctx.scene,
        ctx.text_system,
        theme,
        hit_index,
    );
}

/// The master-section footer below the strips: Limiter toggle · Reverb (toggle +
/// Size/Mix) · Play Test. Split out of `paint` to stay under the fn LOC cap.
#[allow(clippy::too_many_arguments)]
fn paint_master_section(
    mut y: f32,
    content_x: f32,
    content_w: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) {
    // Master output limiter (Accent when engaged) — tames peaks below the clip
    // ceiling instead of hard-clipping.
    paint_toggle(
        Rect::new(content_x, y, content_w, MUTE_H),
        "Limiter",
        snapshot::limiter(),
        ColorToken::Accent,
        AMIX_LIMITER,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += MUTE_H + Spacing::Md.px();

    // Master reverb: enable toggle + Size (decay) + Mix (wet/dry) thin sliders.
    paint_toggle(
        Rect::new(content_x, y, content_w, MUTE_H),
        "Reverb",
        snapshot::reverb_on(),
        ColorToken::Accent,
        AMIX_REVERB,
        scene,
        text_system,
        theme,
        hit_index,
    );
    y += MUTE_H + Spacing::Sm.px();
    for (label, id, value) in [
        ("Size", AMIX_REVERB_SIZE, snapshot::reverb_size()),
        ("Mix", AMIX_REVERB_MIX, snapshot::reverb_mix()),
    ] {
        let label_rect = Rect::new(content_x, y, REVERB_LABEL_W, Spacing::Md.px());
        paint_text_centered(
            text_system,
            scene,
            label,
            label_rect,
            TypeToken::Xs.px(),
            resolve(ColorToken::Text2, theme),
        );
        let slider_x = content_x + REVERB_LABEL_W + Spacing::Sm.px();
        let slider_w = (content_w - REVERB_LABEL_W - Spacing::Sm.px()).max(1.0);
        let slider_rect = Rect::new(slider_x, y, slider_w, Spacing::Md.px());
        let mut slider = Slider::new(id, label).orientation(SliderOrientation::Horizontal);
        slider.set_value(value.clamp(0.0, 1.0));
        paint_slider(&slider, slider_rect, scene, theme);
        hit_index.register(id, slider_rect);
        y += Spacing::Md.px() + Spacing::Sm.px();
    }
    y += Spacing::Sm.px();

    // Built-in test oscillator toggle (Accent when playing) so the mixer can be
    // exercised without an external file. The shell owns the signal.
    let playing = snapshot::play_test();
    paint_toggle(
        Rect::new(content_x, y, content_w, MUTE_H),
        if playing { "Stop" } else { "Play Test" },
        playing,
        ColorToken::Accent,
        AMIX_PLAY,
        scene,
        text_system,
        theme,
        hit_index,
    );
}

/// Paint one channel strip in its column: label · pan · fader (standard
/// `Slider`, unity tick) + meter · dB readout · mute (+ solo on sub-buses).
/// Registers the pan/fader/mute/solo hit rects.
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

    // Tone — a thin full-column horizontal low-pass cutoff Slider (open = 1.0).
    let tone_rect = Rect::new(col_x, y, col_w, Spacing::Md.px());
    let mut tone =
        Slider::new(strip.tone_id, strip.label).orientation(SliderOrientation::Horizontal);
    tone.set_value(strip.tone.clamp(0.0, 1.0));
    paint_slider(&tone, tone_rect, scene, theme);
    hit_index.register(strip.tone_id, tone_rect);
    y += Spacing::Md.px() + Spacing::Sm.px();

    // Fader + meter cluster, centered in the column. The fader carries a tick at
    // the unity (0 dB) position so the dB taper reads at a glance.
    let cluster_w = FADER_W + Spacing::Sm.px() + METER_W;
    let cluster_x = col_x + ((col_w - cluster_w) * 0.5).max(0.0);

    let fader_rect = Rect::new(cluster_x, y, FADER_W, STRIP_H);
    let mut fader =
        Slider::new(strip.fader_id, strip.label).orientation(SliderOrientation::Vertical);
    fader.set_value(strip.gain_pos.clamp(0.0, 1.0));
    fader.ticks = vec![FADER_UNITY_POS];
    paint_slider(&fader, fader_rect, scene, theme);
    hit_index.register(strip.fader_id, fader_rect);

    let meter_rect = Rect::new(cluster_x + FADER_W + Spacing::Sm.px(), y, METER_W, STRIP_H);
    let m = LevelMeter::new(strip.fader_id, strip.label)
        .rms(strip.rms[0], strip.rms[1])
        .peak_hold(strip.peak_hold[0], strip.peak_hold[1])
        .clipped(strip.clipped[0], strip.clipped[1]);
    paint_level_meter(&m, meter_rect, scene, theme);
    // Click the meter to clear its latched clip cap.
    hit_index.register(strip.meter_id, meter_rect);
    y += STRIP_H + Spacing::Sm.px();

    // dB readout — the current fader gain in dB (or "-inf" at the bottom).
    let readout = if strip.gain_pos <= 0.0 {
        "-inf".to_string()
    } else {
        format!("{:.0} dB", fader_db(strip.gain_pos))
    };
    let readout_rect = Rect::new(col_x, y, col_w, TypeToken::Xs.px());
    paint_text_centered(
        text_system,
        scene,
        &readout,
        readout_rect,
        TypeToken::Xs.px(),
        resolve(ColorToken::Text2, theme),
    );
    y += TypeToken::Xs.px() + Spacing::Sm.px();

    // Mute (+ solo on sub-buses) button row. Master gets a full-width Mute; a
    // sub-bus splits the row into M | S (the mixer-console convention).
    match strip.solo_id {
        None => {
            paint_toggle(
                Rect::new(col_x, y, col_w, MUTE_H),
                "Mute",
                strip.muted,
                ColorToken::Danger,
                strip.mute_id,
                scene,
                text_system,
                theme,
                hit_index,
            );
        }
        Some(solo_id) => {
            let gap = Spacing::Sm.px();
            let half = ((col_w - gap) * 0.5).max(1.0);
            paint_toggle(
                Rect::new(col_x, y, half, MUTE_H),
                "M",
                strip.muted,
                ColorToken::Danger,
                strip.mute_id,
                scene,
                text_system,
                theme,
                hit_index,
            );
            paint_toggle(
                Rect::new(col_x + half + gap, y, half, MUTE_H),
                "S",
                strip.soloed,
                ColorToken::Warn,
                solo_id,
                scene,
                text_system,
                theme,
                hit_index,
            );
        }
    }
}

/// Paint one toggle button (mute / solo): `active_bg` tint + `AccentFg` text
/// when engaged, else `Bg3` + `Text1`. Registers `id` as the hit rect.
#[allow(clippy::too_many_arguments)]
fn paint_toggle(
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
