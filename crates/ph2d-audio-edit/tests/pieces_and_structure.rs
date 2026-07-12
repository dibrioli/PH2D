//! Gates for **cuts, pieces, reorder and stretch** — and for the structure that has to travel
//! with them.
//!
//! The load-bearing claims, each stated as something that can go red:
//!
//! 1. A reorder is a **permutation** — every sample survives, exactly once.
//! 2. A reorder is **reversible byte-for-byte**: drag it back and the clip is the file you loaded.
//! 3. Undo restores the **cuts**, not just the samples.
//! 4. Markers are glued to the **audio**, not to frame numbers — through a reorder AND through a
//!    ripple delete, which is where they used to silently slide onto different audio.
//! 5. A stretch changes the **length** and not the **pitch** (that is the whole word "stretch").
//! 6. A stretch keeps the **stereo image**: the channels are sliced at the same places.
//! 7. Splitting costs an undo step but **no audio**, and does not touch one sample.

use ph2d_audio::{AudioFormat, SampleData};
use ph2d_audio_edit::EditClip;

const SR: u32 = 48_000;

/// A stereo clip whose every frame is stamped with its own index, so a permutation is legible:
/// after any reorder you can read off exactly which frame of the original each output frame is.
fn stamped(frames: usize) -> SampleData {
    let x: Vec<f32> = (0..frames).flat_map(|f| [f as f32, -(f as f32)]).collect();
    SampleData::from_interleaved(x, AudioFormat::stereo(SR))
}

/// Which original frame each output frame came from (reads the stamp back).
fn stamps(d: &SampleData) -> Vec<i64> {
    d.samples().chunks(2).map(|c| c[0] as i64).collect()
}

fn tone(hz: f32, frames: usize) -> SampleData {
    let tau = std::f32::consts::TAU;
    let x: Vec<f32> = (0..frames)
        .map(|i| 0.6 * (tau * hz * i as f32 / SR as f32).sin())
        .collect();
    SampleData::from_interleaved(x, AudioFormat::mono(SR))
}

/// Energy at one frequency — a single DFT bin, computed directly.
fn energy_at(x: &[f32], hz: f64) -> f64 {
    let tau = std::f64::consts::TAU;
    let (mut re, mut im) = (0.0f64, 0.0f64);
    for (n, &v) in x.iter().enumerate() {
        let phase = tau * hz * n as f64 / f64::from(SR);
        re += f64::from(v) * phase.cos();
        im -= f64::from(v) * phase.sin();
    }
    (re * re + im * im).sqrt() / x.len() as f64
}

// ---------------------------------------------------------------------------------------------
// 1 + 2. A reorder is a permutation, and it is reversible.
// ---------------------------------------------------------------------------------------------

/// Every sample of the input comes out, exactly once — the strongest thing that can be said about
/// a reorder, and the one that catches an off-by-one in the copy loop that a "sounds fine" listen
/// never would.
#[test]
fn reorder_is_a_permutation_of_the_clip() {
    let mut clip = EditClip::new(stamped(1_000));
    clip.split_at(300);
    clip.split_at(700); // pieces: [0,300) [300,700) [700,1000)
    assert_eq!(clip.pieces().len(), 3);

    // Move the FIRST piece to the end (boundary 3 = past the last piece).
    assert!(clip.move_piece(0, 3));

    let out = stamps(clip.data());
    assert_eq!(out.len(), 1_000, "a permutation cannot change the length");

    let mut sorted = out.clone();
    sorted.sort_unstable();
    let expected: Vec<i64> = (0..1_000).collect();
    assert_eq!(sorted, expected, "every frame survives, exactly once");

    // And it landed in the order asked for: [300..700) [700..1000) [0..300).
    let want: Vec<i64> = (300..700).chain(700..1000).chain(0..300).collect();
    assert_eq!(out, want, "the pieces are in the dropped order");

    // The cuts are re-derived from the new layout, not shifted from the old ones.
    assert_eq!(
        clip.cuts(),
        &[400, 700],
        "seams land on the new piece starts"
    );
}

/// Drag it back and you have the file you loaded — **byte for byte**. Two operations that do not
/// compose to the identity are two operations lying about what they do.
#[test]
fn reordering_a_piece_back_is_byte_identical() {
    let original = stamped(900);
    let mut clip = EditClip::new(original.clone());
    clip.split_at(200);
    clip.split_at(500);

    assert!(clip.move_piece(2, 0)); // last piece to the front
    assert_ne!(clip.data().samples(), original.samples(), "it really moved");

    // It is now piece 0; put it back after the other two (boundary 3, past the end).
    assert!(clip.move_piece(0, 3));
    assert_eq!(
        clip.data().samples(),
        original.samples(),
        "reorder ∘ reorder⁻¹ = identity, bit for bit"
    );
    assert_eq!(clip.cuts(), &[200, 500], "and the seams came home too");
}

