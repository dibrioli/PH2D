//! Integration test for the Make Square island.
//!
//! The new module lives at `src/tools/make_square/` but is not yet
//! re-exported through `tools/mod.rs` (that wiring is the
//! Coordenador's job per `INTEGRATION.md`). To still exercise the
//! algorithm + icon under `cargo test -p ph2d-editor`, we pull the
//! source files in via `#[path]` so they compile as a fresh module
//! inside this test binary. `algorithm.rs` is `std`-only;
//! `icon.rs` pulls `ph2d_vector` (`BezPath`, `RoundedRect`, `Shape`),
//! a regular dependency of `ph2d-editor` and therefore available in
//! integration-test builds.
//!
//! Same shape as `tests/trim_transparency_algorithm.rs` (slug-prefix
//! convention from §7.1 of `03-Agente-Periferico.md`).

#[path = "../src/tools/make_square/algorithm.rs"]
mod algorithm;

#[path = "../src/tools/make_square/icon.rs"]
mod icon;

use algorithm::make_square;

#[test]
fn behaviour_wider_than_tall_preserves_original_pixels_at_centered_offset() {
    // 8 wide × 4 tall RGBA buffer with an opaque rectangle at
    // (2, 1)..(5, 2). After make_square: 8×8 canvas with
    // offset_y = (8-4)/2 = 2. The original rectangle now lands at
    // (2, 1+2)..(5, 2+2) with identical RGBA bytes; padded rows
    // (y=0..1 and y=6..7) are fully transparent.
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
    assert_eq!(r.pixels.len(), s * s * 4);

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
fn behaviour_already_square_is_noop_with_made_square_false() {
    let rgba = vec![42u8; 16 * 16 * 4];
    let r = make_square(&rgba, 16, 16);
    assert!(!r.made_square);
    assert_eq!(r.size, 16);
    assert_eq!(r.offset_x, 0);
    assert_eq!(r.offset_y, 0);
    assert_eq!(r.pixels, rgba);
}

#[test]
fn behaviour_degenerate_input_returns_1x1_transparent_sentinel() {
    let r = make_square(&[], 0, 32);
    assert_eq!(r.size, 1);
    assert_eq!(r.pixels, vec![0, 0, 0, 0]);
    assert!(r.made_square);
}

#[test]
fn icon_module_compiles_and_returns_non_empty_path() {
    // Black-box smoke: the icon helper is callable from outside the
    // source module and yields a non-empty path. Shape detail lives
    // in the unit tests inside `icon.rs`.
    let path = icon::square_bezpath();
    assert!(!path.elements().is_empty());
}
