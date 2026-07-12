//! `EditClip` tests, extracted from lib.rs to keep it under the workspace LOC cap.

use super::*;
use ph2d_audio::AudioFormat;

#[test]
fn duration_and_selection_clamp() {
    let data = SampleData::from_interleaved(vec![0.0; 96_000], AudioFormat::stereo(48_000));
    let mut clip = EditClip::new(data);
    assert_eq!(clip.frame_count(), 48_000);
    assert!((clip.duration_secs() - 1.0).abs() < 1e-9);

    clip.set_selection(Some(10..60_000)); // end past the clip
    assert_eq!(clip.selection(), Some(10..48_000));

    clip.set_selection(Some(500..500)); // empty → cleared
    assert_eq!(clip.selection(), None);
}

#[test]
fn apply_undo_redo_timeline() {
    let d = SampleData::from_interleaved(vec![0.5; 8], AudioFormat::stereo(48_000));
    let mut clip = EditClip::new(d);
    assert!(!clip.can_undo() && !clip.can_redo());

    clip.apply_gain(0.5); // 0.5 → 0.25
    assert_eq!(clip.data().samples()[0], 0.25);
    assert!(clip.can_undo() && !clip.can_redo());

    clip.apply_invert(); // 0.25 → -0.25
    assert_eq!(clip.data().samples()[0], -0.25);

    assert!(clip.undo()); // back to 0.25
    assert_eq!(clip.data().samples()[0], 0.25);
    assert!(clip.undo()); // back to 0.5
    assert_eq!(clip.data().samples()[0], 0.5);
    assert!(!clip.undo(), "at the start of the timeline");

    assert!(clip.redo()); // 0.25 again
    assert_eq!(clip.data().samples()[0], 0.25);

    // A new edit truncates the redo tail.
    clip.apply_gain(2.0); // 0.25 → 0.5
    assert_eq!(clip.data().samples()[0], 0.5);
    assert!(!clip.can_redo(), "new edit dropped the redo branch");
}

#[test]
fn trim_uses_selection_and_clears_it() {
    let d = SampleData::from_interleaved(
        vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0],
        AudioFormat::stereo(48_000),
    );
    let mut clip = EditClip::new(d);
    clip.set_selection(Some(1..2));
    clip.apply_trim();
    assert_eq!(clip.frame_count(), 1);
    assert_eq!(clip.data().samples(), &[2.0, 2.0]);
    assert_eq!(clip.selection(), None);
    assert!(clip.undo(), "trim is undoable");
    assert_eq!(clip.frame_count(), 3);
}

#[test]
fn whole_clip_ops_respect_selection_and_undo() {
    // 4 stereo frames, all 0.5. Select the middle two and gain them to 0.
    let d = SampleData::from_interleaved(vec![0.5; 8], AudioFormat::stereo(48_000));
    let mut clip = EditClip::new(d);
    clip.set_selection(Some(1..3));
    clip.apply_gain(0.0);
    assert_eq!(
        clip.data().samples(),
        &[0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5],
        "gain must scope to the selection, not the whole clip"
    );
    // The selection survives a length-preserving op (chainable).
    assert_eq!(clip.selection(), Some(1..3));
    // Undo brings the selected samples back.
    assert!(clip.undo(), "selection-scoped gain is undoable");
    assert_eq!(clip.data().samples(), &[0.5; 8]);
    // With no selection the same op hits the whole clip.
    clip.set_selection(None);
    clip.apply_gain(0.0);
    assert_eq!(clip.data().samples(), &[0.0; 8]);
}

