//! Stateless per-instance randomness: a well-mixed integer hash of
//! `(seed, index)` → `f32 ∈ [0, 1)`: this crate's wrapper (`rand01(seed, index)`, lane `0`)
//! over the shared law `ph2d_motion_kit::hash::hash3` (bug #11 — one door, not a per-crate
//! copy).
//!
//! **Stateless is the whole point** (Jarzynski & Olano 2020): an instance's draw
//! is a pure function of its identity, never of a stream of draws — so the field
//! is `Effect::Pure`, scrubbing reproduces it bit-for-bit, and a GPU lowering
//! computes the same value per lane. Transcendental-free (HR-5).

use ph2d_motion_kit::hash::hash3;

/// Instance `index`'s draw for `seed`, in `[0, 1)`.
pub(crate) fn rand01(seed: u32, index: u32) -> f32 {
    hash3(seed, index, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_are_in_range_and_reproducible() {
        for i in 0..500u32 {
            let v = rand01(7, i);
            assert!((0.0..1.0).contains(&v), "draw {v} out of range at {i}");
            assert_eq!(v, rand01(7, i), "the same identity always redraws");
        }
    }

    #[test]
    fn seeds_and_indices_decorrelate() {
        assert_ne!(rand01(0, 1), rand01(1, 1), "seeds differ");
        assert_ne!(rand01(0, 1), rand01(0, 2), "indices differ");
    }

    #[test]
    fn the_draw_spreads_over_the_unit_interval() {
        let mut buckets = [0u32; 4];
        for i in 0..1000u32 {
            let q = (rand01(3, i) * 4.0) as usize;
            buckets[q.min(3)] += 1;
        }
        assert!(
            buckets.iter().all(|&c| c > 150),
            "quartiles under-filled: {buckets:?}"
        );
    }
}
