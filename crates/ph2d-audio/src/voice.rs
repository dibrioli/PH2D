//! [`Voice`] — one playing sample, and [`VoiceId`], its opaque handle.

use crate::buffer::SampleData;
use crate::bus::BusId;
use crate::command::PlayParams;
use crate::dsp::{Adsr, SmoothGain, equal_power_pan};
use crate::format::Sample;
use crate::stream::{Pull, StreamHandle};

/// What the window could do when asked to cover the cursor.
enum Slide {
    /// `win` straddles the cursor; play.
    Ready,
    /// The producer has not caught up. Silence this frame; the cursor stays put.
    Stalled,
    /// The stream had no audio at all.
    Empty,
}

/// Where a voice's audio comes from (ADR-0118).
///
/// Both are **dropped off the audio thread**: a `SampleData` frees an `Arc<[f32]>`, a
/// `StreamHandle` frees its chunks, and a `free()` on the RT thread is an allocation running
/// backwards (HR-3). So a finished voice hands its source back through the return ring, whichever
/// kind it is — which is why this is one type and not two fields.
pub(crate) enum Source {
    /// The whole clip, decoded, in memory. What every voice was until ADR-0118.
    Resident(SampleData),
    /// Decoded ahead, in chunks, by a producer thread. The clip is never all here at once.
    Streamed(Box<StreamHandle>),
}

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
    source: Option<Source>,
    /// Fractional read position in *source frames* (f64 spans long samples exactly).
    cursor: f64,
    /// **Streaming only.** The two source frames the interpolation straddles — stream frames
    /// `head` and `head + 1`. A stream has no random access, but a cursor only ever moves
    /// *forward*, so a two-frame sliding window is all the resident path's `frame_stereo(i0)` /
    /// `frame_stereo(i0 + 1)` ever needed.
    win: [[Sample; 2]; 2],
    /// Absolute stream-frame index of `win[0]`.
    head: usize,
    /// **Streaming + looping only.** How many frames of the stream the wrapped cursor no longer
    /// counts. The cursor is wrapped exactly as the resident path wraps it (`cursor -= len`, which
    /// is what keeps the two bit-identical and the `f64` from drifting over a long stream), so the
    /// absolute frame the window must cover is `base + cursor`, not `cursor`.
    base: usize,
    /// How much of `win` has been primed (0, 1, or 2).
    filled: usize,
    /// Total frames — knowable only once the stream says it has ended. A **looping** stream never
    /// does: the producer splices the file's start onto its end, so the sequence is endless and the
    /// interpolation across the loop point lands on frame 0 all by itself, exactly as the resident
    /// path's `frame_stereo(0)` does.
    total: Option<usize>,
    /// Source-frames advanced per output frame = (source_rate / out_rate) * pitch.
    advance: f64,
    gain: SmoothGain,
    pan_gains: [f32; 2],
    envelope: Option<Adsr>,
    looping: bool,
    /// Which mixer bus this voice sums into (set at `start`).
    bus: BusId,
    /// Output frames rendered since `start` — the "oldest" axis for stealing.
    age: u64,
}

