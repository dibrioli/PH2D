//! Gates for the long-operation pattern.
//!
//! The load-bearing claim is not "a thread runs" — it is **"the UI thread keeps its frame"**.
//! So the tests below are written against the two things a frame does: ask for a number to
//! paint, and ask whether the result is here yet. Neither may block.
//!
//! **No test here sleeps.** Every wait is on a *condition* (a channel recv, or a bounded spin on
//! the real predicate), so a slow machine takes more turns of a loop rather than failing.

use super::*;
use ph2d_a11y::Role;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Wait for `try_take` to yield, then return the result.
///
/// This is not a sleep and not a guess: it polls the real predicate and gives up only on a
/// deadline that exists so a genuine hang **fails** instead of hanging CI for ever. On a
/// machine 100× slower this test takes 100× more loop turns and still passes.
fn await_result<T: Send + 'static>(job: &mut Job<T>) -> T {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(v) = job.try_take() {
            return v;
        }
        assert!(
            Instant::now() < deadline,
            "the job never produced a result — it is hung, or `try_take` never yields"
        );
        thread::yield_now();
    }
}

/// **The point of the whole module.** The work runs somewhere else, the UI can read the bar
/// while it runs, `try_take` does not block, and the result comes back.
///
/// Deterministic without a sleep: the worker is held on a channel the test controls, so "the
/// worker has started but has not finished" is a state the test *creates* rather than races for.
///
/// **The order inside the worker is the whole trick, and it is not cosmetic.** `set` happens
/// *before* the "I started" send, because the channel is what publishes the write: `send`/`recv`
/// is the happens-before edge, so by the time the test is unblocked the 0.5 is provably visible.
/// Announcing first and reporting after is a **race** — the test would read the fraction while the
/// worker had only reached the announcement, and 0.0 would come back. It did, on ~1 run in 3.
#[test]
fn the_work_leaves_the_ui_thread_and_the_result_comes_back() {
    let ui = thread::current().id();
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel::<()>();

    let mut job = Job::spawn("Work", move |p| {
        p.set(0.5);
        started_tx.send(thread::current().id()).expect("test alive");
        release_rx.recv().expect("test releases the worker");
        7_u32
    });

    // Blocks until the worker really ran. No sleep, no guess.
    let worker = started_rx.recv().expect("worker starts");
    assert_ne!(
        worker, ui,
        "the work must not run on the thread that paints — a bar that cannot be painted \
         while the work runs is worse than no bar at all"
    );

    // Mid-flight: the UI can read the bar, and asking for the result costs it nothing.
    assert_eq!(job.progress().fraction(), 0.5);
    assert!(
        job.try_take().is_none(),
        "try_take must not block on an unfinished job"
    );
    assert!(!job.is_finished());

    release_tx.send(()).expect("worker alive");
    assert_eq!(await_result(&mut job), 7);
}

/// A result is taken **once**. After that the job is spent — the caller that keeps a `Job`
/// around must not be handed the same edit twice.
#[test]
fn a_result_is_taken_once() {
    let mut job = Job::spawn("Work", |_| 1_u8);
    assert_eq!(await_result(&mut job), 1);
    assert!(job.try_take().is_none());
    assert!(job.is_finished(), "a spent job must not read as running");
}

/// **A panicking worker still takes its bar down.**
///
/// `denoise_ml` panics on a model-init failure, so this is a reachable state, not a theory.
/// Without the drop guard the bar would sit on screen for ever at whatever fraction the worker
/// died at — a widget insisting that work is in progress on a thread that no longer exists.
#[test]
fn a_panicking_worker_still_takes_its_bar_down() {
    let mut q = JobQueue::new();
    // The panic message on the worker's own stderr is expected output of this test.
    let mut job = Job::spawn("Doomed", |p| {
        p.set(0.5);
        panic!("the model failed to initialise");
    });
    q.push(job.progress().clone());
    assert_eq!(q.len(), 1);

    let deadline = Instant::now() + Duration::from_secs(10);
    while !job.is_finished() {
        assert!(Instant::now() < deadline, "worker never unwound");
        thread::yield_now();
    }
    // No result — but the caller is told it is over, and the bar goes.
    let taken: Option<()> = job.try_take();
    assert!(taken.is_none());
    q.tick();
    assert!(
        q.is_empty(),
        "a bar whose worker panicked must not outlive it"
    );
}

