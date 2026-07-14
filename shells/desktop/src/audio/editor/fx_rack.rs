//! The Audio Editor's effects **chain** runtime (docs/Audio/, W3 blocks 3a/3b).
//!
//! A descendant submodule of `audio::editor`, so it reaches the private fields of
//! `AudioEditorRuntime` and `AudioSystem` while keeping `editor.rs` under the
//! HR-18 shell cap.
//!
//! The panel owns the chain (kind indices + normalized 0..1 slider positions); this
//! module renders it. The audition is `clip → stage₀ → … → stageₙ`, rendered over
//! the **pristine** clip and hot-swapped into the sounding preview, so the chain is
//! heard and drawn while it is tuned. Nothing is committed until Apply, which lands
//! exactly the buffer that sounded as **one** undo step.
//!
//! Two invariants earn their keep here:
//! - **What sounds is what commits.** Play, the waveform, the duration readout,
//!   Export and Apply all go through `editor_sounding()`. The global Bypass makes
//!   that the dry clip, so Apply then commits nothing (and the panel dims it).
//! - **A neutral stage costs nothing.** Effects are byte-identical no-ops at their
//!   defaults, so a bypassed or untouched stage is skipped, not rendered.

use super::AudioSystem;
use ph2d_audio_edit::EditClip;
use ph2d_panel_audio_editor::{FxStage, MAX_FX_PARAMS};

/// One stage, quantized: `(kind, slider normals ×1000, enabled)`. Float jitter must
/// not defeat the change-gate — a slider that doesn't really move cannot be allowed
/// to re-render the whole chain every frame.
type StageSig = (usize, [i32; MAX_FX_PARAMS], bool);

/// Everything the audition depends on: every stage, plus the **target range**
/// (moving the selection re-targets every effect in the chain).
pub(super) type ChainSig = (Vec<StageSig>, Option<(usize, usize)>);

/// What produced the cached head buffer: the index of the stage being edited, the
/// stages *before* it, and the target range.
pub(super) type HeadSig = (usize, Vec<StageSig>, Option<(usize, usize)>);

