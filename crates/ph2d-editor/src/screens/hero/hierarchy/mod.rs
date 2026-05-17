//! Hierarchy panel painter — header + add button + entity rows.

use super::HeroLayout;
use super::HeroSelection;
use super::fixture;
use super::ids;
use super::style::{
    HIER_ROW_H, PANEL_HEAD_PAD, paint_panel_corner_dot, paint_panel_surface,
    panel_drag_handle_rect, panel_resize_handle_rect,
};
use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{
    fill_rounded_rect, paint_icon, paint_text, paint_text_title, resolve, stroke_rounded_rect,
};
use crate::panel_registry::{PaintCtx, PanelManifest};
use crate::screens::hero::HeroScreen;
use crate::widget::{
    ButtonState, Tag, TagState, TagTone, TextInput, TextInputState, paint_tag,
    paint_text_input_with_buffer,
};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{
    ColorToken, ICON_BTN_SIZE_PX, ROW_H_PX, Radius, SECTION_GAP_PX, Spacing, StrokeToken, Theme,
    TypeToken,
};
use ph2d_vector::{Color as VelloColor, VectorScene};

/// Register the hierarchy header `+` button + every entity row's hit
/// id. Entity rows are `Plain` (focusable; no per-state visual
/// transitions — selection is driven by `apply_event` below).
/// Hierarchy panel + row painters live in sibling files (Wave 2 PR
/// 11.7b) so this `mod.rs` stays under the HR-18 600-LOC cap. Both
/// painters are re-exported `pub` here so external callers
/// (`screens::hero::paint_hero_screen`) keep using
/// `screens::hero::hierarchy::paint_hierarchy` unchanged.
mod panel_painter;
mod row_painter;
pub use panel_painter::paint_hierarchy;

/// Wave 5 stage C+D — declarative panel manifest. Stage C is a no-op
/// paint thunk; stage D moves the per-frame logic here.
pub static PANEL_MANIFEST: PanelManifest = PanelManifest {
    id: "hierarchy",
    panel_node_id: super::ids::HIER_PANEL,
    default_visible: true,
    paint_fn: paint_thunk,
    apply_event_fn: apply_event_thunk,
    populate_fn: populate,
};

#[allow(clippy::needless_pass_by_ref_mut)]
fn paint_thunk(_ctx: &mut PaintCtx) {}

fn apply_event_thunk(_hero: &mut HeroScreen, _ev: WidgetEvent) -> bool {
    false
}

