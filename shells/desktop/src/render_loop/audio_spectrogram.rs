//! The spectrogram half of the Audio Editor overlay (W5, ADR-0115).
//!
//! Same rectangle as the waveform, a different picture in it: time across, **frequency
//! up**, brightness for level. It is drawn as one **image** (`draw_image_rgba`) rather than
//! as a field of little rectangles — a 700 × 250 view holds ~175 000 cells, and painting
//! that many rounded rects per frame would be absurd. The bytes come from the shell's
//! cache, so a repaint is an upload, not an FFT.
//!
//! The selection is the other half of the story. In the waveform it is a full-height band:
//! it can only say *when*. Here it is a **box** — when and *what* — and that box is the
//! only thing spectral repair can act on, because the beep and the voice beneath it live
//! in the very same samples and differ only in where they sit on this axis.
//!
//! Split out of `audio_overlay` to keep that file under the shell's 600-LOC cap.

use ph2d_editor::paint::{fill_rounded_rect, resolve, stroke_rect};
use ph2d_editor::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{ImageQuality, VectorScene};

use crate::audio::AudioSystem;

/// Draw the spectrogram into `area`. Falls back to painting nothing if the picture is not
/// ready (no clip) — the caller has already drawn the panel surface under it.
pub(super) fn draw_spectrogram(
    scene: &mut VectorScene,
    audio: &mut AudioSystem,
    area: Rect,
    theme: Theme,
) {
    // One image pixel per screen pixel: the cache is resampled to the view, so a resize
    // re-renders (cheaply, from the analysed cache) rather than stretching a stale image.
    let w = area.w.max(1.0) as u32;
    let h = area.h.max(1.0) as u32;
    let Some(img) = audio.editor_spectrogram_rgba(w, h, theme) else {
        return;
    };
    scene.draw_image_rgba(
        &img,
        w,
        h,
        (
            f64::from(area.x),
            f64::from(area.y),
            f64::from(area.x + area.w),
            f64::from(area.y + area.h),
        ),
        // Nearest: a spectrogram cell is a *measurement*, and smoothing between two of them
        // invents a level that was never in the sound. The user is about to select one of
        // these cells to erase it — it had better be the one they can see.
        ImageQuality::Low,
    );
}

/// Draw the time-frequency selection box: the region a Repair would act on.
///
/// Only the time half comes from the clip's selection; the frequency half is the band the
/// user dragged. Without a band there is nothing to draw here — a full-height band would
/// say "the whole spectrum", which is exactly what the user did *not* select.
pub(super) fn draw_band(
    scene: &mut VectorScene,
    audio: &AudioSystem,
    area: Rect,
    frames: u64,
    theme: Theme,
) {
    let (Some((s, e)), Some((a, b))) = (audio.editor_selection(), audio.editor_spectral_band())
    else {
        return;
    };
    let total = frames.max(1) as f32;
    let x0 = area.x + (s as f32 / total).clamp(0.0, 1.0) * area.w;
    let x1 = area.x + (e as f32 / total).clamp(0.0, 1.0) * area.w;
    // Frequency runs UP the image, so the HIGH end of the band is the LOW y.
    let (lo, hi) = (a.min(b), a.max(b));
    let y0 = area.y + (1.0 - hi) * area.h;
    let y1 = area.y + (1.0 - lo) * area.h;
    let box_rect = Rect::new(x0.min(x1), y0, (x1 - x0).abs().max(1.0), (y1 - y0).max(1.0));

    // A translucent wash plus a hard edge: the wash says "this region", the edge says
    // exactly where it stops — and where it stops is what gets rebuilt.
    let mut fill = resolve(ColorToken::Accent, theme);
    fill = fill.multiply_alpha(SELECTION_ALPHA);
    fill_rounded_rect(scene, box_rect, 0.0, fill);
    stroke_rect(scene, box_rect, 1.0, resolve(ColorToken::Accent, theme));
}

/// How solid the selection wash is. Enough to read as selected, sheer enough to still see
/// the thing you are about to erase through it.
const SELECTION_ALPHA: f32 = 0.25; // LITERAL-PX-OK: selection wash opacity (chrome)
