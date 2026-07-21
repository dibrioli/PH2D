//! Deterministic random sources (port of `rng.js`).
//!
//! No global randomness anywhere in the engine: identical seeds reproduce
//! identical states forever. Two primitives — a seeded 32-bit stream
//! generator (splitmix32 flavour) and a stateless integer hash of a 2-D
//! lattice point (for per-pixel jitter that must not depend on visit order).

use crate::jsmath::imul;

/// Seeded stream RNG. `next()` yields f64 in [0, 1).
#[derive(Clone, Debug)]
pub struct Rng {
    state: u32,
}

impl Rng {
    pub fn new(seed: u32) -> Self {
        Rng { state: seed }
    }

    #[inline]
    pub fn next(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x9e37_79b9);
        let mut z = self.state;
        z = imul(z ^ (z >> 16), 0x21f0_aaad);
        z = imul(z ^ (z >> 15), 0x735a_2d97);
        z ^= z >> 15;
        z as f64 / 4_294_967_296.0
    }
}

/// Stateless integer hash of a 2-D lattice point (plus a seed), uniform [0, 1).
#[inline]
pub fn hash2(x: i32, y: i32, seed: u32) -> f64 {
    let mut h = imul(x as u32, 0x27d4_eb2d) ^ imul(y as u32, 0x1656_67b1) ^ seed;
    h = imul(h ^ (h >> 15), 0x85eb_ca6b);
    h = imul(h ^ (h >> 13), 0xc2b2_ae35);
    h ^= h >> 16;
    h as f64 / 4_294_967_296.0
}

/// Signed variant of [`hash2`], uniform in [-1, 1).
#[inline]
pub fn hash2_signed(x: i32, y: i32, seed: u32) -> f64 {
    hash2(x, y, seed) * 2.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next(), b.next());
        }
    }

    #[test]
    fn stream_is_uniform_ish() {
        let mut r = Rng::new(7);
        let mut sum = 0.0;
        for _ in 0..10_000 {
            let v = r.next();
            assert!((0.0..1.0).contains(&v));
            sum += v;
        }
        let mean = sum / 10_000.0;
        assert!((mean - 0.5).abs() < 0.02, "mean {mean}");
    }

    #[test]
    fn hash_is_order_free_and_bounded() {
        let a = hash2(10, 20, 99);
        let b = hash2(10, 20, 99);
        assert_eq!(a, b);
        assert!((0.0..1.0).contains(&a));
        assert!((-1.0..1.0).contains(&hash2_signed(-5, 7, 3)));
    }
}
