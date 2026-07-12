//! [`VoicePool`] — a fixed set of voices with oldest-quietest stealing.

use crate::buffer::SampleData;
use crate::bus::BusId;
use crate::command::PlayParams;
use crate::format::AudioFormat;
use crate::stream::StreamHandle;
use crate::voice::Source;
use crate::voice::{Voice, VoiceId};

/// A fixed-capacity pool of voices. All slots are pre-allocated at construction;
/// `start`/`render`/`stop` never allocate. When full, `start` **steals** the
/// oldest-quietest voice (SKILL §11.6).
pub(crate) struct VoicePool {
    voices: Vec<Voice>,
    format: AudioFormat,
}

impl VoicePool {
    pub(crate) fn new(max_voices: usize, format: AudioFormat) -> Self {
        let mut voices = Vec::with_capacity(max_voices);
        voices.resize_with(max_voices, Voice::silent);
        Self { voices, format }
    }

    /// Number of voice slots (never changes → the HR-3 capacity anchor).
    pub(crate) fn capacity(&self) -> usize {
        self.voices.len()
    }

    /// How many slots are currently sounding.
    pub(crate) fn active_count(&self) -> usize {
        self.voices.iter().filter(|v| !v.is_free()).count()
    }

    /// Start `id`, reusing a free slot or stealing the oldest-quietest one.
    /// A stolen voice's sample is handed to `on_finished` (off-thread drop).
    pub(crate) fn start(
        &mut self,
        id: VoiceId,
        data: SampleData,
        params: PlayParams,
        on_finished: &mut dyn FnMut(Source),
    ) {
        let slot = match self.voices.iter().position(Voice::is_free) {
            Some(i) => i,
            None => {
                let i = self.steal_index();
                if let Some(old) = self.voices[i].free() {
                    on_finished(old);
                }
                i
            }
        };
        self.voices[slot].start(id, data, &params, self.format.sample_rate);
    }

    /// Same as [`VoicePool::start`], but the audio arrives from a stream (ADR-0118). Voice
    /// stealing, bussing and envelopes behave identically — a streamed voice is a voice.
    pub(crate) fn start_stream(
        &mut self,
        id: VoiceId,
        handle: Box<StreamHandle>,
        params: PlayParams,
        on_finished: &mut dyn FnMut(Source),
    ) {
        let slot = match self.voices.iter().position(Voice::is_free) {
            Some(i) => i,
            None => {
                let i = self.steal_index();
                if let Some(old) = self.voices[i].free() {
                    on_finished(old);
                }
                i
            }
        };
        self.voices[slot].start_stream(id, handle, &params, self.format.sample_rate);
    }

    /// Index of the voice to steal: minimum loudness, ties broken by oldest age.
    fn steal_index(&self) -> usize {
        use std::cmp::Ordering;
        let mut best = 0usize;
        let mut best_level = f32::INFINITY;
        let mut best_age = 0u64;
        for (i, v) in self.voices.iter().enumerate() {
            let level = v.level();
            let age = v.age();
            // Prefer quieter; on a tie prefer older (larger age). `partial_cmp`
            // keeps this off the `float_cmp` lint (no `==` on floats).
            let better = match level.partial_cmp(&best_level) {
                Some(Ordering::Less) => true,
                Some(Ordering::Equal) => age > best_age,
                _ => false,
            };
            if better {
                best = i;
                best_level = level;
                best_age = age;
            }
        }
        best
    }

    fn slot_mut(&mut self, id: VoiceId) -> Option<&mut Voice> {
        if id.is_none() {
            return None;
        }
        self.voices.iter_mut().find(|v| v.id() == id)
    }

    pub(crate) fn stop(&mut self, id: VoiceId, on_finished: &mut dyn FnMut(Source)) {
        if let Some(v) = self.slot_mut(id)
            && let Some(data) = v.free()
        {
            on_finished(data);
        }
    }

    pub(crate) fn release(&mut self, id: VoiceId) {
        if let Some(v) = self.slot_mut(id) {
            v.release();
        }
    }

    pub(crate) fn set_gain(&mut self, id: VoiceId, gain: f32) {
        if let Some(v) = self.slot_mut(id) {
            v.set_gain(gain);
        }
    }

    pub(crate) fn set_pan(&mut self, id: VoiceId, pan: f32) {
        if let Some(v) = self.slot_mut(id) {
            v.set_pan(pan);
        }
    }

    /// Mix every active voice routed to `target` into `out`; finished voices'
    /// samples go to `on_finished`. The mixer calls this once per sub-bus (into a
    /// per-bus scratch) and once for [`BusId::Master`] (voices routed straight to
    /// the master mix).
    pub(crate) fn render_bus(
        &mut self,
        target: BusId,
        out: &mut [crate::format::Sample],
        frames: usize,
        on_finished: &mut dyn FnMut(Source),
    ) {
        for v in &mut self.voices {
            if v.is_free() || v.bus() != target {
                continue;
            }
            if let Some(done) = v.render_add(out, frames) {
                on_finished(done);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn silent_sample() -> SampleData {
        // A short quiet sample so voices are "quietest" candidates predictably.
        SampleData::from_interleaved(vec![0.0; 480], AudioFormat::mono(48_000))
    }

    fn noop(_s: Source) {}

    #[test]
    fn fills_free_slots_then_caps_by_stealing() {
        let mut pool = VoicePool::new(4, AudioFormat::stereo(48_000));
        let mut sink: Box<dyn FnMut(Source)> = Box::new(noop);
        for i in 1..=4 {
            pool.start(
                VoiceId(i),
                silent_sample(),
                PlayParams::default(),
                &mut *sink,
            );
        }
        assert_eq!(pool.active_count(), 4);
        // 5th must steal, not grow.
        let mut stolen = 0;
        let mut counting: Box<dyn FnMut(Source)> = Box::new(|_| stolen += 1);
        pool.start(
            VoiceId(5),
            silent_sample(),
            PlayParams::default(),
            &mut *counting,
        );
        drop(counting);
        assert_eq!(pool.capacity(), 4, "pool must never grow");
        assert_eq!(pool.active_count(), 4, "still exactly capacity after steal");
        assert_eq!(stolen, 1, "the stolen voice's sample is returned once");
    }

    #[test]
    fn stop_frees_a_slot() {
        let mut pool = VoicePool::new(2, AudioFormat::stereo(48_000));
        let mut sink: Box<dyn FnMut(Source)> = Box::new(noop);
        pool.start(
            VoiceId(1),
            silent_sample(),
            PlayParams::default(),
            &mut *sink,
        );
        assert_eq!(pool.active_count(), 1);
        pool.stop(VoiceId(1), &mut *sink);
        assert_eq!(pool.active_count(), 0);
        // Stopping an unknown id is a harmless no-op.
        pool.stop(VoiceId(999), &mut *sink);
    }
}
