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
const RULER_H: f32 = 22.0; // LITERAL-PX-OK: time-ruler strip height (chrome; tall = clickable)
/// Width reserved on the right for the docked Audio Editor panel (matches its
/// `PANEL_W`), so the overlay's default position doesn't cover it.
const EDITOR_PANEL_W: f32 = 240.0; // LITERAL-PX-OK: docked editor panel width (chrome)

/// Paint the Audio Editor waveform overlay if the editor panel is open and a
/// clip is loaded. No-op otherwise.
pub(super) fn draw_audio_overlay(
    hero: &mut HeroScreen,
    audio: &mut AudioSystem,
    viewport: Rect,
    scene: &mut VectorScene,
    text: &mut TextSystem,
) {
    if !hero.is_panel_visible("audio_editor") {
        hero.store.clear_panel_rect(ids::AUDIO_OVERLAY_PANEL);
        crate::audio::set_wave_view(None);
        return;
    }
    let Some(frames) = audio.editor_clip().map(|c| c.frame_count()) else {
        crate::audio::set_wave_view(None);
        return;
    };
    let theme = hero.theme;
    // Which picture is showing. The panel owns this toggle (W5); the overlay just obeys.
    let spectral = ph2d_panel_audio_editor::spectral_state::view();

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
        if spectral {
            "Audio Editor \u{00b7} Spectrogram"
        } else {
            "Audio Editor \u{00b7} Waveform"
        },
        header,
        TypeToken::Xs.px(),
        resolve(ColorToken::Text2, theme),
    );

    // Layout: the time RULER sits at the TOP (like a timeline / DAW) — a darker,
    // taller strip the user clicks + drags to position the playhead — then the
    // waveform fills the rest. Ruler and waveform share the same x/width so screen-x
    // maps straight to clip time.
    let inset = Spacing::Sm.px();
    let content_x = rect.x + inset;
    let content_w = (rect.w - inset * 2.0).max(1.0);
    let ruler = Rect::new(content_x, rect.y + HEADER_H, content_w, RULER_H);
    let wave = Rect::new(
        content_x,
        ruler.y + RULER_H,
        content_w,
        (rect.y + rect.h - (ruler.y + RULER_H) - inset).max(1.0),
    );
    // The playhead frame (offset into the loop region while it loops), computed once:
    // it drives both the waveform's played/unplayed shading and the playhead line.
    let ph_frame = audio.editor_playhead_frame();
    let total = frames.max(1) as f32;
    let played_frac = (ph_frame as f32 / total).clamp(0.0, 1.0);
    // Only split played/unplayed once playback has advanced; a stopped clip shows the
    // whole waveform lit.
    let played_x = (ph_frame > 0).then_some(wave.x + played_frac * wave.w);
    if let Some(clip) = audio.editor_clip() {
        draw_ruler(scene, text, clip, ruler, theme);
        if !spectral {
            draw_waveform(scene, clip, wave, played_x, theme);
        }
    }
    // The spectrogram needs the image cache, so it borrows `audio` mutably — after the
    // read-only draws above have let go of the clip.
    if spectral {
        super::audio_spectrogram::draw_spectrogram(scene, audio, wave, theme);
    }
    // The selection. In the waveform it is a full-height band (it can only say WHEN); in
    // the spectrogram it is a box (when AND what), and only the box can be repaired.
    if spectral {
        super::audio_spectrogram::draw_band(scene, audio, wave, frames as u64, theme);
    } else if let Some((s, e)) = audio.editor_selection() {
        draw_selection(scene, wave, frames as u64, s, e, theme);
    }
    // Loop brackets (W6) — the region that will be written to the `smpl` chunk and
    // auditioned click-free. A green frame, distinct from the blue selection band.
    if let Some((ls, le)) = audio.editor_loop_frames() {
        draw_loop_region(scene, wave, frames as u64, ls, le, theme);
    }
    // Cue markers (W6) — named points written to the `cue`+`adtl` chunks. Purple
    // flags in the ruler with a thin line down the wave, distinct from every band.
    draw_markers(
        scene,
        text,
        ruler,
        wave,
        frames as u64,
        &audio.editor_markers(),
    );
    crate::audio::set_wave_view(Some(crate::audio::WaveView {
        rect: wave,
        ruler,
        frames: frames as u64,
    }));
    // The playhead spans the ruler AND the waveform (one continuous bar), so grabbing
    // it in the ruler reads as moving the same line. It stays inside the green loop
    // brackets while looping (the region plays as its own buffer whose frames start at
    // 0 — `ph_frame` already carries the offset).
    let play_area = Rect::new(wave.x, ruler.y, wave.w, wave.y + wave.h - ruler.y);
    draw_playhead(scene, play_area, played_frac, theme);

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
            let col_x = area.x + c as f32 * area.w / columns as f32;
            // Signed min/max envelope (NOT abs): `hi` above the centre, `lo` below.
            let (top_y, h) = column_bar(lo, hi, center, half);
            let color = match played_x {
                Some(px) if col_x > px => unplayed,
                _ => played,
            };
            fill_rounded_rect(scene, Rect::new(col_x, top_y, BAR_W, h), BAR_W * 0.5, color);
        }
    }
}

