//! **A busca da paleta** — o predicado único que o filtro pintado e o `Enter` chamam, com as palavras
//! de intenção dos itens (doc ObjectsAndComponents 01 §7.3, F6).

use super::{PaletteGroup, PaletteItem, PaletteModel, PaletteSub};
use ph2d_a11y::NodeId;

/// Does `label` match the live search `query`? Case-insensitive substring. An empty query matches
/// everything.
#[must_use]
pub fn label_matches(query: &str, label: &str) -> bool {
    query.is_empty() || label.to_lowercase().contains(&query.to_lowercase())
}

/// Does `item` match the live search `query`? This is the SINGLE predicate the palette's on-screen filter
/// and its `Enter` top-match both call — so what the artist SEES filtered and what `Enter` adds can never
/// disagree. The label matches by substring ([`label_matches`]) or as a subsequence ([`fuzzy_score`]); otherwise EVERY word of the query has to
/// be found in the label or the item's intent [`PaletteItem::keywords`] («play sound» ⇒ *Audio Source*).
#[must_use]
pub fn item_matches(query: &str, item: &PaletteItem) -> bool {
    label_matches(query, &item.label)
        || fuzzy_score(query, &item.label).is_some()
        || intent_matches(query, item)
}

/// Every word of `query` is in the label or the item's intent words.
fn intent_matches(query: &str, item: &PaletteItem) -> bool {
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

/// The best item for `query`, in four tiers (the last: the best [`fuzzy_score`]) and display order inside each: a LABEL match, then an item
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
    let best_fuzzy = || {
        items()
            .filter_map(|it| fuzzy_score(query, &it.label).map(|s| (s, it)))
            .fold(None, |b: Option<(i32, &PaletteItem)>, c| match b {
                Some(b) if b.0 >= c.0 => Some(b),
                _ => Some(c),
            })
            .map(|(_, it)| it)
    };
    items()
        .find(|it| label_matches(query, &it.label))
        .or_else(|| items().find(|it| keyword_is(query, it)))
        .or_else(|| items().find(|it| intent_matches(query, it)))
        .or_else(best_fuzzy)
        .map(|it| it.id)
}

/// ⭐ **The fuzzy match** (Aseprite Run Command, `docs/PixelArt/P12_oraculos.md` O10): `query` (no
/// spaces, no case) as a SUBSEQUENCE of `name` — `flp` finds *Flip*; `None` = no match. Higher =
/// better: a letter at a word start and letters in a row score more, gaps cost. The ONE matcher of
/// the app: the filter, `Enter` and the Pixel's own search all rank by it.
#[must_use]
pub fn fuzzy_score(query: &str, name: &str) -> Option<i32> {
    let q: Vec<char> = query
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect();
    if q.is_empty() {
        return Some(0);
    }
    let n: Vec<char> = name.chars().flat_map(char::to_lowercase).collect();
    let (mut s, mut qi, mut last) = (0, 0, None::<usize>);
    for (i, &ch) in n.iter().enumerate() {
        if qi == q.len() {
            break;
        }
        if ch != q[qi] {
            continue;
        }
        s += 10;
        if i == 0 || !n[i - 1].is_alphanumeric() {
            s += 8;
        }
        match last {
            Some(l) if l + 1 == i => s += 6,
            Some(l) => s -= i32::try_from((i - l - 1).min(20)).unwrap_or(20),
            None => s -= i32::try_from(i.min(20)).unwrap_or(20),
        }
        last = Some(i);
        qi += 1;
    }
    (qi == q.len()).then_some(s)
}

/// A copy of `model` keeping only items whose label matches `query`, dropping any sub-cluster or category
/// that ends up empty — so a filtered library never shows a coloured header with nothing under it. An
/// empty query is handled by the caller (it uses the model unfiltered); here every item still matches.
pub(super) fn filter_model(model: &PaletteModel, query: &str) -> PaletteModel {
    let groups = model
        .groups
        .iter()
        .filter_map(|g| {
            let subs: Vec<PaletteSub> = g
                .subs
                .iter()
                .filter_map(|s| {
                    let mut items: Vec<PaletteItem> = s
                        .items
                        .iter()
                        .filter(|it| item_matches(query, it))
                        .cloned()
                        .collect();
                    // Best match first (O10); a stable sort keeps the display order on a tie.
                    items.sort_by_key(|it| {
                        std::cmp::Reverse(fuzzy_score(query, &it.label).unwrap_or(i32::MIN))
                    });
                    (!items.is_empty()).then(|| PaletteSub {
                        title: s.title.clone(),
                        items,
                    })
                })
                .collect();
            (!subs.is_empty()).then(|| PaletteGroup {
                title: g.title.clone(),
                color: g.color,
                subs,
            })
        })
        .collect();
    PaletteModel {
        title: model.title.clone(),
        groups,
        // ⚠️ A caixa ATRAVESSA o filtro: ela é da banda, não do conteúdo — filtrar por texto não a
        // pode desligar, senão escrever no campo de busca apagava o controlo que a mostra.
        toggle: model.toggle.clone(),
        // A dica procura-se pela id: as dos itens filtrados ficam e não pintam.
        tips: model.tips.clone(),
        select: model.select.clone(),
    }
}