/// Set by the knob smoke, which needs the log to be the point of the scene rather than a second
/// env var the Enio has to remember (asking him to remember it is asking him to run it wrong).
static PREVIEW_LOG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Turn the drag-frame log on from code.
pub(crate) fn enable_preview_log() {
    PREVIEW_LOG.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// `PH2D_AUDIO_PREVIEW_LOG=1`, or a scene that asked for it — print what one drag frame cost, and
/// which path paid it.
///
/// The env half is read once: it is asked every drag frame and cannot change mid-session.
fn preview_log() -> bool {
    static ENV: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    PREVIEW_LOG.load(std::sync::atomic::Ordering::Relaxed)
        || *ENV.get_or_init(|| std::env::var_os("PH2D_AUDIO_PREVIEW_LOG").is_some())
}

/// Quantize one stage for the change-gate.
fn stage_sig(stage: &FxStage) -> StageSig {
    let mut q = [0i32; MAX_FX_PARAMS];
    for (slot, n) in q.iter_mut().zip(&stage.norms) {
        *slot = (n * 1_000.0) as i32;
    }
    (stage.kind, q, stage.enabled)
}

/// Whether a stage actually changes the audio: enabled, buildable, and off its
/// neutral point. A chain of fresh stages is silent work — skip it.
pub(super) fn is_audible(stage: &FxStage) -> bool {
    stage.enabled
        && crate::audio::fx_params::build(stage.kind, &stage.norms)
            .is_some_and(|cmd| !cmd.is_bypass())
}

/// Render one stage over `base`, carrying the target range forward so every stage
/// acts on the same selection (clamped if a tail effect grew the buffer).
/// `None` when the stage is transparent — the caller then keeps `base` as-is
/// instead of paying for a copy.
fn render_stage(base: &EditClip, stage: &FxStage) -> Option<EditClip> {
    if !is_audible(stage) {
        return None;
    }
    let data = match crate::audio::fx_params::build(stage.kind, &stage.norms)? {
        crate::audio::fx_params::FxCommand::Plain(fx) => base.render_effect(fx),
        crate::audio::fx_params::FxCommand::Tail(fx) => base.render_tail_effect(&fx),
    };
    let mut next = EditClip::new(data);
    next.set_selection(base.selection());
    Some(next)
}

/// Render `chain[from..]` over `head`, returning the last buffer produced (or a
/// clone of `head` when every remaining stage is transparent).
fn render_from(head: &EditClip, chain: &[FxStage], from: usize) -> EditClip {
    let mut owned: Option<EditClip> = None;
    for stage in &chain[from.min(chain.len())..] {
        let base: &EditClip = owned.as_ref().unwrap_or(head);
        if let Some(next) = render_stage(base, stage) {
            owned = Some(next);
        }
    }
    owned.unwrap_or_else(|| head.clone())
}

impl AudioSystem {
    /// Re-render the chain when the user has touched the rack and something actually
    /// changed, then hot-swap it into the sounding preview so it is heard **live**.
    /// The clip stays pristine — this is non-destructive.
    ///
    /// `sel` is the stage the sliders are editing. Everything before it is cached in
    /// `fx_head`, so dragging a slider re-renders only `chain[sel..]` — usually one
    /// stage. Selecting a range scopes each render, which is what keeps long clips
    /// responsive.
    pub(crate) fn editor_fx_update(&mut self, chain: &[FxStage], sel: usize) {
        let Some(clip) = self.editor.clip.as_ref() else {
            return;
        };
        let selection = clip.selection().map(|r| (r.start, r.end));
        let sig: Vec<StageSig> = chain.iter().map(stage_sig).collect();
        if self
            .editor
            .fx_sig
            .as_ref()
            .is_some_and(|(s, range)| *s == sig && *range == selection)
        {
            return;
        }

        // `chain[..sel]` is everything upstream of the slider under the user's
        // finger. Rebuild it only when it (or the target range) actually moved.
        let sel = sel.min(chain.len().saturating_sub(1));
        let head_sig: HeadSig = (sel, sig[..sel.min(sig.len())].to_vec(), selection);
        if self.editor.fx_head_sig.as_ref() != Some(&head_sig) {
            let mut head = clip.clone();
            for stage in &chain[..sel.min(chain.len())] {
                if let Some(next) = render_stage(&head, stage) {
                    head = next;
                }
            }
            self.editor.fx_head = Some(head);
            self.editor.fx_head_sig = Some(head_sig);
        }

        // An all-neutral chain is a no-op: leave the clip on the wire rather than audition a
        // byte-identical copy (which would arm Cancel/Bypass over nothing, and push a no-op undo
        // step on Apply).
        let audition = if chain.iter().any(is_audible) {
            let head = self.editor.fx_head.as_ref().expect("just built").clone();
            // This block runs ONCE PER DRAG FRAME — it is the 16 ms that ADR-0120 is about, so it
            // is the only honest place to time it. `PH2D_AUDIO_PREVIEW_LOG=1` prints the cost and
            // which path paid it; the knob smoke turns it on for itself.
            let t0 = preview_log().then(std::time::Instant::now);
            // O(selection) when it can be (ADR-0120): rewrite the region of a buffer the mixer has
            // already handed back, instead of copying the whole clip to change 0.55 % of it. It
            // bails out on anything it does not handle, and the full render below always works --
            // the fast path is an optimisation, never a second source of truth.
            let fast = self.fx_preview_incremental(&head, chain, sel);
            let took_fast = fast.is_some();
            let out = fast.unwrap_or_else(|| render_from(&head, chain, sel));
            if let Some(t0) = t0 {
                let ms = t0.elapsed().as_secs_f64() * 1_000.0;
                // A fast frame that had to BUILD its scratch is not the steady state: the first
                // two frames of a drag fill the two slots, one whole-clip copy each, and only from
                // the third on is the drag free. Calling those `region rewrite` would print 32 ms
                // beside the name of the optimisation and make it look broken in its own log.
                let filled = took_fast && self.fx_preview_filled();
                let path = match (took_fast, filled) {
                    (true, true) => {
                        "ADR-0120 warm-up (fills a scratch: one copy, twice per selection)"
                    }
                    (true, false) => "ADR-0120 (region rewrite) -- the steady state of a drag",
                    (false, _) => "full render (whole-clip copy) -- the pre-ADR-0120 path",
                };
                // 16.6 ms is one frame at 60 fps: over that, the drag itself stutters. Only the
                // steady state is judged -- the warm-up is a cost the design ACCEPTS, out loud.
                let verdict = if ms > 16.6 && !filled {
                    "  <-- OVER BUDGET (this is the stutter)"
                } else {
                    ""
                };
                println!("audio: preview frame {ms:6.2} ms  {path}{verdict}");
            }
            Some(out)
        } else {
            None
        };
        self.editor.fx_audition = audition;
        self.editor.fx_sig = Some((sig, selection));
        self.editor_hot_swap();
    }

    /// Commit **the exact buffer that was heard** as one undo step. With the global
    /// Bypass engaged that is the dry clip, so this is a no-op — the panel dims
    /// Apply, and this path agrees with the dim rather than trusting it.
    pub(crate) fn editor_fx_apply(&mut self, chain: &[FxStage], sel: usize) {
        if self.editor.fx_audition.is_none() {
            self.editor_fx_update(chain, sel);
        }
        // Clone out of the shared borrow before reaching for `clip` mutably.
        let sounding = self.editor_sounding().map(|c| c.data().clone());
        if let (Some(data), Some(clip)) = (sounding, self.editor.clip.as_mut())
            && data.samples() != clip.data().samples()
        {
            clip.commit_rendered(data);
        }
        self.editor_fx_discard();
        self.editor_hot_swap();
    }

    /// Throw the chain away and put the committed clip back on the wire.
    pub(crate) fn editor_fx_cancel(&mut self) {
        self.editor_fx_discard();
        self.editor_hot_swap();
    }

    /// Drop the audition, its cache **and the chain** without touching the clip
    /// (used by Apply/Cancel/Load, and by edits that would invalidate them). Does
    /// not restore the preview — callers that need the old sound back hot-swap.
    pub(super) fn editor_fx_discard(&mut self) {
        self.editor.fx_audition = None;
        self.editor.fx_sig = None;
        self.editor.fx_head = None;
        self.editor.fx_head_sig = None;
        self.editor.fx_bypass = false;
        // The chain has just been baked into the clip (Apply) or abandoned
        // (Cancel/Load); re-rendering it would double every effect.
        ph2d_panel_audio_editor::reset_fx_chain();
    }

    /// Mirror the panel's global A/B. Engaged, the dry clip is what sounds, shows and
    /// exports — the chain is untouched and comes straight back when it is released.
    pub(crate) fn editor_fx_set_bypass(&mut self, bypass: bool) {
        if self.editor.fx_bypass != bypass {
            self.editor.fx_bypass = bypass;
            self.editor_hot_swap();
        }
    }

    /// Whether an audition is currently sounding (enables the panel's Cancel/Bypass).
    pub(crate) fn editor_fx_auditioning(&self) -> bool {
        self.editor.fx_audition.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::{AudioFormat, SampleData};

    /// Band-spread, off-centre, stereo-divergent — a filter, a saturator or an M/S
    /// tweak all leave a mark on it.
    fn clip() -> EditClip {
        // 4_000 stereo frames, so a 1_000..2_000 selection sits well inside it.
        let samples: Vec<f32> = (0..8_000)
            .map(|i| {
                let t = i as f32 / 48_000.0;
                0.4 * (t * 220.0).sin() + 0.3 * (t * 9_000.0).sin() + 0.05
            })
            .collect();
        EditClip::new(SampleData::from_interleaved(
            samples,
            AudioFormat::stereo(48_000),
        ))
    }

    /// `kind` at `norms`, enabled.
    fn stage(kind: usize, norms: [f32; MAX_FX_PARAMS]) -> FxStage {
        FxStage {
            kind,
            norms,
            enabled: true,
        }
    }

    /// A tuned Low-Pass (cutoff well down the range) and a tuned Saturate.
    fn low_pass() -> FxStage {
        stage(0, [0.4, 0.25, 0.0, 0.0])
    }
    fn saturate() -> FxStage {
        stage(3, [0.6, 0.0, 0.0, 0.0])
    }

    /// A chain of stages sitting on their neutral points is transparent. Rendering
    /// it must return the audio byte-identical — not "almost", or a rack the user
    /// merely opened would quietly rewrite the clip.
    #[test]
    fn a_neutral_chain_renders_the_audio_untouched() {
        let c = clip();
        let neutral: Vec<FxStage> = (0..8)
            .map(|k| stage(k, crate::audio::fx_params::default_norms(k)))
            .collect();
        assert!(
            !neutral.iter().any(is_audible),
            "neutral stages are audible"
        );
        let out = render_from(&c, &neutral, 0);
        assert_eq!(out.data().samples(), c.data().samples());
        assert_eq!(out.data().frame_count(), c.data().frame_count());
    }

    /// A disabled stage is skipped, however far off neutral it is tuned — that is
    /// what makes the eye toggle a real per-stage A/B and not a cosmetic dim.
    #[test]
    fn a_disabled_stage_is_skipped_however_it_is_tuned() {
        let c = clip();
        let mut off = low_pass();
        off.enabled = false;
        assert!(!is_audible(&off));
        assert_eq!(
            render_from(&c, &[off], 0).data().samples(),
            c.data().samples()
        );
        // ...and enabling it does change the audio, so the test above isn't vacuous.
        assert_ne!(
            render_from(&c, &[low_pass()], 0).data().samples(),
            c.data().samples()
        );
    }

    /// Effects do not commute: filtering before saturating is not the same as
    /// saturating before filtering (the second clips harmonics the first removed).
    /// This is why the chain is ordered and why Up/Down exist.
    #[test]
    fn chain_order_changes_the_result() {
        let c = clip();
        let a = render_from(&c, &[low_pass(), saturate()], 0);
        let b = render_from(&c, &[saturate(), low_pass()], 0);
        assert_ne!(
            a.data().samples(),
            b.data().samples(),
            "swapping two stages must change the audio"
        );
    }

    /// THE cache contract. `editor_fx_update` renders `chain[..sel]` once into
    /// `fx_head` and then only `chain[sel..]` per slider move. That shortcut is only
    /// sound if it is **byte-identical** to rendering the whole chain from the clip.
    /// A drift here would mean the audition sounds one way and Apply commits another.
    #[test]
    fn rendering_from_a_cached_head_matches_a_full_render() {
        let c = clip();
        // A tail effect in the middle grows the buffer, which is where a naive
        // prefix cache would most easily go out of step with a full render.
        let reverb = stage(6, [0.7, 0.5, 0.35, 0.3]);
        let chain = [low_pass(), reverb, saturate()];

        let full = render_from(&c, &chain, 0);
        for sel in 0..chain.len() {
            let mut head = c.clone();
            for st in &chain[..sel] {
                if let Some(next) = render_stage(&head, st) {
                    head = next;
                }
            }
            let cached = render_from(&head, &chain, sel);
            assert_eq!(
                cached.data().samples(),
                full.data().samples(),
                "head cache at sel={sel} drifted from a full render"
            );
        }
    }

    /// Every stage must act on the SAME selection, so the target range has to be
    /// carried forward across the chain. Drop it and stage 1 would silently retarget
    /// the whole clip — the classic "the second effect ignored my selection" bug.
    #[test]
    fn the_target_range_survives_the_whole_chain() {
        let mut c = clip();
        let range = 1_000..2_000;
        c.set_selection(Some(range.clone()));
        let out = render_from(&c, &[low_pass(), saturate()], 0);
        assert_eq!(out.selection(), Some(range.clone()));

        // Outside the selection the audio is untouched, even after two stages.
        // `channel_count()`, not `channels as usize` — `ChannelLayout` is an enum,
        // so the cast yields its discriminant (Stereo = 1) and silently halves every
        // sample offset below.
        let (a, b) = (c.data().samples(), out.data().samples());
        let ch = c.data().format().channel_count();
        assert_eq!(a.len(), b.len(), "a length-preserving chain grew the clip");
        assert_eq!(
            a[..range.start * ch],
            b[..range.start * ch],
            "audio before the selection moved"
        );
        assert_eq!(
            a[range.end * ch..],
            b[range.end * ch..],
            "audio after the selection moved"
        );
        assert_ne!(
            a[range.start * ch..range.end * ch],
            b[range.start * ch..range.end * ch],
            "the chain did not touch the selection at all"
        );
    }
}
