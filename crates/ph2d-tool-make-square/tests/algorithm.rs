//! Black-box integration test for the Make Square algorithm. Mirrors
//! the legacy `crates/ph2d-editor/tests/make_square_algorithm.rs`
//! behaviour-coverage cases but consumes the public crate API instead
//! of `#[path]` private modules.

use ph2d_tool_make_square::{make_square, square_bezpath};

#[test]
fn wider_than_tall_preserves_original_pixels_at_centered_offset() {
    let w = 8usize;
    let h = 4usize;
    let mut rgba = vec![0u8; w * h * 4];
    for y in 1..=2 {
        for x in 2..=5 {
            let i = (y * w + x) * 4;
            rgba[i] = 200;
            rgba[i + 1] = 100;
            rgba[i + 2] = 50;
            rgba[i + 3] = 255;
        }
    }
    let r = make_square(&rgba, w as u32, h as u32);
    assert!(r.made_square);
    assert_eq!(r.size, 8);
    assert_eq!(r.offset_x, 0);
    assert_eq!(r.offset_y, 2);
    let s = r.size as usize;

    for y in [0usize, 1, 6, 7] {
        for x in 0..s {
            let i = (y * s + x) * 4;
            assert_eq!(&r.pixels[i..i + 4], &[0, 0, 0, 0], "pad y={y} x={x}");
        }
    }
    for y in 1..=2 {
        for x in 2..=5 {
            let i = ((y + 2) * s + x) * 4;
            assert_eq!(&r.pixels[i..i + 4], &[200, 100, 50, 255]);
        }
    }
}

#[test]
fn already_square_is_noop_with_made_square_false() {
    let rgba = vec![42u8; 16 * 16 * 4];
    let r = make_square(&rgba, 16, 16);
    assert!(!r.made_square);
    assert_eq!(r.size, 16);
    assert_eq!(r.offset_x, 0);
    assert_eq!(r.offset_y, 0);
    assert_eq!(r.pixels, rgba);
}

#[test]
fn degenerate_input_returns_1x1_transparent_sentinel() {
    let r = make_square(&[], 0, 32);
    assert_eq!(r.size, 1);
    assert_eq!(r.pixels, vec![0, 0, 0, 0]);
    assert!(r.made_square);
}

#[test]
fn icon_exposes_non_empty_bezpath() {
    let path = square_bezpath();
    assert!(!path.elements().is_empty());
}