/// Dropping a piece back onto its own seam changes nothing — and says so, so it never lands an
/// empty step on the undo timeline.
#[test]
fn dropping_a_piece_where_it_already_is_costs_nothing() {
    let mut clip = EditClip::new(stamped(600));
    clip.split_at(300);
    let before = clip.can_undo();
    assert!(!clip.move_piece(0, 0), "onto its own left seam");
    assert!(!clip.move_piece(0, 1), "onto its own right seam");
    assert_eq!(clip.can_undo(), before, "no phantom undo step");
}

// ---------------------------------------------------------------------------------------------
// 3 + 4. The structure travels with the audio.
// ---------------------------------------------------------------------------------------------

/// Undo has to restore the **cuts** as well as the samples. Restoring only the audio would leave
/// every seam drawn across the wrong part of it — the picture would lie about the clip.
#[test]
fn undo_of_a_reorder_restores_the_cuts_and_the_samples() {
    let original = stamped(900);
    let mut clip = EditClip::new(original.clone());
    clip.split_at(200);
    clip.split_at(500);

    assert!(clip.move_piece(2, 0));
    assert_eq!(clip.cuts(), &[400, 600], "moved: [500..900) is now first");

    assert!(clip.undo());
    assert_eq!(
        clip.data().samples(),
        original.samples(),
        "samples restored"
    );
    assert_eq!(clip.cuts(), &[200, 500], "cuts restored");

    // And redo is the same swap, back the other way.
    assert!(clip.redo());
    assert_eq!(clip.cuts(), &[400, 600], "redo re-applies the structure");
}

/// A marker names a **moment in the audio**. Move the audio and the marker goes with it: a cue on
/// a footstep is still on that footstep after the footstep is dragged somewhere else.
#[test]
fn a_reorder_carries_the_markers_with_their_audio() {
    let mut clip = EditClip::new(stamped(900));
    clip.split_at(300);
    clip.split_at(600);
    clip.add_marker(350, "M"); // 50 frames into piece 1

    // Piece 1 ([300..600)) to the front.
    assert!(clip.move_piece(1, 0));
    assert_eq!(
        clip.markers()[0].frame,
        50,
        "the marker is still 50 frames into its piece, which now starts at 0"
    );
    // The audio under it is unchanged — that is what "carried" means.
    assert_eq!(stamps(clip.data())[50], 350);
}

/// **The bug that predates pieces.** A ripple delete slides everything after it to the left; the
/// markers used to sit still and end up on different audio. This is the gate that says they move.
#[test]
fn a_ripple_delete_slides_the_markers_it_did_not_delete() {
    let mut clip = EditClip::new(stamped(1_000));
    clip.add_marker(200, "before");
    clip.add_marker(500, "inside");
    clip.add_marker(800, "after");

    // Delete 400..600 (200 frames) — "inside" is destroyed, "after" slides left by 200.
    clip.set_selection(Some(400..600));
    clip.apply_delete();

    let m: Vec<(usize, &str)> = clip
        .markers()
        .iter()
        .map(|m| (m.frame, m.name.as_str()))
        .collect();
    assert_eq!(
        m,
        vec![(200, "before"), (600, "after")],
        "the marker before stays; the one after slides by exactly what was removed; \
         the one standing on deleted audio is gone"
    );
    // The proof it is on the SAME audio: frame 600 of the new clip is old frame 800.
    assert_eq!(stamps(clip.data())[600], 800);
}

/// Cuts ride the same rails: a paste before a seam pushes the seam along by what was inserted.
#[test]
fn a_paste_slides_the_cuts_after_it() {
    let mut clip = EditClip::new(stamped(1_000));
    clip.split_at(600);

    let clipboard = stamped(100);
    clip.set_selection(None);
    clip.apply_paste(&clipboard, 200); // insert 100 frames at 200

    assert_eq!(clip.frame_count(), 1_100);
    assert_eq!(
        clip.cuts(),
        &[700],
        "the seam moved with the audio it names"
    );
    assert_eq!(stamps(clip.data())[700], 600, "and still names that audio");
}

/// A loop set **inside one piece** survives a reorder (it moves with the piece). One that
/// **straddles** two pieces has nowhere coherent to go, so it is cleared rather than silently
/// re-pointed at whatever audio slid into those frame numbers.
#[test]
fn a_loop_follows_its_piece_or_is_cleared() {
    let mut clip = EditClip::new(stamped(900));
    clip.split_at(300);
    clip.split_at(600);

    clip.set_loop_region(Some(350..500)); // inside piece 1
    assert!(clip.move_piece(1, 0));
    assert_eq!(clip.loop_region(), Some(50..200), "moved with its piece");

    // Now a loop across the seam between the (new) pieces 0 and 1.
    clip.set_loop_region(Some(250..400));
    assert!(clip.move_piece(2, 0));
    assert_eq!(clip.loop_region(), None, "a straddling loop is cleared");
}

