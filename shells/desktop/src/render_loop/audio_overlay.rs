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
    PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, clamp_panel_rect, paint_panel_corner_dot,
    paint_panel_corner_dot_bl, paint_panel_surface, panel_drag_handle_rect,
    panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

use crate::audio::AudioSystem;

const MIN_W: f32 = 260.0; // LITERAL-PX-OK: overlay default minimum width (chrome)
const DEFAULT_H: f32 = 260.0; // LITERAL-PX-OK: overlay default height (chrome)
const TOP_MARGIN: f32 = 72.0; // LITERAL-PX-OK: below the TopBar (chrome)
const HEADER_H: f32 = 26.0; // LITERAL-PX-OK: overlay title-bar height (chrome)
const RULER_H: f32 = 16.0; // LITERAL-PX-OK: time-ruler strip height (chrome)
/// Width reserved on the right for the docked Audio Editor panel (matches its
/// `PANEL_W`), so the overlay's default position doesn't cover it.
const EDITOR_PANEL_W: f32 = 240.0; // LITERAL-PX-OK: docked editor panel width (chrome)

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
        crate::audio::set_wave_view(None);
        return;
    }
    let Some(clip) = audio.editor_clip() else {
        crate::audio::set_wave_view(None);
        return;
    };
    let theme = hero.theme;

    // Drag/resize use the SAME proven math as the Widget Gallery: `clamp_panel_rect`
    // against the full viewport, with a `base` strictly SMALLER than that envelope
    // so the stored drag offset + resize delta have real travel. (A base that fills
    // its clamp envelope collapses the range to a single point → dead drag/resize —
    // the original bug.) Default position sits in the canvas, left of the docked
    // Audio Editor panel; from there it floats/resizes anywhere like the Gallery.
    let base = default_rect(viewport);
    let off = hero.store.blender_picker_offset(ids::AUDIO_OVERLAY_PANEL);
    let resize = hero.store.panel_resize_delta(ids::AUDIO_OVERLAY_PANEL);
    let (rect, _, _) = clamp_panel_rect(base, off, resize, viewport);
    hero.store.set_panel_rect(ids::AUDIO_OVERLAY_PANEL, rect);
    // Body hit-barrier FIRST — clicks on the empty overlay body must not fall
    // through to the canvas tool. The handles below register AFTER, so (newest
    // wins in the hit-index) they still outrank this barrier.
    hero.hit_index.register(ids::AUDIO_OVERLAY_PANEL, rect);

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
    // The playhead frame (offset into the loop region while it loops), computed once:
    // it drives both the waveform's played/unplayed shading and the playhead line.
    let ph_frame = audio.editor_playhead_frame();
    let total = clip.frame_count().max(1) as f32;
    let played_frac = (ph_frame as f32 / total).clamp(0.0, 1.0);
    // Only split played/unplayed once playback has advanced; a stopped clip shows the
    // whole waveform lit.
    let played_x = (ph_frame > 0).then(|| wave.x + played_frac * wave.w);
    draw_waveform(scene, clip, wave, played_x, theme);
    // Selection highlight (under the playhead) + publish the wave viewport so the
    // shell can hit-test a press over it and map screen-x → clip frame.
    if let Some((s, e)) = audio.editor_selection() {
        draw_selection(scene, wave, clip.frame_count() as u64, s, e, theme);
    }
    // Loop brackets (W6) — the region that will be written to the `smpl` chunk and
    // auditioned click-free. A green frame, distinct from the blue selection band.
    if let Some((ls, le)) = audio.editor_loop_frames() {
        draw_loop_region(scene, wave, clip.frame_count() as u64, ls, le, theme);
    }
    crate::audio::set_wave_view(Some(crate::audio::WaveView {
        rect: wave,
        ruler: Rect::new(wave.x, ruler_top, wave.w, RULER_H),
        frames: clip.frame_count() as u64,
    }));
    draw_ruler(scene, text, clip, wave, ruler_top, theme);
    // The line stays inside the green brackets while looping (the region plays as its
    // own buffer, whose frames start at 0 — `ph_frame` already carries the offset).
    draw_playhead(scene, wave, played_frac, theme);

    // Register drag + resize handle hit rects into the hero hit-index so the
    // shared BlenderHit dispatch moves/resizes the overlay next frame.
    let drag = panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
    hero.hit_index
        .register(ids::AUDIO_OVERLAY_DRAG_HANDLE, drag);
    hero.hit_index.register(
        ids::AUDIO_OVERLAY_RESIZE_HANDLE,
        panel_resize_handle_rect(rect),
    );
    hero.hit_index.register(
        ids::AUDIO_OVERLAY_RESIZE_HANDLE_BL,
        panel_resize_handle_rect_bl(rect),
    );
}

/// The overlay's DEFAULT rect (before any drag/resize): a comfortable band in the
/// canvas, to the LEFT of the docked Audio Editor panel (reserved on the right),
/// below the TopBar. Strictly smaller than the viewport so the viewport-clamped
/// drag/resize have real travel.
fn default_rect(viewport: Rect) -> Rect {
    let hier_right = viewport.x + RAIL_W + EDGE_PAD + HIERARCHY_W;
    let insp_left = viewport.x + viewport.w - EDGE_PAD - INSPECTOR_W;
    let x = hier_right + EDGE_PAD;
    // Stop before the docked Audio Editor panel column (width + a gap).
    let right = (insp_left - EDITOR_PANEL_W - EDGE_PAD * 2.0).max(x + MIN_W);
    let w = (right - x).max(MIN_W);
    Rect::new(x, viewport.y + TOP_MARGIN, w, DEFAULT_H)
}

