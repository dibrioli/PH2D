//! ADR-0115 A1-A3: the strip's time map and the lane's blend weights.
//!
//! Everything here is asserted on a **sampled value**, never on a field being
//! present. A strip that stores a `speed` it never applies would pass a
//! field-shaped test and fail an animator.

use ph2d_timeline::{ClipLane, ClipStrip, LaneMode, StripLoop};

/// A strip playing 2 s of clip 0 over `[t0, t0 + 2)`.
fn strip(t0: f64) -> ClipStrip {
    ClipStrip::new(0, t0, t0 + 2.0, 2.0)
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
        doc.strip(lane, above).map(|s| s.clip),
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
