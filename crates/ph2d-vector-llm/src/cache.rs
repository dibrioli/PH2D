//! Result cache keyed by `(prompt_hash, seed)` (ADR-0061 §2.1). Two roles: skip
//! a repeat LLM call (latency 2-10 s), and serve the **timeout fallback** (§2.6)
//! — on a 15 s LLM timeout the host returns the last cached result instead of
//! blocking. Capacity-bounded, deterministic eviction (oldest insert first).

use std::collections::BTreeMap;

use crate::semantic_tokens::SemanticTokens;

/// `(blake3(prompt), seed)` — same prompt + seed ⇒ same key.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CacheKey {
    prompt_hash: [u8; 32],
    seed: u64,
}

impl CacheKey {
    pub fn new(prompt: &str, seed: u64) -> Self {
        Self {
            prompt_hash: *blake3::hash(prompt.as_bytes()).as_bytes(),
            seed,
        }
    }
}

/// A bounded `(prompt, seed) → SemanticTokens` cache.
#[derive(Debug)]
pub struct ResultCache {
    entries: BTreeMap<CacheKey, (u64, SemanticTokens)>,
    capacity: usize,
    tick: u64,
}

impl ResultCache {
    /// New cache holding at most `capacity` results.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            capacity: capacity.max(1),
            tick: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The cached result for `key`, if present.
    pub fn get(&self, key: &CacheKey) -> Option<&SemanticTokens> {
        self.entries.get(key).map(|(_, t)| t)
    }

    /// Insert (or replace) a result, evicting the oldest insert if at capacity.
    pub fn insert(&mut self, key: CacheKey, tokens: SemanticTokens) {
        if !self.entries.contains_key(&key)
            && self.entries.len() >= self.capacity
            && let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, (t, _))| *t)
                .map(|(k, _)| *k)
        {
            self.entries.remove(&oldest);
        }
        let t = self.tick;
        self.tick += 1;
        self.entries.insert(key, (t, tokens));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic_tokens::{Shape, StyleTokens};
    use glam::Vec2;

    fn toks(sides: u32) -> SemanticTokens {
        SemanticTokens {
            shape: Shape::Polygon {
                center: Vec2::ZERO,
                radius: 100.0,
                sides,
                rotation: 0.0,
            },
            style: StyleTokens::default(),
        }
    }

    #[test]
    fn same_prompt_seed_hits() {
        let mut c = ResultCache::new(8);
        let k = CacheKey::new("a spiral", 42);
        assert!(c.get(&k).is_none());
        c.insert(k, toks(6));
        assert!(c.get(&CacheKey::new("a spiral", 42)).is_some());
    }

    #[test]
    fn seed_is_part_of_the_key() {
        let mut c = ResultCache::new(8);
        c.insert(CacheKey::new("p", 1), toks(6));
        assert!(
            c.get(&CacheKey::new("p", 2)).is_none(),
            "different seed misses"
        );
    }

    #[test]
    fn evicts_oldest_at_capacity() {
        let mut c = ResultCache::new(2);
        c.insert(CacheKey::new("a", 0), toks(3));
        c.insert(CacheKey::new("b", 0), toks(4));
        c.insert(CacheKey::new("c", 0), toks(5)); // evicts "a"
        assert_eq!(c.len(), 2);
        assert!(c.get(&CacheKey::new("a", 0)).is_none());
        assert!(c.get(&CacheKey::new("c", 0)).is_some());
    }
}
