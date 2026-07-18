//! ADR-0115 A1-A3: the strip's time map and the lane's blend weights.
//!
//! Everything here is asserted on a **sampled value**, never on a field being
//! present. A strip that stores a `speed` it never applies would pass a
//! field-shaped test and fail an animator.

use ph2d_timeline::{ClipLane, ClipStrip, LaneMode, StripLoop, StripSource};

/// A strip playing 2 s of clip 0 over `[t0, t0 + 2)`.
fn strip(t0: f64) -> ClipStrip {
    ClipStrip::new(StripSource::Clip(0), t0, t0 + 2.0, 2.0)
}

// ── The time map ────────────────────────────────────────────────────────────

#[test]
fn a_strip_maps_timeline_time_into_clip_time() {
    let s = strip(3.0);
    assert_eq!(
        s.source_time(2.99),
        None,
        "before the strip: no contribution"
    );
    assert_eq!(
        s.source_time(3.0),
        Some(0.0),
        "its start is the clip's start"
    );
    assert_eq!(
        s.source_time(4.0),
        Some(1.0),
        "one second in, one second in"
    );
    assert_eq!(s.source_time(5.0), None, "the end is exclusive");
}

#[test]
fn the_source_slice_crops_the_clip() {
    let mut s = strip(0.0);
    (s.src_in, s.src_out) = (1.5, 2.0); // the clip's last half-second only
    assert_eq!(
        s.source_time(0.0),
        Some(1.5),
        "starts at the slice, not at 0"
    );
    // A 2 s span reading a 0.5 s slice at speed 1 runs out after 0.5 s and holds.
    assert_eq!(s.source_time(0.25), Some(1.75));
    assert_eq!(s.source_time(0.5), Some(2.0));
    assert_eq!(s.source_time(1.9), Some(2.0), "Once holds the last frame");
}

#[test]
fn speed_retimes_the_source() {
    let mut s = strip(0.0);
    s.speed = 2.0;
    assert_eq!(s.source_time(0.5), Some(1.0), "twice as fast");
    assert_eq!(s.source_time(1.0), Some(2.0), "the 2 s clip is done in 1 s");
    assert_eq!(s.source_time(1.5), Some(2.0), "and then holds (Once)");

    s.speed = 0.5;
    assert_eq!(s.source_time(1.0), Some(0.5), "half speed: slow motion");
}

#[test]
fn loop_wraps_and_ping_pong_reflects() {
    let mut s = strip(0.0);
    s.t_end = 6.0; // three times the clip's length

    s.loop_mode = StripLoop::Loop;
    assert_eq!(s.source_time(0.5), Some(0.5));
    assert_eq!(s.source_time(2.5), Some(0.5), "wrapped once");
    assert_eq!(s.source_time(4.5), Some(0.5), "wrapped twice");

    s.loop_mode = StripLoop::PingPong;
    assert_eq!(s.source_time(0.5), Some(0.5), "forward");
    assert_eq!(
        s.source_time(2.5),
        Some(1.5),
        "reflected: running backwards"
    );
    assert_eq!(s.source_time(3.5), Some(0.5), "still backwards");
    assert_eq!(s.source_time(4.5), Some(0.5), "forward again");
}

#[test]
fn a_reversed_or_empty_strip_never_panics_and_never_covers() {
    let mut s = strip(0.0);
    s.t_end = s.t_start; // zero span
    assert_eq!(s.source_time(0.0), None);
    s.t_end = -1.0; // reversed
    assert_eq!(s.source_time(0.0), None);
    assert_eq!(s.span(), 0.0);
}

// ── The blend weights ───────────────────────────────────────────────────────

#[test]
fn a_lone_strip_is_at_full_weight_throughout() {
    let mut lane = ClipLane::new("Base");
    lane.insert(strip(0.0));
    for step in 0..20 {
        let t = f64::from(step) * 0.1;
        assert_eq!(lane.weight_at(0, t), 1.0, "no neighbour, no ease, at t={t}");
    }
    assert_eq!(lane.weight_at(0, 2.0), 0.0, "past the end it is gone");
}