/// Every clone is the same progress. This is why the bar cannot drift from the work: there is
/// nothing to drift *from* — one number, two readers.
#[test]
fn clones_share_one_number() {
    let a = Progress::new("Work");
    let b = a.clone();
    a.set(0.25);
    assert_eq!(b.fraction(), 0.25);
    b.set(0.75);
    assert_eq!(a.fraction(), 0.75);
}

/// Garbage in never becomes a scene the painter cannot draw. `i / n` with `n == 0` is NaN, and a
/// bar of width NaN is a hole in the frame.
#[test]
fn out_of_range_and_nan_are_clamped_not_trusted() {
    let p = Progress::new("Work");
    p.set(2.0);
    assert_eq!(p.fraction(), 1.0);
    p.set(-1.0);
    assert_eq!(p.fraction(), 0.0);
    p.set(f32::NAN);
    assert_eq!(p.fraction(), 0.0, "NaN must land at the low end, not paint");
    p.set(f32::INFINITY);
    assert_eq!(p.fraction(), 1.0);
}

/// `tick` drops what is finished and keeps what is not — the mirror of `ToastQueue::tick`.
#[test]
fn tick_drops_the_finished_and_keeps_the_running() {
    let mut q = JobQueue::new();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let mut running = Job::spawn("Running", move |_| release_rx.recv().expect("released"));
    let mut done = Job::spawn("Done", |_| ());
    q.push(running.progress().clone());
    q.push(done.progress().clone());
    assert_eq!(q.len(), 2);

    await_result(&mut done);
    q.tick();
    assert_eq!(q.len(), 1, "the finished bar goes");
    assert_eq!(
        q.iter().next().map(Progress::label),
        Some("Running"),
        "and the one still working stays — a job that finishes out of order must not take a \
         neighbour's bar with it"
    );

    release_tx.send(()).expect("worker alive");
    await_result(&mut running);
    q.tick();
    assert!(q.is_empty());
}

/// Full queue drops silently — and the *work* is untouched. Same backpressure as `ToastQueue`.
#[test]
fn a_full_queue_drops_the_bar_not_the_work() {
    let mut q = JobQueue::with_cap(1);
    let a = Progress::new("a");
    let b = Progress::new("b");
    assert!(q.push(a));
    assert!(!q.push(b.clone()), "over cap, so the bar is dropped");
    assert_eq!(q.len(), 1);
    // The dropped one is still a perfectly live handle: its worker never knew.
    b.set(0.5);
    assert_eq!(b.fraction(), 0.5);
}

/// Paint the queue and report how many paths the scene actually encodes.
///
/// **Counting, not merely surviving.** A paint test that only proves "it did not panic" is green
/// against `fn paint() {}` — the bar could be deleted outright and nothing here would notice
/// ([[feedback_painted_is_not_populated_paint_gate]]). `Scene::encoding()` says what was really
/// emitted, so the assertions below can be about pixels rather than about the absence of a crash.
fn painted_paths(q: &JobQueue, rows_above: usize) -> u32 {
    let mut scene = VectorScene::new();
    let mut text = ph2d_text::TextSystem::without_system_fonts();
    let mut ctx = PaintCtx {
        theme: ph2d_tokens::Theme::Forge,
        viewport: Rect::new(0.0, 0.0, 1920.0, 1080.0),
        text: &mut text,
    };
    q.paint_below(rows_above, &mut scene, &mut ctx);
    scene.inner().encoding().n_paths
}

/// **The bar is really drawn, and an empty queue really draws nothing.**
///
/// The pair matters: "a full queue paints something" alone would pass if the painter drew the
/// card unconditionally, and "an empty queue paints nothing" alone would pass if the painter
/// drew nothing ever.
#[test]
fn a_bar_paints_and_an_empty_queue_does_not() {
    let mut q = JobQueue::new();
    assert_eq!(painted_paths(&q, 0), 0, "an empty queue painted something");

    q.push(Progress::new("AI Denoise"));
    let one = painted_paths(&q, 0);
    assert!(one > 0, "a queued job painted no bar at all");

    q.push(Progress::new("Export"));
    assert!(
        painted_paths(&q, 0) > one,
        "the second bar added nothing to the scene"
    );
}

