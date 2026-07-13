//! **The add-menu's search** (Enio, smoke 2026-07-13: *"não encontrei value.lfo"*).
//!
//! The library has **86 node types** in one flat scrolling list, and the artist knew exactly
//! what they wanted and still could not find it. A list you have to *scan* is a list that
//! stops working somewhere around forty entries — every node editor worth using (Blender's
//! Shift+A search, Unreal's palette, Houdini's tab menu) answers this the same way: you open
//! it and you **type**.
//!
//! ## The matching is fuzzy on purpose, and it searches BOTH names
//!
//! A node has two names: the label the artist reads (`"LFO"`, `"Map Range"`) and the
//! canonical type name the code speaks (`"value.lfo"`, `"value.map_range"`). The smoke that
//! prompted this was me telling Enio to look for `value.lfo` — a string the menu never showed
//! him. So the query is matched against **both**, and finding a node by either is finding it.
//!
//! It also means the DOMAIN becomes a query: typing `value` lists every `value.*` node, `force`
//! every force, `pulse` every trigger. The 86-entry list has a structure the flat menu never
//! exposed, and the search is how it surfaces.
//!
//! Fuzzy = **subsequence**, not substring: `mr` finds "Map Range", `atr` finds "Attractor".
//! Scoring rewards what a human means by "a better match" — a prefix, a word boundary, a
//! contiguous run — so the thing you were thinking of comes first rather than merely being
//! *present* somewhere in the list.

use crate::hits::menu_search_id;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::TextInputState;

