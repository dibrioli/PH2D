//! **ADR-0124.** A range edit now rewrites the clip's buffer where it lies instead of rebuilding it.
//! That is an optimisation, and an optimisation is only allowed to change how long something takes.
//!
//! So everything here compares the fast path against the slow one — which is not a mock: holding a
//! second owner of the buffer is exactly what the mixer does while it plays the clip, and the splice
//! it forces is the code the editor ran before this ADR. **The oracle is the old path itself.**
//!
//! The dangerous one is `undo`: an edit that reports the wrong range produces a step that restores
//! the wrong audio, and nothing about that is visible until a user presses Ctrl+Z and their work is
//! quietly different. So undo is checked byte-for-byte against a whole-buffer snapshot oracle, over
//! random edit sequences, with the selection at the start, the middle, the end, and over everything.

use std::ops::Range;

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};
use ph2d_audio_edit::{EditClip, FadeDir, FadeShape, PeakCache, column_peaks};

const SR: usize = 48_000;

fn noise(frames: usize, ch: usize, seed: u32) -> SampleData {
    let mut s = seed.wrapping_mul(2_654_435_761).wrapping_add(1);
    let format = AudioFormat::new(
        SR as u32,
        if ch == 2 {
            ChannelLayout::Stereo
        } else {
            ChannelLayout::Mono
        },
    );
    SampleData::from_fn(frames * ch, format, |_| {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        (s as f32 / u32::MAX as f32) * 2.0 - 1.0
    })
}