pub fn populate(store: &mut WidgetStore) {
    store.register(
        ids::HIERARCHY_ADD,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    // M14.6 E: search/filter TextInput in the header. Pre-registered so
    // the dispatcher can route clicks/typing without per-frame allocs.
    store.register(
        ids::HIER_SEARCH,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    // Placeholder: only Scene Root is registered. The pilot project
    // populates real entities and reuses the reserved HIER_* ids
    // (see screens/hero/ids.rs). Live-data mode swaps this out via
    // [`repopulate`] at runtime (ADR-0025 M14.4a).
    let entities = [ids::HIER_PLAYER];
    for id in entities {
        store.register(id, InteractiveState::Plain);
    }
    // Seed the hierarchy display order (drag-and-drop reorders this
    // list at runtime).
    store.init_hierarchy_order(entities.to_vec());
}

/// Live-data variant of [`populate`]: register every id in `ids` as a
/// `Plain` interactive row + seed the hierarchy display order. Called
/// from [`crate::screens::hero::HeroScreen::sync_from_hierarchy`] per
/// frame with the entity-id list produced by the host's
/// `EntityNodeMap` bridge.
///
/// **Stale-id behavior:** `WidgetStore::register` is idempotent and
/// monotonic — stale rows from a previous frame remain in the store's
/// `states` map but are absent from `init_hierarchy_order`, so the
/// painter never iterates them. This is an acceptable bounded leak
/// for a session-long editor (typical: < 10k entity creates per
/// session). M15+ may add a `WidgetStore::retain` if it ever
/// matters.
pub fn repopulate(store: &mut WidgetStore, ids: &[ph2d_a11y::NodeId]) {
    // Register the `+` button once. `register_if_absent` so the
    // button's Pressed / Hovered transitions survive re-entry — a
    // plain `register` would reset to `Normal` every frame.
    store.register_if_absent(
        super::ids::HIERARCHY_ADD,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    // M14.6 E search-field state: must NOT clobber. Plain
    // `register` (which `states.insert` always replaces) was
    // resetting `text` / `caret` to empty every frame in live mode
    // → user couldn't type. `register_if_absent` inserts on the
    // first frame and is a true no-op thereafter, preserving the
    // typed query across the per-frame `repopulate`.
    store.register_if_absent(
        super::ids::HIER_SEARCH,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    for &id in ids {
        store.register_if_absent(id, InteractiveState::Plain);
    }
    // Use `set_hierarchy_order` (force-overwrite), NOT
    // `init_hierarchy_order` (idempotent guard) — the latter is a
    // no-op after `populate()` seeded `[HIER_PLAYER]` at boot, so
    // live mode would paint zero rows.
    store.set_hierarchy_order(ids.to_vec());
}

/// Apply a [`WidgetEvent`] against hierarchy widgets. A click on an
/// entity row updates `selection`; everything else is ignored.
/// Returns true iff the event was consumed.
///
/// When `live_entries` is `Some`, click-id resolution prefers it over
/// the static `ids::hierarchy_label_for_id` table — this is how
/// live-ECS mode (ADR-0025 M14.4a) names rows that don't have a
/// hardcoded `HIER_*` constant.
pub fn apply_event(
    _store: &mut WidgetStore,
    selection: &mut Option<HeroSelection>,
    live_entries: Option<&std::collections::BTreeMap<ph2d_a11y::NodeId, fixture::HierarchyEntity>>,
    event: WidgetEvent,
) -> bool {
    if let WidgetEvent::Click(id) = event {
        // Live mode first: looking up `id` in the live entries gives
        // us the entity's own Name (which the host writes there).
        if let Some(live) = live_entries
            && let Some(entry) = live.get(&id)
        {
            *selection = Some(HeroSelection {
                label: entry.name.clone(),
                kind: entry.badge.clone().unwrap_or_else(|| "ENT".to_string()),
                world_pos: (0.0, 0.0),
            });
            return true;
        }
        // Fixture fallback (single Scene Root row).
        if let Some(label) = ids::hierarchy_label_for_id(id) {
            *selection = Some(HeroSelection {
                label: label.into(),
                kind: ids::hierarchy_kind_for_label(label).into(),
                world_pos: (0.0, 0.0),
            });
            return true;
        }
    }
    false
}

thread_local! {
    /// Total height of the hierarchy entity list painted last
    /// frame. Hero clamps the scroll offset against this each
    /// frame so wheeling at the bottom doesn't overshoot.
    static LAST_HIER_CONTENT_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
}

pub fn last_hierarchy_content_h() -> f32 {
    LAST_HIER_CONTENT_H.with(|c| c.get())
}

fn set_last_hierarchy_content_h(h: f32) {
    LAST_HIER_CONTENT_H.with(|c| c.set(h));
}

// `paint_hierarchy` reads the current selection label via this
// thread-local since the painter takes the layout/store but not the
// hero-level selection. Set by `paint_hero_screen` before calling
// `paint_hierarchy`.
thread_local! {
    static CURRENT_SELECTION_LABEL: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
}

fn current_selection_label() -> Option<String> {
    CURRENT_SELECTION_LABEL.with(|c| c.borrow().clone())
}

pub(super) fn set_selection_label(label: Option<String>) {
    CURRENT_SELECTION_LABEL.with(|c| *c.borrow_mut() = label);
}

// Live-mode entity rows published by `HeroScreen::sync_from_hierarchy`
// (ADR-0025 M14.4a). When set, `paint_hierarchy` uses these instead of
// `fixture::hierarchy()`. Cleared by `paint_hero_screen` after the
// hierarchy paint so the next frame's `set_*` is the single source.
thread_local! {
    static CURRENT_LIVE_ENTRIES: std::cell::RefCell<
        Option<std::collections::BTreeMap<ph2d_a11y::NodeId, fixture::HierarchyEntity>>,
    > = const { std::cell::RefCell::new(None) };
}

fn current_live_entries()
-> Option<std::collections::BTreeMap<ph2d_a11y::NodeId, fixture::HierarchyEntity>> {
    CURRENT_LIVE_ENTRIES.with(|c| c.borrow().clone())
}

// M14.7 polish: row currently in inline-rename mode. Painter uses
// this to replace that row's name label with a TextInput overlay.
thread_local! {
    static CURRENT_RENAME_TARGET: std::cell::Cell<Option<ph2d_a11y::NodeId>> =
        const { std::cell::Cell::new(None) };
}

fn current_rename_target() -> Option<ph2d_a11y::NodeId> {
    CURRENT_RENAME_TARGET.with(|c| c.get())
}

pub(super) fn set_rename_target(target: Option<ph2d_a11y::NodeId>) {
    CURRENT_RENAME_TARGET.with(|c| c.set(target));
}

pub(super) fn set_live_entries(
    entries: Option<std::collections::BTreeMap<ph2d_a11y::NodeId, fixture::HierarchyEntity>>,
) {
    CURRENT_LIVE_ENTRIES.with(|c| *c.borrow_mut() = entries);
}

// Total component count across the live SimWorld, surfaced into the
// hierarchy header next to the entity count. Host writes this once
// per frame before `paint_hero_screen`; defaults to 0 when no host
// is publishing (e.g. fixture-only smoke tests).
thread_local! {
    static CURRENT_COMPONENT_COUNT: std::cell::Cell<u32> =
        const { std::cell::Cell::new(0) };
}

pub(crate) fn current_component_count() -> u32 {
    CURRENT_COMPONENT_COUNT.with(|c| c.get())
}

pub fn set_live_component_count(count: u32) {
    CURRENT_COMPONENT_COUNT.with(|c| c.set(count));
}

/// Compute which rows survive the M14.6 E hierarchy search filter.
///
/// Inputs are arrays in the DFS visit order produced by
/// [`build_hierarchy_snapshot`] (and threaded through the bridge):
/// `order[i]` is the row's `NodeId`, `depths[i]` is its depth from
/// root, and `entities_by_id` carries the per-row name. `query` is the
/// pre-lowercased search string the user typed; callers handle the
/// "empty query → show all" case before invoking this function.
///
/// Returns two parallel vectors of the same length as `order`:
/// - `display[i] == true` when row `i` should remain painted (either
///   it matched the query directly, or one of its descendants did)
/// - `direct[i] == true` when row `i` matched the query literally
///   (used by `paint_hierarchy_row` to render the name in Accent).
///
/// Algorithm: O(N × max_depth) worst case, O(N) when the matched
/// rows are sparse. A running stack of open-ancestor indices lets
/// each match propagate "visible" up to every parent of the match
/// without revisiting subtrees.
pub(crate) fn compute_match_filter(
    order: &[ph2d_a11y::NodeId],
    depths: &[u32],
    entities_by_id: &std::collections::BTreeMap<ph2d_a11y::NodeId, fixture::HierarchyEntity>,
    query: &str,
) -> (Vec<bool>, Vec<bool>) {
    let n = order.len();
    let mut display = vec![false; n];
    let mut direct = vec![false; n];
    // Stack of (index, depth) for ancestors whose subtree is still
    // open. Popped when the next row's depth dips back to or below
    // the ancestor — at which point the ancestor's subtree is sealed.
    let mut stack: Vec<usize> = Vec::with_capacity(16);
    for i in 0..n {
        let d = depths[i];
        while let Some(&top) = stack.last() {
            if depths[top] >= d {
                stack.pop();
            } else {
                break;
            }
        }
        let name_lower = entities_by_id
            .get(&order[i])
            .map(|e| e.name.to_lowercase())
            .unwrap_or_default();
        let is_match = !name_lower.is_empty() && name_lower.contains(query);
        if is_match {
            direct[i] = true;
            display[i] = true;
            // Mark every open ancestor so the path to the hit stays
            // painted even when the ancestor name itself doesn't
            // contain the query.
            for &a in &stack {
                display[a] = true;
            }
        }
        stack.push(i);
    }
    (display, direct)
}

#[cfg(test)]
mod search_tests {
    use super::*;
    use crate::icons::IconId;
    use crate::screens::hero::fixture::HierarchyEntity;
    use ph2d_a11y::NodeId;
    use std::collections::BTreeMap;

    fn entity(name: &str, indent: u8) -> HierarchyEntity {
        HierarchyEntity {
            name: name.to_string(),
            icon: IconId::Sprite,
            indent,
            badge: None,
            swatch: None,
            visible: true,
            selected: false,
            muted: false,
        }
    }

    fn build_tree() -> (Vec<NodeId>, Vec<u32>, BTreeMap<NodeId, HierarchyEntity>) {
        // group_a (0)
        //   ├── sprite_alpha (1)
        //   └── sprite_beta  (1)
        // group_b (0)
        //   └── sprite_gamma (1)
        let order = vec![NodeId(1), NodeId(2), NodeId(3), NodeId(4), NodeId(5)];
        let depths = vec![0, 1, 1, 0, 1];
        let mut map = BTreeMap::new();
        map.insert(NodeId(1), entity("group_a", 0));
        map.insert(NodeId(2), entity("sprite_alpha", 1));
        map.insert(NodeId(3), entity("sprite_beta", 1));
        map.insert(NodeId(4), entity("group_b", 0));
        map.insert(NodeId(5), entity("sprite_gamma", 1));
        (order, depths, map)
    }

    #[test]
    fn direct_match_keeps_ancestor_visible() {
        let (order, depths, map) = build_tree();
        let (display, direct) = compute_match_filter(&order, &depths, &map, "alpha");
        // alpha matched directly
        assert!(direct[1]);
        // group_a is its ancestor → kept visible for context
        assert!(display[0]);
        assert!(display[1]);
        // beta is a sibling, not on the matched path → hidden
        assert!(!display[2]);
        // group_b + gamma in a sibling subtree → hidden
        assert!(!display[3]);
        assert!(!display[4]);
        // ancestors don't get the "direct" highlight
        assert!(!direct[0]);
    }

    #[test]
    fn ancestor_match_does_not_pull_descendants() {
        let (order, depths, map) = build_tree();
        let (display, direct) = compute_match_filter(&order, &depths, &map, "group_a");
        // Only group_a matches directly
        assert!(direct[0]);
        assert!(display[0]);
        // Its children stay hidden — search is not a tree-expander
        // for parents, only a path-preserver for descendants.
        assert!(!display[1]);
        assert!(!display[2]);
        // group_b's subtree is untouched
        assert!(!display[3]);
        assert!(!display[4]);
    }

    #[test]
    fn case_insensitive_and_substring() {
        let (order, depths, map) = build_tree();
        // Caller normalizes to lowercase before invoking; verify a
        // partial substring across casing works once lowered.
        let (display, direct) = compute_match_filter(&order, &depths, &map, "gamm");
        assert!(direct[4]);
        // group_b is gamma's ancestor → visible
        assert!(display[3]);
        assert!(display[4]);
    }

    #[test]
    fn no_match_hides_everything() {
        let (order, depths, map) = build_tree();
        let (display, direct) = compute_match_filter(&order, &depths, &map, "zzz_does_not_exist");
        assert!(display.iter().all(|&b| !b));
        assert!(direct.iter().all(|&b| !b));
    }

    #[test]
    fn deep_chain_marks_every_ancestor() {
        // root → mid → leaf (depths 0, 1, 2)
        let order = vec![NodeId(10), NodeId(11), NodeId(12)];
        let depths = vec![0, 1, 2];
        let mut map = BTreeMap::new();
        map.insert(NodeId(10), entity("root", 0));
        map.insert(NodeId(11), entity("mid", 1));
        map.insert(NodeId(12), entity("leaf_xyz", 2));
        let (display, direct) = compute_match_filter(&order, &depths, &map, "xyz");
        assert!(display[0]);
        assert!(display[1]);
        assert!(display[2]);
        assert!(direct[2]);
        assert!(!direct[0]);
        assert!(!direct[1]);
    }
}
