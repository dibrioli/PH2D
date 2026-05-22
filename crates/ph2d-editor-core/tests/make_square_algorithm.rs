//! Integration test for the Make Square island + its round-trip with
//! `trim_transparency`.
//!
//! Make Square lives in crate `ph2d-tool-make-square`; this test
//! consumes it as a `[dev-dependency]` (ADR-0040 T1.5 — the editor-core
//! facade was removed so foundation has no tool dep). The cross-island
//! round-trip with `trim_transparency` still pulls that algorithm in
//! via `#[path]` because trim has not migrated to a crate yet.
//!
//! Slug-prefix convention canonicalized in
//! `docs/IntegracaoMultiAgente/DIRETRIZ.md`.

// Both islands now live in their own crates (ADR-0040 T1.5 / T3); this
// test consumes their public APIs as dev-dependencies. The editor-core
// facades were removed so the foundation has no tool dep at runtime.
use ph2d_tool_make_square::{make_square, square_bezpath};
use ph2d_tool_trim_transparency::trim_transparency;

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
    let path = square_bezpath();
    assert!(!path.elements().is_empty());
}

/// Audit fix N1 — cross-island invariant. A sprite with opaque content
/// surrounded by transparent padding should survive a
/// Trim → MakeSquare → Trim cycle with the **same** bounding box and
/// **byte-identical opaque pixels** as the input. This is the
/// regression test for the "inverso exato" claim in `INTEGRATION.md` —
/// pad expands by transparent rows/cols only, trim recovers the bbox,
/// and the second trim should be a no-op (already-trimmed input).
#[test]
fn round_trip_trim_then_make_square_then_trim_preserves_bbox_and_pixels() {
    // 16×16 canvas with a 5×3 opaque rectangle at (4, 6).
    let w = 16usize;
    let h = 16usize;
    let mut rgba = vec![0u8; w * h * 4];
    let rect = (4usize, 6usize, 5usize, 3usize); // x, y, w, h
    for y in rect.1..rect.1 + rect.3 {
        for x in rect.0..rect.0 + rect.2 {
            let i = (y * w + x) * 4;
            rgba[i] = 0xAA;
            rgba[i + 1] = 0xBB;
            rgba[i + 2] = 0xCC;
            rgba[i + 3] = 0xFF;
        }
    }

    // Trim 1 — crops to the 5×3 bbox.
    let t1 = trim_transparency(&rgba, w as u32, h as u32, 0);
    assert!(t1.trimmed);
    assert_eq!(t1.width, rect.2 as u32);
    assert_eq!(t1.height, rect.3 as u32);
    // Every pixel of the trimmed buffer should be opaque (the bbox
    // contains only the rectangle).
    for chunk in t1.pixels.chunks_exact(4) {
        assert_eq!(chunk, &[0xAA, 0xBB, 0xCC, 0xFF]);
    }

    // MakeSquare — pad 5×3 → 5×5 (height diff = 2, even).
    let ms = make_square(&t1.pixels, t1.width, t1.height);
    assert!(ms.made_square);
    assert_eq!(ms.size, 5);
    assert_eq!(ms.offset_x, 0);
    assert_eq!(ms.offset_y, 1);

    // Trim 2 — should recover the original 5×3 bbox with identical
    // pixels (round-trip invariant).
    let t2 = trim_transparency(&ms.pixels, ms.size, ms.size, 0);
    assert!(t2.trimmed);
    assert_eq!(t2.width, rect.2 as u32, "width after round-trip");
    assert_eq!(t2.height, rect.3 as u32, "height after round-trip");
    assert_eq!(
        t2.pixels, t1.pixels,
        "pixels byte-identical after round-trip"
    );
}