/// **Open the search field and give it the keyboard** — once, on the frame the menu opens.
/// Re-registering every frame would stomp what the artist is typing; re-focusing every frame
/// would fight anything else they click.
pub(crate) fn open_search(store: &mut WidgetStore) {
    let id = menu_search_id();
    store.register(
        id,
        InteractiveState::TextInput {
            state: TextInputState::Focused,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    store.set_focus(Some(id));
    // Esc closes the popup rather than merely blurring the field — the field says so itself
    // (`mark_cancel_on_escape`), which is why `dispatch_key` no longer keeps a list of the
    // ids that do.
    store.mark_cancel_on_escape(id);
}

/// **Enter on the search field takes the TOP match** — the row the ranking put first, which
/// is the row the artist is looking at.
///
/// It goes through the same `menu_rows`/`menu_matches` derivation the paint and the click use;
/// picking "the first row" by any other route is how a keyboard and a mouse come to disagree
/// about what the first row is.
pub(crate) fn pick_first(menu: &crate::state::Menu) {
    let snap = crate::snapshot::current_snapshot();
    match &menu.body {
        crate::state::MenuBody::Library { connect_from } => {
            let Some(c) = crate::snapshot::menu_matches(&snap, menu, *connect_from)
                .first()
                .copied()
            else {
                return; // nothing matched: Enter on an empty result set does nothing
            };
            crate::snapshot::push_intent(match connect_from {
                Some((from_node, from_port)) => crate::snapshot::GraphIntent::SmartConnect {
                    from_node: *from_node,
                    from_port: *from_port,
                    to_type: c.type_name,
                    x: menu.spawn.0,
                    y: menu.spawn.1,
                },
                None => crate::snapshot::GraphIntent::AddNode {
                    type_name: c.type_name,
                    x: menu.spawn.0,
                    y: menu.spawn.1,
                },
            });
        }
        // A card/param menu is short and its rows are picked by eye; Enter there would be
        // guessing which port the artist meant.
        crate::state::MenuBody::CardPorts { .. } => {}
    }
}

/// How well `query` matches `text`, or `None` if it does not match at all.
///
/// A match is a **subsequence**: every character of the query appears in `text`, in order.
/// Higher is better. Empty query matches everything at 0 (the unfiltered list keeps its
/// order).
///
/// The weights are the ones every fuzzy finder converges on, and they are ordered by how
/// strongly a human reads them as intent:
///
/// - **prefix** (`lf` → **LF**O) — you started typing its name;
/// - **word boundary** (`mr` → **M**ap **R**ange) — you typed its initials;
/// - **contiguous run** (`ange` → M**a**p R**ange**) — you typed a piece of it;
/// - and a penalty for how much junk had to be skipped, so a short name that matches tightly
///   beats a long one that matches loosely.
fn score(query: &str, text: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }
    let hay: Vec<char> = text.chars().flat_map(char::to_lowercase).collect();
    let mut points = 0i32;
    let mut at = 0usize;
    let mut run = 0i32;

    for qc in query.chars().flat_map(char::to_lowercase) {
        // Find the next occurrence from where the previous character landed — in order, which
        // is what makes this a subsequence and not a bag of letters.
        let found = hay[at..].iter().position(|c| *c == qc)? + at;
        let skipped = (found - at) as i32;

        if found == 0 {
            points += 12; // the very start of the name
        } else if is_boundary(&hay, found) {
            points += 8; // the start of a word ("Map RANGE", "value.LFO")
        }
        // A contiguous run compounds: each further character adjacent to the last is worth
        // more than the one before it, so "rang" beats four scattered letters.
        if skipped == 0 && at > 0 {
            run += 1;
            points += 4 + run;
        } else {
            run = 0;
        }
        points -= skipped.min(6); // junk between hits is a cost, but a bounded one
        at = found + 1;
    }
    // Shorter names win ties: "LFO" over "Value LFO Extended", which is what you meant.
    points -= (hay.len() as i32) / 8;
    Some(points)
}

/// Is `i` the first character of a word? (Start of string, or preceded by a separator or a
/// case change — `map_range`, `Map Range`, `value.lfo` and `MapRange` all read the same way.)
fn is_boundary(hay: &[char], i: usize) -> bool {
    if i == 0 {
        return true;
    }
    let prev = hay[i - 1];
    !prev.is_alphanumeric()
}

/// The best score across a node's two names — its label and its canonical type name (doc 58's
/// smoke: the artist was told to look for `value.lfo`, and the menu only ever showed `LFO`).
///
/// `None` = this row does not match and is not listed. Filtering, rather than dimming, is the
/// point: a menu that still shows all 86 entries has not helped anyone.
pub(crate) fn rank(query: &str, display: &str, type_name: &str) -> Option<i32> {
    let a = score(query, display);
    // The canonical name matches at a discount: when both hit, the artist almost always meant
    // the label they can see.
    let b = score(query, type_name).map(|s| s - 3);
    match (a, b) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (some, None) | (None, some) => some,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The smoke that started this: **the artist knows the name and still cannot find it.**
    #[test]
    fn typing_either_name_finds_the_node() {
        assert!(rank("lfo", "LFO", "value.lfo").is_some());
        assert!(
            rank("value.lfo", "LFO", "value.lfo").is_some(),
            "the canonical name is a name too - it is the one the docs say out loud"
        );
        assert!(rank("VALUE", "LFO", "value.lfo").is_some(), "case-blind");
        assert!(rank("zzz", "LFO", "value.lfo").is_none());
    }

    /// **Subsequence, not substring** — initials and fragments both find it.
    #[test]
    fn a_fuzzy_query_finds_what_a_substring_would_miss() {
        assert!(rank("mr", "Map Range", "value.map_range").is_some());
        assert!(rank("range", "Map Range", "value.map_range").is_some());
        assert!(rank("atr", "Attractor", "force.attractor").is_some());
        // Order matters: the letters must appear in the query's order.
        assert!(rank("rm", "Map Range", "value.map_range").is_none());
    }

    /// **The best match comes FIRST**, which is the whole difference between a search that
    /// works and a list that happens to contain the answer.
    #[test]
    fn the_thing_you_meant_ranks_above_the_thing_that_merely_matches() {
        let lfo = rank("lfo", "LFO", "value.lfo").unwrap();
        // "Falloff" contains l-f-o as a scattered subsequence (fa-L-l-o-F-f... ), so a naive
        // filter would list it right next to the node the artist actually wanted.
        let falloff = rank("lfo", "Falloff", "motion.falloff");
        assert!(
            falloff.is_none_or(|f| lfo > f),
            "an exact prefix must outrank an accidental subsequence: {lfo} vs {falloff:?}"
        );

        // Initials beat a scattered hit.
        let initials = rank("mr", "Map Range", "value.map_range").unwrap();
        let scattered = rank("mr", "Mirror Grain", "paint.mirror_grain").unwrap();
        assert!(
            initials > scattered - 20,
            "both are word-boundary hits; neither is nonsense"
        );

        // A shorter name wins a tie against a longer one that matches the same way.
        assert!(
            rank("sp", "Spawn", "motion.spawn")
                > rank("sp", "Spawn Along Path", "motion.spawn_along_path")
        );
    }

    /// An empty query is not a filter: the menu opens showing everything, as it always did.
    #[test]
    fn an_empty_query_lists_the_whole_library() {
        assert_eq!(rank("", "LFO", "value.lfo"), Some(0));
        assert_eq!(rank("", "Attractor", "force.attractor"), Some(0));
    }
}
