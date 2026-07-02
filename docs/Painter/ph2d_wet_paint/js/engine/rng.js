// Deterministic random sources for the engine.
// The whole app must reproduce identical state from identical input scripts,
// so no Math.random() anywhere in the engine: everything random flows through
// a seeded 32-bit stream generator or a stateless integer hash.

/**
 * Seeded stream RNG (splitmix32 flavour). Returns a function yielding
 * floats in [0, 1). Identical seeds give identical sequences forever.
 */
export function makeRng(seed) {
  let state = seed >>> 0;
  return function next() {
    state = (state + 0x9e3779b9) >>> 0;
    let z = state;
    z = Math.imul(z ^ (z >>> 16), 0x21f0aaad);
    z = Math.imul(z ^ (z >>> 15), 0x735a2d97);
    z = (z ^ (z >>> 15)) >>> 0;
    return z / 4294967296;
  };
}

/**
 * Stateless integer hash of a 2-D lattice point (plus a seed), uniform in [0, 1).
 * Used for per-pixel jitter that must not depend on visit order.
 */
export function hash2(x, y, seed) {
  let h = (Math.imul(x | 0, 0x27d4eb2d) ^ Math.imul(y | 0, 0x165667b1) ^ (seed | 0)) >>> 0;
  h = Math.imul(h ^ (h >>> 15), 0x85ebca6b);
  h = Math.imul(h ^ (h >>> 13), 0xc2b2ae35);
  h = (h ^ (h >>> 16)) >>> 0;
  return h / 4294967296;
}

/** Signed variant of hash2, uniform in [-1, 1). */
export function hash2Signed(x, y, seed) {
  return hash2(x, y, seed) * 2 - 1;
}
