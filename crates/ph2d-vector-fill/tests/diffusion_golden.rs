//! W7 golden + visual smoke for the diffusion-curve `ColorField`.
//!
//! Scaffolded by the Coordinator at the impl's request (HANDOFF_vector_w7_poisson
//! _cpu_impl §4): the deterministic CPU multigrid solver is the stable oracle the
//! W7 **step-3 GPU Walk-on-Spheres port** will be validated against. This test
//! pins one canonical scene two ways:
//!
//! 1. **Oracle hash** — an FNV-1a over the sRGB8 quantisation of the field.
//!    Catches an accidental change to the CPU multigrid solver (same machine).
//!    sRGB8 quantisation (not raw f32 bits) is the comparison granularity the GPU
//!    port will match against (a different algorithm never bit-matches f32, but
//!    must agree on the displayed image within tolerance).
//! 2. **Visual smoke** — an ANSI 24-bit-colour preview printed to the terminal,
//!    so the diffusion result is eyeball-checkable with zero file/format friction
//!    (`cargo test -p ph2d-vector-fill --test diffusion_golden -- --ignored --nocapture`).
//!
//! `#[ignore]` (dev/oracle convention, like the GPU parity tests): cross-OS f32
//! bit-identity is the det-mode opt-in deferred to a later wave, so this is a
//! same-machine oracle, not a CI gate. The impl's non-ignored unit tests
//! (`harmonic_*`, `straight_red_blue_*`, `solve_is_bit_deterministic`) guard
//! correctness in CI.

use glam::Vec2;
use ph2d_color::OklchColor;
use ph2d_color::srgb::linear_to_srgb_byte;
use ph2d_vector_fill::{
    ColorField, DiffusionCurve, DiffusionCurveSet, Resolution, solve_color_field,
};

/// THE canonical golden scene (the W7 step-3 GPU port reuses this exact set):
/// a full-height red↔blue wall + a green band near the top + a warm diagonal.
fn golden_scene() -> DiffusionCurveSet {
    let red = OklchColor::opaque(0.63, 0.26, 29.0);
    let blue = OklchColor::opaque(0.45, 0.31, 264.0);
    let green = OklchColor::opaque(0.70, 0.20, 142.0);
    let amber = OklchColor::opaque(0.82, 0.16, 75.0);
    DiffusionCurveSet::from_curves([
        // Vertical wall x=0.5: red left, blue right.
        DiffusionCurve::straight(Vec2::new(0.5, 0.0), Vec2::new(0.5, 1.0), red, blue),
        // Horizontal band near the top, green both sides.
        DiffusionCurve::straight(Vec2::new(0.1, 0.22), Vec2::new(0.9, 0.22), green, green),
        // Warm diagonal accent in the lower-right.
        DiffusionCurve::straight(Vec2::new(0.55, 0.95), Vec2::new(0.95, 0.55), amber, amber),
    ])
}

/// Encode a linear-light field to straight-sRGB8 RGBA bytes (the displayed image,
/// and the comparison granularity for the oracle).
fn encode_srgb8(field: &ColorField) -> Vec<u8> {
    let mut out = Vec::with_capacity(field.w * field.h * 4);
    for px in &field.texel {
        out.push(linear_to_srgb_byte(px[0]));
        out.push(linear_to_srgb_byte(px[1]));
        out.push(linear_to_srgb_byte(px[2]));
        out.push((px[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
    }
    out
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Print a downsampled ANSI 24-bit-colour preview (background-colour blocks) so
/// the diffusion field is visible in the terminal.
fn print_ansi_preview(field: &ColorField) {
    let cols = 64usize.min(field.w);
    let rows = 32usize.min(field.h);
    eprintln!(
        "\n  diffusion ColorField ({}×{}) — visual smoke:",
        field.w, field.h
    );
    for ry in 0..rows {
        let mut line = String::from("  ");
        let y = ry * (field.h - 1) / (rows - 1);
        for rx in 0..cols {
            let x = rx * (field.w - 1) / (cols - 1);
            let p = field.at(x, y);
            let r = linear_to_srgb_byte(p[0]);
            let g = linear_to_srgb_byte(p[1]);
            let b = linear_to_srgb_byte(p[2]);
            line.push_str(&format!("\x1b[48;2;{r};{g};{b}m "));
        }
        line.push_str("\x1b[0m");
        eprintln!("{line}");
    }
    eprintln!();
}

/// The committed oracle for `golden_scene()` @ 129², default V-cycles. Re-pin
/// (run with `--ignored --nocapture`, copy the printed value) only when the CPU
/// multigrid solver changes ON PURPOSE.
const GOLDEN_HASH: u64 = 0x3fcf_9e8a_f30a_d1ff;

#[test]
#[ignore = "dev oracle + visual smoke — run with --ignored --nocapture"]
fn diffusion_field_golden_oracle() {
    let set = golden_scene();
    let res = Resolution::square(129).expect("129 = 2^7+1");
    let field = solve_color_field(&set, res);

    // Same-machine determinism (cheap; the impl's gate covers this too).
    let field2 = solve_color_field(&set, res);
    assert_eq!(field, field2, "solve_color_field is not deterministic");

    print_ansi_preview(&field);

    let hash = fnv1a(&encode_srgb8(&field));
    eprintln!("  diffusion golden hash = {hash:#018x}\n");
    assert_eq!(
        hash, GOLDEN_HASH,
        "diffusion field changed — if intentional, re-pin GOLDEN_HASH to {hash:#018x}"
    );
}
