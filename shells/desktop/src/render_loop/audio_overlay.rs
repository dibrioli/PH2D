//! Audio Editor floating waveform overlay (docs/Audio/, W1).
//!
//! A resizable window floating over the canvas in the gap between the Hierarchy
//! and Inspector docks, showing the loaded clip's **waveform + time ruler +
//! playhead**. Painted AFTER `paint_hero_screen` (on top of the hero chrome; it
//! never overlaps the docks since it lives in the gap).
//!
//! Drag/resize reuse the panel-agnostic `BlenderHit` mechanism keyed to
//! `AUDIO_OVERLAY_PANEL` (the handles are registered as `InteractiveState` by the
//! editor panel's `populate`; this bridge registers their hit rects each frame
//! and applies the stored `blender_picker_offset` + `panel_resize_delta`) —
//! exactly the Inspector dock's move/resize recipe under a different id.

use ph2d_editor::ids;
use ph2d_editor::paint::{fill_rounded_rect, paint_text_centered, resolve};
use ph2d_editor::screens::HeroScreen;
use ph2d_editor::screens::layout::{EDGE_PAD, HIERARCHY_W, INSPECTOR_W, RAIL_W};
use ph2d_editor::widget::panel_chrome::{
    PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, paint_panel_corner_dot,
    paint_panel_corner_dot_bl, paint_panel_surface, panel_drag_handle_rect,
    panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

use crate::audio::AudioSystem;

const MIN_W: f32 = 260.0; // LITERAL-PX-OK: overlay minimum width (chrome)
const MIN_H: f32 = 140.0; // LITERAL-PX-OK: overlay minimum height (chrome)
const DEFAULT_H: f32 = 260.0; // LITERAL-PX-OK: overlay default height (chrome)
const TOP_MARGIN: f32 = 72.0; // LITERAL-PX-OK: below the TopBar (chrome)
const HEADER_H: f32 = 26.0; // LITERAL-PX-OK: overlay title-bar height (chrome)
const RULER_H: f32 = 16.0; // LITERAL-PX-OK: time-ruler strip height (chrome)

/// Paint the Audio Editor waveform overlay if the editor panel is open and a
/// clip is loaded. No-op otherwise.
pub(super) fn draw_audio_overlay(
    hero: &mut HeroScreen,
    audio: &AudioSystem,
    viewport: Rect,
    scene: &mut VectorScene,
    text: &mut TextSystem,
) {
    if !hero.is_panel_visible("audio_editor") {
        hero.store.clear_panel_rect(ids::AUDIO_OVERLAY_PANEL);
        return;
    }
    let Some(clip) = audio.editor_clip() else {
        return;
    };
    let theme = hero.theme;

    // Default rect: a band in the Hierarchy↔Inspector gap, below the TopBar.
    let gap = gap_rect(viewport);
    let base = Rect::new(
        gap.x,
        gap.y,
        gap.w,
        DEFAULT_H.min((gap.h - TOP_MARGIN).max(MIN_H)),
    );
    // Apply the stored drag offset + resize delta, then clamp into the gap.
    let (ox, oy) = hero.store.blender_picker_offset(ids::AUDIO_OVERLAY_PANEL);
    let (dw, dh) = hero.store.panel_resize_delta(ids::AUDIO_OVERLAY_PANEL);
    let rect = clamp_to_gap(base, ox, oy, dw, dh, gap);
    hero.store.set_panel_rect(ids::AUDIO_OVERLAY_PANEL, rect);

    // Frame surface + corner dots.
    fill_rounded_rect(
        scene,
        rect,
        Radius::Sm.px(),
        resolve(ColorToken::BgElev, theme),
    );
    paint_panel_surface(rect, scene, theme);
    paint_panel_corner_dot(rect, scene, theme);
    paint_panel_corner_dot_bl(rect, scene, theme);

    // Title bar.
    let header = Rect::new(rect.x, rect.y, rect.w, HEADER_H);
    paint_text_centered(
        text,
        scene,
        "Audio Editor \u{00b7} Waveform",
        header,
        TypeToken::Xs.px(),
        resolve(ColorToken::Text2, theme),
    );

    // Waveform + ruler area (below the header).
    let body_top = rect.y + HEADER_H;
    let ruler_top = rect.y + rect.h - RULER_H;
    let wave = Rect::new(
        rect.x + Spacing::Sm.px(),
        body_top,
        (rect.w - Spacing::Sm.px() * 2.0).max(1.0),
        (ruler_top - body_top).max(1.0),
    );
    draw_waveform(scene, clip, wave, theme);
    draw_ruler(scene, text, clip, wave, ruler_top, theme);
    draw_playhead(scene, clip, wave, audio.editor_preview_frame(), theme);

    // Register drag + resize handle hit rects into the hero hit-index so the
    // shared BlenderHit dispatch moves/resizes the overlay next frame.
    let drag = panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
    hero.hit_index.register(ids::AUDIO_OVERLAY_DRAG_HANDLE, drag);
    hero.hit_index
        .register(ids::AUDIO_OVERLAY_RESIZE_HANDLE, panel_resize_handle_rect(rect));
    hero.hit_index.register(
        ids::AUDIO_OVERLAY_RESIZE_HANDLE_BL,
        panel_resize_handle_rect_bl(rect),
    );
}

/// The Hierarchy↔Inspector gap rect (unmirrored default layout), below the
/// TopBar. The overlay's default position + clamp bounds.
fn gap_rect(viewport: Rect) -> Rect {
    let hier_right = viewport.x + RAIL_W + EDGE_PAD + HIERARCHY_W;
    let insp_left = viewport.x + viewport.w - EDGE_PAD - INSPECTOR_W;
    let x = hier_right + EDGE_PAD;
    let w = (insp_left - EDGE_PAD - x).max(MIN_W);
    let y = viewport.y + TOP_MARGIN;
    let h = (viewport.h - TOP_MARGIN - EDGE_PAD).max(MIN_H);
    Rect::new(x, y, w, h)
}

/// Apply drag offset + resize delta to `base`, clamp size to `MIN_*`, and keep
/// the whole rect inside `gap` (full-containment, like `clamp_panel_rect`).
fn clamp_to_gap(base: Rect, ox: f32, oy: f32, dw: f32, dh: f32, gap: Rect) -> Rect {
    let w = (base.w + dw).clamp(MIN_W, gap.w.max(MIN_W));
    let h = (base.h + dh).clamp(MIN_H, gap.h.max(MIN_H));
    let x = (base.x + ox).clamp(gap.x, (gap.x + gap.w - w).max(gap.x));
    let y = (base.y + oy).clamp(gap.y, (gap.y + gap.h - h).max(gap.y));
    Rect::new(x, y, w, h)
}

/// Draw the clip's min/max envelope across `area`, one lane per channel.
fn draw_waveform(scene: &mut VectorScene, clip: &ph2d_audio_edit::EditClip, area: Rect, theme: Theme) {
    let channels = clip.data().format().channel_count().max(1);
    let columns = (area.w as usize).clamp(1, 4096);
    let peaks = clip.column_peaks(0, clip.frame_count(), columns);
    let color = resolve(ColorToken::Accent, theme);
    let mid = resolve(ColorToken::Border, theme);

    let lane_h = area.h / channels as f32;
    for ch in 0..channels {
        let lane_top = area.y + ch as f32 * lane_h;
        let center = lane_top + lane_h * 0.5;
        let half = (lane_h * 0.5 - 1.0).max(1.0);
        // Zero line.
        fill_rounded_rect(
            scene,
            Rect::new(area.x, center, area.w, 1.0),
            0.0,
            mid,
        );
        for c in 0..columns {
            let (lo, hi) = peaks.get(c, ch);
            let col_x = area.x + c as f32 * area.w / columns as f32;
            let y_top = center - hi.clamp(-1.0, 1.0) * half;
            let y_bot = center - lo.clamp(-1.0, 1.0) * half;
            let h = (y_bot - y_top).max(1.0);
            fill_rounded_rect(
                scene,
                Rect::new(col_x, y_top, (area.w / columns as f32).max(1.0), h),
                0.0,
                color,
            );
        }
    }
}

/// Draw a few evenly-spaced time ticks + labels along the ruler strip.
fn draw_ruler(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    clip: &ph2d_audio_edit::EditClip,
    area: Rect,
    ruler_top: f32,
    theme: Theme,
) {
    let dur = clip.duration_secs();
    let ticks = 5;
    let tick_col = resolve(ColorToken::Border, theme);
    let label_col = resolve(ColorToken::Text2, theme);
    for i in 0..=ticks {
        let frac = i as f32 / ticks as f32;
        let x = area.x + frac * area.w;
        fill_rounded_rect(scene, Rect::new(x, ruler_top, 1.0, RULER_H), 0.0, tick_col);
        let secs = dur * frac as f64;
        let label = format!("{secs:.1}s");
        let lw = 34.0; // LITERAL-PX-OK: tick label box width (chrome)
        paint_text_centered(
            text,
            scene,
            &label,
            Rect::new(x - lw * 0.5, ruler_top + 2.0, lw, TypeToken::Xs.px()),
            TypeToken::Xs.px(),
            label_col,
        );
    }
}

/// Draw the playback playhead as a vertical line at the current preview frame.
fn draw_playhead(
    scene: &mut VectorScene,
    clip: &ph2d_audio_edit::EditClip,
    area: Rect,
    frame: u64,
    theme: Theme,
) {
    let total = clip.frame_count().max(1) as f64;
    let frac = (frame as f64 / total).clamp(0.0, 1.0) as f32;
    let x = area.x + frac * area.w;
    fill_rounded_rect(
        scene,
        Rect::new(x, area.y, 2.0, area.h),
        0.0,
        resolve(ColorToken::Warn, theme),
    );
}