#[test]
fn tail_effect_grows_the_clip_and_undo_restores_it() {
    let d = SampleData::from_interleaved(vec![0.5; 2_000], AudioFormat::stereo(48_000));
    let mut clip = EditClip::new(d);
    assert_eq!(clip.frame_count(), 1_000);

    let echo = TailEffect::Delay {
        time_secs: 0.002,
        feedback: 0.3,
        mix: 0.5,
        tail_secs: 0.01, // 480 frames
    };
    clip.apply_tail_effect(&echo);
    assert_eq!(
        clip.frame_count(),
        1_480,
        "no selection → whole clip + tail"
    );
    assert!(clip.can_undo());
    assert!(clip.undo());
    assert_eq!(
        clip.frame_count(),
        1_000,
        "undo restores the original length"
    );

    // A selection mid-clip: the tail bleeds in, the clip does NOT grow.
    clip.set_selection(Some(0..100));
    clip.apply_tail_effect(&echo);
    assert_eq!(clip.frame_count(), 1_000, "tail fits inside → no growth");
}

/// The live-audition contract: the buffer the user HEARS (`render_*`) is
/// bit-for-bit the one `commit_rendered` puts in the undo timeline, and it is
/// identical to what the one-shot `apply_*` would have produced. A drift here
/// means Apply silently sounds different from the preview.
#[test]
fn rendered_audition_is_what_gets_committed() {
    let d = SampleData::from_interleaved(vec![0.6; 2_000], AudioFormat::stereo(48_000));
    let fx = Effect::Saturate { drive: 4.0 };

    let mut auditioned = EditClip::new(d.clone());
    let heard = auditioned.render_effect(fx);
    assert_eq!(auditioned.frame_count(), 1_000, "render must NOT mutate");
    assert!(!auditioned.can_undo(), "render must NOT touch the timeline");
    auditioned.commit_rendered(heard.clone());

    let mut applied = EditClip::new(d);
    applied.apply_effect(fx);
    assert_eq!(auditioned.data().samples(), applied.data().samples());
    assert_eq!(heard.samples(), applied.data().samples());
    assert!(auditioned.can_undo() && auditioned.undo());
}

/// Same contract for the tail family, where the audition also changes length.
#[test]
fn rendered_tail_audition_matches_apply() {
    let d = SampleData::from_interleaved(vec![0.5; 2_000], AudioFormat::stereo(48_000));
    let fx = TailEffect::Delay {
        time_secs: 0.002,
        feedback: 0.3,
        mix: 0.5,
        tail_secs: 0.01,
    };
    let clip = EditClip::new(d.clone());
    let heard = clip.render_tail_effect(&fx);
    assert_eq!(clip.frame_count(), 1_000, "render must NOT mutate");

    let mut applied = EditClip::new(d);
    applied.apply_tail_effect(&fx);
    assert_eq!(heard.samples(), applied.data().samples());
    assert_eq!(heard.frame_count(), 1_480);
}

/// A filter applied to a MID-CLIP selection must not click at its leading edge.
/// Found by the 2026-07-09 audit: `in_range` hands the op an isolated region, so
/// the biquad starts with cleared memory — as if silence preceded the selection —
/// and its first output collapses toward zero while the untouched neighbour is
/// still at full level. `render_effect` now pre-rolls the real preceding audio.
#[test]
fn filtering_a_mid_clip_selection_does_not_click_at_the_leading_edge() {
    // 4000 frames of steady DC: any level jump at the splice IS the artifact.
    let d = SampleData::from_interleaved(vec![0.7; 8_000], AudioFormat::stereo(48_000));
    let fx = Effect::LowPass {
        cutoff: 1_000.0,
        q: 0.707,
    };
    let sel = 2_000..3_000;
    let step = |x: &SampleData| (x.samples()[2_000 * 2] - x.samples()[1_999 * 2]).abs();

    // The cold splice (what plain `in_range` does) really does click.
    let cold = in_range(&d, sel.clone(), |x| fx.apply(x));
    assert!(
        step(&cold) > 0.5,
        "expected the cold-start click to reproduce, got {}",
        step(&cold)
    );

    // `render_effect` pre-rolls the filter → the edge is continuous.
    let mut clip = EditClip::new(d.clone());
    clip.set_selection(Some(sel));
    let warm = clip.render_effect(fx);
    assert!(
        step(&warm) < 0.01,
        "warm-up must remove the edge click, got {}",
        step(&warm)
    );

    // No selection: nothing precedes the range, so nothing to warm up on — and
    // the result stays byte-identical to applying the op directly.
    let whole = EditClip::new(d.clone());
    assert_eq!(whole.render_effect(fx).samples(), fx.apply(&d).samples());
}