/// **The fill tracks the fraction.** A bar whose geometry does not change with its number is a
/// picture of a bar. `paint_progress_bar` skips a zero-width fill, so the path count is the
/// cheapest true statement about it: a running job draws strictly more than a fresh one.
#[test]
fn the_fill_appears_once_there_is_progress_to_show() {
    let p = Progress::new("AI Denoise");
    let mut q = JobQueue::new();
    q.push(p.clone());

    let at_zero = painted_paths(&q, 0);
    p.set(0.5);
    assert!(
        painted_paths(&q, 0) > at_zero,
        "progress moved from 0 to 50 % and the scene did not change — the bar is a picture"
    );
}

/// The column is a stack, not a pile: no two rows overlap, whoever the tenants are.
#[test]
fn column_rows_never_overlap() {
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    let rows: Vec<Rect> = (0..8).map(|i| column_row(viewport, i)).collect();
    for pair in rows.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        assert!(
            b.y >= a.y + a.h,
            "row at y={} overlaps the row above it (y={}, h={})",
            b.y,
            a.y,
            a.h
        );
        assert_eq!(a.x, b.x, "the column is a column");
        assert_eq!(a.w, b.w);
    }
}

/// The column is centred on the viewport, not on the window origin — a second monitor or a
/// letterboxed viewport must not push the stack off to one side.
#[test]
fn the_column_is_centred_on_the_viewport() {
    let viewport = Rect::new(100.0, 50.0, 800.0, 600.0);
    let r = column_row(viewport, 0);
    let left = r.x - viewport.x;
    let right = (viewport.x + viewport.w) - (r.x + r.w);
    assert!((left - right).abs() < f32::EPSILON, "off-centre column");
    assert!(
        r.y > viewport.y,
        "the first row sits below the viewport top"
    );
}

/// The a11y node is a real progress indicator carrying a real value — not a label that happens
/// to contain a number. A screen reader has to be able to *report* the value, which means the
/// value must be in the value field.
///
/// The exactness is deliberately loose: the fraction round-trips through `u32` fixed point
/// (`0.42` comes back as `0.41999998…`), which is four orders of magnitude finer than the
/// pixel it draws. Asserting on the bits would be asserting on the storage, not the contract.
#[test]
fn the_a11y_node_is_a_progress_indicator_with_a_value() {
    let p = Progress::new("AI Denoise");
    p.set(0.42);
    let n = p.build_a11y(Rect::new(0.0, 0.0, 100.0, 12.0));
    assert_eq!(n.role(), Role::ProgressIndicator);
    assert_eq!(n.label(), Some("AI Denoise"));
    let v = n
        .numeric_value()
        .expect("a determinate bar publishes its value");
    assert!((v - 0.42).abs() < 1e-5, "value was {v}");
    assert_eq!(n.min_numeric_value(), Some(0.0));
    assert_eq!(n.max_numeric_value(), Some(1.0));
    assert_eq!(
        n.bounds().map(|b| (b.x1 - b.x0, b.y1 - b.y0)),
        Some((100.0, 12.0)),
        "the tree must describe where the bar actually is"
    );
}

/// **The a11y node and the paint describe the same rectangle.** Two rects computed two ways
/// drift, and a screen reader pointing a magnifier at empty canvas is a silent bug — nothing
/// on screen looks wrong.
#[test]
fn the_a11y_bounds_are_the_track_that_gets_painted() {
    let viewport = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    let mut q = JobQueue::new();
    q.push(Progress::new("Work"));
    let nodes = q.build_a11y(viewport, 2);
    let want = track_rect(column_row(viewport, 2));
    let b = nodes[0].bounds().expect("bounded");
    assert_eq!((b.x0 as f32, b.y0 as f32), (want.x, want.y));
    assert_eq!((b.x1 - b.x0) as f32, want.w);
    assert!(
        want.y + want.h <= column_row(viewport, 2).y + column_row(viewport, 2).h,
        "the track must sit inside its own card"
    );
}
