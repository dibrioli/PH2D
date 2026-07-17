//! **Pricing an asset is EXPORT work, and this is where it stops being edit work** (ADR-0125).
//!
//! # The bug
//!
//! Enio, on a 3-minute clip: *"1 seg e meio para mudar ganho"*. A Gain click cost **1562 ms**, and
//! **1549 ms of it (99.2%)** was `editor_publish_platforms` — three `conform`s and three **real
//! encodes of the whole clip**, on the UI thread, eighteen lines after the edit, to redraw a
//! three-line readout. The DSP the user asked for was 25 ms of that.
//!
//! # Why the cache was no defence
//!
//! Every readout here is keyed on the buffer, and **an edit moves the buffer by definition** — that
//! is what an edit *is*. So the cache hit on every frame except the one frame the user was waiting
//! on. The old docstring said it in as many words — *"a cache hit on all but the frame after the
//! buffer actually changed"* — without noticing that the frame in which the buffer changes is the
//! click. A cache is not a budget.
//!
//! # The shape of the answer
//!
//! ADR-0117 said *an edit is an INTERVAL*, and ADR-0124 said the same thing on the time axis: a
//! consumer downstream must be **told** what changed rather than made to rediscover it. This is the
//! third face of it. The consumers here cannot be told anything useful — a re-encode is a re-encode
//! — so they get the two answers that are left:
//!
//! 1. **Do not price what nobody is looking at.** The rows are painted inside a collapsible section
//!    that **ships folded** (`populate_sections`). The default case was paying 1.5 s to compute
//!    three strings that were not on screen.
//! 2. **When someone IS looking, do not price on the UI thread.** [`OffThread`] runs it on a worker
//!    and lands the result whenever it lands.
//!
//! # Why there is no progress bar
//!
//! This is the second consumer of `ph2d_editor_core::progress`, and it deliberately takes only half
//! of it: [`Job`] (the worker + the way back), never `JobQueue` (the bar). The bar's own docs say
//! what it is for — *"user-initiated, seconds-long operations"* — and pricing is neither. It is
//! automatic and it is sub-second, so a bar would flash into the toast column on every nudge of a
//! knob, competing with real messages for a spot the user is supposed to learn.
//!
//! The honest indicator for a readout is **the readout**: it says `…` while it is working, in the
//! place the user is already looking. Which is also the rule that mattered most here — a stale
//! number left on screen after the audio changed would be a *wrong* number presented as a right
//! one, and that is worse than the 1.5 s stall this file exists to delete.

use ph2d_editor::Job;
use std::time::{Duration, Instant};

/// How still the clip must be before a thread is spent pricing it.
///
/// Without this, a knob drag is a **thread per frame**: the rack's audition hands us a new buffer
/// every frame it is dragged, every one of them a new key, and each would spawn a worker to price
/// an intermediate state nobody will ever ship. The user is *mid-gesture*; there is no number they
/// want yet.
///
/// 250 ms is the gap between "still dragging" and "stopped to look" — long enough that a gesture
/// never spawns, short enough that letting go feels like it answered immediately.
const SETTLE: Duration = Duration::from_millis(250);

/// An expensive readout, computed off the UI thread, with the staleness question answered once.
///
/// `K` is what the value depends on (a buffer version, or that plus the codec); `V` is the finished
/// readout. **One state machine, two consumers** (`platforms`, `delivery`) — hand-rolling it twice
/// would be two answers to "is this number current?", and the day they drift the panel would print
/// one readout that is honest and one that is not, with nothing on screen to tell them apart.
pub(crate) struct OffThread<K, V> {
    /// The value we have, and the key it describes. **Never published for a different key** — that
    /// is the whole point of keeping them married.
    have: Option<(K, V)>,
    /// In flight: what is being computed, and the worker computing it.
    job: Option<(K, Job<V>)>,
    /// The key we last saw and when we first saw it — [`SETTLE`]'s clock.
    seen: Option<(K, Instant)>,
    /// A key whose worker **died**. Without this, a panicking worker is respawned every [`SETTLE`]
    /// for the rest of the session: a bug that cost one thread becomes a thread storm, and the only
    /// symptom is a readout that never fills in. (`cost` returns its failures as values, so this
    /// should stay unreachable — which is exactly why it must not be silent if it is not.)
    poisoned: Option<K>,
}

// Hand-written: `#[derive(Default)]` would demand `K: Default + V: Default`, which neither is.
impl<K, V> Default for OffThread<K, V> {
    fn default() -> Self {
        Self {
            have: None,
            job: None,
            seen: None,
            poisoned: None,
        }
    }
}

impl<K: Copy + PartialEq, V: Send + 'static> OffThread<K, V> {
    /// Land a finished worker's result. Call once per frame, **before** [`Self::current`].
    ///
    /// Never blocks: `try_take` asks and does not wait (that is `Job`'s whole contract, and this is
    /// the frame loop).
    fn land(&mut self) {
        let Some((key, job)) = self.job.as_mut() else {
            return;
        };
        if !job.is_finished() {
            return;
        }
        let key = *key;
        let taken = job.try_take();
        self.job = None;
        match taken {
            Some(v) => self.have = Some((key, v)),
            // The worker panicked. It has already said so on stderr; do not ask it again.
            None => self.poisoned = Some(key),
        }
    }

    /// The value **for this exact key**, or `None` while it is not known.
    ///
    /// `None` is not a failure — it means *"do not print a number"*. The caller answers it with
    /// `…`, never with the last value it happened to have: the last value describes audio the user
    /// has since changed.
    ///
    /// Spawns `work` on a worker once `key` has been still for [`SETTLE`] and nothing else is in
    /// flight. At most one worker exists at a time, so a long drag cannot pile them up.
    pub(crate) fn current(
        &mut self,
        key: K,
        label: &'static str,
        work: impl FnOnce() -> V + Send + 'static,
    ) -> Option<&V> {
        self.land();

        // Do we already have this one? Then it is fresh, and the clock is irrelevant.
        if self.have.as_ref().is_some_and(|(k, _)| *k == key) {
            self.seen = None;
            return self.have.as_ref().map(|(_, v)| v);
        }
        // Already being computed. Wait — do not spawn a second worker for the same answer.
        if self.job.as_ref().is_some_and(|(k, _)| *k == key) {
            return None;
        }
        // A worker already died on this one. Asking again would just kill another.
        if self.poisoned == Some(key) {
            return None;
        }

        // The key moved. Start (or continue) the settle clock.
        let now = Instant::now();
        let settled = match self.seen {
            Some((k, at)) if k == key => now.duration_since(at) >= SETTLE,
            _ => {
                self.seen = Some((key, now));
                false
            }
        };
        // `job.is_none()`: a worker on a now-stale key still has to finish before the next one
        // starts. `seen` is deliberately NOT cleared while we wait for it — the key stays settled,
        // so the moment the old worker lands, this one goes out on the very next frame.
        if settled && self.job.is_none() {
            self.job = Some((key, Job::spawn(label, move |_| work())));
            self.seen = None;
        }
        None
    }

    /// Forget everything. The clip was closed or swapped — a price is about a clip, and there is no
    /// clip. (An in-flight worker is dropped, not joined: its result is about audio that is gone,
    /// and blocking the UI thread to collect an answer nobody wants is the bug this file is
    /// deleting, in miniature.)
    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests;