// ---------------------------------------------------------------------------------------------
// 5 + 6. Stretch.
// ---------------------------------------------------------------------------------------------

/// The whole word: the clip gets **shorter**, and the note stays where it was. A resample would
/// also shorten it — and take the pitch up with it, which is a different tool.
#[test]
fn stretch_changes_the_length_and_not_the_pitch() {
    let mut clip = EditClip::new(tone(220.0, 24_000)); // 0.5 s
    assert!(clip.stretch_piece(0, 16_000)); // squeeze to 2/3

    assert_eq!(clip.frame_count(), 16_000, "it got shorter");

    let out = clip.data().samples();
    let at_220 = energy_at(out, 220.0);
    // A naive resample would have taken 220 Hz up to 330 Hz. WSOLA must not.
    let at_330 = energy_at(out, 330.0);
    assert!(
        at_220 > 0.15,
        "the note is still there at 220 Hz (got {at_220:.3})"
    );
    assert!(
        at_330 < at_220 * 0.15,
        "and it did NOT slide up to 330 Hz — that is a resample, not a stretch \
         (220: {at_220:.3}, 330: {at_330:.3})"
    );
}

/// Stretching a piece ripples what follows, and the cuts follow the audio.
#[test]
fn stretching_a_piece_ripples_the_rest() {
    let mut clip = EditClip::new(stamped(3_000));
    clip.split_at(1_000);
    clip.split_at(2_000);

    assert!(clip.stretch_piece(1, 500)); // middle piece: 1000 frames -> 500
    assert_eq!(
        clip.frame_count(),
        2_500,
        "the clip lost what the piece lost"
    );
    assert_eq!(
        clip.cuts(),
        &[1_000, 1_500],
        "the seam before it holds; the seam after slides by the difference"
    );
    // The audio AFTER the stretched piece is untouched — a stretch is local.
    assert_eq!(stamps(clip.data())[1_500], 2_000);
    assert_eq!(stamps(clip.data())[2_499], 2_999);
}

/// **The channels are sliced at the same places.** Run WSOLA's similarity search once per channel
/// instead and each one phase-aligns to *its own* waveform — so the two channels get different
/// time-warps, and an event that happened at one instant comes out at two.
///
/// The signal is built to make that visible: **different carriers** (200 Hz left, 700 Hz right, so
/// the per-channel searches genuinely disagree about where to splice) under **one shared
/// envelope** — a burst that, in the real world, is one sound arriving at both ears. Sliced
/// together, the burst lands at the same output frame in both channels. Sliced apart, it does not.
///
/// (An earlier version of this gate used the same waveform in both channels, delayed. It passed
/// even with the per-channel bug deliberately reintroduced — a delayed copy has *identical*
/// self-similarity, so both searches return the same answer and the gate could not tell the two
/// implementations apart. It measured nothing. Mutation is how that was found.)
#[test]
fn stretching_keeps_the_channels_locked_together() {
    let n = 24_000;
    let tau = std::f32::consts::TAU;
    // One envelope, two carriers: a raised-cosine burst centred at frame 12_000. The width is
    // chosen: too narrow and both implementations happen to pick the same offsets (measured), so
    // the gate would separate nothing.
    const HALF: f32 = 2_000.0;
    let env = |i: usize| {
        let d = (i as f32 - 12_000.0).abs();
        if d > HALF {
            0.0
        } else {
            0.5 * (1.0 + (std::f32::consts::PI * d / HALF).cos())
        }
    };
    let x: Vec<f32> = (0..n)
        .flat_map(|i| {
            let e = env(i);
            [
                e * (tau * 200.0 * i as f32 / SR as f32).sin(),
                e * (tau * 700.0 * i as f32 / SR as f32).sin(),
            ]
        })
        .collect();
    let mut clip = EditClip::new(SampleData::from_interleaved(x, AudioFormat::stereo(SR)));

    assert!(clip.stretch_piece(0, 36_000)); // stretch 1.5x

    let out = clip.data().samples();
    // Where each channel's energy sits, in frames. One sound, one arrival time — in both ears.
    let centroid = |c: usize| {
        let (mut num, mut den) = (0.0f64, 0.0f64);
        for (f, s) in out.chunks(2).enumerate() {
            let e = f64::from(s[c]) * f64::from(s[c]);
            num += e * f as f64;
            den += e;
        }
        num / den.max(1e-12)
    };
    // Slicing both channels at the same offsets makes the two output envelopes *the same
    // envelope*, so the centroids agree to a rounding error — measured at 0.01 frames apart.
    // With the search run per channel they land 20 frames apart. The bar sits between.
    let (l, r) = (centroid(0), centroid(1));
    assert!(
        (l - r).abs() < 2.0,
        "the burst came out at frame {l:.1} on the left and {r:.1} on the right — \
         the channels were time-warped independently, which is what smears a stereo image"
    );
}