/// Loop metadata: adopted from the selection, survives undo/redo (it is not
/// sample data), clamps when an edit shrinks the clip, and clears on a new load.
#[test]
fn loop_region_is_metadata_that_survives_undo_and_clamps() {
    let d = SampleData::from_interleaved(vec![0.5; 20_000], AudioFormat::stereo(48_000));
    let mut clip = EditClip::new(d); // 10_000 frames
    assert!(!clip.has_loop());

    clip.set_selection(Some(2_000..8_000));
    clip.set_loop_from_selection();
    assert_eq!(clip.loop_region(), Some(2_000..8_000));

    // An edit (gain) does NOT disturb the loop, and undo leaves it alone.
    clip.apply_gain(0.5);
    assert_eq!(
        clip.loop_region(),
        Some(2_000..8_000),
        "edit keeps the loop"
    );
    assert!(clip.undo());
    assert_eq!(
        clip.loop_region(),
        Some(2_000..8_000),
        "undo keeps the loop"
    );

    // Trimming to 0..5_000 shrinks the clip → the loop clamps to the new length.
    clip.set_selection(Some(0..5_000));
    clip.apply_trim();
    assert_eq!(clip.frame_count(), 5_000);
    assert_eq!(
        clip.loop_region(),
        Some(2_000..5_000),
        "loop clamps to the clip"
    );

    // A new clip clears the loop.
    clip.set_data(SampleData::from_interleaved(
        vec![0.0; 1_000],
        AudioFormat::stereo(48_000),
    ));
    assert!(!clip.has_loop(), "load clears the loop");
}

/// Snap moves both endpoints onto zero crossings; the audition buffer is the
/// region length and loops without a click.
#[test]
fn loop_snap_and_audition_buffer() {
    // A mono sine: zero crossings are dense, so a small window always finds one.
    let step = std::f32::consts::TAU * 200.0 / 48_000.0;
    let v: Vec<f32> = (0..4_800).map(|i| (i as f32 * step).sin()).collect();
    let mut clip = EditClip::new(SampleData::from_interleaved(v, AudioFormat::mono(48_000)));
    clip.set_selection(Some(1_001..3_099));
    clip.set_loop_from_selection();
    clip.snap_loop_to_zero_crossing(64);
    let lp = clip.loop_region().unwrap();
    let sample = |f: usize| clip.data().samples()[f];
    let crosses = |f: usize| f > 0 && (sample(f - 1) <= 0.0) != (sample(f) <= 0.0);
    assert!(crosses(lp.start), "loop start on a zero crossing");
    assert!(crosses(lp.end), "loop end on a zero crossing");

    let buf = clip.loop_audition_buffer(256).expect("loop is set");
    assert_eq!(buf.frame_count(), lp.len());
    // The crossfade drives the seam down to the source's OWN continuity at the
    // loop point — `|data[start] − data[start-1]|`, one adjacent-sample step — not
    // to zero. That is the click-free floor: the wrap becomes the source's natural
    // `start-1 → start` transition.
    let natural = (sample(lp.start) - sample(lp.start - 1)).abs();
    assert!(
        loops::seam_step(&buf) <= natural + 1e-3,
        "audition seam {} must reach the natural step {natural}",
        loops::seam_step(&buf)
    );
    // No loop → no audition buffer.
    clip.clear_loop();
    assert!(clip.loop_audition_buffer(256).is_none());
}