/// A named edit that acts on the clip's target range.
type NamedOp = (&'static str, fn(&mut EditClip));

/// A named selection to run one against.
type NamedSelection = (&'static str, Option<Range<usize>>);

/// Every op that acts on the target range, by name.
fn ops() -> Vec<NamedOp> {
    vec![
        ("gain", |c: &mut EditClip| c.apply_gain(0.5)),
        ("gain_up", |c: &mut EditClip| c.apply_gain(1.7)),
        ("invert", |c: &mut EditClip| c.apply_invert()),
        ("reverse", |c: &mut EditClip| c.apply_reverse()),
        ("normalize_peak", |c: &mut EditClip| {
            c.apply_normalize_peak(0.9)
        }),
        ("normalize_lufs", |c: &mut EditClip| {
            c.apply_normalize_lufs(-16.0)
        }),
        ("remove_dc", |c: &mut EditClip| c.apply_remove_dc_offset()),
        ("silence", |c: &mut EditClip| c.apply_silence()),
        ("fade_in", |c: &mut EditClip| {
            c.apply_fade(FadeShape::SCurve, FadeDir::In)
        }),
        ("fade_out", |c: &mut EditClip| {
            c.apply_fade(FadeShape::Linear, FadeDir::Out)
        }),
    ]
}

/// The selections worth testing: the edges are where an off-by-one in the range lands.
fn selections(frames: usize) -> Vec<NamedSelection> {
    vec![
        ("at the start", Some(0..frames / 4)),
        ("in the middle", Some(frames / 3..frames / 2)),
        ("at the end", Some(frames - frames / 4..frames)),
        ("one frame", Some(frames / 2..frames / 2 + 1)),
        ("the whole clip, explicitly", Some(0..frames)),
        ("no selection at all", None),
    ]
}

/// A clip that owns its buffer **alone**.
///
/// `map_in_place`, NOT `data.clone()`: cloning a `SampleData` bumps the `Arc`, it does not copy —
/// so the fixture would still be holding the buffer, `get_mut` would refuse forever, and every
/// comparison below would be the slow path against itself. Green, and proving nothing.
///
/// That is not hypothetical. This file was written with `clone()` here and
/// `a_sole_owner_writes_the_range_where_it_lies` failed immediately — which is the only reason that
/// test is in the suite.
fn clip_with(data: &SampleData, sel: Option<Range<usize>>) -> EditClip {
    let mut c = EditClip::new(SampleData::map_in_place(data, |_| {}));
    c.set_selection(sel);
    c
}

/// Run `f` on a clip **nobody else holds** — the in-place path.
fn fast(data: &SampleData, sel: Option<Range<usize>>, f: fn(&mut EditClip)) -> EditClip {
    let mut c = clip_with(data, sel);
    f(&mut c);
    c
}

/// Run `f` with a **second owner of the buffer alive** — which is what the mixer is while it plays
/// the clip, and which forces the splice the editor used before ADR-0124.
fn slow(data: &SampleData, sel: Option<Range<usize>>, f: fn(&mut EditClip)) -> EditClip {
    let mut c = clip_with(data, sel);
    let keep = c.data().clone(); // an `Arc` bump: `get_mut` must now refuse
    f(&mut c);
    drop(keep);
    c
}

/// A **copy** of a buffer, not a second owner of it.
///
/// `clone()` would bump the `Arc`, and an oracle that holds a clone of the document's buffer stops
/// the document from owning it — every edit after the first would quietly take the splice path and
/// the fast path would go untested. Snapshots must observe the audio without laying a finger on it.
fn copy(d: &SampleData) -> SampleData {
    SampleData::map_in_place(d, |_| {})
}

/// The waveform, read through the cache at bin resolution.
fn waveform(c: &EditClip) -> (Vec<f32>, Vec<f32>) {
    let frames = c.frame_count();
    let cols = (frames / c.peaks().bin_size()).max(1);
    let p = c.column_peaks(0, frames, cols);
    (p.min, p.max)
}

/// **The whole claim.** Same op, same selection, same audio — whichever path ran.
#[test]
fn the_fast_path_and_the_slow_path_are_the_same_edit() {
    for &ch in &[1usize, 2] {
        let frames = 5_000;
        let data = noise(frames, ch, 7);
        for (op_name, f) in ops() {
            for (sel_name, sel) in selections(frames) {
                let a = fast(&data, sel.clone(), f);
                let b = slow(&data, sel.clone(), f);
                assert_eq!(
                    a.data().samples(),
                    b.data().samples(),
                    "{op_name} {sel_name} (ch={ch}): the in-place path and the splice path \
                     disagree about the audio"
                );
                assert_eq!(a.data().format(), b.data().format(), "{op_name} {sel_name}");
                // The waveform is derived from the audio, so it has to agree too — a patched cache
                // that drifted would draw a clip nobody can hear.
                assert_eq!(
                    waveform(&a),
                    waveform(&b),
                    "{op_name} {sel_name} (ch={ch}): the patched waveform is not the rebuilt one"
                );
            }
        }
    }
}

/// **The fast path actually fires.** Without this the suite above would pass just as happily with
/// the optimisation deleted — every comparison would be the slow path against itself, and the gates
/// would be green over dead code (the trap ADR-0120 documented and this one inherits).
///
/// The observable needs no test-only API: rewriting in place is *defined* by the samples not moving.
#[test]
fn a_sole_owner_writes_the_range_where_it_lies() {
    let frames = 5_000;
    let data = noise(frames, 2, 3);
    let sel = Some(frames / 3..frames / 2);

    let mut c = clip_with(&data, sel.clone());
    let before = c.data().samples().as_ptr();
    c.apply_gain(0.5);
    assert_eq!(
        c.data().samples().as_ptr(),
        before,
        "the buffer moved: the edit rebuilt the clip instead of rewriting the range, and ADR-0124 \
         buys nothing"
    );

    // ...and a second owner sends it back down the splice path, which is the correct thing to do:
    // the mixer may be reading those samples on the RT thread (HR-3).
    let mut c = clip_with(&data, sel);
    let keep = c.data().clone();
    let before = c.data().samples().as_ptr();
    c.apply_gain(0.5);
    assert_ne!(
        c.data().samples().as_ptr(),
        before,
        "the clip was scribbled on while a second owner held it -- the mixer would have torn"
    );
    drop(keep);
}

/// **The cache-staleness trap.** Six caches in the shell identify a buffer by its address, on the
/// documented reasoning that "a new buffer is a new pointer, and any edit hands us a different one".
/// Rewriting in place falsifies exactly that sentence, so the version has to move even though the
/// address does not — otherwise the spectrogram draws the pre-edit waveform and the delivery panel
/// prices the pre-edit bytes, both in silence.
#[test]
fn an_in_place_edit_moves_the_version_even_though_it_does_not_move_the_buffer() {
    let frames = 5_000;
    let data = noise(frames, 2, 11);
    let mut c = clip_with(&data, Some(frames / 3..frames / 2));

    let ptr = c.data().samples().as_ptr();
    let v0 = c.data().version();
    c.apply_gain(0.5);

    assert_eq!(
        c.data().samples().as_ptr(),
        ptr,
        "expected the in-place path"
    );
    assert_ne!(
        c.data().version(),
        v0,
        "the samples changed and the version did not: every cache keyed on this buffer now serves \
         the pre-edit audio, and nothing looks broken"
    );
}

/// A clip nobody edits keeps its version — a cache that re-derived on every frame would be no cache.
#[test]
fn reading_a_buffer_does_not_move_its_version() {
    let data = noise(1_000, 2, 4);
    let c = clip_with(&data, None);
    let v = c.data().version();
    let _ = c.data().samples().iter().sum::<f32>();
    let _ = c.column_peaks(0, 1_000, 64);
    assert_eq!(c.data().version(), v);
    // A clone is the same audio, so it is the same version: that is what lets a cache hold a key
    // rather than the buffer.
    assert_eq!(c.data().clone().version(), v);
}

/// **Undo restores the original, byte for byte.** The step is built from the range the caller
/// *declared*; if that declaration is ever wrong, this is what goes red.
#[test]
fn undo_restores_the_original_byte_for_byte() {
    for &ch in &[1usize, 2] {
        let frames = 5_000;
        let data = noise(frames, ch, 21);
        for (op_name, f) in ops() {
            for (sel_name, sel) in selections(frames) {
                let mut c = clip_with(&data, sel.clone());
                f(&mut c);
                let edited = c.data().clone();
                let could = c.undo();

                assert_eq!(
                    c.data().samples(),
                    data.samples(),
                    "{op_name} {sel_name} (ch={ch}): undo did not restore the original audio"
                );
                assert_eq!(
                    waveform(&c),
                    waveform(&clip_with(&data, sel.clone())),
                    "{op_name} {sel_name}: undo restored the audio but not the waveform"
                );
                // ...and redo puts the edit back, exactly.
                if could {
                    assert!(c.redo(), "{op_name} {sel_name}: redo refused after an undo");
                    assert_eq!(
                        c.data().samples(),
                        edited.samples(),
                        "{op_name} {sel_name}: redo did not reproduce the edit"
                    );
                }
            }
        }
    }
}

/// **The A7 oracle of ADR-0117, aimed at the informed step.** Any sequence of range edits, any walk
/// of undo/redo through it, against a timeline that keeps every whole buffer. A delta that lands one
/// sample off the snapshot has corrupted the user's audio.
#[test]
fn undo_and_redo_land_where_whole_snapshots_would() {
    for &ch in &[1usize, 2] {
        for seed in 0..8u32 {
            let frames = 2_000;
            let start = noise(frames, ch, seed);
            let mut c = clip_with(&start, None);
            // Copies, not clones — see `copy`. With clones in here the document would never own its
            // buffer again after the first edit, and this oracle would be checking the splice path
            // against itself while the in-place path went unexercised.
            let mut snapshots = vec![copy(&start)];

            for k in 0..10u32 {
                // Move the selection around, including off entirely.
                let n = c.frame_count();
                let sel = match (seed + k) % 4 {
                    0 => Some(0..n / 5),
                    1 => Some(n / 4..n / 2),
                    2 => Some(n - n / 5..n),
                    _ => None,
                };
                // A selection here is always a proper sub-range of the clip; `None` is the whole
                // thing, which takes the old path on purpose — you cannot change every sample for
                // less than every sample.
                let is_sub_range = sel.is_some();
                c.set_selection(sel);
                let before = copy(c.data());
                // For a sub-range the buffer must be the document's alone as the edit runs, or the
                // edit under test is not the one this file exists to check.
                let ptr = c.data().samples().as_ptr();
                match (seed + k) % 5 {
                    0 => c.apply_gain(0.7),
                    1 => c.apply_invert(),
                    2 => c.apply_reverse(),
                    3 => c.apply_fade(FadeShape::Linear, FadeDir::Out),
                    _ => c.apply_gain(1.0), // a NO-OP: it must not cost a step
                }
                if is_sub_range {
                    assert_eq!(
                        c.data().samples().as_ptr(),
                        ptr,
                        "the oracle stopped exercising the in-place path (ch={ch}, seed={seed}, \
                         k={k}) — it is checking the code this ADR replaced"
                    );
                }
                if c.data().samples() != before.samples() {
                    snapshots.push(copy(c.data()));
                }
            }

            // All the way back...
            let mut i = snapshots.len() - 1;
            while c.undo() {
                i -= 1;
                assert_eq!(
                    c.data().samples(),
                    snapshots[i].samples(),
                    "undo diverged at step {i} (ch={ch}, seed={seed})"
                );
            }
            assert_eq!(
                i, 0,
                "undo stopped short of the original (ch={ch}, seed={seed})"
            );

            // ...and all the way forward.
            while c.redo() {
                i += 1;
                assert_eq!(
                    c.data().samples(),
                    snapshots[i].samples(),
                    "redo diverged at step {i} (ch={ch}, seed={seed})"
                );
            }
            assert_eq!(i, snapshots.len() - 1, "redo stopped short of the tip");
        }
    }
}

/// A no-op still costs nothing — the guarantee ADR-0117 made "by construction" must survive an
/// informed step that could have taken the caller's word for it.
#[test]
fn an_edit_that_changes_nothing_costs_no_undo_step() {
    let data = noise(2_000, 2, 6);
    let mut c = clip_with(&data, Some(100..900));
    c.apply_gain(1.0); // exactly 1.0: every sample is bit-identical afterwards
    assert!(
        !c.can_undo(),
        "a gain of 1.0 changed nothing and still lit the Undo button"
    );
}

/// The patched waveform **is** the rebuilt waveform — for a range anywhere, including one that
/// starts and ends inside a bin.
#[test]
fn a_patched_cache_is_a_rebuilt_cache() {
    let frames = 5_000;
    let bin = 256;
    for &(start, end) in &[
        (0usize, 100usize),
        (100, 101),
        (255, 257), // straddles a bin boundary
        (256, 512), // exactly one bin
        (1_000, 4_000),
        (0, 5_000),
        (4_900, 5_000), // the ragged last bin
    ] {
        let before = noise(frames, 2, 31);
        let after = SampleData::map_in_place(&before, |s| {
            for x in &mut s[start * 2..end * 2] {
                *x *= 0.25;
            }
        });

        let mut patched = PeakCache::build(&before, bin);
        patched.patch(&after, start..end);
        let rebuilt = PeakCache::build(&after, bin);

        let cols = frames / bin;
        let a = column_peaks(&after, &patched, 0, frames, cols);
        let b = column_peaks(&after, &rebuilt, 0, frames, cols);
        assert_eq!(
            (a.min, a.max),
            (b.min, b.max),
            "patching {start}..{end} did not reproduce the rebuilt cache"
        );
    }
}
