//! Gates for the command-palette search filter. Sibling of `command_palette.rs` (kept a CHILD module by
//! `#[path]` so it reaches the private `filter_model`, and to keep the widget file under the LOC cap).

use super::*;
use ph2d_tokens::ColorToken;

fn item(label: &'static str) -> PaletteItem {
    PaletteItem {
        label: label.into(),
        id: hash_node_id(label),
    }
}

/// Source has Grid + Boids; Forces has a "Physics" sub with Gravity + Wind — enough to exercise the
/// display order (Source before Forces) and the drop-empty-category rule.
fn model() -> PaletteModel {
    PaletteModel {
        title: "Add Node".into(),
        groups: vec![
            PaletteGroup {
                title: "Source".into(),
                color: ColorToken::NodeCatSource,
                subs: vec![PaletteSub {
                    title: None,
                    items: vec![item("Grid"), item("Boids")],
                }],
            },
            PaletteGroup {
                title: "Forces".into(),
                color: ColorToken::NodeCatSource,
                subs: vec![PaletteSub {
                    title: Some("Physics".into()),
                    items: vec![item("Gravity"), item("Wind")],
                }],
            },
        ],
    }
}

/// **The one predicate is a case-insensitive substring, and empty matches everything.** The filter and
/// the `Enter` top-match both call this, so what is shown and what is added cannot disagree.
#[test]
fn item_matches_is_case_insensitive_substring_and_empty_matches_all() {
    assert!(item_matches("", "anything"), "empty query matches all");
    assert!(item_matches("boi", "Boids"));
    assert!(item_matches("BOI", "Boids"), "case-insensitive");
    assert!(!item_matches("xyz", "Boids"));
}

/// **A filtered library drops sub-clusters and categories that end up empty** — no coloured header sits
/// over nothing. FALSIFIED if `filter_model` kept a group whose items all filtered out.
#[test]
fn filter_drops_emptied_subs_and_categories() {
    let f = filter_model(&model(), "wind");
    assert_eq!(f.groups.len(), 1, "only the category with a match survives");
    assert_eq!(f.item_count(), 1);
    assert_eq!(f.groups[0].title, "Forces");
}

/// **Enter adds the first FILTERED (visible) item, using the same predicate the filter paints with.** So
/// a click and an Enter can never disagree about what the palette shows vs adds. FALSIFIED if `top_match`
/// used a looser rule than `filter_model` (it would return an id the filter hid).
#[test]
fn enter_picks_the_first_filtered_match_and_it_is_visible() {
    let m = model();
    let q = "g"; // matches Grid (Source) + Gravity (Forces); display order puts Grid first.
    let filtered = filter_model(&m, q);
    let top = top_match(&m, q).expect("`g` matches something");
    assert!(
        filtered.is_item(top),
        "the Enter pick is one of the visible, filtered items"
    );
    assert_eq!(top, hash_node_id("Grid"), "first match in display order");
}

/// **Enter is a no-op with no search text or no match** — it never adds a node the artist did not narrow
/// to. FALSIFIED if `top_match` returned the first item for an empty query.
#[test]
fn enter_is_a_noop_on_empty_or_no_match() {
    assert_eq!(top_match(&model(), ""), None, "empty search adds nothing");
    assert_eq!(top_match(&model(), "zzz"), None, "no match adds nothing");
}