/// Force-to-mono downmixes the whole clip (mean of channels), preserves the frame
/// count, is undoable, and is a no-op on an already-mono clip.
#[test]
fn force_mono_downmixes_preserves_frames_and_undoes() {
    // Stereo: frame 0 = (0.2, 0.6) → 0.4; frame 1 = (-0.4, 0.0) → -0.2.
    let d = SampleData::from_interleaved(vec![0.2, 0.6, -0.4, 0.0], AudioFormat::stereo(48_000));
    let mut clip = EditClip::new(d);
    assert_eq!(clip.frame_count(), 2);
    clip.set_loop_region(Some(0..2));

    clip.apply_force_mono();
    assert_eq!(clip.data().format().channel_count(), 1, "now mono");
    assert_eq!(clip.frame_count(), 2, "frames preserved");
    assert_eq!(clip.data().samples(), &[0.4, -0.2]);
    assert_eq!(clip.loop_region(), Some(0..2), "loop survives the downmix");

    assert!(clip.undo(), "force-mono is undoable");
    assert_eq!(clip.data().format().channel_count(), 2, "back to stereo");

    // Already mono → no-op, no new undo step.
    let mut mono = EditClip::new(SampleData::from_interleaved(
        vec![0.1, 0.2],
        AudioFormat::mono(48_000),
    ));
    mono.apply_force_mono();
    assert!(!mono.can_undo(), "no-op on a mono clip");
}

#[test]
fn set_data_rebuilds_and_clamps_selection() {
    let big = SampleData::from_interleaved(vec![0.1; 20_000], AudioFormat::mono(48_000));
    let mut clip = EditClip::new(big);
    clip.set_selection(Some(100..19_000));
    // Shrink the clip; selection must clamp.
    let small = SampleData::from_interleaved(vec![0.2; 1_000], AudioFormat::mono(48_000));
    clip.set_data(small);
    assert_eq!(clip.frame_count(), 1_000);
    assert_eq!(clip.selection(), Some(100..1_000));
}

/// Markers stay sorted by frame, dedupe on the same frame, delete-nearest honours a
/// window, survive undo, clamp when an edit shrinks the clip, and clear on load.
#[test]
fn markers_sort_dedupe_delete_and_clamp() {
    let d = SampleData::from_interleaved(vec![0.5; 20_000], AudioFormat::stereo(48_000));
    let mut clip = EditClip::new(d); // 10_000 frames

    assert!(clip.add_marker(5_000, "M2"));
    assert!(clip.add_marker(1_000, "M1")); // inserted BEFORE M2 → sorted
    assert!(!clip.add_marker(5_000, "dup"), "same frame is a no-op");
    let frames: Vec<_> = clip.markers().iter().map(|m| m.frame).collect();
    assert_eq!(frames, vec![1_000, 5_000], "kept sorted by frame");

    // Delete-nearest within a window; too-far leaves it.
    assert!(
        clip.remove_marker_near(1_050, 10).is_none(),
        "outside window"
    );
    assert_eq!(
        clip.remove_marker_near(1_050, 100).map(|m| m.name),
        Some("M1".to_string())
    );
    assert_eq!(clip.markers().len(), 1);

    // Markers survive an edit + undo; a trim that shrinks the clip drops the far one.
    clip.apply_gain(0.5);
    assert_eq!(clip.markers().len(), 1, "edit keeps markers");
    assert!(clip.undo());
    assert_eq!(clip.markers().len(), 1, "undo keeps markers");
    clip.set_selection(Some(0..2_000));
    clip.apply_trim(); // → 2_000 frames; the 5_000 marker is now past the end
    assert!(clip.markers().is_empty(), "out-of-range marker dropped");

    // A new clip clears markers.
    clip.add_marker(100, "M");
    clip.set_data(SampleData::from_interleaved(
        vec![0.0; 100],
        AudioFormat::stereo(48_000),
    ));
    assert!(clip.markers().is_empty(), "load clears markers");
}