/// The vertical extent of one waveform column as a **signed** min/max envelope:
/// `hi` (column max) above the centre, `lo` (column min) below (screen y grows down).
/// Returns `(top_y, height)`. A 2 px floor keeps silence a visible seam. Unlike an
/// `abs()` envelope this preserves polarity — so **Invert** (which flips `(lo,hi)` to
/// `(-hi,-lo)`), DC-offset and asymmetric transients are visible, not collapsed to the
/// same bar (2026-07-11 audit: the old `|hi|.max(|lo|)` made Invert invisible).
fn column_bar(lo: f32, hi: f32, center: f32, half: f32) -> (f32, f32) {
    let hi = hi.clamp(-1.0, 1.0);
    let lo = lo.clamp(-1.0, 1.0);
    let top = center - hi * half; // hi ≥ lo, and up = smaller y
    let bot = center - lo * half;
    let h = (bot - top).max(2.0);
    let midpoint = (top + bot) * 0.5;
    (midpoint - h * 0.5, h)
}

/// The time ruler: a **darker, inset strip** (so it reads as the click-to-scrub zone)
/// filling `ruler`, with evenly-spaced time ticks + labels. Sits at the TOP of the
/// overlay (timeline convention); `ruler` shares the waveform's x/width so a click maps
/// straight to clip time.
fn draw_ruler(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    clip: &ph2d_audio_edit::EditClip,
    ruler: Rect,
    theme: Theme,
) {
    // Darker inset backing — the visual affordance that this strip is grabbable.
    fill_rounded_rect(
        scene,
        ruler,
        Radius::Xs.px(),
        resolve(ColorToken::Bg1, theme),
    );
    let dur = clip.duration_secs();
    let ticks = 5;
    let tick_col = resolve(ColorToken::Border, theme);
    let label_col = resolve(ColorToken::Text2, theme);
    let tick_h = ruler.h * 0.4; // short marks along the bottom edge, ruler-style
    for i in 0..=ticks {
        let frac = i as f32 / ticks as f32;
        let x = ruler.x + frac * ruler.w;
        fill_rounded_rect(
            scene,
            Rect::new(x, ruler.y + ruler.h - tick_h, 1.0, tick_h),
            0.0,
            tick_col,
        );
        let secs = dur * frac as f64;
        let label = format!("{secs:.1}s");
        let lw = 34.0; // LITERAL-PX-OK: tick label box width (chrome)
        // Keep the whole label box inside the strip so the first/last don't spill.
        let lx = (x - lw * 0.5).clamp(ruler.x, (ruler.x + ruler.w - lw).max(ruler.x));
        paint_text_centered(
            text,
            scene,
            &label,
            Rect::new(lx, ruler.y + 1.0, lw, TypeToken::Xs.px()),
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

/// Draw cue markers: a thin **purple** line from the ruler down through the waveform,
/// with the name at the top of the wave — distinct from the loop (green), selection
/// (blue) and playhead (orange).
fn draw_markers(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    ruler: Rect,
    wave: Rect,
    total: u64,
    markers: &[(u64, String)],
) {
    if total == 0 {
        return;
    }
    // Purple — a shell overlay may use literal colors (the `no_literal_color` gate
    // scans panels + editor-core, not the shell); mirrors the selection band.
    let col = ph2d_vector::Color::from_rgba8(198, 140, 246, 230);
    let lw = 26.0; // LITERAL-PX-OK: marker label box width (chrome)
    for (frame, name) in markers {
        let x = wave.x + (*frame as f32 / total as f32).clamp(0.0, 1.0) * wave.w;
        fill_rounded_rect(
            scene,
            Rect::new(x, ruler.y, 1.0, wave.y + wave.h - ruler.y),
            0.0,
            col,
        );
        let lx = (x + 2.0).min(wave.x + wave.w - lw);
        paint_text_centered(
            text,
            scene,
            name,
            Rect::new(lx, wave.y, lw, TypeToken::Xs.px()),
            TypeToken::Xs.px(),
            col,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::column_bar;

    /// The signed envelope must DISTINGUISH a column from its polarity-inverse — the
    /// property the old `abs()` bar violated, making Invert invisible. Inverting a
    /// column `(lo,hi) → (-hi,-lo)` mirrors the bar across the centre: same height (same
    /// energy), but the top edge moves. Guards the 2026-07-11 audit fix.
    #[test]
    fn invert_mirrors_the_bar_instead_of_repeating_it() {
        let (center, half) = (100.0f32, 50.0f32);
        // Asymmetric column: mostly positive.
        let (top_a, h_a) = column_bar(-0.3, 0.8, center, half);
        // Its polarity inverse: (lo,hi) = (-0.8, 0.3), mostly negative.
        let (top_b, h_b) = column_bar(-0.8, 0.3, center, half);
        assert!(
            (h_a - h_b).abs() < 1e-4,
            "inverting must preserve the bar height (energy): {h_a} vs {h_b}"
        );
        assert!(
            (top_a - top_b).abs() > 1.0,
            "inverting must move the bar (abs() left it identical): {top_a} vs {top_b}"
        );
        // The positive column reaches higher (smaller y) than its negative inverse.
        assert!(
            top_a < top_b,
            "the mostly-positive column must sit higher on screen"
        );
    }

    /// Silence (all-zero column) still draws a visible 2 px seam centred on the axis.
    #[test]
    fn silence_keeps_a_two_pixel_seam() {
        let (top, h) = column_bar(0.0, 0.0, 100.0, 50.0);
        assert_eq!(h, 2.0);
        assert!((top - 99.0).abs() < 1e-4, "the seam straddles the centre");
    }
}
