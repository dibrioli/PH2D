//! **A busca da paleta** — o predicado único que o filtro pintado e o `Enter` chamam, com as palavras
//! de intenção dos itens (doc ObjectsAndComponents 01 §7.3, F6).

use super::{PaletteItem, PaletteModel};
use ph2d_a11y::NodeId;

/// Does `label` match the live search `query`? Case-insensitive substring. An empty query matches
/// everything.
#[must_use]
pub fn label_matches(query: &str, label: &str) -> bool {
    query.is_empty() || label.to_lowercase().contains(&query.to_lowercase())
}

/// Does `item` match the live search `query`? This is the SINGLE predicate the palette's on-screen filter
/// and its `Enter` top-match both call — so what the artist SEES filtered and what `Enter` adds can never
/// disagree. The label matches by substring ([`label_matches`]); otherwise EVERY word of the query has to
/// be found in the label or the item's intent [`PaletteItem::keywords`] («play sound» ⇒ *Audio Source*).
#[must_use]
pub fn item_matches(query: &str, item: &PaletteItem) -> bool {
    if label_matches(query, &item.label) {
        return true;
    }
    if item.keywords.is_empty() {
        return false;
    }
    let hay = format!("{} {}", item.label, item.keywords).to_lowercase();
    query
        .to_lowercase()
        .split_whitespace()
        .all(|word| hay.contains(word))
}

/// Is `query` one of the item's intent words, WHOLE (`"jump"` is in `"fall, jump"` but not in
/// `"jump link"`)?
fn keyword_is(query: &str, item: &PaletteItem) -> bool {
    let q = query.trim().to_lowercase();
    item.keywords
        .split(',')
        .any(|k| k.trim().to_lowercase() == q)
}

/// The best item for `query`, in three tiers and display order inside each: a LABEL match, then an item
/// that has the query as one WHOLE intent word, then any [`item_matches`]; `None` otherwise. The shell
/// calls this on `Enter` to add the top result — every tier is a subset of [`item_matches`], the predicate
/// the filter paints with, so `Enter` never adds a hidden item. ⚠️ The NAME wins over the intent, and a
/// whole word over a fragment: «jump» adds *Physics Body* («fall, jump») before *Nav Link* («jump link»).
/// An EMPTY query returns `None` (Enter with no search is a no-op, never "add the first random node").
#[must_use]
pub fn top_match(model: &PaletteModel, query: &str) -> Option<NodeId> {
    if query.is_empty() {
        return None;
    }
    let items = || {
        model
            .groups
            .iter()
            .flat_map(|g| &g.subs)
            .flat_map(|s| &s.items)
    };
    items()
        .find(|it| label_matches(query, &it.label))
        .or_else(|| items().find(|it| keyword_is(query, it)))
        .or_else(|| items().find(|it| item_matches(query, it)))
        .map(|it| it.id)
}
