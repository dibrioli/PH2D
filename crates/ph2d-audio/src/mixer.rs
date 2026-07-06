//! [`Mixer`] — active voices summed through sub-buses into the master bus.
//!
//! Each voice routes to a [`BusId`]: sub-bus voices are summed into a reused
//! per-bus scratch, pass through that bus's fader (smoothed), and fold into the
//! master mix; voices routed straight to [`BusId::Master`] sum directly. The
//! master mix then gets the master gain + low-pass filter. Per-bus post-fader
//! peaks are reported out so the UI can meter every strip.

use crate::buffer::SampleData;
use crate::bus::{BusId, SUB_BUS_COUNT};
use crate::command::AudioCommand;
use crate::dsp::{Biquad, SmoothGain};
use crate::format::{AudioFormat, Sample};
use crate::pool::VoicePool;

/// Linear stereo balance gains `[left, right]` for `pan` in `-1.0..=1.0`:
/// center = `[1, 1]` (unity, unlike an equal-power *mono* pan), full-left =
/// `[1, 0]`, full-right = `[0, 1]`. Transcendental-free (HR-5).
fn balance_gains(pan: f32) -> [f32; 2] {
    let p = pan.clamp(-1.0, 1.0);
    [(1.0 - p).min(1.0), (1.0 + p).min(1.0)]
}

pub(crate) struct Mixer {
    pool: VoicePool,
    master_gain: SmoothGain,
    /// Master stereo balance gains `[L, R]`.
    master_pan: [f32; 2],
    /// Master low-pass filter (per channel). Identity coeffs by default = bypass.
    filter_l: Biquad,
    filter_r: Biquad,
    /// Per-sub-bus fader, indexed by `BusId::sub_index`.
    bus_gain: [SmoothGain; SUB_BUS_COUNT],
    /// Per-sub-bus stereo balance gains `[L, R]`.
    bus_pan: [[f32; 2]; SUB_BUS_COUNT],
}

impl Mixer {
    pub(crate) fn new(format: AudioFormat, max_voices: usize) -> Self {
        Self {
            pool: VoicePool::new(max_voices, format),
            master_gain: SmoothGain::immediate(1.0),
            master_pan: balance_gains(0.0),
            filter_l: Biquad::default(),
            filter_r: Biquad::default(),
            bus_gain: std::array::from_fn(|_| SmoothGain::immediate(1.0)),
            bus_pan: [balance_gains(0.0); SUB_BUS_COUNT],
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
            AudioCommand::SetBusGain { bus, gain } => match bus.sub_index() {
                Some(i) => self.bus_gain[i].set_target(gain),
                // A `Master`-targeted bus gain is just the master fader.
                None => self.master_gain.set_target(gain),
            },
            AudioCommand::SetBusPan { bus, pan } => match bus.sub_index() {
                Some(i) => self.bus_pan[i] = balance_gains(pan),
                None => self.master_pan = balance_gains(pan),
            },
        }
    }

    /// Sum active voices into `master` (interleaved stereo): each sub-bus through
    /// its fader (into `bus_scratch`, then folded in), then master-direct voices,
    /// then the master gain + low-pass filter. `bus_peaks[i]` receives sub-bus
    /// `i`'s post-fader `[L, R]` peak. Finished voices' samples go to `on_finished`.
    pub(crate) fn render(
        &mut self,
        master: &mut [Sample],
        bus_scratch: &mut [Sample],
        bus_peaks: &mut [[f32; 2]; SUB_BUS_COUNT],
        bus_rms: &mut [[f32; 2]; SUB_BUS_COUNT],
        frames: usize,
        on_finished: &mut dyn FnMut(SampleData),
    ) {
        let n = frames * 2;
        let inv_frames = 1.0 / frames.max(1) as f32;
        // Sub-buses: render into the shared scratch, apply the fader, fold into
        // master, and capture the post-fader peak + RMS for the strip meter.
        for (i, &bus) in BusId::SUB_BUSES.iter().enumerate() {
            for s in bus_scratch[..n].iter_mut() {
                *s = 0.0;
            }
            self.pool.render_bus(bus, bus_scratch, frames, on_finished);
            let gain = &mut self.bus_gain[i];
            let [pan_l, pan_r] = self.bus_pan[i];
            let mut peak_l = 0.0f32;
            let mut peak_r = 0.0f32;
            let mut sq_l = 0.0f32;
            let mut sq_r = 0.0f32;
            for f in 0..frames {
                let g = gain.tick();
                // Post-fader level (pre-pan) is the strip meter reading, so
                // panning a bus doesn't drop its meter.
                let l = bus_scratch[2 * f] * g;
                let r = bus_scratch[2 * f + 1] * g;
                peak_l = peak_l.max(l.abs());
                peak_r = peak_r.max(r.abs());
                sq_l += l * l;
                sq_r += r * r;
                master[2 * f] += l * pan_l;
                master[2 * f + 1] += r * pan_r;
            }
            bus_peaks[i] = [peak_l, peak_r];
            bus_rms[i] = [(sq_l * inv_frames).sqrt(), (sq_r * inv_frames).sqrt()];
        }

        // Voices routed straight to the master mix (no sub-bus fader).
        self.pool
            .render_bus(BusId::Master, master, frames, on_finished);

        // Master gain, the master low-pass filter (identity = bypass), then the
        // master stereo balance.
        let [mpan_l, mpan_r] = self.master_pan;
        for f in 0..frames {
            let g = self.master_gain.tick();
            master[2 * f] = self.filter_l.process(master[2 * f] * g) * mpan_l;
            master[2 * f + 1] = self.filter_r.process(master[2 * f + 1] * g) * mpan_r;
        }
    }

    pub(crate) fn active_voices(&self) -> usize {
        self.pool.active_count()
    }

    pub(crate) fn voice_capacity(&self) -> usize {
        self.pool.capacity()
    }
}
