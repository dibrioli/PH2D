//! Integration test for the Trim Transparency island.
//!
//! The new module lives at `src/tools/trim_transparency/` but is not
//! yet re-exported through `tools/mod.rs` (that wiring is the
//! Integrator's job per `INTEGRATION.md`). To still exercise the
//! algorithm + icon under `cargo test -p ph2d-editor`, we pull the
//! source files in via `#[path]` so they compile as a fresh module
//! inside this test binary. `algorithm.rs` is `std`-only;
//! `icon.rs` pulls `ph2d_vector::BezPath`, which is a regular
//! dependency of `ph2d-editor` and therefore available in
//! integration-test builds.

#[path = "../src/tools/trim_transparency/algorithm.rs"]
mod algorithm;

#[path = "../src/tools/trim_transparency/icon.rs"]
mod icon;

use algorithm::{Bounds, trim_transparency};

#[test]
fn legacy_parity_fully_transparent_returns_1x1_sentinel() {
    // Matches the legacy `trimTransparency` behaviour on a fully
    // transparent canvas: 1×1 fallback so downstream stores don't
    // see a zero-sized image.
    let rgba = vec![0u8; 32 * 32 * 4];
    let r = trim_transparency(&rgba, 32, 32, 0);
    assert_eq!(
        r.bounds,
        Bounds {
            x: 0,
            y: 0,
            width: 1,
            height: 1
        }
    );
    assert_eq!(r.pixels, vec![0, 0, 0, 0]);
    assert!(r.trimmed);
}

#[test]
fn legacy_parity_off_center_object_trims_to_tight_bounds() {
    // 64×64 sprite, opaque 16×16 patch at (20, 24). Expected output:
    // tight 16×16 buffer starting at bounds (20, 24).
    let w = 64usize;
    let h = 64usize;
    let mut rgba = vec![0u8; w * h * 4];
    for y in 24..40 {
        for x in 20..36 {
            let i = (y * w + x) * 4;
            rgba[i] = 0;
            rgba[i + 1] = 128;
            rgba[i + 2] = 255;
            rgba[i + 3] = 255;
        }
    }
    let r = trim_transparency(&rgba, w as u32, h as u32, 0);
    assert_eq!(
        r.bounds,
        Bounds {
            x: 20,
            y: 24,
            width: 16,
            height: 16
        }
    );
    assert_eq!(r.width, 16);
    assert_eq!(r.height, 16);
    assert_eq!(r.pixels.len(), 16 * 16 * 4);
    assert!(r.trimmed);
    // Every pixel of the trimmed buffer must match the source patch.
    for chunk in r.pixels.chunks(4) {
        assert_eq!(chunk, &[0, 128, 255, 255]);
    }
}

#[test]
fn icon_module_compiles_and_returns_path() {
    // Black-box check that the icon helper is callable from outside
    // the source module and produces a non-empty path. Detailed
    // shape assertions live in the unit tests inside `icon.rs`.
    let path = icon::crop_bezpath();
    assert!(!path.elements().is_empty());
}