/// **The gesture** (ADR-0115 R1): the overlap IS the crossfade. Nobody typed a
/// duration; the two strips were merely dragged into each other.
#[test]
fn overlapping_two_strips_creates_the_crossfade() {
    let mut lane = ClipLane::new("Base");
    lane.insert(strip(0.0)); // [0, 2)
    lane.insert(strip(1.0)); // [1, 3)  -> a 1 s overlap

    assert_eq!(lane.weight_at(0, 0.5), 1.0, "before the overlap: A owns it");
    assert_eq!(lane.weight_at(1, 0.5), 0.0, "B has not started");

    assert_eq!(lane.weight_at(0, 1.5), 0.5, "mid-overlap: half and half");
    assert_eq!(lane.weight_at(1, 1.5), 0.5);

    assert_eq!(lane.weight_at(0, 2.5), 0.0, "A is over");
    assert_eq!(lane.weight_at(1, 2.5), 1.0, "B owns it");
}

/// The property the whole design leans on. Complementary weights need **no base
/// value** to blend against — which is what makes the crossfade immune to the
/// "sag toward the default pose" that Unity ships a dedicated weight processor
/// to prevent, and what lets a partial clip (one that keys X but not Y) blend
/// correctly with no mask.
///
/// It holds because smoothstep is antisymmetric about its midpoint:
/// `s(1 - u) == 1 - s(u)`.
#[test]
fn the_crossfade_weights_sum_to_exactly_one_through_the_whole_overlap() {
    let mut lane = ClipLane::new("Base");
    lane.insert(strip(0.0)); // [0, 2)
    lane.insert(strip(1.0)); // [1, 3)

    for step in 0..=100 {
        let t = 1.0 + f64::from(step) * 0.01; // sweep the overlap [1, 2]
        let sum = lane.weight_at(0, t) + lane.weight_at(1, t);
        assert!(
            (sum - 1.0).abs() < 1e-12,
            "weights sum to {sum} at t={t}, not 1 — the blend would sag"
        );
    }
}

/// An authored ease applies only where there is no neighbour to blend against.
/// Where there IS one, the overlap wins — the two never fight, and there are no
/// two numbers to keep in agreement (the ergonomic hole in Blender's NLA).
#[test]
fn an_overlap_overrides_the_authored_ease() {
    let mut a = strip(0.0);
    a.ease_out = 0.1; // authored: a fast 0.1 s fade
    let mut lane = ClipLane::new("Base");
    lane.insert(a);
    lane.insert(strip(1.0)); // overlaps by 1 s

    // If the authored 0.1 s ease had won, A would still be at full weight here.
    assert_eq!(
        lane.weight_at(0, 1.5),
        0.5,
        "the 1 s overlap defines the blend, not the authored 0.1 s ease"
    );
}

#[test]
fn an_authored_ease_ramps_a_lone_strip_in_and_out() {
    let mut s = strip(0.0);
    (s.ease_in, s.ease_out) = (0.5, 0.5);
    let mut lane = ClipLane::new("Base");
    lane.insert(s);

    assert_eq!(lane.weight_at(0, 0.0), 0.0, "starts silent");
    assert_eq!(lane.weight_at(0, 0.25), 0.5, "half way in");
    assert_eq!(lane.weight_at(0, 1.0), 1.0, "full in the middle");
    assert_eq!(lane.weight_at(0, 1.75), 0.5, "half way out");
}

#[test]
fn strips_stay_ordered_however_they_are_added() {
    let mut lane = ClipLane::new("Base");
    lane.insert(strip(4.0));
    lane.insert(strip(0.0));
    lane.insert(strip(2.0));
    let starts: Vec<f64> = lane.strips.iter().map(|s| s.t_start).collect();
    assert_eq!(
        starts,
        [0.0, 2.0, 4.0],
        "neighbours only mean something sorted"
    );
}

#[test]
fn a_lane_defaults_to_override_at_full_weight() {
    let lane = ClipLane::new("Base");
    assert_eq!(lane.mode, LaneMode::Override);
    assert_eq!(lane.weight, 1.0);
    assert!(!lane.muted);
    assert!(lane.strips.is_empty());
}

// ── The document-level authoring API (ADR-0115, slice B's data seam) ─────────

