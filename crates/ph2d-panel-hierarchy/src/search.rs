//! M14.6 E hierarchy search filter — DFS ancestor-preserving match
//! mask. Ported verbatim from
//! `ph2d_editor_core::screens::hero::hierarchy::compute_match_filter`
//! in Phase C.2.

use ph2d_editor_core::screens::hero::fixture;

/// Compute which rows survive the search filter.
///
/// Inputs are arrays in DFS visit order: `order[i]` is the row's
/// `NodeId`, `depths[i]` its tree depth, and `entities_by_id`
/// carries the per-row name. `query` is the pre-lowercased search
/// string the user typed; callers handle the "empty query → show
/// all" case before invoking.
///
/// Returns parallel `(display, direct)` vectors of the same length
/// as `order`:
/// - `display[i] == true` when row `i` should remain painted (it
///   matched directly OR one of its descendants did).
/// - `direct[i] == true` when row `i` matched the query literally
///   (used by `paint_hierarchy_row` to paint the name in Accent).
///
/// Algorithm: O(N × max_depth) worst case, O(N) when matches are
/// sparse. A running stack of open-ancestor indices lets each match
/// propagate "visible" up to every parent without revisiting
/// subtrees.
pub(crate) fn compute_match_filter(
    order: &[ph2d_a11y::NodeId],
    depths: &[u32],
    entities_by_id: &std::collections::BTreeMap<ph2d_a11y::NodeId, fixture::HierarchyEntity>,
    query: &str,
) -> (Vec<bool>, Vec<bool>) {
    let n = order.len();
    let mut display = vec![false; n];
    let mut direct = vec![false; n];
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
            for &a in &stack {
                display[a] = true;
            }
        }
        stack.push(i);
    }
    (display, direct)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_a11y::NodeId;
    use ph2d_editor_core::icons::IconId;
    use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
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
            locked: false,
            group_locked: false,
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
        assert!(direct[1]);
        assert!(display[0]);
        assert!(display[1]);
        assert!(!display[2]);
        assert!(!display[3]);
        assert!(!display[4]);
        assert!(!direct[0]);
    }

    #[test]
    fn ancestor_match_does_not_pull_descendants() {
        let (order, depths, map) = build_tree();
        let (display, direct) = compute_match_filter(&order, &depths, &map, "group_a");
        assert!(direct[0]);
        assert!(display[0]);
        assert!(!display[1]);
        assert!(!display[2]);
        assert!(!display[3]);
        assert!(!display[4]);
    }

    #[test]
    fn case_insensitive_and_substring() {
        let (order, depths, map) = build_tree();
        let (display, direct) = compute_match_filter(&order, &depths, &map, "gamm");
        assert!(direct[4]);
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
