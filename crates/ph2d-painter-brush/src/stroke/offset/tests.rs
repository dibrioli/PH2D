//! Unit tests for the perpendicular path offset.

use super::*;

#[test]
fn zero_offset_is_identity() {
    let p = vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]];
    assert_eq!(offset_polyline(&p, 0.0, false), p);
    assert_eq!(offset_polyline(&p, 0.0, true), p);
}

#[test]
fn open_horizontal_segment_offsets_perpendicular() {
    // A horizontal A→B run: left-normal of +x is (0, +1) in image space, so +d moves the run down by d.
    let p = vec![[0.0, 0.0], [10.0, 0.0]];
    let o = offset_polyline(&p, 5.0, false);
    assert!(
        (o[0][1] - 5.0).abs() < 1e-4 && (o[1][1] - 5.0).abs() < 1e-4,
        "shifted +y by d: {o:?}"
    );
    assert!(
        (o[0][0] - 0.0).abs() < 1e-4 && (o[1][0] - 10.0).abs() < 1e-4,
        "x unchanged: {o:?}"
    );
}

#[test]
fn open_offset_flips_sign() {
    let p = vec![[0.0, 0.0], [10.0, 0.0]];
    let up = offset_polyline(&p, -5.0, false);
    assert!(
        (up[0][1] + 5.0).abs() < 1e-4,
        "negative d moves the other way: {up:?}"
    );
}

#[test]
fn closed_square_insets_uniformly() {
    // A CCW (image-space) square; the left-normal points inward, so a positive offset shrinks it toward
    // the centre. Corners move along the 45° diagonal (averaged normal of the two edges).
    let sq = vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
    let o = offset_polyline(&sq, 2.0, true);
    // Each corner moves inward by 2px along both axes (averaged unit normal is (±1,±1)/√2, ×√2 span... the
    // averaged-then-normalized normal is a 45° unit vector, so the corner shifts 2px along it = √2 each
    // axis ≈ 1.414). Assert the bbox shrank symmetrically.
    let minx = o.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
    let maxx = o.iter().map(|p| p[0]).fold(f32::NEG_INFINITY, f32::max);
    let span = maxx - minx;
    assert!(
        span < 10.0,
        "the closed loop offset inward (span {span} < 10)"
    );
    // Symmetric: the four corners stay a centred diamond/box around (5,5).
    let cx = o.iter().map(|p| p[0]).sum::<f32>() / 4.0;
    let cy = o.iter().map(|p| p[1]).sum::<f32>() / 4.0;
    assert!(
        (cx - 5.0).abs() < 1e-3 && (cy - 5.0).abs() < 1e-3,
        "centre preserved: ({cx},{cy})"
    );
}

#[test]
fn degenerate_short_input_is_returned_as_is() {
    let one = vec![[3.0, 4.0]];
    assert_eq!(offset_polyline(&one, 9.0, false), one);
    assert_eq!(
        offset_polyline(&Vec::new(), 9.0, true),
        Vec::<[f32; 2]>::new()
    );
}
