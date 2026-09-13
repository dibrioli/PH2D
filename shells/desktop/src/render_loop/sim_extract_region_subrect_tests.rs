use super::region_subrect;

const FULL: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

fn close(a: [f32; 4], b: [f32; 4]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-6)
}

#[test]
fn zero_or_negative_region_is_identity() {
    // A degenerate (zero-area) region is treated as "no region".
    assert_eq!(
        region_subrect(FULL, [0.0, 0.0, 0.0, 0.0], 64.0, 64.0, None),
        FULL
    );
    assert_eq!(
        region_subrect(FULL, [10.0, 10.0, -5.0, 8.0], 64.0, 64.0, None),
        FULL
    );
    // Unknown source dims (0) is also identity.
    assert_eq!(
        region_subrect(FULL, [0.0, 0.0, 32.0, 32.0], 0.0, 0.0, None),
        FULL
    );
}

#[test]
fn full_source_region_is_identity() {
    // region == whole source → the base rect, unchanged.
    assert!(close(
        region_subrect(FULL, [0.0, 0.0, 100.0, 100.0], 100.0, 100.0, None),
        FULL
    ));
}

#[test]
fn right_half_region_maps_to_right_half_uv() {
    // 100×100 source, region = right half → U in [0.5, 1.0].
    assert!(close(
        region_subrect(FULL, [50.0, 0.0, 50.0, 100.0], 100.0, 100.0, None),
        [0.5, 0.0, 1.0, 1.0]
    ));
}

#[test]
fn region_maps_into_a_non_unit_atlas_rect() {
    // The source occupies atlas-UV [0.2, 0.2, 0.6, 0.6] (a 0.4×0.4
    // patch); a centered quarter region [25,25,50,50] of the 100px
    // source → the middle 0.2×0.2 of that patch = [0.3, 0.3, 0.5, 0.5].
    let base = [0.2, 0.2, 0.6, 0.6];
    assert!(close(
        region_subrect(base, [25.0, 25.0, 50.0, 50.0], 100.0, 100.0, None),
        [0.3, 0.3, 0.5, 0.5]
    ));
}

#[test]
fn region_beyond_source_edges_is_clamped() {
    // A region wider than the source can't sample past the base rect.
    let r = region_subrect(FULL, [0.0, 0.0, 200.0, 200.0], 100.0, 100.0, None);
    assert!(close(r, FULL));
}

#[test]
fn filter_clip_insets_by_half_a_texel() {
    // 64px atlas texel = 1/64; half-texel = 1/128. Right-half region
    // [0.5,0,1,1] inset by 1/128 on every side.
    let ht = 0.5 / 64.0;
    let r = region_subrect(FULL, [32.0, 0.0, 32.0, 64.0], 64.0, 64.0, Some((ht, ht)));
    assert!((r[0] - (0.5 + ht)).abs() < 1e-6, "u0 {r:?}");
    assert!((r[2] - (1.0 - ht)).abs() < 1e-6, "u1 {r:?}");
    assert!((r[1] - ht).abs() < 1e-6, "v0 {r:?}");
    assert!((r[3] - (1.0 - ht)).abs() < 1e-6, "v1 {r:?}");
}

#[test]
fn sub_texel_region_collapses_to_center_under_filter_clip() {
    // A 1px-wide region on a 64px source spans 1/64 in U; insetting by
    // a half-texel (1/128) each side would invert → collapse to center.
    let ht = 0.5 / 64.0;
    let r = region_subrect(FULL, [10.0, 0.0, 1.0, 64.0], 64.0, 64.0, Some((ht, ht)));
    assert!((r[0] - r[2]).abs() < 1e-6, "U collapsed to a point: {r:?}");
    // Center sits at 10.5/64 in U.
    assert!((r[0] - 10.5 / 64.0).abs() < 1e-6, "{r:?}");
}
