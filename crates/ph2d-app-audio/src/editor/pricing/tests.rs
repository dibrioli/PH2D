//! Gates for [`OffThread`] — the state machine that decides **whether an edit frame pays for a
//! price**, and whether the number on screen is allowed to be shown.
//!
//! The two halves, deliberately paired (`feedback_absence_gate_needs_a_presence_sibling`): "the
//! edit frame does not compute" is green on a machine that never computes anything, so every
//! absence gate here has a presence sibling that watches the same counter go **up**.

use super::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A key that is cheap to move — stands in for a `BufferVersion`.
type K = u32;

/// A work function that **counts how many times it actually ran**.
///
/// The counter is the whole apparatus: "did the UI thread pay for an encode?" is not a question you
/// can answer by reading the source, and the audit that found this bug found it by measuring. So
/// the gates measure too — they just measure a cheap stand-in instead of a 941 ms one.
fn counted(runs: &Arc<AtomicUsize>) -> impl FnOnce() -> u64 + Send + 'static {
    let runs = runs.clone();
    move || {
        runs.fetch_add(1, Ordering::SeqCst);
        42
    }
}

/// Spin until the in-flight worker has landed, or give up. Jobs are threads; a test that asserted
/// on the very next call would be asserting on the scheduler.
fn settle<V: Send + 'static + Copy>(
    o: &mut OffThread<K, V>,
    key: K,
    work: impl Fn() -> V + Send + Sync + 'static + Clone,
) -> Option<V> {
    for _ in 0..2_000 {
        if let Some(v) = o.current(key, "test", work.clone()) {
            return Some(*v);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    None
}

/// **THE gate: the frame that moves the key does not do the work.**
///
/// This is the bug, in one assertion. `editor_apply(Gain)` moves the buffer version, and eighteen
/// lines later the old code re-encoded the whole clip three times, on this thread, before the frame
/// could end. Whatever else is true, the call that first sees a new key must come back having spent
/// nothing.
///
/// Mutation-tested: delete the `SETTLE` check (spawn on first sight) and this still passes — the
/// work would be on a *worker*, which is the point. Make `current` compute inline instead of
/// spawning, and it goes red immediately.
#[test]
fn the_frame_that_sees_a_new_key_does_not_do_the_work() {
    let runs = Arc::new(AtomicUsize::new(0));
    let mut o: OffThread<K, u64> = OffThread::default();

    let v = o.current(1, "test", counted(&runs));

    assert!(
        v.is_none(),
        "a key never seen before cannot already have a value -- and if it does, it was computed \
         on this thread, which is the entire bug"
    );
    assert_eq!(
        runs.load(Ordering::SeqCst),
        0,
        "the edit frame ran the pricing work. On the real clip that is 1549 ms of UI thread \
         (ADR-0125) to redraw three strings."
    );
}

/// **The presence sibling: it DOES compute, and it hands back the answer.**
///
/// Without this, the gate above is satisfied by an `OffThread` that returns `None` for ever — a
/// readout that is permanently `…` is fast and useless, and every absence gate in the file would
/// still be green.
#[test]
fn a_key_that_holds_still_is_eventually_priced_and_the_value_comes_back() {
    let runs = Arc::new(AtomicUsize::new(0));
    let mut o: OffThread<K, u64> = OffThread::default();
    let r = runs.clone();

    let v = settle(&mut o, 1, move || {
        r.fetch_add(1, Ordering::SeqCst);
        42
    });

    assert_eq!(v, Some(42), "the worker's result never came home");
    assert_eq!(
        runs.load(Ordering::SeqCst),
        1,
        "the work must run exactly once for one settled key"
    );
}

/// **A value is never published under a key it does not describe.**
///
/// The staleness rule, which matters more than the speed: after an edit, the price we hold is of
/// the audio *before* the edit. Returning it would put a wrong number on screen dressed as a right
/// one — and unlike a slow readout, nobody would ever notice.
#[test]
fn a_stale_value_is_never_handed_out_for_the_new_key() {
    let mut o: OffThread<K, u64> = OffThread::default();
    assert_eq!(settle(&mut o, 1, || 42), Some(42));

    // The clip is edited: same readout, new audio.
    assert!(
        o.current(2, "test", || 99).is_none(),
        "the price of the PREVIOUS buffer was published for the new one -- the panel would print \
         a byte count for audio that no longer exists, and it would look perfectly reasonable"
    );
}

/// **A key that keeps moving never spawns a worker.**
///
/// A knob drag hands the rack's audition a new buffer every frame. Without the settle clock that is
/// a thread per frame, each pricing an intermediate state nobody will ship — and the machine would
/// be busier than the 1.5 s stall this replaced.
#[test]
fn a_key_that_keeps_moving_never_spawns_anything() {
    let runs = Arc::new(AtomicUsize::new(0));
    let mut o: OffThread<K, u64> = OffThread::default();

    // 60 "frames" of a drag, well past SETTLE in wall-clock, each with a different buffer.
    for frame in 0..60u32 {
        assert!(o.current(frame, "test", counted(&runs)).is_none());
        std::thread::sleep(Duration::from_millis(10));
    }

    assert_eq!(
        runs.load(Ordering::SeqCst),
        0,
        "a drag spawned {} workers -- SETTLE exists so that a gesture in progress prices nothing",
        runs.load(Ordering::SeqCst)
    );
}

/// ...and when the drag **stops**, the price arrives. The presence sibling of the debounce: a
/// settle clock that never fires is just a broken readout.
#[test]
fn when_the_drag_stops_the_price_arrives() {
    let runs = Arc::new(AtomicUsize::new(0));
    let mut o: OffThread<K, u64> = OffThread::default();
    for frame in 0..10u32 {
        o.current(frame, "test", counted(&runs));
        std::thread::sleep(Duration::from_millis(5));
    }
    // Pen up: the key holds still.
    let r = runs.clone();
    let v = settle(&mut o, 9, move || {
        r.fetch_add(1, Ordering::SeqCst);
        7
    });
    assert_eq!(v, Some(7));
    assert_eq!(
        runs.load(Ordering::SeqCst),
        1,
        "letting go of the knob must price the clip exactly once"
    );
}

/// **At most one worker at a time.** Two workers on one readout is two encodes of one clip, and the
/// loser's answer is thrown away — the cost is real and the benefit is zero.
///
/// Note the shape: the *first* `current` on a key only starts the settle clock, it never spawns. So
/// reaching the state this gate is about takes two calls per key, with a `SETTLE` between — which
/// is exactly what the frame loop does for free.
#[test]
fn only_one_worker_is_ever_in_flight() {
    let runs = Arc::new(AtomicUsize::new(0));
    let mut o: OffThread<K, u64> = OffThread::default();
    let r = runs.clone();
    // Slower than SETTLE, deliberately: worker 1 has to be *still running* when key 2 comes due, or
    // the gate would pass by accident on a worker that had already landed.
    let slow = move || {
        r.fetch_add(1, Ordering::SeqCst);
        std::thread::sleep(SETTLE * 3);
        1
    };

    o.current(1, "test", slow.clone()); // starts key 1's clock
    std::thread::sleep(SETTLE);
    o.current(1, "test", slow.clone()); // settled -> worker 1 goes out

    // **Wait for the worker to actually reach its first line before counting it.** `spawn` returns
    // as soon as the thread exists, not once it has run — asserting here without this is asserting
    // on the scheduler, which is how the last async gate on this line managed to be a race itself
    // (`38b8d207`).
    for _ in 0..2_000 {
        if runs.load(Ordering::SeqCst) >= 1 {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(runs.load(Ordering::SeqCst), 1, "worker 1 never went out");

    o.current(2, "test", slow.clone()); // the clip was edited: starts key 2's clock
    std::thread::sleep(SETTLE);
    o.current(2, "test", slow.clone()); // settled -- but worker 1 is still running

    assert_eq!(
        runs.load(Ordering::SeqCst),
        1,
        "a second worker went out while the first was still running -- during a long edit session \
         that is a thread per settled key, all of them encoding the same clip"
    );
}

/// **A worker that dies is not resurrected on a loop.**
///
/// `cost` returns its failures as values, so this should stay unreachable — which is exactly why it
/// must not be silent if it is not. Without the poison, one panicking worker becomes a fresh thread
/// every 250 ms for the rest of the session, and the only visible symptom is a readout stuck on `…`.
#[test]
fn a_panicking_worker_is_not_retried_for_ever() {
    let runs = Arc::new(AtomicUsize::new(0));
    let mut o: OffThread<K, u64> = OffThread::default();
    let r = runs.clone();
    let boom = move || -> u64 {
        r.fetch_add(1, Ordering::SeqCst);
        panic!("the worker died");
    };
    std::thread::sleep(SETTLE);
    o.current(1, "test", boom.clone());
    // Let it die, then keep asking for a good while.
    for _ in 0..40 {
        o.current(1, "test", boom.clone());
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        runs.load(Ordering::SeqCst),
        1,
        "the dead worker was respawned {} times",
        runs.load(Ordering::SeqCst)
    );
}

/// Closing the clip forgets the price. A price is *about* a clip; there is no clip.
#[test]
fn clearing_forgets_the_value() {
    let mut o: OffThread<K, u64> = OffThread::default();
    assert_eq!(settle(&mut o, 1, || 42), Some(42));
    o.clear();
    assert!(
        o.current(1, "test", || 42).is_none(),
        "a cleared cache still answered for the clip it was told to forget"
    );
}