/// **Deleting a clip must not re-point somebody else's strip.** Clips live in a
/// `Vec` and a strip names one by INDEX, so removing one slides every later clip
/// down. Without a fix-up, every strip above the hole quietly starts playing its
/// neighbour — a wrong animation, with nothing on screen to say why.
#[test]
fn deleting_a_clip_repoints_the_strips_above_it_and_drops_its_own() {
    use ph2d_timeline::TimelineDoc;
    let mut doc = TimelineDoc::new(); // clip 0 = "Main"
    doc.add_clip("B".to_string()); // 1
    doc.add_clip("C".to_string()); // 2
    let lane = doc.add_lane("L".to_string()).unwrap();
    doc.add_strip(lane, 0, 0.0, 1.0).unwrap();
    let doomed = doc.add_strip(lane, 1, 1.0, 2.0).unwrap();
    let above = doc.add_strip(lane, 2, 2.0, 3.0).unwrap();

    assert!(doc.remove_clip(1), "clip B is gone");

    assert_eq!(
        doc.strip(lane, doomed),
        None,
        "the strip that played it goes with it — a dead item paints and cannot evaluate"
    );
    assert_eq!(
        doc.strip(lane, above).and_then(|s| s.source.clip_index()),
        Some(1),
        "clip C slid from index 2 to 1, and its strip followed"
    );
    assert_eq!(doc.stack()[lane].strips.len(), 2);
}

/// A strip is grabbed by identity, never by position: dragging one past its
/// neighbour renumbers both, and an index-anchored drag would swap victims at the
/// exact instant the animator is watching most closely.
#[test]
fn a_strip_keeps_its_identity_when_a_drag_reorders_the_lane() {
    use ph2d_timeline::TimelineDoc;
    let mut doc = TimelineDoc::new();
    let lane = doc.add_lane("L".to_string()).unwrap();
    let first = doc.add_strip(lane, 0, 0.0, 1.0).unwrap();
    let second = doc.add_strip(lane, 0, 2.0, 3.0).unwrap();
    assert_eq!(doc.stack()[lane].index_of(first), Some(0));

    // Drag the first strip past the second.
    doc.strip_mut(lane, first).unwrap().t_start = 5.0;
    doc.strip_mut(lane, first).unwrap().t_end = 6.0;
    doc.stack_mut()[lane].resort();

    assert_eq!(
        doc.stack()[lane].index_of(first),
        Some(1),
        "it moved to the back"
    );
    assert_eq!(doc.stack()[lane].index_of(second), Some(0));
    assert_eq!(
        doc.strip(lane, first).map(|s| s.t_start),
        Some(5.0),
        "and it is still the strip we grabbed"
    );
}

#[test]
fn a_new_strip_is_as_long_as_the_clip_it_plays() {
    use ph2d_anim::{AnimValue, Interp, RationalTime};
    use ph2d_timeline::{PropKind, TimelineDoc};
    let mut doc = TimelineDoc::new();
    // A clip whose duration was never authored: it is as long as its last key.
    doc.insert_key(
        7,
        PropKind::TranslationX,
        RationalTime::from_seconds(3.0),
        AnimValue::Float(1.0),
        Interp::Linear,
    );
    let lane = doc.add_lane("L".to_string()).unwrap();
    let id = doc.add_strip(lane, 0, 0.0, 3.0).unwrap();
    assert_eq!(
        doc.strip(lane, id).map(|s| s.src_out),
        Some(3.0),
        "sized to the clip, not to zero — a zero-length strip is one nobody can grab"
    );
}

#[test]
fn the_lane_count_is_bounded_because_the_panel_ids_are() {
    use ph2d_timeline::{MAX_LANES, TimelineDoc};
    let mut doc = TimelineDoc::new();
    for _ in 0..MAX_LANES {
        assert!(doc.add_lane("L".to_string()).is_some());
    }
    assert_eq!(
        doc.add_lane("one too many".to_string()),
        None,
        "the doc refuses what the panel cannot address"
    );
}

// ── Intents: one gesture, one undo step (ADR-0115 acceptance §3.9's data half) ─

mod intents {
    use ph2d_core::Playhead;
    use ph2d_timeline::{
        ClipStrip, LaneMode, StripLoop, TimelineIntent as I, TimelineState, apply_intent,
    };

    fn app() -> (TimelineState, Playhead) {
        (TimelineState::default(), Playhead::default())
    }

