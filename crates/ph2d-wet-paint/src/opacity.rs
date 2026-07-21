//! Saturating pigment-opacity table (port of `opacity.js`, SPEC §3).
//!
//! Coverage is not linear in pigment mass: each unit of mass covers 0.2% of
//! whatever paper is still showing, i.e. `alpha(m) = 1 - 0.998^m`. Precomputed
//! once as a 3001-entry table; every consumer truncates its mass to an integer
//! index and clamps into the table, so mass >= 3000 reads fully opaque.
//! Half-opacity sits near mass ~346 — a light wash (100..400) lets the paper
//! glow through, which is what makes deposits read as watercolor.
//!
//! The table is a COMPILE-TIME static, not a lazy singleton: `alpha_of_mass`
//! runs several times per cell in the drying/trail hot loops, and a OnceLock
//! acquire per lookup was measurable on the flood upper bound.

pub const OPACITY_TABLE_MAX: usize = 3000;

const fn build_table() -> [f32; OPACITY_TABLE_MAX + 1] {
    let mut t = [0.0f32; OPACITY_TABLE_MAX + 1];
    // The JS recurrence runs in f64 reading back the f32 it just stored.
    let mut m = 1;
    while m <= OPACITY_TABLE_MAX {
        t[m] = (0.002 + 0.998 * t[m - 1] as f64) as f32;
        m += 1;
    }
    t[OPACITY_TABLE_MAX] = 1.0; // force exact saturation at the top entry
    t
}

static OPACITY: [f32; OPACITY_TABLE_MAX + 1] = build_table();

/// Opacity lookup: truncate mass to int (JS `m | 0` — ToInt32 WRAPS, it
/// never saturates; port-verify finding), clamp into the table.
#[inline]
pub fn alpha_of_mass(m: f64) -> f64 {
    let i = crate::jsmath::to_int32_wrapping(m);
    if i <= 0 {
        return 0.0;
    }
    if i >= OPACITY_TABLE_MAX as i32 {
        return 1.0;
    }
    OPACITY[i as usize] as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_curve_saturates_like_the_spec_says() {
        assert_eq!(alpha_of_mass(0.0), 0.0);
        assert_eq!(alpha_of_mass(3000.0), 1.0);
        assert_eq!(alpha_of_mass(5000.0), 1.0);
        // Half-opacity sits near mass ~346 (SPEC §3).
        let a346 = alpha_of_mass(346.0);
        assert!((a346 - 0.5).abs() < 0.01, "alpha(346) = {a346}");
        // A light wash lets the paper show through (SPEC: 100..400 -> 0.18..0.55).
        let a100 = alpha_of_mass(100.0);
        assert!(a100 > 0.15 && a100 < 0.22, "alpha(100) = {a100}");
    }

    #[test]
    fn the_const_table_matches_the_runtime_recurrence() {
        // The compile-time evaluation must be the SAME arithmetic the JS
        // runs at startup — recompute at runtime and compare bit-for-bit.
        let mut prev = 0.0f32;
        for m in 1..OPACITY_TABLE_MAX {
            let v = (0.002 + 0.998 * prev as f64) as f32;
            assert_eq!(v.to_bits(), OPACITY[m].to_bits(), "table diverges at {m}");
            prev = v;
        }
    }
}
