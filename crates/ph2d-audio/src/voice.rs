//! [`Voice`] — one playing sample, and [`VoiceId`], its opaque handle.

use crate::buffer::SampleData;
use crate::command::PlayParams;
use crate::dsp::{Adsr, SmoothGain, equal_power_pan};
use crate::format::Sample;

/// Opaque handle to a playing voice (HR-8 style: a monotonic token, never a raw
/// index). Minted by the control thread; the audio thread binds it to a pool
/// slot. Tokens are never reused, so a `Stop`/`Set` on a finished voice is a
/// harmless no-op rather than hitting a recycled voice.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct VoiceId(pub(crate) u64);

impl VoiceId {
    /// The "no voice" sentinel — a free pool slot, or a `play` that was dropped.
    pub const NONE: VoiceId = VoiceId(0);

    /// True when this is [`VoiceId::NONE`].
    pub fn is_none(&self) -> bool {
        self.0 == 0
    }
}

/// One voice: a sample read cursor + per-voice gain/pan/envelope. Pre-allocated
/// in the pool; [`Voice::start`] resets it in place (no allocation) and
/// [`Voice::render_add`] mixes it into a block.
pub(crate) struct Voice {
    id: VoiceId,
    data: Option<SampleData>,
    /// Fractional read position in *source frames* (f64 spans long samples exactly).
    cursor: f64,
    /// Source-frames advanced per output frame = (source_rate / out_rate) * pitch.
    advance: f64,
    gain: SmoothGain,
    pan_gains: [f32; 2],
    envelope: Option<Adsr>,
    looping: bool,
    /// Output frames rendered since `start` — the "oldest" axis for stealing.
    age: u64,
}

impl Voice {
    /// A free (silent) voice slot.
    pub(crate) fn silent() -> Self {
        Self {
            id: VoiceId::NONE,
            data: None,
            cursor: 0.0,
            advance: 1.0,
            gain: SmoothGain::immediate(0.0),
            pan_gains: [0.0, 0.0],
            envelope: None,
            looping: false,
            age: 0,
        }
    }

    /// (Re)start this slot for `id`, playing `data` under `params`. `out_rate`
    /// is the mixer's output sample rate (for pitch/resampling).
    pub(crate) fn start(
        &mut self,
        id: VoiceId,
        data: SampleData,
        params: &PlayParams,
        out_rate: u32,
    ) {
        let advance_base = data.format().sample_rate as f64 / out_rate.max(1) as f64;
        self.envelope = params.envelope.map(|p| {
            let mut e = Adsr::new(p, out_rate as f32);
            e.trigger();
            e
        });
        self.id = id;
        self.data = Some(data);
        self.cursor = 0.0;
        self.advance = advance_base * params.pitch.max(0.0) as f64;
        self.gain = SmoothGain::immediate(params.gain);
        self.pan_gains = equal_power_pan(params.pan);
        self.looping = params.looping;
        self.age = 0;
    }

    pub(crate) fn id(&self) -> VoiceId {
        self.id
    }

    pub(crate) fn is_free(&self) -> bool {
        self.id.is_none()
    }

    pub(crate) fn age(&self) -> u64 {
        self.age
    }

    /// Effective loudness right now — the "quietest" axis for stealing.
    pub(crate) fn level(&self) -> f32 {
        let env = self.envelope.as_ref().map(|e| e.level()).unwrap_or(1.0);
        self.gain.current().abs() * env
    }

    /// Free the slot, returning any sample it held (for off-thread drop).
    pub(crate) fn free(&mut self) -> Option<SampleData> {
        self.id = VoiceId::NONE;
        self.data.take()
    }

    pub(crate) fn set_gain(&mut self, gain: f32) {
        self.gain.set_target(gain);
    }

    pub(crate) fn set_pan(&mut self, pan: f32) {
        self.pan_gains = equal_power_pan(pan);
    }

    pub(crate) fn release(&mut self) {
        if let Some(e) = self.envelope.as_mut() {
            e.release();
        }
    }

    /// Mix this voice into `master` (interleaved stereo, `frames` frames).
    /// Returns the voice's [`SampleData`] if it finished during this block (the
    /// slot is now free); the caller ships it to the return ring.
    pub(crate) fn render_add(
        &mut self,
        master: &mut [Sample],
        frames: usize,
    ) -> Option<SampleData> {
        // Own the sample locally for the block so we can freely mutate the rest
        // of `self` (cursor/gain/envelope) without a borrow conflict.
        let data = self.data.take()?;
        let frame_count = data.frame_count();
        if frame_count == 0 {
            self.id = VoiceId::NONE;
            return Some(data);
        }
        let [pan_l, pan_r] = self.pan_gains;

        for f in 0..frames {
            if self.cursor >= frame_count as f64 {
                if self.looping {
                    self.cursor -= frame_count as f64;
                } else {
                    self.id = VoiceId::NONE;
                    return Some(data);
                }
            }

            let env = self.envelope.as_mut().map(|e| e.tick()).unwrap_or(1.0);
            let g = self.gain.tick() * env;

            let i0 = self.cursor as usize;
            let frac = (self.cursor - i0 as f64) as f32;
            let [l0, r0] = data.frame_stereo(i0);
            let [l1, r1] = if i0 + 1 < frame_count {
                data.frame_stereo(i0 + 1)
            } else if self.looping {
                data.frame_stereo(0)
            } else {
                [l0, r0]
            };
            let l = l0 + (l1 - l0) * frac;
            let r = r0 + (r1 - r0) * frac;

            master[2 * f] += l * g * pan_l;
            master[2 * f + 1] += r * g * pan_r;

            self.cursor += self.advance;
            self.age = self.age.saturating_add(1);

            if self.envelope.as_ref().is_some_and(|e| e.is_finished()) {
                self.id = VoiceId::NONE;
                return Some(data);
            }
        }

        // Survived the block — put the sample back.
        self.data = Some(data);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::AudioFormat;

    fn mono(samples: Vec<f32>) -> SampleData {
        SampleData::from_interleaved(samples, AudioFormat::mono(48_000))
    }

    #[test]
    fn plays_then_finishes() {
        let mut v = Voice::silent();
        assert!(v.is_free());
        v.start(
            VoiceId(1),
            mono(vec![1.0, 1.0]),
            &PlayParams::default(),
            48_000,
        );
        assert!(!v.is_free());

        // 2-frame sample: block of 2 renders both, voice still alive; next block
        // is past the end → finishes and hands the sample back.
        let mut buf = [0.0f32; 4];
        assert!(v.render_add(&mut buf, 2).is_none());
        assert!(v.render_add(&mut buf, 2).is_some());
        assert!(v.is_free());
    }

    #[test]
    fn free_returns_the_sample() {
        let mut v = Voice::silent();
        v.start(VoiceId(7), mono(vec![0.5]), &PlayParams::default(), 48_000);
        assert!(v.free().is_some());
        assert!(v.is_free());
        assert!(v.free().is_none());
    }
}
