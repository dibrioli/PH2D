//! Stateless hash of `(seed, index, lane)` → `f32 ∈ [0, 1)`: a draw is a pure
//! function of its identity, never of a stream of draws.
//!
//! **Stateless is the whole point** (plan §1.3/§1.7, Jarzynski & Olano 2020): a node
//! that draws needs no RNG state, is `Effect::Pure`, scrubbing backwards reproduces
//! the same draws bit-for-bit, and a GPU lowering computes the same values per lane
//! without a sequence. Transcendental-free (HR-5).
//!
//! UMA porta (bug #11): era copiada em treze crates de nó em duas embalagens — a
//! [`hash3`] crua (lattice, scatter, voronoi, lsystem, boids, distribute.poisson) e o
//! [`rand01`] por partícula (emitter, randomize, sim.lifetime, sim.spawn, sort). Os dois
//! nomes ficam, porque cada um lê-se certo no seu chamador; a lei é uma. Quem embrulha a
//! lei noutra assinatura (o `rand01(seed, index)` de `sim.collide` e
//! `value.instance_field`, o `Draws` do poisson) guarda só o embrulho e chama esta porta.

/// splitmix-style avalanche on a 32-bit lattice → `[0, 1)`.
pub fn hash3(a: u32, b: u32, lane: u32) -> f32 {
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
pub fn rand01(seed: u32, id: u32, lane: u32) -> f32 {
    hash3(seed, id, lane)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash3_draws_are_in_range_and_reproducible() {
        for i in 0..500u32 {
            let v = hash3(7, i, 0);
            assert!((0.0..1.0).contains(&v), "draw {v} out of range at {i}");
            assert_eq!(v, hash3(7, i, 0), "the same identity always redraws");
        }
    }

    #[test]
    fn hash3_lanes_and_indices_decorrelate() {
        assert_ne!(hash3(0, 1, 0), hash3(0, 1, 1), "lanes differ (x vs y)");
        assert_ne!(hash3(0, 1, 0), hash3(0, 2, 0), "indices differ");
    }

    #[test]
    fn rand01_draws_are_in_range_and_reproducible() {
        for id in 0..500u32 {
            let v = rand01(7, id, 0);
            assert!((0.0..1.0).contains(&v), "draw {v} out of range at {id}");
            assert_eq!(v, rand01(7, id, 0), "the same identity always redraws");
        }
    }

    #[test]
    fn rand01_lanes_seeds_and_ids_decorrelate() {
        assert_ne!(rand01(0, 1, 0), rand01(0, 1, 1), "lanes differ");
        assert_ne!(rand01(0, 1, 0), rand01(1, 1, 0), "seeds differ");
        assert_ne!(rand01(0, 1, 0), rand01(0, 2, 0), "ids differ");
    }

    /// ⭐ **O GOLDEN da lei**, em bits: os mesmos três pares que o `sim.collide` deriva para o
    /// oráculo do `sc_rand01` do WGSL dele (lá `rand01(seed, index)` = `hash3(seed, index, 0)`).
    /// Um literal trocado ou um `>>` a menos reprova aqui, sem cena, sem GPU, sem olho. E
    /// `(0, 0, 0)` dá **zero** por aritmética: a avalanche parte de `0·k + 0·k + 0·k`.
    #[test]
    fn hash3_golden_bits() {
        assert_eq!(hash3(0, 0, 0).to_bits(), 0x0000_0000, "(0, 0, 0)");
        assert_eq!(hash3(7, 3, 0).to_bits(), 0x3edd_93de, "(7, 3, 0)");
        assert_eq!(hash3(11, 1000, 0).to_bits(), 0x3dba_59d0, "(11, 1000, 0)");
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
