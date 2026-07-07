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
use crate::dsp::{Biquad, Delay, Reverb, SmoothGain};
use crate::format::{AudioFormat, Sample};
use crate::pool::VoicePool;

/// Linear stereo balance gains `[left, right]` for `pan` in `-1.0..=1.0`:
/// center = `[1, 1]` (unity, unlike an equal-power *mono* pan), full-left =
/// `[1, 0]`, full-right = `[0, 1]`. Transcendental-free (HR-5).
fn balance_gains(pan: f32) -> [f32; 2] {
    let p = pan.clamp(-1.0, 1.0);
    [(1.0 - p).min(1.0), (1.0 + p).min(1.0)]
}

/// Master limiter ceiling (~ -2 dBFS) — loud peaks are turned down toward this
/// level. Comfortably below full scale so a limited mix never hard-clips and the
/// gain reduction is clearly audible when the master is pushed.
const LIMIT_CEILING: f32 = 0.8;
/// Gain-reduction attack coefficient (one-pole, per sample): near-instant so a
/// transient is caught in a fraction of a millisecond.
const LIMIT_ATTACK: f32 = 0.4;
/// Gain-reduction release coefficient — slow (~50 ms) so the level eases back up
/// without audible chatter once the loud part passes.
const LIMIT_RELEASE: f32 = 0.0004;

/// Soft-knee clip: below `THRESHOLD` the signal is untouched; above it, the
/// excess is smoothly saturated so the output asymptotes to `CEILING` (< 1.0).
/// The limiter's final brickwall — catches the transient overshoot before the
/// gain-reduction envelope has fully engaged. C1-continuous at the knee (slope
/// 1), transcendental-free (HR-5) — a single divide per sample.
fn soft_clip(x: f32) -> f32 {
    const THRESHOLD: f32 = 0.8; // knee start (~ -2 dBFS)
    const CEILING: f32 = 0.98; // asymptotic output ceiling (< full scale)
    let a = x.abs();
    if a <= THRESHOLD {
        return x;
    }
    let over = a - THRESHOLD;
    let range = CEILING - THRESHOLD;
    let compressed = THRESHOLD + range * (over / (over + range));
    x.signum() * compressed
}

pub(crate) struct Mixer {
    pool: VoicePool,
    master_gain: SmoothGain,
    /// Master stereo balance gains `[L, R]`.
    master_pan: [f32; 2],
    /// Master high-pass (low-cut) filter (per channel), in series ahead of the
    /// low-pass. Identity coeffs by default = off (bypass).
    hp_l: Biquad,
    hp_r: Biquad,
    /// Master low-pass filter (per channel). Identity coeffs by default = bypass.
    filter_l: Biquad,
    filter_r: Biquad,
    /// Master 3-band EQ (low shelf, mid peak, high shelf) per channel, in series
    /// after the master low-pass. Identity coeffs by default = flat (transparent).
    eq_l: [Biquad; 3],
    eq_r: [Biquad; 3],
    /// Per-sub-bus fader, indexed by `BusId::sub_index`.
    bus_gain: [SmoothGain; SUB_BUS_COUNT],
    /// Per-sub-bus stereo balance gains `[L, R]`.
    bus_pan: [[f32; 2]; SUB_BUS_COUNT],
    /// Per-sub-bus high-pass (low-cut) filter (per channel), ahead of the
    /// low-pass. Identity = off (bypass).
    bus_hp_l: [Biquad; SUB_BUS_COUNT],
    bus_hp_r: [Biquad; SUB_BUS_COUNT],
    /// Per-sub-bus low-pass filter (per channel). Identity = open (bypass).
    bus_filter_l: [Biquad; SUB_BUS_COUNT],
    bus_filter_r: [Biquad; SUB_BUS_COUNT],
    /// Per-sub-bus reverb aux-send amount (0..1) — how much of that bus's
    /// post-fader signal feeds the reverb return. `0` = dry (default).
    bus_send: [f32; SUB_BUS_COUNT],
    /// Per-sub-bus delay aux-send amount (0..1) — feeds the delay return.
    bus_delay_send: [f32; SUB_BUS_COUNT],
    /// Master limiter engaged?
    limiter: bool,
    /// Current limiter gain reduction (`1.0` = none) — a linked-stereo envelope
    /// carried across blocks so the release is continuous.
    limiter_gr: f32,
    /// Master reverb as a parallel **return** (fed by the per-bus sends, not a
    /// master insert) + its enable + return level (`reverb_mix`).
    reverb: Reverb,
    reverb_on: bool,
    reverb_mix: f32,
    /// Master delay/echo as a parallel **return** (fed by the per-bus delay
    /// sends) + its enable + return level (`delay_mix`).
    delay: Delay,
    delay_on: bool,
    delay_mix: f32,
}

