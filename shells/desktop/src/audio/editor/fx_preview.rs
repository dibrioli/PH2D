//! **The knob-drag preview, O(selection)** — ADR-0120, closing ADR-0117 §5.
//!
//! # The measurement that made this worth doing
//!
//! Dragging a slider re-renders the audition **every frame**. The DSP is already scoped to the
//! selection, so on a 3-minute clip with a 1-second selection it touches 0.55 % of the audio —
//! but the result has to be a contiguous `SampleData` for the mixer to play, and building one is
//! a whole-clip copy. Measured (`measure_preview.rs`):
//!
//! ```text
//!   whole-clip copy (the contiguity tax)   11.31 ms   <- 69 % of the frame
//!   the DSP on the 1 s selection            5.11 ms
//!   one frame of a knob drag               16.43 ms   <- the entire 60 fps budget
//! ```
//!
//! Two thirds of every frame was memcpy of audio that did not change.
//!
//! # Why it could not simply be "mutate the buffer"
//!
//! `SampleData` is an immutable `Arc<[f32]>` *on purpose*: the RT thread holds it, and a buffer
//! that changed under the mixer would tear. So there is no in-place path — **unless you are the
//! only owner**, which is exactly what `SampleData::get_mut` asks (`Arc::get_mut`, safe, and it
//! simply refuses when a clone exists).
//!
//! And you can BE the only owner, because the machinery was already there: a preview hot-swap
//! makes the mixer hand its old buffer back through the **return ring**, and the control thread
//! drops it (`AudioSystem::poll`, HR-3 — a `free()` on the audio thread is an allocation running
//! backwards). So a buffer sent two frames ago is *yours* again.
//!
//! Hence **two scratch buffers, alternating**: send A, and while the mixer plays it, rewrite B —
//! which the mixer handed back last frame. Neither the preview contract nor HR-3 moves an inch.
//!
//! # The one thing that can go wrong, and the gate for it
//!
//! A scratch buffer is only correct if the audio **outside** the region is still the head's. Move
//! the selection, or change an upstream stage, and it is stale — and stale audio is silently
//! WRONG audio, which is worse than slow audio. So the scratch is keyed on
//! (head buffer identity, selection) and thrown away the moment either moves, and
//! `the_incremental_preview_is_byte_identical_to_a_full_render` drives a whole sequence of drags,
//! selection moves and stage edits through both paths and compares the bytes.
//!
//! Every bail-out falls back to the full render, which always works. The fast path is an
//! optimisation, never a second source of truth.

use ph2d_audio::SampleData;
use ph2d_audio_edit::EditClip;
use ph2d_panel_audio_editor::FxStage;

use super::fx_rack::is_audible;
use crate::audio::fx_params::{FxCommand, build};

/// What the scratch's out-of-region audio belongs to: the head buffer (identified by pointer +
/// length — `SampleData` is an immutable `Arc`, so a new buffer is a new pointer) and the
/// selection. Either moves and every byte outside the region is stale.
type ScratchKey = (usize, usize, Option<(usize, usize)>);

/// The two alternating audition buffers, and what they were built from.
#[derive(Default)]
pub(crate) struct PreviewScratch {
    buffers: [Option<SampleData>; 2],
    slot: usize,
    key: Option<ScratchKey>,
    /// Did the last `step` have to BUILD its buffer (one whole-clip copy) rather than rewrite a
    /// region of one it already had?
    ///
    /// This is not bookkeeping for its own sake: the first two frames of a drag fill the two
    /// slots, so they each pay a copy, and only from the third frame on is the drag free. Without
    /// this flag the frame log calls a fill frame `region rewrite` and prints 32 ms next to it —
    /// and the smoke would say, in its own output, that the optimisation does not work.
    filled: bool,
}

impl PreviewScratch {
    /// Throw the scratch away — the base it was built from is gone.
    fn reset(&mut self, key: ScratchKey) {
        self.buffers = [None, None];
        self.key = Some(key);
    }

