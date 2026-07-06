//! [`Mixer`] — active voices summed through the master bus.
//!
//! Fundamentals implement a single master bus (master gain + clamp on write).
//! Each [`crate::voice::Voice`] carries the routing data a future sub-bus layer
//! needs, so adding named buses in the features phase is additive, not a rewrite.

use crate::buffer::SampleData;
use crate::command::AudioCommand;
use crate::dsp::{Biquad, SmoothGain};
use crate::format::{AudioFormat, Sample};
use crate::pool::VoicePool;

pub(crate) struct Mixer {
    pool: VoicePool,
    master_gain: SmoothGain,
    /// Master low-pass filter (per channel). Identity coeffs by default = bypass.
    filter_l: Biquad,
    filter_r: Biquad,
}

impl Mixer {
    pub(crate) fn new(format: AudioFormat, max_voices: usize) -> Self {
        Self {
            pool: VoicePool::new(max_voices, format),
            master_gain: SmoothGain::immediate(1.0),
            filter_l: Biquad::default(),
            filter_r: Biquad::default(),
        }
    }

    /// Apply one control command. `on_finished` receives any sample freed as a
    /// side effect (a stolen or stopped voice), for off-thread drop.
    pub(crate) fn apply(&mut self, cmd: AudioCommand, on_finished: &mut dyn FnMut(SampleData)) {
        match cmd {
            AudioCommand::Play {
                voice,
                data,
                params,
            } => self.pool.start(voice, data, params, on_finished),
            AudioCommand::Stop { voice } => self.pool.stop(voice, on_finished),
            AudioCommand::Release { voice } => self.pool.release(voice),
            AudioCommand::SetVoiceGain { voice, gain } => self.pool.set_gain(voice, gain),
            AudioCommand::SetVoicePan { voice, pan } => self.pool.set_pan(voice, pan),
            AudioCommand::SetMasterGain { gain } => self.master_gain.set_target(gain),
            AudioCommand::SetMasterFilter { coeffs } => {
                self.filter_l.set_coeffs(coeffs);
                self.filter_r.set_coeffs(coeffs);
            }
        }
    }

    /// Sum active voices into `master` (interleaved stereo) and apply the
    /// smoothed master gain. Finished voices' samples go to `on_finished`.
    pub(crate) fn render(
        &mut self,
        master: &mut [Sample],
        frames: usize,
        on_finished: &mut dyn FnMut(SampleData),
    ) {
        self.pool.render_into(master, frames, on_finished);
        for f in 0..frames {
            let g = self.master_gain.tick();
            // Master gain, then the master low-pass filter (identity = bypass).
            master[2 * f] = self.filter_l.process(master[2 * f] * g);
            master[2 * f + 1] = self.filter_r.process(master[2 * f + 1] * g);
        }
    }

    pub(crate) fn active_voices(&self) -> usize {
        self.pool.active_count()
    }

    pub(crate) fn voice_capacity(&self) -> usize {
        self.pool.capacity()
    }
}
