//! [`Progress`] · [`Job`] · [`JobQueue`] — the app's pattern for work that outlives a frame.
//!
//! ## Why this is infrastructure, and not an audio feature
//!
//! The first caller is the audio editor's AI Denoise (a 3-minute take is ~5 s of inference),
//! but nothing here knows that. Batch LUFS, export, upscale and the painter's whole-layer
//! effects have the identical shape: **the user clicks, the main thread disappears for
//! seconds, and the window stops answering**. Each of them inventing its own thread + its own
//! bar would give the app four progress idioms and four sets of bugs, so this lands as the one
//! pattern they all copy.
//!
//! ## The one thing that makes it work: the UI thread never does the work
//!
//! A bar that does not move is worse than no bar — it reads as a freeze *plus* a broken widget.
//! And the bar cannot move while the thread that paints it is busy computing. So the whole
//! design falls out of one rule: **the work leaves the UI thread, and what comes back is a
//! number the painter can read for free**.
//!
//! [`Progress`] is that number. It is an `Arc` the worker and the painter *share* — the worker
//! stores into it, the painter loads from it, and there is exactly one of them. The obvious
//! alternative (the worker sends progress events down a channel, the UI accumulates them into
//! its own copy) has two copies of the same fact, and two copies drift: drop an event, or drain
//! the channel on a frame that never paints, and the bar is telling a story the work is not
//! living. One `Arc`, no drift.
//!
//! ## The shape of the seam
//!
//! ```ignore
//! // Shell, on the click:
//! let job = Job::spawn("AI Denoise", move |p| ph2d_audio_ml::denoise_ml_with_progress(&clip, amt, &|f| p.set(f)));
//! gfx.jobs.push(job.progress().clone());   // the bar
//! self.ml_job = Some(job);                 // the result
//!
//! // Shell, every frame:
//! if let Some(job) = self.ml_job.as_mut() && job.is_finished() {
//!     if let Some(out) = job.try_take() { install(out) }
//!     self.ml_job = None;
//! }
//! ```
//!
//! Note what the worker closure does **not** take: a `Progress`. It takes a `&dyn Fn(f32)` at
//! the far end (see `ph2d_audio_ml::denoise_ml_with_progress`), so the crate doing the work
//! never depends on this one. A DSP crate that has to link the editor's UI in order to say
//! "42 %" is a DSP crate that can no longer be used without the editor.

use crate::paint::{Paint, PaintCtx, fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use crate::widget::{ProgressBar, paint_progress_bar};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeId};
use ph2d_tokens::{ColorToken, Radius, Spacing, TypeToken};
use ph2d_vector::VectorScene;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::thread::JoinHandle;

/// Fixed-point denominator for the published fraction.
///
/// There is no `AtomicF32` in `std`, and there is no need for one: a bar is ~360 px wide, so a
/// millionth of the range is four orders of magnitude finer than a pixel. Storing the fraction
/// as `u32` parts-per-million keeps the publish a single relaxed store.
const SCALE: u32 = 1_000_000;

// ---------------------------------------------------------------------------
// The shared column (this module owns the ruler — see `column_row`)
// ---------------------------------------------------------------------------

/// Width of a card in the top-center notification column.
// LITERAL-PX-OK: the toast card's established width — this is the same column, and the two
// tenants must agree to the pixel. Not a design token (no token describes "notification card").
const COLUMN_W: f32 = 360.0;
/// Height of one row in the column.
// LITERAL-PX-OK: as COLUMN_W — the toast row height, preserved exactly so adding this second
// tenant moves no existing pixel.
const ROW_H: f32 = 44.0;

/// Geometry of row `index` of the **top-center notification column**.
///
/// This is the one ruler for a column with two tenants: the toast stream and these job bars.
/// It lives here, rather than beside the toast painter in `paint.rs`, for a boring reason —
/// `paint.rs` sits at its frozen LOC ceiling (`architecture_workspace_file_loc_cap`) and may
/// shrink but never grow. The tenant that arrived second brought the shared ruler with it, and
/// the toast painter now measures against it too, so the two cannot drift apart.
///
/// **Rows are uniform.** A column whose rows are different heights needs every tenant to know
/// every other tenant's height in order to place itself; one height means a row index is all
/// anyone needs.
pub(crate) fn column_row(viewport: Rect, index: usize) -> Rect {
    let gap = Spacing::Md.px();
    Rect {
        x: viewport.x + (viewport.w - COLUMN_W) / 2.0,
        y: viewport.y + Spacing::Xl.px() + (ROW_H + gap) * index as f32,
        w: COLUMN_W,
        h: ROW_H,
    }
}