/// Pitch (px) between waveform bars — the bar plus its gap.
const BAR_PITCH: f32 = 3.0; // LITERAL-PX-OK: waveform bar pitch (chrome)
/// Width (px) of each waveform bar; the remainder of the pitch is the gap.
const BAR_W: f32 = 2.0; // LITERAL-PX-OK: waveform bar width (chrome)

/// Draw the clip as a **mirrored, rounded bar** waveform, one lane per channel.
///
/// Each bar is the column's peak amplitude reflected around the centre (the classic
/// symmetric look, cleaner than a raw min/max envelope), pill-rounded with a thin gap
/// — a modern, calm read. `played_x` (screen x of the playhead) splits the bars into
/// **played** (accent) and **unplayed** (muted); `None` lights the whole waveform
/// (stopped / at the very start).
fn draw_waveform(
    scene: &mut VectorScene,
    clip: &ph2d_audio_edit::EditClip,
    area: Rect,
    played_x: Option<f32>,
    theme: Theme,
) {
    let channels = clip.data().format().channel_count().max(1);
    let columns = ((area.w / BAR_PITCH) as usize).clamp(1, 4096);
    let peaks = clip.column_peaks(0, clip.frame_count(), columns);
    let played = resolve(ColorToken::Accent, theme);
    let unplayed = resolve(ColorToken::Text2, theme);
    let mid = resolve(ColorToken::Border, theme);

    let lane_h = area.h / channels as f32;
    for ch in 0..channels {
        let lane_top = area.y + ch as f32 * lane_h;
        let center = lane_top + lane_h * 0.5;
        let half = (lane_h * 0.5 - BAR_PITCH).max(1.0);
        // Zero line.
        fill_rounded_rect(
            scene,
            Rect::new(area.x, center - 0.5, area.w, 1.0),
            0.0,
            mid,
        );
        for c in 0..columns {
            let (lo, hi) = peaks.get(c, ch);
            let amp = hi.abs().max(lo.abs()).clamp(0.0, 1.0);
            let col_x = area.x + c as f32 * area.w / columns as f32;
            // Reflect around the centre; a 2 px floor keeps silence a visible seam.
            let h = (amp * half * 2.0).max(2.0);
            let color = match played_x {
                Some(px) if col_x > px => unplayed,
                _ => played,
            };
            fill_rounded_rect(
                scene,
                Rect::new(col_x, center - h * 0.5, BAR_W, h),
                BAR_W * 0.5,
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
        // Keep the whole label box inside the waveform area so the first/last
        // ticks don't spill their text past the timeline edges.
        let lx = (x - lw * 0.5).clamp(area.x, (area.x + area.w - lw).max(area.x));
        paint_text_centered(
            text,
            scene,
            &label,
            Rect::new(lx, ruler_top + 2.0, lw, TypeToken::Xs.px()),
            TypeToken::Xs.px(),
            label_col,
        );
    }
}

/// Draw the playback playhead as a vertical line at `frac` (0..1) across `area`.
fn draw_playhead(scene: &mut VectorScene, area: Rect, frac: f32, theme: Theme) {
    let x = area.x + frac.clamp(0.0, 1.0) * area.w;
    fill_rounded_rect(
        scene,
        Rect::new(x, area.y, 2.0, area.h),
        0.0,
        resolve(ColorToken::Warn, theme),
    );
}

/// Draw the selection `[s, e)` (frames) as a translucent band with crisp edges.
/// (A shell canvas overlay may use literal colors — the `no_literal_color` gate
/// scans panels + editor-core, not the shell; this mirrors the rubber-band.)
fn draw_selection(scene: &mut VectorScene, area: Rect, total: u64, s: u64, e: u64, _theme: Theme) {
    if total == 0 || e <= s {
        return;
    }
    let x_of = |f: u64| area.x + (f as f32 / total as f32) * area.w;
    let x0 = x_of(s);
    let x1 = x_of(e);
    // ColorToken::Selection sRGB (~#3a8ee6), translucent fill + opaque edges.
    let fill = ph2d_vector::Color::from_rgba8(58, 142, 230, 72);
    let edge = ph2d_vector::Color::from_rgba8(58, 142, 230, 220);
    fill_rounded_rect(
        scene,
        Rect::new(x0, area.y, (x1 - x0).max(1.0), area.h),
        0.0,
        fill,
    );
    fill_rounded_rect(scene, Rect::new(x0, area.y, 1.0, area.h), 0.0, edge);
    fill_rounded_rect(scene, Rect::new(x1 - 1.0, area.y, 1.0, area.h), 0.0, edge);
}

/// Draw the loop region `[s, e)` (frames) as a green bracket frame — two vertical
/// edges joined by thin top + bottom rails — so it reads as the loop span without
/// obscuring the waveform (unlike the translucent selection band it may overlap).
fn draw_loop_region(scene: &mut VectorScene, area: Rect, total: u64, s: u64, e: u64, theme: Theme) {
    if total == 0 || e <= s {
        return;
    }
    let x_of = |f: u64| area.x + (f as f32 / total as f32) * area.w;
    let x0 = x_of(s);
    let x1 = x_of(e);
    let col = resolve(ColorToken::Success, theme);
    let w = (x1 - x0).max(1.0);
    // Vertical edges (2 px) + top/bottom rails (2 px) framing the region.
    fill_rounded_rect(scene, Rect::new(x0, area.y, 2.0, area.h), 0.0, col);
    fill_rounded_rect(scene, Rect::new(x1 - 2.0, area.y, 2.0, area.h), 0.0, col);
    fill_rounded_rect(scene, Rect::new(x0, area.y, w, 2.0), 0.0, col);
    fill_rounded_rect(
        scene,
        Rect::new(x0, area.y + area.h - 2.0, w, 2.0),
        0.0,
        col,
    );
}