    fn run(st: &mut TimelineState, ph: &mut Playhead, i: I) {
        apply_intent(st, ph, i);
    }

    /// A strip authored, dragged, trimmed and deleted — each one gesture, each one
    /// Ctrl+Z. Nothing here touches the panel: intents are pure over
    /// `(state, playhead)`, so the whole feature is testable with no UI at all.
    #[test]
    fn every_stack_intent_is_exactly_one_undo_step() {
        let (mut st, mut ph) = app();
        run(&mut st, &mut ph, I::AddLane);
        assert_eq!(st.doc.stack().len(), 1);

        run(
            &mut st,
            &mut ph,
            I::AddStrip {
                lane: 0,
                clip: 0,
                t_start: 1.0,
                t_end: 3.0,
            },
        );
        let id = st.doc.stack()[0].strips[0].id;

        run(
            &mut st,
            &mut ph,
            I::MoveStrip {
                lane: 0,
                id,
                t_start: 5.0,
            },
        );
        assert_eq!(st.doc.strip(0, id).map(ClipStrip::span), Some(2.0), "rigid");

        run(&mut st, &mut ph, I::Undo);
        assert_eq!(
            st.doc.strip(0, id).map(|s| s.t_start),
            Some(1.0),
            "one Ctrl+Z puts the strip back"
        );
        run(&mut st, &mut ph, I::Undo);
        assert!(
            st.doc.stack()[0].strips.is_empty(),
            "and one more removes it"
        );
        run(&mut st, &mut ph, I::Undo);
        assert!(st.doc.stack().is_empty(), "and one more, the lane");
    }

    /// **A trim is not a stretch.** Dragging the start edge one second in must HIDE
    /// the clip's first second, not squeeze the whole clip into a smaller box. The
    /// tell is `src_in`: it has to travel with the edge.
    #[test]
    fn trimming_an_edge_moves_the_source_slice_with_it() {
        let (mut st, mut ph) = app();
        run(&mut st, &mut ph, I::AddLane);
        run(
            &mut st,
            &mut ph,
            I::AddStrip {
                lane: 0,
                clip: 0,
                t_start: 0.0,
                t_end: 4.0,
            },
        );
        let id = st.doc.stack()[0].strips[0].id;
        st.doc.strip_mut(0, id).unwrap().src_out = 4.0; // a 4 s clip, played 1:1

        run(
            &mut st,
            &mut ph,
            I::TrimStrip {
                lane: 0,
                id,
                edge: 0,
                t: 1.0,
                from: 0.0,
            },
        );
        let s = st.doc.strip(0, id).unwrap();
        assert_eq!(s.t_start, 1.0);
        assert_eq!(
            s.src_in, 1.0,
            "the clip's first second is HIDDEN, not compressed"
        );
        assert_eq!(s.src_out, 4.0, "and the rest of the slice did not move");
    }

    #[test]
    fn an_edge_cannot_cross_the_other_and_paint_the_strip_inside_out() {
        let (mut st, mut ph) = app();
        run(&mut st, &mut ph, I::AddLane);
        run(
            &mut st,
            &mut ph,
            I::AddStrip {
                lane: 0,
                clip: 0,
                t_start: 0.0,
                t_end: 2.0,
            },
        );
        let id = st.doc.stack()[0].strips[0].id;

        run(
            &mut st,
            &mut ph,
            I::TrimStrip {
                lane: 0,
                id,
                edge: 0,
                t: 9.0, // way past the end
                from: 0.0,
            },
        );
        let s = st.doc.strip(0, id).unwrap();
        assert!(
            s.t_start < s.t_end,
            "a strip of negative span covers no time"
        );
    }