// ---------------------------------------------------------------------------
// Progress — the shared number
// ---------------------------------------------------------------------------

struct ProgressInner {
    /// Fraction done, in parts of [`SCALE`]. Clamped to `0..=SCALE` on write.
    frac: AtomicU32,
    /// Set once, when the worker stops — by a drop guard, so a panicking worker still
    /// takes its bar off the screen.
    done: AtomicBool,
    /// Immutable for the life of the job. A label that changed mid-flight would have to be a
    /// lock or a second channel, and "AI Denoise" says everything the user needs while it runs.
    label: String,
}

/// A cloneable, thread-safe handle on one long operation's progress.
///
/// Clone it freely: every clone is the *same* progress (an `Arc`), which is the entire point —
/// the worker writes through its clone and the painter reads through another, and there is no
/// third copy to fall out of step.
#[derive(Clone)]
pub struct Progress {
    inner: Arc<ProgressInner>,
}

impl Progress {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(ProgressInner {
                frac: AtomicU32::new(0),
                done: AtomicBool::new(false),
                label: label.into(),
            }),
        }
    }

    /// What the bar is called. Shown to the user, so it is a sentence, not an id.
    pub fn label(&self) -> &str {
        &self.inner.label
    }

    /// **Worker side**: publish how far along the work is, `0.0..=1.0`. Out-of-range input is
    /// clamped rather than rejected — a caller that computes `i / n` with `n == 0` gets a NaN,
    /// and a NaN that reached the painter would be a bar of width NaN. `clamp` maps NaN to the
    /// low end, so the worst case is a bar that does not move, not a scene that cannot paint.
    ///
    /// `Relaxed` is deliberate: the fraction guards nothing. Publishing it neither hands the
    /// reader ownership of anything nor gates any other memory — it is a number to draw. The
    /// *result* travels through the join handle, which carries its own happens-before.
    pub fn set(&self, fraction: f32) {
        let f = (fraction.clamp(0.0, 1.0) * SCALE as f32) as u32;
        self.inner.frac.store(f.min(SCALE), Ordering::Relaxed);
    }

    /// **Painter side**: how far along, `0.0..=1.0`.
    pub fn fraction(&self) -> f32 {
        self.inner.frac.load(Ordering::Relaxed) as f32 / SCALE as f32
    }

    /// Has the worker stopped (returned *or* panicked)?
    pub fn is_done(&self) -> bool {
        self.inner.done.load(Ordering::Relaxed)
    }

    fn finish(&self) {
        self.inner.done.store(true, Ordering::Relaxed);
    }

    /// This progress as the design system's [`ProgressBar`] — the widget is what gets painted
    /// and what describes itself to a screen reader. See [`paint_bar`] for why the bar is
    /// borrowed rather than reinvented.
    fn widget(&self) -> ProgressBar {
        // NodeId(0): the bar is output-only — nothing hit-tests it, nothing dispatches to it, so
        // it has no identity to collide with. (`ToastQueue` makes the same call by having no id
        // at all.)
        ProgressBar::new(NodeId(0), self.label()).determinate(self.fraction())
    }

    /// The a11y node for this bar, bounded by `rect` — **the rect of the track itself**, not of
    /// the card around it. `Role::ProgressIndicator` carrying a real numeric value, built by the
    /// widget, so a screen reader hears the same thing about a job bar as about any other
    /// progress in the app.
    pub fn build_a11y(&self, rect: Rect) -> Node {
        self.widget()
            .build_a11y(rect.x as f64, rect.y as f64, rect.w as f64, rect.h as f64)
    }
}

/// Sets `done` when the worker's stack unwinds *or* returns.
///
/// Not a line after the closure: `denoise_ml` reaches for `.expect()` in two places (model
/// init, inference), and a panicking worker that left `done` false would leave a bar on screen
/// that could never advance and never leave — the exact "frozen UI" this module exists to
/// prevent, only now with a widget lying about it.
struct FinishOnDrop(Progress);

impl Drop for FinishOnDrop {
    fn drop(&mut self) {
        self.0.finish();
    }
}