impl Mixer {
    pub(crate) fn new(format: AudioFormat, max_voices: usize) -> Self {
        Self {
            pool: VoicePool::new(max_voices, format),
            master_gain: SmoothGain::immediate(1.0),
            master_pan: balance_gains(0.0),
            hp_l: Biquad::default(),
            hp_r: Biquad::default(),
            filter_l: Biquad::default(),
            filter_r: Biquad::default(),
            eq_l: std::array::from_fn(|_| Biquad::default()),
            eq_r: std::array::from_fn(|_| Biquad::default()),
            bus_gain: std::array::from_fn(|_| SmoothGain::immediate(1.0)),
            bus_pan: [balance_gains(0.0); SUB_BUS_COUNT],
            bus_hp_l: std::array::from_fn(|_| Biquad::default()),
            bus_hp_r: std::array::from_fn(|_| Biquad::default()),
            bus_filter_l: std::array::from_fn(|_| Biquad::default()),
            bus_filter_r: std::array::from_fn(|_| Biquad::default()),
            bus_send: [0.0; SUB_BUS_COUNT],
            bus_delay_send: [0.0; SUB_BUS_COUNT],
            limiter: false,
            limiter_gr: 1.0,
            reverb: Reverb::new(format.sample_rate),
            reverb_on: false,
            reverb_mix: 0.3,
            delay: Delay::new(format.sample_rate),
            delay_on: false,
            delay_mix: 0.3,
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
            AudioCommand::SetMasterHighpass { coeffs } => {
                self.hp_l.set_coeffs(coeffs);
                self.hp_r.set_coeffs(coeffs);
            }
            AudioCommand::SetMasterEq { low, mid, high } => {
                let bands = [low, mid, high];
                for (b, c) in self.eq_l.iter_mut().zip(bands) {
                    b.set_coeffs(c);
                }
                for (b, c) in self.eq_r.iter_mut().zip(bands) {
                    b.set_coeffs(c);
                }
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
            AudioCommand::SetMasterLimiter { on } => self.limiter = on,
            AudioCommand::SetBusFilter { bus, coeffs } => {
                if let Some(i) = bus.sub_index() {
                    self.bus_filter_l[i].set_coeffs(coeffs);
                    self.bus_filter_r[i].set_coeffs(coeffs);
                }
            }
            AudioCommand::SetBusHighpass { bus, coeffs } => {
                if let Some(i) = bus.sub_index() {
                    self.bus_hp_l[i].set_coeffs(coeffs);
                    self.bus_hp_r[i].set_coeffs(coeffs);
                }
            }
            AudioCommand::SetReverb { on, mix, room_size } => {
                self.reverb_on = on;
                // `mix` is the return level (wet fold-back), not a wet/dry insert.
                self.reverb_mix = mix.clamp(0.0, 1.0);
                // Fixed, musical damping; Size drives the decay length.
                self.reverb.set_params(room_size, 0.5);
            }
            AudioCommand::SetBusSend { bus, amount } => {
                if let Some(i) = bus.sub_index() {
                    self.bus_send[i] = amount.clamp(0.0, 1.0);
                }
            }
            AudioCommand::SetDelay {
                on,
                time,
                feedback,
                mix,
            } => {
                self.delay_on = on;
                self.delay_mix = mix.clamp(0.0, 1.0);
                self.delay.set_params(time, feedback);
            }
            AudioCommand::SetBusDelaySend { bus, amount } => {
                if let Some(i) = bus.sub_index() {
                    self.bus_delay_send[i] = amount.clamp(0.0, 1.0);
                }
            }
        }
    }

    /// Sum active voices into `master` (interleaved stereo): each sub-bus through
    /// its fader (into `bus_scratch`, then folded in) while accumulating its
    /// reverb aux-send into `send` and delay aux-send into `delay_send`, then
    /// master-direct voices, then the reverb + delay returns + master
    /// gain/filters/pan/limiter. `bus_peaks[i]` receives sub-bus `i`'s post-fader
    /// `[L, R]` peak. Finished voices' samples go to `on_finished`.
    // The scratch buffers + meter outputs are all distinct borrows the hot path
    // needs at once; bundling them into a struct would only hide the data flow of
    // the mixer's single per-block entry point.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render(
        &mut self,
        master: &mut [Sample],
        bus_scratch: &mut [Sample],
        send: &mut [Sample],
        delay_send: &mut [Sample],
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
            let [pan_l, pan_r] = self.bus_pan[i];
            let send_amt = self.bus_send[i];
            let delay_amt = self.bus_delay_send[i];
            let gain = &mut self.bus_gain[i];
            let hp_l = &mut self.bus_hp_l[i];
            let hp_r = &mut self.bus_hp_r[i];
            let filter_l = &mut self.bus_filter_l[i];
            let filter_r = &mut self.bus_filter_r[i];
            let mut peak_l = 0.0f32;
            let mut peak_r = 0.0f32;
            let mut sq_l = 0.0f32;
            let mut sq_r = 0.0f32;
            for f in 0..frames {
                let g = gain.tick();
                // Fader, then the bus low-cut high-pass, then the low-pass (both
                // identity = open). Post-filter, pre-pan is the strip meter
                // reading, so panning doesn't drop it.
                let l = filter_l.process(hp_l.process(bus_scratch[2 * f] * g));
                let r = filter_r.process(hp_r.process(bus_scratch[2 * f + 1] * g));
                peak_l = peak_l.max(l.abs());
                peak_r = peak_r.max(r.abs());
                sq_l += l * l;
                sq_r += r * r;
                let (fold_l, fold_r) = (l * pan_l, r * pan_r);
                master[2 * f] += fold_l;
                master[2 * f + 1] += fold_r;
                // Effect aux sends (post-fader, post-pan): scaled copies into the
                // shared reverb + delay send buses, accumulated across every
                // sub-bus this block.
                send[2 * f] += fold_l * send_amt;
                send[2 * f + 1] += fold_r * send_amt;
                delay_send[2 * f] += fold_l * delay_amt;
                delay_send[2 * f + 1] += fold_r * delay_amt;
            }
            bus_peaks[i] = [peak_l, peak_r];
            bus_rms[i] = [(sq_l * inv_frames).sqrt(), (sq_r * inv_frames).sqrt()];
        }