    /// One frame of the fast path: rewrite the region of the slot the mixer is not holding.
    ///
    /// A free method rather than a method on `AudioSystem` **so it can be gated**:
    /// `AudioSystem::new()` needs an audio device and no headless test can build one, so anything
    /// that only exists behind it is smoke-only. The alternation with the mixer is the part most
    /// worth proving — an optimisation that silently never fires is worse than no optimisation,
    /// because it looks done.
    pub(crate) fn step(
        &mut self,
        head: &EditClip,
        fx: ph2d_audio_edit::Effect,
    ) -> Option<EditClip> {
        let hd = head.data();
        let selection = head.selection().map(|r| (r.start, r.end));
        let key = (
            hd.samples().as_ptr() as usize,
            hd.samples().len(),
            selection,
        );
        if self.key != Some(key) {
            self.reset(key);
        }

        // The region, and only the region. `None` = the effect is not length-preserving here, so
        // the splice would not fit anyway.
        let (range, region) = head.render_effect_region(fx)?;
        let ch = hd.format().channel_count().max(1);

        let slot = self.slot;
        // A slot with no buffer yet is one whole copy — paid once per selection, not per frame.
        // (Both slots fill over the first two frames of a drag; every frame after that is free.)
        //
        // `map_in_place`, NOT `hd.clone()`: cloning a `SampleData` bumps the `Arc`, it does not
        // copy the data — so the head would still be holding the buffer, `get_mut` would refuse
        // FOREVER, and the fast path would be dead code that silently never ran. The gates caught
        // exactly that.
        self.filled = self.buffers[slot].is_none();
        let buf = self.buffers[slot].get_or_insert_with(|| SampleData::map_in_place(hd, |_| {}));

        // The buffer the mixer is still holding cannot be touched. It comes back on the next
        // `poll` (the return ring, HR-3) — until then, the caller re-renders in full.
        let out = buf.get_mut()?;
        out[range.start * ch..range.end * ch].copy_from_slice(region.samples());

        // The clone is an `Arc` bump, not a copy: this is the whole point.
        let mut clip = EditClip::new(buf.clone());
        clip.set_selection(head.selection());
        self.slot ^= 1;
        Some(clip)
    }
}

/// The single audible **Plain** stage in `chain[from..]`, if that is exactly what is there.
///
/// The incremental path rewrites ONE region of ONE buffer. Two audible stages would each need
/// their own base (the second acts on the first's output), and a tail effect changes the buffer's
/// LENGTH — neither fits, and both fall back to the full render. This is not a narrow case: it is
/// the one a knob drag is, almost always.
pub(crate) fn lone_plain_stage(chain: &[FxStage], from: usize) -> Option<ph2d_audio_edit::Effect> {
    let mut found = None;
    for stage in &chain[from.min(chain.len())..] {
        if !is_audible(stage) {
            continue;
        }
        if found.is_some() {
            return None; // two audible stages: not our case
        }
        match build(stage.kind, &stage.norms)? {
            FxCommand::Plain(fx) => found = Some(fx),
            FxCommand::Tail(_) => return None, // grows the buffer
        }
    }
    found
}

/// **The A/B switch** (`PH2D_AUDIO_SLOW_PREVIEW=1`): make the fast path refuse, so every drag
/// frame goes down the whole-clip render this ADR replaced.
///
/// It exists because the optimisation is **invisible by construction** — the output is byte-
/// identical, so there is nothing to hear, and "it feels smooth now" is not something a human can
/// check against a memory of last week. A smoke you cannot A/B is a demo. This makes the *old*
/// behaviour reachable, in the shipping code path, with one env var.
///
/// Read once: a drag reads it every frame, and the answer cannot change mid-session.
fn slow_preview_forced() -> bool {
    static FORCED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *FORCED.get_or_init(|| std::env::var_os("PH2D_AUDIO_SLOW_PREVIEW").is_some())
}