// ---------------------------------------------------------------------------
// Job — the work, and the way back
// ---------------------------------------------------------------------------

/// One unit of off-thread work and the result it will hand back.
///
/// The UI never blocks on this. [`Job::try_take`] asks; it does not wait.
pub struct Job<T> {
    progress: Progress,
    /// `None` once the result has been taken (or a panicking worker's has been dropped).
    handle: Option<JoinHandle<T>>,
}

impl<T: Send + 'static> Job<T> {
    /// Run `f` on a new thread. `f` is handed the job's [`Progress`] and should call
    /// [`Progress::set`] as it goes — a job that never reports is a bar pinned at 0 %.
    ///
    /// One thread per job, not a pool: these are seconds-long, user-initiated, and rare (a
    /// person clicks one button at a time). A pool would add a queue, a policy for what happens
    /// when it is full, and a shutdown story — all to save a `clone` and a `mmap` that the work
    /// itself dwarfs by four orders of magnitude.
    pub fn spawn(
        label: impl Into<String>,
        f: impl FnOnce(&Progress) -> T + Send + 'static,
    ) -> Self {
        let progress = Progress::new(label);
        let worker = progress.clone();
        let handle = std::thread::spawn(move || {
            let guard = FinishOnDrop(worker);
            f(&guard.0)
        });
        Self {
            progress,
            handle: Some(handle),
        }
    }

    /// The shared progress. Clone it into a [`JobQueue`] to get a bar.
    pub fn progress(&self) -> &Progress {
        &self.progress
    }

    /// Has the worker stopped? True once it returned **or panicked** — the two are the same
    /// question to a caller that has to decide whether to keep polling.
    pub fn is_finished(&self) -> bool {
        self.handle.as_ref().is_none_or(JoinHandle::is_finished)
    }

    /// Take the result if it is ready. `None` while the work runs — and `None` *for ever* after
    /// the result was taken, or if the worker panicked.
    ///
    /// **This never blocks**, which is the whole contract: it is called from the frame loop.
    /// The `join` below only runs once `is_finished` is true, where it returns immediately.
    ///
    /// Pair it with [`Job::is_finished`] to decide when to drop the job, or a panicking worker
    /// leaves the caller polling a `None` for ever — with, in the audio case, the buttons that
    /// the job disabled still disabled.
    pub fn try_take(&mut self) -> Option<T> {
        if !self.handle.as_ref()?.is_finished() {
            return None;
        }
        self.handle.take()?.join().ok()
    }
}

// ---------------------------------------------------------------------------
// JobQueue — the bars
// ---------------------------------------------------------------------------

/// The bars the app is currently showing.
///
/// It holds [`Progress`] handles and **not** `Job<T>` values, which is not a compromise but the
/// point: `Job<T>` is generic over what the work returns, so a queue of jobs could only ever
/// hold one kind of work. Progress is the part every job has in common. The typed job stays
/// with whoever knows what to do with the result; the bar comes here.
///
/// Sibling of [`crate::toast::ToastQueue`], deliberately: same push-with-cap, same
/// once-per-frame `tick` that drops what is finished, same silent drop when full.
#[derive(Default)]
pub struct JobQueue {
    inner: VecDeque<Progress>,
    cap: usize,
}

impl JobQueue {
    /// Bars past this are dropped. Far below the toast cap of 32 on purpose: these are
    /// user-initiated, seconds-long operations. Eight at once is already a screenful of column,
    /// and a *ninth* means something is spawning jobs in a loop — which the drop makes visible
    /// as a missing bar instead of hiding as a wall of them.
    pub const DEFAULT_CAP: usize = 8;

    pub fn new() -> Self {
        Self::with_cap(Self::DEFAULT_CAP)
    }

    pub fn with_cap(cap: usize) -> Self {
        Self {
            inner: VecDeque::new(),
            cap,
        }
    }

    /// Show a bar for this progress. Returns false if the queue is full (silent drop).
    ///
    /// Dropping the *bar* never drops the *work*: the job owns its thread and its result, and
    /// this queue only draws. A full queue costs the user a progress bar, not an edit.
    pub fn push(&mut self, progress: Progress) -> bool {
        if self.inner.len() >= self.cap {
            return false;
        }
        self.inner.push_back(progress);
        true
    }

