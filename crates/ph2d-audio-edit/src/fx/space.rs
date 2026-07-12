//! The wet-tail driver shared by the rack's space effects (reverb, delay, ping-pong),
//! plus the ping-pong delay itself.
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
    // One allocation (ADR-0117 D2). The output is *longer* than the input (the tail), and
    // the dry signal is read from the pristine `src` — so this builds, it does not map.
    // Each frame writes exactly `ch` samples (`ch` is 1 or 2), starting at `f * ch`.
    SampleData::build((frames + tail_frames) * ch, data.format(), |out| {
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
            let o = f * ch;
            if ch >= 2 {
                out[o] = (dry_l * dry_gain + wet_l * mix).clamp(-1.0, 1.0);
                out[o + 1] = (dry_r * dry_gain + wet_r * mix).clamp(-1.0, 1.0);
            } else {
                let w = (wet_l + wet_r) * 0.5;
                out[o] = (dry_l * dry_gain + w * mix).clamp(-1.0, 1.0);
            }
        }
    })
}

/// **Ping-pong delay**: a plain feedback delay whose repeats **alternate channels**.
/// The input is summed to mono and injected on the left; each side's delayed output
/// feeds the *other* side's line, so an echo walks L→R→L, `feedback` quieter per
/// bounce. On a mono clip [`render_wet`] collapses the bounce back to a single decaying
/// echo train — the stereo walk needs two output channels to be heard.
pub(super) fn pingpong(
    data: &SampleData,
    tail_frames: usize,
    time_secs: f32,
    feedback: f32,
    mix: f32,
) -> SampleData {
    let sr = data.format().sample_rate as f32;
    // One delay of `time_secs` per side, as a simple ring buffer read-then-write.
    let n = ((time_secs.clamp(0.01, 0.99) * sr) as usize).max(1);
    let fb = feedback.clamp(0.0, 0.95); // < 1 or the bounce never decays
    let mut dl = vec![0.0f32; n];
    let mut dr = vec![0.0f32; n];
    let mut i = 0usize;
    render_wet(data, tail_frames, mix, move |l, r| {
        let out_l = dl[i];
        let out_r = dr[i];
        let input = (l + r) * 0.5;
        // Left line takes the dry input plus the right line's feedback; the right line
        // takes only the left line's feedback. That cross is what makes it bounce.
        dl[i] = input + fb * out_r;
        dr[i] = fb * out_l;
        i = (i + 1) % n;
        (out_l, out_r)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    /// THE ping-pong contract: a single hit lands on the left, its first repeat on the
    /// left, the next on the right, the next on the left — bouncing and decaying by the
    /// feedback each time. 1 kHz makes 10 ms exactly 10 samples.
    #[test]
    fn ping_pong_bounces_between_channels() {
        let mut x = vec![0.0f32; 8]; // 4 stereo frames
        x[0] = 1.0; // an impulse on both channels at frame 0 (mono-sums to 1)
        x[1] = 1.0;
        let d = SampleData::from_interleaved(x, AudioFormat::stereo(1_000));
        let out = pingpong(&d, 60, 0.01, 0.5, 1.0); // n=10, fb 0.5, fully wet
        let s = out.samples();
        let l = |f: usize| s[f * 2];
        let r = |f: usize| s[f * 2 + 1];
        // Frame 10: left bounce, right silent.
        assert!(
            l(10) > 0.9 && r(10).abs() < 1e-3,
            "1st bounce not left: L{} R{}",
            l(10),
            r(10)
        );
        // Frame 20: right bounce at feedback, left silent.
        assert!(
            r(20) > 0.4 && l(20).abs() < 1e-3,
            "2nd bounce not right: L{} R{}",
            l(20),
            r(20)
        );
        // Frame 30: left again at feedback², quieter than the first.
        assert!(
            l(30) > 0.2 && l(30) < l(10) && r(30).abs() < 1e-3,
            "3rd bounce not left: L{} R{}",
            l(30),
            r(30)
        );
    }

    /// The tail is appended: output is region + `tail_frames`, never truncated.
    #[test]
    fn ping_pong_appends_the_tail() {
        let d = SampleData::from_interleaved(vec![0.1f32; 200], AudioFormat::stereo(48_000));
        let out = pingpong(&d, 480, 0.05, 0.4, 0.5); // 100 frames + 480 tail
        assert_eq!(out.frame_count(), 100 + 480);
    }
}
