//! Device-buffer write-out: the final stage that copies the mixed master scratch
//! into the callback's output buffer, clamped + sanitized. Split out of
//! [`crate::engine`] to keep that file under the workspace LOC cap.

use crate::format::{AudioFormat, ChannelLayout, Sample};

/// Clamp a sample to `[-1, 1]`, mapping any non-finite value (NaN/±∞) to silence.
/// `f32::clamp` returns NaN for a NaN input, so the plain clamp does not by itself
/// keep the device buffer finite — a NaN escaping an unstable effect (reverb
/// feedback, limiter) would reach the device as silence or a pop. This is the
/// last line: the device only ever sees finite samples in range.
#[inline]
fn sanitize(x: Sample) -> Sample {
    if x.is_finite() {
        x.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

/// Write the interleaved-stereo `master` scratch into the device `out` buffer,
/// down-mixing to mono if the output layout is mono, clamped to `[-1, 1]` with
/// non-finite samples flushed to silence ([`sanitize`]).
pub(crate) fn write_out(out: &mut [Sample], master: &[Sample], frames: usize, format: AudioFormat) {
    match format.channels {
        ChannelLayout::Stereo => {
            let n = (frames * 2).min(out.len());
            for i in 0..n {
                out[i] = sanitize(master[i]);
            }
        }
        ChannelLayout::Mono => {
            let n = frames.min(out.len());
            for f in 0..n {
                out[f] = sanitize(0.5 * (master[2 * f] + master[2 * f + 1]));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_out_flushes_non_finite_to_silence() {
        // A NaN/∞ escaping an unstable effect must never reach the device buffer.
        let fmt = AudioFormat::stereo(48_000);
        let master = [0.5, f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 2.0, -2.0];
        let mut out = [7.0f32; 6]; // sentinel: must be overwritten
        write_out(&mut out, &master, 3, fmt);
        assert!(
            out.iter().all(|s| s.is_finite()),
            "device buffer must be finite"
        );
        assert_eq!(out[0], 0.5, "finite in-range sample passes through");
        assert_eq!(out[1], 0.0, "NaN flushed to silence");
        assert_eq!(out[2], 0.0, "+∞ flushed to silence");
        assert_eq!(out[3], 0.0, "-∞ flushed to silence");
        assert_eq!(out[4], 1.0, "over-range clamps to +1");
        assert_eq!(out[5], -1.0, "under-range clamps to -1");
    }

    #[test]
    fn write_out_mono_downmix_sanitizes() {
        // Mono down-mix path must also stay finite even if one channel is NaN.
        let fmt = AudioFormat::mono(48_000);
        let master = [0.4, 0.4, f32::NAN, 0.2];
        let mut out = [7.0f32; 2];
        write_out(&mut out, &master, 2, fmt);
        assert!(out.iter().all(|s| s.is_finite()));
        assert_eq!(out[0], 0.4, "average of two equal channels");
        assert_eq!(
            out[1], 0.0,
            "NaN in either channel flushes the frame to silence"
        );
    }
}