    /// Dragging a strip past its neighbour renumbers the lane — and the ORDER is
    /// what the crossfade reads to find that neighbour. `settle()` restores it at
    /// the single choke point, so the drawn blend and the evaluated one cannot
    /// disagree.
    #[test]
    fn a_strip_dragged_past_its_neighbour_leaves_the_lane_sorted() {
        let (mut st, mut ph) = app();
        run(&mut st, &mut ph, I::AddLane);
        for (a, b) in [(0.0, 2.0), (4.0, 6.0)] {
            run(
                &mut st,
                &mut ph,
                I::AddStrip {
                    lane: 0,
                    clip: 0,
                    t_start: a,
                    t_end: b,
                },
            );
        }
        let first = st.doc.stack()[0].strips[0].id;
        run(
            &mut st,
            &mut ph,
            I::MoveStrip {
                lane: 0,
                id: first,
                t_start: 8.0,
            },
        );

        let starts: Vec<f64> = st.doc.stack()[0].strips.iter().map(|s| s.t_start).collect();
        assert_eq!(starts, [4.0, 8.0], "the lane re-sorted itself");
        assert_eq!(
            st.doc.stack()[0].index_of(first),
            Some(1),
            "and the strip we dragged is the one that moved"
        );
    }

    #[test]
    fn lane_and_strip_settings_round_trip_through_intents() {
        let (mut st, mut ph) = app();
        run(&mut st, &mut ph, I::AddLane);
        run(
            &mut st,
            &mut ph,
            I::AddStrip {
                lane: 0,
                clip: 0,
                t_start: 0.0,
                t_end: 2.0,
            },
        );
        let id = st.doc.stack()[0].strips[0].id;

        run(
            &mut st,
            &mut ph,
            I::SetLaneMode {
                lane: 0,
                mode: LaneMode::Additive,
            },
        );
        run(
            &mut st,
            &mut ph,
            I::SetLaneWeight {
                lane: 0,
                weight: 9.0, // out of range on purpose
            },
        );
        run(
            &mut st,
            &mut ph,
            I::SetLaneMuted {
                lane: 0,
                muted: true,
            },
        );
        run(
            &mut st,
            &mut ph,
            I::SetStripLoop {
                lane: 0,
                id,
                loop_mode: StripLoop::PingPong,
            },
        );
        run(
            &mut st,
            &mut ph,
            I::SetStripSpeed {
                lane: 0,
                id,
                speed: 0.0, // a strip frozen on one frame is not a speed
            },
        );

        let lane = &st.doc.stack()[0];
        assert_eq!(lane.mode, LaneMode::Additive);
        assert_eq!(lane.weight, 1.0, "weight is clamped, not trusted");
        assert!(lane.muted);
        assert_eq!(lane.strips[0].loop_mode, StripLoop::PingPong);
        assert!(lane.strips[0].speed > 0.0, "speed never reaches zero");
    }

    /// A lane holding one 2-second strip of a 2-second clip, at real time.
    fn app_with_a_strip() -> (TimelineState, Playhead, ph2d_timeline::StripId) {
        use ph2d_anim::{AnimValue, Interp, RationalTime};
        use ph2d_timeline::PropKind;
        let (mut st, mut ph) = app();
        // The clip's duration is its last key — a clip of nothing is a strip with
        // no slice, and a strip with no slice has no rate to stretch.
        st.doc.insert_key(
            7,
            PropKind::TranslationX,
            RationalTime::from_seconds(2.0),
            AnimValue::Float(1.0),
            Interp::Linear,
        );
        run(&mut st, &mut ph, I::AddLane);
        run(
            &mut st,
            &mut ph,
            I::AddStrip {
                lane: 0,
                clip: 0,
                t_start: 0.0,
                t_end: 2.0,
            },
        );
        let id = st.doc.stack()[0].strips[0].id;
        (st, ph, id)
    }

