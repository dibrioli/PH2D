//! Drawing the **pieces**: the seams, and the drag in flight.
//!
//! Four things already claim a colour on this overlay — the selection is blue, the loop green, the
//! markers purple, the playhead orange — so a cut takes the one that is left and the one it should
//! have had anyway: **white**, full height, with a notch at the top. A cut is not an annotation
//! like a marker (which names a moment); it is a *break in the material*, and it reads like one.
//!
//! The drag draws an **outline, not the result**. Nothing is committed until the release
//! (`audio/editor/pieces.rs`), so what the user sees while dragging is where the piece *would* go —
//! an insertion caret at the seam it would drop onto, or a ghost edge at the length it would become.

use ph2d_editor::paint::fill_rounded_rect;
use ph2d_editor::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Color, VectorScene};

use crate::audio::AudioSystem;
use crate::audio::editor::pieces::PieceDrag;

/// A shell canvas overlay may use literal colours (the `no_literal_color` gate scans panels +
/// editor-core, not the shell) — this mirrors the selection band and the markers, which do the same.
///
/// White, and nearly opaque: a seam has to be findable at a glance against a lit waveform.
const CUT: Color = Color::from_rgba8(236, 240, 245, 235);
/// The drop caret: the same white, but heavier, so the place the piece will land outranks the seams
/// it is being dragged past.
const CARET_W: f32 = 3.0; // LITERAL-PX-OK: insertion caret width (chrome)
const CUT_W: f32 = 1.0; // LITERAL-PX-OK: cut seam width (chrome)
const NOTCH: f32 = 5.0; // LITERAL-PX-OK: the triangle-ish nub that marks a seam at the top (chrome)

/// Draw the cut seams over the waveform. No-op on an uncut clip — which is most of them, so this
/// costs nothing until the user asks for it.
pub(super) fn draw_cuts(
    scene: &mut VectorScene,
    audio: &AudioSystem,
    wave: Rect,
    ruler: Rect,
    total: u64,
) {
    if total == 0 {
        return;
    }
    let top = ruler.y + ruler.h;
    for cut in audio.editor_cuts() {
        let x = wave.x + (cut as f32 / total as f32).clamp(0.0, 1.0) * wave.w;
        fill_rounded_rect(
            scene,
            Rect::new(x - CUT_W * 0.5, top, CUT_W, wave.y + wave.h - top),
            0.0,
            CUT,
        );
        // The nub: a small square at the seam's head, so a cut is legible even where the waveform
        // behind it is bright.
        fill_rounded_rect(
            scene,
            Rect::new(x - NOTCH * 0.5, top, NOTCH, NOTCH),
            1.0,
            CUT,
        );
    }
}

/// Draw the drag in flight: where the piece would land, or how long it would become.
pub(super) fn draw_piece_drag(
    scene: &mut VectorScene,
    audio: &AudioSystem,
    wave: Rect,
    total: u64,
    theme: Theme,
) {
    let (Some(drag), Some(clip)) = (audio.editor_piece_drag(), audio.editor_clip()) else {
        return;
    };
    if total == 0 {
        return;
    }
    let x_of = |f: usize| wave.x + (f as f32 / total as f32).clamp(0.0, 1.0) * wave.w;
    let accent = ph2d_editor::paint::resolve(ColorToken::Accent, theme);

    match drag {
        PieceDrag::Move { to, .. } => {
            // The seam the piece would drop onto — a caret, not a line: it says "between", which is
            // the whole question the drag is asking.
            let bounds = ph2d_audio_edit::boundaries(clip.cuts(), clip.frame_count());
            let Some(&frame) = bounds.get(to) else {
                return;
            };
            let x = x_of(frame);
            fill_rounded_rect(
                scene,
                Rect::new(x - CARET_W * 0.5, wave.y, CARET_W, wave.h),
                0.0,
                accent,
            );
            // Flared head and foot, so the caret reads as an insertion point rather than one more
            // seam among the seams it is being dragged past.
            let flare = CARET_W * 3.0;
            for y in [wave.y, wave.y + wave.h - CARET_W] {
                fill_rounded_rect(
                    scene,
                    Rect::new(x - flare * 0.5, y, flare, CARET_W),
                    0.0,
                    accent,
                );
            }
        }
        PieceDrag::Scale { piece, new_len } => {
            // The edge the piece would end at, and a rail from its start to there — the new extent,
            // drawn over the old one so the change in length is the thing you actually see.
            let Some(r) = clip.pieces().get(piece).cloned() else {
                return;
            };
            let (x0, x1) = (x_of(r.start), x_of(r.start + new_len));
            fill_rounded_rect(
                scene,
                Rect::new(x1 - CARET_W * 0.5, wave.y, CARET_W, wave.h),
                0.0,
                accent,
            );
            let (lo, hi) = if x1 >= x0 { (x0, x1) } else { (x1, x0) };
            fill_rounded_rect(
                scene,
                Rect::new(lo, wave.y, (hi - lo).max(1.0), CARET_W),
                0.0,
                accent,
            );
            fill_rounded_rect(
                scene,
                Rect::new(lo, wave.y + wave.h - CARET_W, (hi - lo).max(1.0), CARET_W),
                0.0,
                accent,
            );
        }
    }
}