impl Voice {
    /// A free (silent) voice slot.
    pub(crate) fn silent() -> Self {
        Self {
            id: VoiceId::NONE,
            source: None,
            cursor: 0.0,
            win: [[0.0; 2]; 2],
            head: 0,
            base: 0,
            filled: 0,
            total: None,
            advance: 1.0,
            gain: SmoothGain::immediate(0.0),
            pan_gains: [0.0, 0.0],
            envelope: None,
            looping: false,
            bus: BusId::Master,
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
        let rate = data.format().sample_rate;
        self.start_source(id, Source::Resident(data), rate, params, out_rate);
    }

    /// (Re)start this slot playing a **stream** (ADR-0118). Same knobs, same arithmetic — the only
    /// difference is where the frames come from.
    pub(crate) fn start_stream(
        &mut self,
        id: VoiceId,
        handle: Box<StreamHandle>,
        params: &PlayParams,
        out_rate: u32,
    ) {
        let rate = handle.sample_rate();
        self.start_source(id, Source::Streamed(handle), rate, params, out_rate);
    }

    fn start_source(
        &mut self,
        id: VoiceId,
        source: Source,
        src_rate: u32,
        params: &PlayParams,
        out_rate: u32,
    ) {
        let advance_base = f64::from(src_rate) / f64::from(out_rate.max(1));
        self.envelope = params.envelope.map(|p| {
            let mut e = Adsr::new(p, out_rate as f32);
            e.trigger();
            e
        });
        self.id = id;
        self.source = Some(source);
        self.cursor = 0.0;
        self.win = [[0.0; 2]; 2];
        self.head = 0;
        self.base = 0;
        self.filled = 0;
        self.total = None;
        self.advance = advance_base * params.pitch.max(0.0) as f64;
        self.gain = SmoothGain::immediate(params.gain);
        self.pan_gains = equal_power_pan(params.pan);
        self.looping = params.looping;
        self.bus = params.bus;
        self.age = 0;
    }

    pub(crate) fn id(&self) -> VoiceId {
        self.id
    }

    /// Current read position in whole source frames — the editor preview
    /// transport publishes this as the playhead.
    pub(crate) fn cursor_frame(&self) -> u64 {
        self.cursor.max(0.0) as u64
    }

    /// Jump the read cursor to `frame` (clamped to the sample length). Used by
    /// the preview transport's seek/scrub; only meaningful on a started voice.
    ///
    /// **A stream cannot seek** (ADR-0118 §5): the frames it has are the ones already decoded, and
    /// jumping means re-positioning the decoder and throwing the ring away — a producer operation,
    /// not a mixer one. Seeking a streamed voice is a no-op rather than a lie, and the editor's
    /// preview transport is unaffected: it seeks a clip that is *open*, and an open clip is
    /// resident by definition.
    pub(crate) fn set_cursor(&mut self, frame: u64) {
        let max = match self.source.as_ref() {
            Some(Source::Resident(d)) => d.frame_count() as f64,
            Some(Source::Streamed(_)) => return,
            None => 0.0,
        };
        self.cursor = (frame as f64).clamp(0.0, max);
    }

    /// The bus this voice sums into.
    pub(crate) fn bus(&self) -> BusId {
        self.bus
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

    /// Free the slot, returning whatever source it held (for off-thread drop — HR-3).
    pub(crate) fn free(&mut self) -> Option<Source> {
        self.id = VoiceId::NONE;
        self.source.take()
    }

    pub(crate) fn set_gain(&mut self, gain: f32) {
        self.gain.set_target(gain);
    }

    pub(crate) fn set_pan(&mut self, pan: f32) {
        self.pan_gains = equal_power_pan(pan);
    }

    /// Enable/disable looping live (e.g. the editor's Loop toggle mid-playback).
    pub(crate) fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }

    /// Hot-swap the playing sample for `data`, KEEPING the read cursor (clamped to
    /// the new length) so playback continues seamlessly — used by the editor to
    /// apply an edit without stopping the preview. Returns the previous sample (to
    /// drop off the RT thread), or `Some(data)` unchanged if this voice is free.
    pub(crate) fn replace_data(&mut self, data: SampleData) -> Option<Source> {
        if self.id.is_none() {
            return Some(Source::Resident(data));
        }
        let max = data.frame_count() as f64;
        let old = self.source.replace(Source::Resident(data));
        if self.cursor > max {
            self.cursor = max;
        }
        old
    }

    pub(crate) fn release(&mut self) {
        if let Some(e) = self.envelope.as_mut() {
            e.release();
        }
    }

    /// Mix this voice into `master` (interleaved stereo, `frames` frames).
    /// Returns the voice's [`Source`] if it finished during this block (the
    /// slot is now free); the caller ships it to the return ring.
    ///
    /// Dispatches **once**, at the top: the resident loop stays exactly as tight as it was, with no
    /// per-frame branch bought by a feature it does not use.
    pub(crate) fn render_add(&mut self, master: &mut [Sample], frames: usize) -> Option<Source> {
        match self.source.take()? {
            Source::Resident(data) => self
                .render_resident(data, master, frames)
                .map(Source::Resident),
            Source::Streamed(h) => self
                .render_streamed(h, master, frames)
                .map(Source::Streamed),
        }
    }

    fn render_resident(
        &mut self,
        data: SampleData,
        master: &mut [Sample],
        frames: usize,
    ) -> Option<SampleData> {
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
        self.source = Some(Source::Resident(data));
        None
    }

    /// The streaming loop (ADR-0118). **The arithmetic is the resident loop's, line for line** —
    /// same `i0`/`frac`, same lerp, same `g`, same pan. The only difference is that `win[0]`/`win[1]`
    /// come from a sliding window over the stream instead of from an index into a buffer. That is
    /// what makes a streamed voice **bit-identical** to a resident one (A2), and a streaming path
    /// that only sounds *nearly* the same would be a bug nobody could ever find.
    fn render_streamed(
        &mut self,
        mut h: Box<StreamHandle>,
        master: &mut [Sample],
        frames: usize,
    ) -> Option<Box<StreamHandle>> {
        let [pan_l, pan_r] = self.pan_gains;

        let loop_len = h.loop_frames();

        for f in 0..frames {
            // Wrap the cursor at the loop point, **exactly as the resident loop does**. Not an
            // optimisation: letting the cursor run on would accumulate a different sequence of f64
            // roundings, and the two paths would part company by an ulp — inaudible, unfalsifiable,
            // and the sort of thing someone loses a week to. `base` remembers what was subtracted,
            // so the window still knows which absolute frame of the stream it is looking for.
            if self.looping
                && let Some(n) = loop_len
                && self.cursor >= n as f64
            {
                self.cursor -= n as f64;
                self.base += n;
            }

            // The stream ended and the cursor has played out everything it had.
            if self.total.is_some_and(|t| self.cursor >= t as f64) {
                self.id = VoiceId::NONE;
                return Some(h);
            }

            let i0 = self.cursor as usize;
            match self.slide(&mut h, self.base + i0) {
                Slide::Ready => {}
                // The producer is late. Silence for this frame, and **the cursor does not move** —
                // nothing is skipped, playback simply waits and resumes exactly where it was. A
                // dropout you can hear beats audio you silently lost, and the counter in the handle
                // makes it visible instead of mysterious.
                Slide::Stalled => continue,
                // An empty stream: nothing to play, and no reason to hold a slot.
                Slide::Empty => {
                    self.id = VoiceId::NONE;
                    return Some(h);
                }
            }

            let env = self.envelope.as_mut().map(|e| e.tick()).unwrap_or(1.0);
            let g = self.gain.tick() * env;

            let frac = (self.cursor - i0 as f64) as f32;
            let [l0, r0] = self.win[0];
            let [l1, r1] = self.win[1];
            let l = l0 + (l1 - l0) * frac;
            let r = r0 + (r1 - r0) * frac;

            master[2 * f] += l * g * pan_l;
            master[2 * f + 1] += r * g * pan_r;

            self.cursor += self.advance;
            self.age = self.age.saturating_add(1);

            if self.envelope.as_ref().is_some_and(|e| e.is_finished()) {
                self.id = VoiceId::NONE;
                return Some(h);
            }
        }

        self.source = Some(Source::Streamed(h));
        None
    }

    /// Slide the two-frame window so it straddles stream frames `i0` and `i0 + 1`.
    ///
    /// A stream has no random access — but a cursor only ever moves **forward**, so the window only
    /// ever needs to be pushed forward too, one frame at a time, which is exactly what a stream can
    /// do.
    fn slide(&mut self, h: &mut StreamHandle, i0: usize) -> Slide {
        // Prime: load frames 0 and 1.
        while self.filled < 2 {
            match h.pull() {
                Pull::Frame(fr) => {
                    self.win[self.filled] = fr;
                    self.filled += 1;
                }
                Pull::Underrun => return Slide::Stalled,
                Pull::Ended => {
                    if self.filled == 0 {
                        return Slide::Empty;
                    }
                    // A one-frame stream. Its last frame HOLDS, exactly as a resident clip's does
                    // (`else { [l0, r0] }` in the resident loop).
                    self.win[1] = self.win[0];
                    self.filled = 2;
                    self.total = Some(1);
                    return Slide::Ready;
                }
            }
        }

        while self.head < i0 {
            // Already sitting on the last frame — hold it, do not ask for more.
            if self.total.is_some_and(|t| self.head + 1 >= t) {
                return Slide::Ready;
            }
            match h.pull() {
                Pull::Frame(fr) => {
                    self.win[0] = self.win[1];
                    self.win[1] = fr;
                    self.head += 1;
                }
                Pull::Underrun => return Slide::Stalled,
                Pull::Ended => {
                    // `win[1]` is the final frame. Step onto it and hold — the resident loop's
                    // last-frame behaviour, reached from the other direction.
                    self.win[0] = self.win[1];
                    self.head += 1;
                    self.total = Some(self.head + 1);
                    return Slide::Ready;
                }
            }
        }
        Slide::Ready
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