    /// Drop the bars of finished jobs. Called once per frame by the shell.
    ///
    /// `retain`, not the toast's `pop_front`-while-expired: toasts expire in the order they
    /// arrived, so the front is always the next to go. Jobs finish in whatever order the work
    /// takes, so the one that is done may be anywhere.
    pub fn tick(&mut self) {
        self.inner.retain(|p| !p.is_done());
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Progress> {
        self.inner.iter()
    }

    /// One a11y node per bar, bounded by the track each is actually painted at — the tree has to
    /// describe the screen, so both go through `track_rect`. `rows_above` is what
    /// [`JobQueue::paint_below`] was given.
    pub fn build_a11y(&self, viewport: Rect, rows_above: usize) -> Vec<Node> {
        self.inner
            .iter()
            .enumerate()
            .map(|(i, p)| p.build_a11y(track_rect(column_row(viewport, rows_above + i))))
            .collect()
    }

    /// Paint the bars into the top-center column, starting `rows_above` rows down.
    ///
    /// **The toast stream owns the top of the column and these stack under it.** The order is
    /// not arbitrary: a toast self-destructs after three seconds, so it gets one chance to be
    /// read, and its position must not depend on whether some unrelated background job happens
    /// to be running — a message that moves to a different place for reasons the user cannot
    /// see is a message they cannot learn to find. A bar is persistent and self-announcing (it
    /// is the thing on screen that is *moving*), so it can afford to be the one that shifts.
    pub fn paint_below(&self, rows_above: usize, scene: &mut VectorScene, ctx: &mut PaintCtx) {
        for (i, p) in self.inner.iter().enumerate() {
            paint_bar(p, column_row(ctx.viewport, rows_above + i), scene, ctx);
        }
    }
}

impl Paint for JobQueue {
    /// The column to ourselves. The shell has toasts above it and calls
    /// [`JobQueue::paint_below`] with their row count; this is the same painter with nothing
    /// above — one implementation, so the two can never disagree about what a bar looks like.
    fn paint(&self, scene: &mut VectorScene, ctx: &mut PaintCtx) {
        self.paint_below(0, scene, ctx);
    }
}

/// The track's rect inside a card at `r`. Its own function because the a11y bounds and the
/// paint must describe the *same* rectangle.
fn track_rect(r: Rect) -> Rect {
    let pad = Spacing::Lg.px();
    Rect::new(
        r.x + pad,
        r.y + r.h - Spacing::Md.px() - Spacing::Lg.px(),
        r.w - 2.0 * pad,
        Spacing::Lg.px(),
    )
}

/// One bar: a toast-shaped card carrying a label and the design system's [`ProgressBar`].
///
/// **The bar is borrowed, not drawn here.** `widget::progress_bar` already answers "what does
/// progress look like in this app" — track token, fill token, the pill radius, the percent
/// readout, and the small-value radius clamp that stops 5 % from painting a stadium wider than
/// its own fill. A second hand-rolled track would be a second answer, and the day someone
/// retunes the widget the job bars would quietly keep the old look. The card around it (the
/// elevated surface, the border, the label) is the *column's* chrome and is this module's to
/// own — it is what makes a bar sit in the same stack as a toast.
fn paint_bar(p: &Progress, r: Rect, scene: &mut VectorScene, ctx: &mut PaintCtx) {
    // Same body as a toast — same column, same surface, so the two read as one stack.
    fill_rounded_rect(
        scene,
        r,
        Radius::Md.px(),
        resolve(ColorToken::BgElev, ctx.theme),
    );
    stroke_rounded_rect(
        scene,
        r,
        Radius::Md.px(),
        1.0,
        resolve(ColorToken::Border, ctx.theme),
    );

    paint_text(
        ctx.text,
        scene,
        p.label(),
        r.x + Spacing::Lg.px(),
        r.y + Spacing::Md.px(),
        TypeToken::Xs.px(),
        (r.w - 2.0 * Spacing::Lg.px()).max(0.0),
        resolve(ColorToken::Text1, ctx.theme),
    );

    // `show_percent`: the number is the honesty check on the shape beside it. A track that is
    // moving slowly and a track that is stuck look alike; "3 %" and "3 %" a second later do not.
    paint_progress_bar(
        &p.widget().show_percent(true),
        track_rect(r),
        scene,
        ctx.text,
        ctx.theme,
    );
}

#[cfg(test)]
mod tests;