        // Voices routed straight to the master mix (no sub-bus fader).
        self.pool
            .render_bus(BusId::Master, master, frames, on_finished);

        // The reverb return (the accumulated per-bus sends through the reverb,
        // folded back at `reverb_mix`), then master gain, the master low-cut
        // high-pass then low-pass filters (both identity = bypass), the master
        // stereo balance, then the limiter (bypassed when disengaged).
        let [mpan_l, mpan_r] = self.master_pan;
        let limiter = self.limiter;
        let reverb_on = self.reverb_on;
        let reverb_mix = self.reverb_mix;
        let delay_on = self.delay_on;
        let delay_mix = self.delay_mix;
        let mut gr = self.limiter_gr;
        for f in 0..frames {
            let g = self.master_gain.tick();
            // Effect returns: fold the wet reverb + delay send-buses back into the
            // dry master before the master fader/filters so they govern the wet
            // too (each bypassed when off).
            let (mut ml, mut mr) = (master[2 * f], master[2 * f + 1]);
            if reverb_on {
                let (wet_l, wet_r) = self.reverb.process(send[2 * f], send[2 * f + 1]);
                ml += wet_l * reverb_mix;
                mr += wet_r * reverb_mix;
            }
            if delay_on {
                let (wet_l, wet_r) = self.delay.process(delay_send[2 * f], delay_send[2 * f + 1]);
                ml += wet_l * delay_mix;
                mr += wet_r * delay_mix;
            }
            let mut l = self.filter_l.process(self.hp_l.process(ml * g));
            let mut r = self.filter_r.process(self.hp_r.process(mr * g));
            // Master 3-band EQ in series (identity bands = flat/transparent).
            for b in 0..3 {
                l = self.eq_l[b].process(l);
                r = self.eq_r[b].process(r);
            }
            l *= mpan_l;
            r *= mpan_r;
            if limiter {
                // Linked-stereo gain reduction: fast attack pulls the level down
                // to the ceiling on loud peaks, slow release eases it back — the
                // audible level control. `soft_clip` is the brickwall for the
                // transient overshoot before the envelope catches.
                let peak = l.abs().max(r.abs());
                let target = if peak > LIMIT_CEILING {
                    LIMIT_CEILING / peak
                } else {
                    1.0
                };
                let coeff = if target < gr {
                    LIMIT_ATTACK
                } else {
                    LIMIT_RELEASE
                };
                gr += (target - gr) * coeff;
                l = soft_clip(l * gr);
                r = soft_clip(r * gr);
            }
            master[2 * f] = l;
            master[2 * f + 1] = r;
        }
        // Reset the envelope when disengaged so re-engaging starts clean.
        self.limiter_gr = if limiter { gr } else { 1.0 };
    }

    pub(crate) fn active_voices(&self) -> usize {
        self.pool.active_count()
    }

    pub(crate) fn voice_capacity(&self) -> usize {
        self.pool.capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::soft_clip;

    #[test]
    fn soft_clip_passes_small_signals_and_caps_loud_ones() {
        // Below the knee: identity.
        assert_eq!(soft_clip(0.5), 0.5);
        assert_eq!(soft_clip(-0.3), -0.3);
        // Way over full scale: pulled below the ceiling (never hard-clips).
        assert!(soft_clip(4.0) < 0.99, "loud input must stay under ceiling");
        assert!(soft_clip(4.0) > 0.8, "…but above the knee");
        // Symmetric.
        assert!((soft_clip(4.0) + soft_clip(-4.0)).abs() < 1e-6);
        // Monotonic across the knee.
        assert!(soft_clip(0.9) < soft_clip(1.5));
    }
}