    /// **The stretch is the trim's opposite, and that is the whole point.**
    ///
    /// Dragging the end edge from 2 s to 4 s with the modifier held must play the
    /// SAME two seconds of clip over twice the time — slice untouched, rate halved.
    /// The identical drag without the modifier hides nothing and reveals two more
    /// seconds of source at rate 1. Asserting both here, side by side, is the only
    /// way to prove they did not quietly become the same edit.
    #[test]
    fn stretching_an_edge_holds_the_slice_and_changes_the_rate() {
        let (mut st, mut ph, id) = app_with_a_strip();
        let slice = st.doc.strip(0, id).map(ClipStrip::slice);

        run(
            &mut st,
            &mut ph,
            I::StretchStrip {
                lane: 0,
                id,
                edge: 1,
                t: 4.0,
                from: 2.0,
            },
        );
        let s = st.doc.strip(0, id).unwrap();
        assert_eq!(s.span(), 2.0 * 2.0, "the box doubled");
        assert_eq!(Some(s.slice()), slice, "and the source slice did NOT move");
        assert!(
            (s.speed - 0.5).abs() < 1e-12,
            "twice the box, same content: half speed, got {}",
            s.speed
        );
        assert_eq!(s.t_start, 0.0, "the edge you are not dragging stays put");

        // And the same gesture WITHOUT the modifier is a trim: the rate is
        // untouched and the slice grows to fill the new box.
        run(&mut st, &mut ph, I::Undo);
        run(
            &mut st,
            &mut ph,
            I::TrimStrip {
                lane: 0,
                id,
                edge: 1,
                t: 4.0,
                from: 2.0,
            },
        );
        let s = st.doc.strip(0, id).unwrap();
        assert_eq!(s.speed, 1.0, "a trim never retimes");
        assert!(s.slice() > slice.unwrap(), "it reveals more source");
    }

    /// Stretching from the START edge pins the END — the frame under the edge you
    /// are not touching must not move, or the strip walks across the timeline while
    /// you retime it.
    #[test]
    fn stretching_the_start_edge_pins_the_end() {
        let (mut st, mut ph, id) = app_with_a_strip();
        run(
            &mut st,
            &mut ph,
            I::MoveStrip {
                lane: 0,
                id,
                t_start: 4.0,
            },
        ); // [4, 6)
        run(
            &mut st,
            &mut ph,
            I::StretchStrip {
                lane: 0,
                id,
                edge: 0,
                t: 2.0,
                from: 4.0,
            },
        );
        let s = st.doc.strip(0, id).unwrap();
        assert_eq!(s.t_end, 6.0, "the end is pinned");
        assert_eq!(s.t_start, 2.0);
        assert!((s.speed - 0.5).abs() < 1e-12, "4 s of box, 2 s of clip");
    }

    /// **Reset Speed re-lengthens the box.** Setting the rate back to 1 while
    /// leaving a stretched span would leave the strip playing a slice that no
    /// longer fits it — the source would run out mid-span and the number in the
    /// menu would describe a strip nobody authored. Rate and span are one edit.
    #[test]
    fn resetting_the_speed_restores_real_time_and_the_span_with_it() {
        let (mut st, mut ph, id) = app_with_a_strip();
        run(
            &mut st,
            &mut ph,
            I::StretchStrip {
                lane: 0,
                id,
                edge: 1,
                t: 8.0,
                from: 2.0,
            },
        );
        assert!((st.doc.strip(0, id).unwrap().speed - 0.25).abs() < 1e-12);

        run(
            &mut st,
            &mut ph,
            I::SetStripSpeed {
                lane: 0,
                id,
                speed: 1.0,
            },
        );
        let s = st.doc.strip(0, id).unwrap();
        assert_eq!(s.speed, 1.0);
        assert_eq!(
            s.span(),
            s.slice(),
            "at real time the box is exactly as long as what it plays"
        );
    }

    /// A duplicate lands BUTTED AGAINST its original, never on top of it: a copy
    /// overlapping its source would come up as a crossfade of a clip with itself —
    /// a null edit dressed as a blend. End-to-start it reads as a repeat, and
    /// dragging it left BY HAND is the crossfade (that is the gesture, ADR-0115 R1).
    #[test]
    fn a_duplicate_lands_next_to_its_original_not_on_top_of_it() {
        let (mut st, mut ph, id) = app_with_a_strip();
        run(&mut st, &mut ph, I::DuplicateStrip { lane: 0, id });

        let lane = &st.doc.stack()[0];
        assert_eq!(lane.strips.len(), 2);
        let (a, b) = (&lane.strips[0], &lane.strips[1]);
        assert_ne!(a.id, b.id, "a copy is a NEW strip, not an alias");
        assert_eq!(b.t_start, a.t_end, "butted, not overlapping");
        assert_eq!(b.span(), a.span());
        assert_eq!(
            lane.blend_in(1),
            0.0,
            "and therefore no crossfade was invented on the animator's behalf"
        );

        run(&mut st, &mut ph, I::Undo);
        assert_eq!(st.doc.stack()[0].strips.len(), 1, "one gesture, one Ctrl+Z");
    }
}

