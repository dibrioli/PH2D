//! Stateless per-particle randomness: a well-mixed integer hash of
//! `(seed, spawn_index, lane)` → `f32 ∈ [0, 1)`.
//!
//! **Stateless is the whole point** (plan §1.3/§1.7, Jarzynski & Olano 2020): a
//! particle's randomness is a pure function of its identity, never of a stream
//! of draws. So the emitter needs no RNG state, scrubbing backwards reproduces
//! the same particles bit-for-bit, and a GPU lowering computes the same values
//! per lane without a sequence. Transcendental-free (HR-5).

/// splitmix64-style avalanche on a 32-bit lattice → `[0, 1)`.
fn hash3(a: u32, b: u32, lane: u32) -> f32 {
    let mut h = a
        .wrapping_mul(0x9e37_79b9)
        .wrapping_add(b.wrapping_mul(0x85eb_ca6b))
        .wrapping_add(lane.wrapping_mul(0xc2b2_ae35));
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    h ^= h >> 16;
    // 24 bits into the mantissa → exactly representable, uniform, and never 1.0.
    (h >> 8) as f32 / (1u32 << 24) as f32
}

/// Particle `id`'s draw on `lane`, in `[0, 1)`. Different lanes are independent
/// (angle vs speed vs anything a later param needs).
pub(crate) fn rand01(seed: u32, id: u32, lane: u32) -> f32 {
    hash3(seed, id, lane)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_are_in_range_and_reproducible() {
        for id in 0..500u32 {
            let v = rand01(7, id, 0);
            assert!((0.0..1.0).contains(&v), "draw {v} out of range at {id}");
            assert_eq!(v, rand01(7, id, 0), "the same identity always redraws");
        }
    }

    #[test]
    fn lanes_seeds_and_ids_decorrelate() {
        assert_ne!(rand01(0, 1, 0), rand01(0, 1, 1), "lanes differ");
        assert_ne!(rand01(0, 1, 0), rand01(1, 1, 0), "seeds differ");
        assert_ne!(rand01(0, 1, 0), rand01(0, 2, 0), "ids differ");
    }

    #[test]
    fn the_draw_spreads_over_the_unit_interval() {
        // A crude uniformity check: 1000 ids should fill all four quartiles.
        let mut buckets = [0u32; 4];
        for id in 0..1000u32 {
            let q = (rand01(3, id, 0) * 4.0) as usize;
            buckets[q.min(3)] += 1;
        }
        assert!(
            buckets.iter().all(|&c| c > 150),
            "quartiles under-filled: {buckets:?}"
        );
    }
}
