//! The wet-tail driver shared by the rack's space effects (reverb, delay).
//!
//! Control thread only.

use ph2d_audio::SampleData;

use crate::ops::channels;

/// Drive a **wet-only** stereo processor (the `dsp` kit's contract) across the
/// region then `tail_frames` of silence, crossfading dry/wet by `mix`. Mono clips
/// feed the processor `(x, x)` and collapse its stereo wet back to one channel.
pub(super) fn render_wet(
    data: &SampleData,
    tail_frames: usize,
    mix: f32,
    mut wet: impl FnMut(f32, f32) -> (f32, f32),
) -> SampleData {
    let ch = channels(data);
    let frames = data.frame_count();
    let src = data.samples();
    let mix = mix.clamp(0.0, 1.0);
    let dry_gain = 1.0 - mix;
    let mut out = Vec::with_capacity((frames + tail_frames) * ch);
    for f in 0..frames + tail_frames {
        // Past the region the dry signal is silence — the processor keeps ringing.
        let (dry_l, dry_r) = if f < frames {
            let b = f * ch;
            if ch >= 2 {
                (src[b], src[b + 1])
            } else {
                (src[b], src[b])
            }
        } else {
            (0.0, 0.0)
        };
        let (wet_l, wet_r) = wet(dry_l, dry_r);
        if ch >= 2 {
            out.push((dry_l * dry_gain + wet_l * mix).clamp(-1.0, 1.0));
            out.push((dry_r * dry_gain + wet_r * mix).clamp(-1.0, 1.0));
        } else {
            let w = (wet_l + wet_r) * 0.5;
            out.push((dry_l * dry_gain + w * mix).clamp(-1.0, 1.0));
        }
    }
    SampleData::from_interleaved(out, data.format())
}