/// A drag that ends where it started must not run the audio through the grain mill for nothing.
#[test]
fn a_zero_move_stretch_is_byte_identical() {
    let original = tone(300.0, 8_000);
    let mut clip = EditClip::new(original.clone());
    assert!(!clip.stretch_piece(0, 8_000), "asking for the same length");
    assert_eq!(clip.data().samples(), original.samples());
    assert!(!clip.can_undo(), "and it costs no undo step");
}

// ---------------------------------------------------------------------------------------------
// 7. Splitting is structure, and only structure.
// ---------------------------------------------------------------------------------------------

/// Split is an undo step that carries **no audio at all** — not one sample moves.
#[test]
fn splitting_costs_an_undo_step_but_moves_no_audio() {
    let original = stamped(1_000);
    let mut clip = EditClip::new(original.clone());

    assert!(clip.split_at(400));
    assert_eq!(
        clip.data().samples(),
        original.samples(),
        "a cut is a boundary, not a knife"
    );
    assert_eq!(clip.pieces(), vec![0..400, 400..1_000]);

    assert!(clip.can_undo(), "but it IS an undo step");
    assert!(clip.undo());
    assert_eq!(clip.cuts(), &[] as &[usize], "undo removes the cut");
    assert_eq!(clip.pieces(), vec![0..1_000], "one piece again");
}

/// Split at Markers now **only splits the clip** (it used to encode files and adopt a variation
/// set — a delivery verb wearing an edit verb's name).
#[test]
fn split_at_markers_only_splits_the_clip() {
    let original = stamped(1_000);
    let mut clip = EditClip::new(original.clone());
    clip.add_marker(250, "a");
    clip.add_marker(500, "b");
    clip.add_marker(750, "c");

    assert!(clip.split_at_markers());
    assert_eq!(clip.cuts(), &[250, 500, 750]);
    assert_eq!(clip.pieces().len(), 4);
    assert_eq!(
        clip.data().samples(),
        original.samples(),
        "the audio is untouched"
    );

    // The pieces are still available AS clips — that is what Export Pieces writes out.
    let clips = clip.piece_clips();
    assert_eq!(clips.len(), 4);
    assert_eq!(clips[0].frame_count(), 250);
    assert_eq!(clips[3].frame_count(), 250);
}

/// Splitting on a seam that already exists, or on the very edges, is not a split.
#[test]
fn a_cut_that_names_nothing_is_refused() {
    let mut clip = EditClip::new(stamped(1_000));
    assert!(!clip.split_at(0), "a cut at 0 names an empty piece");
    assert!(!clip.split_at(1_000), "and so does one at the end");
    assert!(clip.split_at(500));
    assert!(!clip.split_at(500), "and a seam is not cut twice");
}

/// Clearing the cuts un-splits without un-doing the reorder they let you make: the audio is
/// wherever you dragged it to, and the boundaries are simply gone.
#[test]
fn clearing_the_cuts_keeps_the_audio_where_you_put_it() {
    let mut clip = EditClip::new(stamped(900));
    clip.split_at(300);
    clip.split_at(600);
    clip.move_piece(2, 0);
    let arranged = clip.data().clone();

    assert!(clip.clear_cuts());
    assert_eq!(clip.cuts(), &[] as &[usize]);
    assert_eq!(
        clip.data().samples(),
        arranged.samples(),
        "the arrangement survives; only the seams are gone"
    );
}

/// The piece a frame falls in, and the seam a drop would snap to — the two lookups the Move tool
/// runs on every mouse event.
#[test]
fn piece_lookup_and_drop_targets() {
    let mut clip = EditClip::new(stamped(1_000));
    clip.split_at(300);
    clip.split_at(700);

    assert_eq!(clip.piece_at(0), 0);
    assert_eq!(clip.piece_at(299), 0);
    assert_eq!(
        clip.piece_at(300),
        1,
        "a cut belongs to the piece it starts"
    );
    assert_eq!(clip.piece_at(999), 2);

    // Boundaries are 0, 300, 700, 1000.
    assert_eq!(clip.nearest_boundary(10), 0);
    assert_eq!(clip.nearest_boundary(280), 1);
    assert_eq!(clip.nearest_boundary(690), 2);
    assert_eq!(clip.nearest_boundary(990), 3);
}
