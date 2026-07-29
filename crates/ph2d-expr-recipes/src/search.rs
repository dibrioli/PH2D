//! **The search** — with a catalog this size, the search IS the interface.
//!
//! Two things make it work, and neither is fuzzy matching:
//!
//! 1. **Aliases.** The artist types the name they learned in another product —
//!    `wiggle`, `posterizeTime`, `Oscillate`, `linear`, `clamp`. A card that only
//!    answers to its own label is invisible to everyone who has used After Effects.
//! 2. **The refusals answer too.** Typing `loop` returns a card that says where the
//!    loop lives and offers to take you there. Refusing silently teaches the artist
//!    that the tool cannot do it.

use crate::catalog::CATALOG;
use crate::recipe::Recipe;
use crate::refusal::{REFUSALS, Refusal};

/// A hit: something the artist can add, or somewhere they should go instead.
#[derive(Clone, Copy, Debug)]
pub enum SearchHit {
    Recipe(&'static Recipe),
    /// ⚠️ NOT an error state. This is routing: the thing exists, elsewhere.
    Refusal(&'static Refusal),
}

/// Lower-case and drop everything that is not a letter or a digit.
///
/// ⚠️ This is not cosmetic, and the gate found it: the artist types the
/// IDENTIFIER they read in the other product — `posterizeTime`, `loopOut`,
/// `sourceRectAtTime` — while a catalog is written in prose (`posterize time`).
/// Normalising both sides makes the two spellings the same question. It is also
/// why `pick whip` finds `Follow` when typed as `pickwhip`.
fn norm(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Rank: an exact label match beats a label prefix beats an alias beats a blurb.
/// Lower is better. `query` arrives already normalised.
fn rank(query: &str, label: &str, aliases: &[&str], blurb: &str) -> Option<u8> {
    let l = norm(label);
    if l == query {
        return Some(0);
    }
    if l.starts_with(query) {
        return Some(1);
    }
    if aliases.iter().any(|a| norm(a) == query) {
        return Some(2);
    }
    if aliases.iter().any(|a| norm(a).contains(query)) {
        return Some(3);
    }
    if l.contains(query) {
        return Some(4);
    }
    if norm(blurb).contains(query) {
        return Some(5);
    }
    None
}

/// Everything matching `query`, best first. An empty query returns the whole
/// catalog in gallery order (the browse case), with no refusals — those are an
/// ANSWER to a question, and browsing has not asked one.
#[must_use]
pub fn search(query: &str) -> Vec<SearchHit> {
    let q = norm(query);
    if q.is_empty() {
        return CATALOG.iter().copied().map(SearchHit::Recipe).collect();
    }

    let mut hits: Vec<(u8, usize, SearchHit)> = Vec::new();
    for (i, r) in CATALOG.iter().enumerate() {
        if let Some(k) = rank(&q, r.label, r.aliases, r.blurb) {
            hits.push((k, i, SearchHit::Recipe(r)));
        }
    }
    for (i, r) in REFUSALS.iter().enumerate() {
        if let Some(k) = rank(&q, r.title, r.aliases, r.body) {
            // Refusals sort after recipes of the same rank: when something IS
            // buildable, offer it before explaining where it is not.
            hits.push((k, CATALOG.len() + i, SearchHit::Refusal(r)));
        }
    }
    hits.sort_by_key(|(k, i, _)| (*k, *i));
    hits.into_iter().map(|(_, _, h)| h).collect()
}