// ── The invariant `slice == span * speed` must be TRUE at birth ─────────────

/// A clip SHORTER than the panel's minimum strip length used to be dropped into a
/// 1 s box at speed 1.0 — a strip whose slice does not fill its span. Nothing
/// complained until the first stretch: `stretch_strip` re-derives the rate from the
/// span, so a 1 ms drag snapped a 0.4 s clip from speed 1.0 to 0.4 and the frame
/// under the playhead jumped from 0.2 s to 0.08 s of clip time. The invariant holds
/// at birth or it does not hold at all.
#[test]
fn a_strip_is_born_with_the_invariant_its_stretch_assumes() {
    use ph2d_anim::{AnimValue, Interp, RationalTime};
    use ph2d_timeline::{PropKind, TimelineDoc};
    let mut doc = TimelineDoc::new();
    // A clip 0.4 s long — shorter than any floor the panel might apply.
    doc.insert_key(
        7,
        PropKind::TranslationX,
        RationalTime::from_seconds(0.4),
        AnimValue::Float(1.0),
        Interp::Linear,
    );
    let lane = doc.add_lane("L".to_string()).unwrap();
    // The caller asks for a 1 s span (what a floor would have produced).
    let id = doc.add_strip(lane, 0, 0.0, 1.0).unwrap();
    let s = doc.strip(lane, id).unwrap();

    assert!(
        (s.slice() - s.span() * s.speed).abs() < 1e-12,
        "slice {} != span {} * speed {}",
        s.slice(),
        s.span(),
        s.speed
    );
    // And the frame it shows is the frame the span says it shows.
    assert!((s.source_time(0.5).unwrap() - 0.2).abs() < 1e-12);
}

/// **The lock the panel greys out means EXACTLY what the evaluator does with the number**
/// (ADR-0115 B4).
///
/// The corner handle authors `ease_*`, and it is read-only where an overlap defines the
/// window. That "where" is a judgement call made twice — once by the panel (which handle
/// to offer) and once by the evaluator (which number to honour) — so it is asked ONCE,
/// here: `neighbour_reach_*`. Let the two drift apart and the artist gets a live handle
/// that does nothing, or is denied one that would have worked.
#[test]
fn the_ease_lock_means_exactly_what_the_evaluator_does_with_the_ease() {
    let mut a = strip(0.0); // [0, 2)
    (a.ease_in, a.ease_out) = (0.5, 0.5);
    let mut lane = ClipLane::new("Base");
    lane.insert(a);
    lane.insert(strip(1.0)); // [1, 3): overlaps A's END by 1 s, not its start

    // A's END: a neighbour owns it. The panel locks the handle…
    assert!(
        lane.neighbour_reach_out(0) > 0.0,
        "the panel must grey the fade-out handle"
    );
    // …and the evaluator ignores the authored 0.5 s, taking the 1 s overlap instead.
    assert_eq!(
        lane.blend_out(0),
        1.0,
        "the overlap defines the window — the authored ease is not read"
    );

    // A's START: nothing reaches it. The panel offers the handle…
    assert_eq!(
        lane.neighbour_reach_in(0),
        0.0,
        "nothing overlaps A's start: the fade-in handle is the artist's"
    );
    // …and the evaluator honours exactly the number that handle writes.
    assert_eq!(
        lane.blend_in(0),
        0.5,
        "and it is the authored ease that shapes the ramp"
    );
}

