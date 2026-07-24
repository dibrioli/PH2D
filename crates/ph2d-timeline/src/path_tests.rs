//! Gates for [`MotionPath`] — the trajectory of a Position binding (ADR-0141).

use super::*;

/// A swooping two-segment path whose parameterisation is deliberately UNEVEN: the
/// handles are lopsided, so a curve walked by `t` bunches up where a curve walked by
/// arc length does not. Several gates below only mean something on a fixture that
/// contains that phenomenon.
fn swoop() -> MotionPath {
    MotionPath::new(vec![
        PathAnchor {
            anchor: [0.0, 0.0],
            in_handle: [0.0, 0.0],
            out_handle: [0.0, 6.0],
        },
        PathAnchor {
            anchor: [10.0, 10.0],
            in_handle: [-1.0, 0.0],
            out_handle: [4.0, 0.0],
        },
        PathAnchor {
            anchor: [20.0, 2.0],
            in_handle: [-0.5, 5.0],
            out_handle: [0.0, 0.0],
        },
    ])
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// **The single door.** Moving one anchor changes where every anchor after it sits
/// along the path, and the table that says so is rewritten in the same call — never
/// left for a caller to remember.
#[test]
fn moving_an_anchor_rewrites_the_arclengths_in_one_operation() {
    let mut p = swoop();
    let before = p.arclen_at(2).unwrap();
    let total_before = p.length();

    // Drag the middle anchor far off to the side: the trip to anchor 2 gets longer.
    let mut mid = p.anchors()[1];
    mid.anchor = [10.0, 40.0];
    assert!(p.set_anchor(1, mid));

    let after = p.arclen_at(2).unwrap();
    assert!(
        after > before + 20.0,
        "anchor 2 sits {after:.3} along the path after the drag, {before:.3} before it: \
         the distances did not follow the geometry"
    );
    assert!(
        (p.length() - after).abs() < 1e-9,
        "the last anchor's distance IS the total length"
    );
    assert!(p.length() > total_before);

    // Anchor 0 is the origin no matter what anyone drags.
    assert_eq!(p.arclen_at(0), Some(0.0));
    assert_eq!(p.arclen_at(3), None, "there is no fourth anchor");
}

/// The rebuild cannot be forgotten because there is exactly one place that can
/// mutate the anchors, and it rebuilds. This reads the source rather than the
/// behaviour: a second `&mut self.anchors` would be invisible to every other gate
/// here until the day someone used it without rebuilding.
#[test]
fn only_one_place_in_this_file_borrows_the_anchors_mutably() {
    let src = include_str!("path.rs");
    let n = src.matches("&mut self.anchors").count();
    assert_eq!(
        n, 1,
        "found {n} mutable borrows of the anchor list; `edit` is meant to be the only one \
         (it is what guarantees the arc-length table is never stale)"
    );
    // ...and the control: the string is really there, so a rename cannot make this
    // gate pass by matching nothing.
    assert!(src.contains("pub fn edit<R>"));
}

/// **Seed == sample, in geometry.** The number a key holds for anchor `i` is
/// `arclen_at(i)`; feeding exactly that number back to the sampler has to land on
/// that anchor, or the object would sit somewhere the artist never put a key.
#[test]
fn a_distance_lands_on_the_anchor_that_measures_it() {
    let p = swoop();
    let mut worst = 0.0f32;
    for i in 0..p.len() {
        // Through `f32`, because that is the trip the number really makes: the
        // distance is computed in `f64` and PARKED IN A KEY, whose value is an
        // `AnimValue::Float`. Handing the sampler the `f64` straight back tests a
        // path the product never takes — and lands on exactly zero every time, which
        // is the shape of a gate that is measuring nothing.
        let s = f64::from(p.arclen_at(i).unwrap() as f32);
        let got = p.at(s).unwrap().point;
        worst = worst.max(dist(got, p.anchors()[i].anchor));
    }
    println!("MEASURED worst anchor miss = {worst:e}");
    assert!(
        worst < 1e-5,
        "worst anchor miss {worst:e} world units; the sampler and the key disagree"
    );
}

/// **Why arc length at all.** Equal distances must cover equal ground — that is the
/// single property that separates this from walking the curve by `t`, and the one an
/// artist sees as constant speed. The control on the same curve is the whole gate:
/// without it, "the spread is small" would be a claim about the fixture.
#[test]
fn equal_distances_cover_equal_ground_where_equal_t_does_not() {
    let p = swoop();
    let n = 40;

    let spread = |pts: &[[f32; 2]]| {
        let steps: Vec<f32> = pts.windows(2).map(|w| dist(w[0], w[1])).collect();
        let max = steps.iter().copied().fold(0.0f32, f32::max);
        let min = steps.iter().copied().fold(f32::INFINITY, f32::min);
        max / min
    };

    let by_arclen: Vec<[f32; 2]> = (0..=n)
        .map(|k| p.at(p.length() * k as f64 / n as f64).unwrap().point)
        .collect();

    // The control: the SAME curve walked by the parameter, which is what an
    // implementation that skipped the arc-length table would produce.
    let seg = p.segment(0);
    let by_t: Vec<[f32; 2]> = (0..=n)
        .map(|k| {
            let q = point_at(&seg, k as f64 / n as f64);
            [q[0] as f32, q[1] as f32]
        })
        .collect();

    let (a, t) = (spread(&by_arclen), spread(&by_t));
    println!("MEASURED spread: arclen {a:.4}x, by-t {t:.4}x");
    // Measured: 1.007x by arc length, 5.05x by parameter, on this fixture.
    assert!(
        a < 1.15,
        "steps of equal distance varied by {a:.2}x - that is not constant speed"
    );
    assert!(
        t > 2.0 * a,
        "the fixture does not contain the phenomenon: walking it by t varied by only \
         {t:.2}x against {a:.2}x by arc length, so this gate would pass on a broken engine"
    );
}

/// [`MotionPath::project`] is the inverse of [`MotionPath::at`] for a point that is
/// not on the curve — which is what a `rest` pose is, and what a click is.
#[test]
fn projecting_a_point_beside_the_path_finds_the_distance_it_is_beside() {
    let p = swoop();
    for frac in [0.0, 0.17, 0.5, 0.83, 1.0] {
        let s = p.length() * frac;
        let on = p.at(s).unwrap();
        let tan = on.tangent.unwrap();
        // Step off the path along its own normal, so the nearest point is exactly
        // the one we started from.
        let off = [on.point[0] - tan[1] * 0.35, on.point[1] + tan[0] * 0.35];
        let back = p.project(off).unwrap();
        assert!(
            (back - s).abs() < 1e-3,
            "projected {back:.6} where the point was placed beside {s:.6}"
        );
    }
    // A point far off the end projects to the end, not past it.
    let far = p.project([200.0, 200.0]).unwrap();
    assert!((0.0..=p.length()).contains(&far));
}

/// The file carries the anchors and nothing else: the arc-length table is derived,
/// and a derived number in a file is a second copy that can disagree with the first.
#[test]
fn a_path_travels_as_its_anchors_alone_and_the_table_is_rebuilt() {
    let p = swoop();
    let bytes = postcard::to_allocvec(&p).unwrap();
    let bare = postcard::to_allocvec(p.anchors()).unwrap();
    assert_eq!(
        bytes, bare,
        "the encoded path is not byte-identical to its bare anchor list, so the derived \
         table is riding along in the file"
    );

    let back: MotionPath = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(back, p, "including the table, which the load rebuilt");
    assert!((back.length() - p.length()).abs() < 1e-12);
}

/// Degenerate paths answer instead of panicking: a Position track can exist before
/// its second key does.
#[test]
fn an_empty_or_pointlike_path_answers_without_panicking() {
    let empty = MotionPath::new(vec![]);
    assert!(empty.is_empty());
    assert_eq!(empty.length(), 0.0);
    assert_eq!(empty.at(0.0), None);
    assert_eq!(empty.project([1.0, 2.0]), None);

    let dot = MotionPath::new(vec![PathAnchor::corner([3.0, 4.0])]);
    assert_eq!(dot.length(), 0.0);
    let s = dot.at(99.0).unwrap();
    assert_eq!(s.point, [3.0, 4.0]);
    assert_eq!(s.tangent, None, "a point has no direction");
    assert_eq!(dot.project([9.0, 9.0]), Some(0.0));

    // Two anchors in the same place: zero length, and the sampler still answers.
    let squashed = MotionPath::new(vec![
        PathAnchor::corner([1.0, 1.0]),
        PathAnchor::corner([1.0, 1.0]),
    ]);
    assert_eq!(squashed.length(), 0.0);
    assert_eq!(squashed.at(0.0).unwrap().point, [1.0, 1.0]);
}

/// Auto Bezier is the default a fresh spatial key is born with (ADR-0141 §4). It has
/// to round a corner without moving the anchor, and it has to leave a straight line
/// straight — an artist who keys three points in a row expects a line, not a wobble.
#[test]
fn auto_smooth_rounds_a_corner_and_leaves_a_straight_line_straight() {
    let corner = MotionPath::auto_smooth(Some([0.0, 0.0]), [10.0, 0.0], Some([10.0, 10.0]));
    assert_eq!(corner.anchor, [10.0, 0.0], "the anchor never moves");
    assert!(
        corner.in_handle[0] < 0.0 && corner.in_handle[1] < 0.0,
        "the incoming handle points back along the chord of the neighbours"
    );
    assert!(corner.out_handle[0] > 0.0 && corner.out_handle[1] > 0.0);

    // Collinear: the smoothed path through three points on a line stays that line.
    let pts = [[0.0f32, 0.0], [4.0, 0.0], [10.0, 0.0]];
    let line = MotionPath::new(
        (0..3)
            .map(|i| {
                MotionPath::auto_smooth(
                    (i > 0).then(|| pts[i - 1]),
                    pts[i],
                    (i + 1 < 3).then(|| pts[i + 1]),
                )
            })
            .collect(),
    );
    assert!(
        (line.length() - 10.0).abs() < 1e-6,
        "a smoothed straight line measured {:.6}, not 10",
        line.length()
    );
    for k in 0..=10 {
        let y = line.at(line.length() * f64::from(k) / 10.0).unwrap().point[1];
        assert!(y.abs() < 1e-5, "the line bulged to y = {y:e}");
    }
}