impl super::super::AudioSystem {
    /// The audition for this frame, rendered O(selection) if it can be — `None` when it cannot,
    /// and the caller does the full render.
    pub(crate) fn fx_preview_incremental(
        &mut self,
        head: &EditClip,
        chain: &[FxStage],
        sel: usize,
    ) -> Option<EditClip> {
        if slow_preview_forced() {
            return None; // the fallback below IS the old path — no second implementation to drift
        }
        let fx = lone_plain_stage(chain, sel)?;
        self.fx_scratch.step(head, fx)
    }

    /// Did the last fast-path frame BUILD its buffer (a whole-clip copy, paid once per slot) or
    /// rewrite a region of one it already owned? Only the second is the steady state of a drag.
    pub(crate) fn fx_preview_filled(&self) -> bool {
        self.fx_scratch.filled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::{AudioFormat, ChannelLayout};
    use ph2d_audio_edit::Effect;

    const SR: usize = 48_000;

    fn clip(secs: usize) -> SampleData {
        SampleData::from_fn(
            secs * SR * 2,
            AudioFormat::new(SR as u32, ChannelLayout::Stereo),
            |i| ((i % 977) as f32 / 977.0) * 0.5 - 0.25,
        )
    }

    /// The incremental path, spelled out without the shell's caches: take a buffer whose
    /// out-of-region audio is the base, rewrite the region, and compare against the full render.
    fn incremental(base: &EditClip, fx: Effect, scratch: &mut SampleData) -> SampleData {
        let (range, region) = base.render_effect_region(fx).expect("length-preserving");
        let ch = base.data().format().channel_count().max(1);
        let out = scratch.get_mut().expect("sole owner in the test");
        out[range.start * ch..range.end * ch].copy_from_slice(region.samples());
        scratch.clone()
    }

    /// **The gate the whole thing rests on: the fast path and the slow path agree, byte for
    /// byte.**
    ///
    /// Not "sound the same" — *byte for byte*, across a sequence that moves the selection and
    /// re-tunes the effect, which is exactly what a knob drag is. A stale scratch would produce
    /// audio that is plausible and wrong, and nothing downstream could tell.
    #[test]
    fn the_incremental_preview_is_byte_identical_to_a_full_render() {
        let base_data = clip(4);
        let ranges = [SR..SR * 2, SR / 2..SR * 3, 0..SR, SR * 3..SR * 4];
        let ratios = [2.0f32, 8.0, 20.0, 4.0];

        for range in ranges {
            let mut base = EditClip::new(base_data.clone());
            base.set_selection(Some(range.clone()));
            // A fresh scratch per selection — which is exactly what the shell's `key` forces.
            // A COPY, not an `Arc` clone: a clone would leave `base_data` holding the buffer and
            // `get_mut` would (correctly) refuse.
            let mut scratch = SampleData::map_in_place(&base_data, |_| {});

            for ratio in ratios {
                let fx = Effect::Compress {
                    threshold: 0.3,
                    ratio,
                    attack_secs: 0.005,
                    release_secs: 0.1,
                };
                let full = base.render_effect(fx);
                let fast = incremental(&base, fx, &mut scratch);
                assert_eq!(
                    fast.samples(),
                    full.samples(),
                    "selection {range:?}, ratio {ratio}: the incremental preview does not match \
                     the full render -- the scratch is stale and the user is hearing audio the \
                     rack never produced"
                );
            }
        }
    }

    /// ...and it is not vacuously equal: the region really is being written, and the audio
    /// outside it really is being left alone. Without this, a scratch that never wrote anything
    /// would pass the gate above whenever the effect happened to be a no-op.
    #[test]
    fn it_rewrites_the_region_and_nothing_else() {
        let base_data = clip(2);
        let range = SR / 2..SR;
        let mut base = EditClip::new(base_data.clone());
        base.set_selection(Some(range.clone()));
        let mut scratch = SampleData::map_in_place(&base_data, |_| {});
        let out = incremental(
            &base,
            Effect::Compress {
                threshold: 0.1,
                ratio: 20.0,
                attack_secs: 0.001,
                release_secs: 0.05,
            },
            &mut scratch,
        );
        let ch = 2;
        assert_ne!(
            &out.samples()[range.start * ch..range.end * ch],
            &base_data.samples()[range.start * ch..range.end * ch],
            "the region was not touched"
        );
        assert_eq!(
            &out.samples()[..range.start * ch],
            &base_data.samples()[..range.start * ch],
            "audio BEFORE the selection was modified"
        );
        assert_eq!(
            &out.samples()[range.end * ch..],
            &base_data.samples()[range.end * ch..],
            "audio AFTER the selection was modified"
        );
    }

    /// **The fast path actually FIRES, with the mixer holding a buffer** — which is the whole
    /// question, and the one no amount of byte-identity proves.
    ///
    /// This drives the real alternation: each frame the "mixer" takes the buffer we just produced
    /// and hands the previous one back through the return ring, which the control thread drops
    /// (`AudioSystem::poll`, HR-3). If the two slots did not alternate — or if the scratch were an
    /// `Arc` clone of the head rather than a copy — `get_mut` would refuse **every frame**, the
    /// step would return `None` for ever, and the shell would quietly fall back to the full render
    /// on all of them. The optimisation would be dead, every other gate would still be green, and
    /// the only symptom would be that the knobs were exactly as slow as before.
    #[test]
    fn the_fast_path_fires_every_frame_while_the_mixer_is_holding_a_buffer() {
        let base = clip(2);
        let mut head = EditClip::new(base.clone());
        head.set_selection(Some(SR / 2..SR));
        let mut scratch = PreviewScratch::default();

        // The mixer holds exactly one preview buffer at a time; the previous one goes to the
        // return ring, and the control thread drops it once per frame.
        let mut mixer: Option<SampleData> = None;
        let mut fired = 0usize;
        let frames = 8;
        for f in 0..frames {
            let fx = Effect::Compress {
                threshold: 0.3,
                // A different value every frame: this is a knob being dragged.
                ratio: 2.0 + f as f32,
                attack_secs: 0.005,
                release_secs: 0.1,
            };
            if let Some(clip) = scratch.step(&head, fx) {
                fired += 1;
                // Hot-swap: the mixer takes the new buffer and returns the old one, which is
                // dropped HERE -- on the control thread, exactly as `collect_returns` does.
                let returned = mixer.replace(clip.data().clone());
                drop(returned);
            }
        }
        assert_eq!(
            fired, frames,
            "the fast path fired on only {fired} of {frames} frames -- the slots are not \
             alternating, so `get_mut` is being refused and every drag frame is falling back to \
             the full whole-clip render. The optimisation is dead and nothing else would say so."
        );
        // ...and it is still the right audio, with the mixer in the loop.
        let expected = head.render_effect(Effect::Compress {
            threshold: 0.3,
            ratio: 2.0 + (frames - 1) as f32,
            attack_secs: 0.005,
            release_secs: 0.1,
        });
        assert_eq!(
            mixer.expect("the mixer holds the last buffer").samples(),
            expected.samples(),
            "the last buffer the mixer was handed is not what a full render produces"
        );
    }

    /// **`get_mut` refuses while anyone else holds the buffer** — which is the entire safety
    /// argument. If it ever handed out a slice while the mixer had a clone, the RT thread would
    /// read audio that is being rewritten under it.
    #[test]
    fn a_buffer_the_mixer_still_holds_cannot_be_mutated() {
        let mut a = clip(1);
        assert!(
            a.get_mut().is_some(),
            "the sole owner must be able to write"
        );
        let mixer_holds_it = a.clone();
        assert!(
            a.get_mut().is_none(),
            "get_mut handed out a slice while a clone existed -- the mixer would tear"
        );
        drop(mixer_holds_it); // ...as `collect_returns` does, on the control thread (HR-3).
        assert!(
            a.get_mut().is_some(),
            "the buffer did not become writable again after the mixer let go"
        );
    }
}