/// **Duas fades não podem tomar o strip duas vezes** (B4).
///
/// `weight_at` é `mixIn(t) * mixOut(t)` (a forma da Unity), então fades que se SOBREPÕEM
/// multiplicam: um strip de 2 s com 2 s de fade-in e 2 s de fade-out — cada uma perfeitamente
/// legal sozinha — teria pico de peso **0,25** e nunca chegaria a 1. Numa faixa `Override` isso
/// é um sprite permanentemente meio-misturado com a pose de baixo, sem nada na tela explicando
/// por quê. O clamp é na SOMA, e é por isso.
#[test]
fn two_fades_cannot_take_the_strip_twice() {
    use ph2d_timeline::{StripId, TimelineIntent as I, TimelineState, apply_intent};
    let mut st = TimelineState::new();
    apply_intent(
        &mut st,
        &mut ph2d_core::Playhead::new(1.0 / 60.0),
        I::AddLane,
    );
    apply_intent(
        &mut st,
        &mut ph2d_core::Playhead::new(1.0 / 60.0),
        I::AddStrip {
            lane: 0,
            clip: 0,
            t_start: 0.0,
            t_end: 2.0,
        },
    );
    let id = st.doc.stack()[0].strips[0].id;
    let span = st.doc.stack()[0].strips[0].span();
    // O artista arrasta as DUAS alças o mais fundo que consegue.
    for edge in [0, 1] {
        apply_intent(
            &mut st,
            &mut ph2d_core::Playhead::new(1.0 / 60.0),
            I::SetStripEase {
                lane: 0,
                id: StripId(id.0),
                edge,
                seconds: 999.0,
            },
        );
    }
    let s = &st.doc.stack()[0].strips[0];
    assert!(
        s.ease_in + s.ease_out <= span + 1e-9,
        "a soma das fades não pode passar do strip: {} + {} > {span}",
        s.ease_in,
        s.ease_out
    );
    // E o que importa de verdade: o strip AINDA chega a peso cheio em algum lugar.
    let lane = &st.doc.stack()[0];
    let peak = (0..=200)
        .map(|i| lane.weight_at(0, s.t_start + span * f64::from(i) / 200.0))
        .fold(0.0_f64, f64::max);
    assert!(
        peak > 0.999,
        "o strip nunca chega a peso cheio (pico {peak}) — na Override o sprite fica \
         permanentemente meio-misturado com a pose de baixo"
    );
}

/// **Um clip de 5 s adicionado vira um strip de 5 s a speed 1 — não um de 1 s a 5×.**
///
/// O bug que o Enio achou: uma animação de 5 s, ao virar strip, tocava inteira no primeiro
/// segundo, acelerada 5×. Causa: DUAS PORTAS para "qual o tamanho do clip?". O botão "+strip" lia
/// a duração AUTORADA (`duration()` = 0 para um clip feito à mão) e caía num mínimo de 1 s; o
/// `add_strip` dimensionava a FATIA pela última key (5 s). Fatia 5 / span 1 = speed 5.
///
/// Este gate atravessa a costura real: o snapshot (o que o botão lê) e o `add_strip` (o que o
/// botão chama) têm de concordar. FALSIFICADO por repopular `clip_length_seconds` de `duration()`.
#[test]
fn a_five_second_clip_becomes_a_five_second_strip_at_real_time() {
    use ph2d_anim::{AnimValue, Interp, RationalTime};
    use ph2d_core::Playhead;
    use ph2d_timeline::{PropKind, TimelineState, TimelineViewSnapshot};

    let mut state = TimelineState::new();
    // Uma animação feita à mão: uma key a 5 s. A duração AUTORADA do clip continua 0.
    state.doc.insert_key(
        7,
        PropKind::TranslationX,
        RationalTime::from_seconds(5.0),
        AnimValue::Float(1.0),
        Interp::Linear,
    );
    assert_eq!(
        state.doc.active_clip().duration().to_seconds(),
        0.0,
        "premissa: um clip feito à mão não tem duração autorada"
    );

    // O que o botão "+strip" LÊ.
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut state, &Playhead::new(1.0 / 60.0), false);
    assert_eq!(
        snap.clip_length_seconds, 5.0,
        "o botão tem de ver 5 s (a última key), não 0 (a duração autorada)"
    );

    // O que o botão FAZ com isso: add_strip no mesmo comprimento.
    let lane = state.doc.add_lane("L".to_string()).unwrap();
    let id = state
        .doc
        .add_strip(lane, 0, 0.0, snap.clip_length_seconds)
        .unwrap();
    let s = state.doc.strip(lane, id).unwrap();
    assert!(
        (s.speed - 1.0).abs() < 1e-9,
        "o strip nasce em tempo REAL, não espremido: speed {}",
        s.speed
    );
    assert!(
        (s.span() - 5.0).abs() < 1e-9 && (s.slice() - 5.0).abs() < 1e-9,
        "span == fatia == 5 s: a animação ocupa os 5 s que ela dura"
    );
}
