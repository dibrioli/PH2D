use super::*;

#[test]
fn three_rasters_and_a_nested_group() {
    // T3.1 criterion 1: create 3 raster layers + 1 nested group.
    let mut s = LayerStack::new();
    let a = s.add_raster("Layer 1", 64, 64).unwrap();
    let b = s.add_raster("Layer 2", 64, 64).unwrap();
    let _c = s.add_raster("Layer 3", 64, 64).unwrap();
    let g = s.add_group("Group").unwrap();
    // Newest is on top: root = [Group, L3, L2, L1].
    assert_eq!(s.root().first(), Some(&g));
    assert_eq!(s.len(), 4);
    // Nest two rasters into the group.
    assert!(s.move_into_group(a, g));
    assert!(s.move_into_group(b, g));
    assert_eq!(s.depth(a), 1);
    assert_eq!(s.depth(g), 0);
    // a and b left the root; group + L3 remain at root.
    assert!(!s.root().contains(&a));
    assert!(!s.root().contains(&b));
    assert_eq!(s.root().len(), 2);
}

#[test]
fn reorder_updates_stack_order() {
    // T3.1 criterion 2.
    let mut s = LayerStack::new();
    let a = s.add_raster("a", 8, 8).unwrap();
    let b = s.add_raster("b", 8, 8).unwrap();
    let c = s.add_raster("c", 8, 8).unwrap();
    // root top-to-bottom = [c, b, a]
    assert_eq!(s.root(), &[c, b, a]);
    // Move `a` to the top.
    s.reorder(a, 0);
    assert_eq!(s.root(), &[a, c, b]);
    // Move `a` back to the bottom.
    s.reorder(a, 2);
    assert_eq!(s.root(), &[c, b, a]);
}

#[test]
fn visibility_opacity_blend_setters() {
    // T3.1 criterion 3 (+ opacity/blend).
    let mut s = LayerStack::new();
    let a = s.add_raster("a", 8, 8).unwrap();
    s.set_visible(a, false);
    assert!(!s.get(a).unwrap().visible);
    s.set_opacity(a, 1.5); // clamps
    assert_eq!(s.get(a).unwrap().opacity, 1.0);
    s.set_opacity(a, -0.2);
    assert_eq!(s.get(a).unwrap().opacity, 0.0);
    s.set_blend_mode(a, BlendMode::Multiply);
    assert_eq!(s.get(a).unwrap().blend_mode, BlendMode::Multiply);
}

#[test]
fn add_raster_sets_active() {
    let mut s = LayerStack::new();
    let a = s.add_raster("a", 8, 8).unwrap();
    assert_eq!(s.active(), Some(a));
    let b = s.add_raster("b", 8, 8).unwrap();
    assert_eq!(s.active(), Some(b));
}

#[test]
fn group_nesting_capped_at_8_levels_matching_savefile() {
    let mut s = LayerStack::new();
    let mut groups = Vec::new();
    for i in 0..MAX_GROUP_DEPTH + 2 {
        groups.push(s.add_group(format!("g{i}")).unwrap());
    }
    // Nest the chain as deep as the cap allows.
    let mut deepest = groups[0]; // depth 0
    for &g in &groups[1..] {
        if s.move_into_group(g, deepest) {
            deepest = g;
        } else {
            break;
        }
    }
    // Deepest group sits at depth MAX_GROUP_DEPTH-1 = 7 → 8 levels (0..=7),
    // matching the frozen savefile validator (rejects a Group at depth ≥ 8).
    assert_eq!(s.depth(deepest), MAX_GROUP_DEPTH - 1);
    // A further nest (would reach depth MAX_GROUP_DEPTH) is rejected.
    let extra = *groups.last().unwrap();
    assert!(
        !s.move_into_group(extra, deepest),
        "nesting past {MAX_GROUP_DEPTH} levels must be rejected (savefile parity)"
    );
}

#[test]
fn move_into_group_rejects_subtree_that_would_overflow_depth() {
    // Regression (audit 2026-06-18): move_into_group must account for the MOVED
    // item's subtree HEIGHT, not just its own landing depth — else a reparent-
    // into-group drag builds a tree deeper than the FROZEN savefile validator
    // accepts (unsaveable). The old `group_nesting_capped` test never caught this
    // because it nests one empty group at a time (subtree_height 0 at each move).
    let mut s = LayerStack::new();

    // Target group G nested to depth 6 (one empty group per level — the leaf
    // case the old guard handled correctly).
    let mut g = s.add_group("g0").unwrap(); // depth 0
    for i in 1..=6 {
        let next = s.add_group(format!("g{i}")).unwrap();
        assert!(s.move_into_group(next, g));
        g = next;
    }
    assert_eq!(s.depth(g), 6, "target group G must sit at depth 6");

    // A NON-trivial subtree M = { N = { raster } }, subtree_height(M) == 2.
    let m = s.add_group("M").unwrap();
    let n = s.add_group("N").unwrap();
    let r = s.add_raster("r", 8, 8).unwrap();
    assert!(s.move_into_group(r, n));
    assert!(s.move_into_group(n, m));
    assert_eq!(s.subtree_height(m), 2, "M={{N={{raster}}}} has height 2");

    // Moving M into G would place N (a Group) at depth 8 → the frozen savefile
    // validator rejects Group@depth≥MAX_GROUP_DEPTH. Must be refused. The OLD
    // guard only checked depth(G)+1 = 7 < 8 and wrongly allowed it.
    assert!(
        !s.move_into_group(m, g),
        "moving a 2-deep subtree into a depth-6 group overflows the frozen \
         savefile depth cap — must be rejected"
    );

    // Positive control: a bare leaf still fits at depth 7 (≤ cap).
    let leaf = s.add_raster("leaf", 8, 8).unwrap();
    assert!(
        s.move_into_group(leaf, g),
        "a leaf raster lands at depth 7 (≤ cap) and must still be accepted"
    );
}

#[test]
fn remove_scrubs_dangling_mask_reference() {
    // audit W3: removing a layer that another layer references as its mask
    // must scrub the now-dangling ref (no `mask: Some(dead_id)`).
    let mut s = LayerStack::new();
    let owner = s.add_raster("owner", 8, 8).unwrap();
    let mask = s.add_raster("mask", 8, 8).unwrap();
    s.get_mut(owner).unwrap().mask = Some(mask);
    s.remove(mask);
    assert!(s.get(owner).is_some(), "owner survives mask removal");
    assert_eq!(
        s.get(owner).unwrap().mask,
        None,
        "dangling mask ref scrubbed"
    );
}

#[test]
fn cannot_nest_group_into_its_own_descendant() {
    let mut s = LayerStack::new();
    let outer = s.add_group("outer").unwrap();
    let inner = s.add_group("inner").unwrap();
    assert!(s.move_into_group(inner, outer));
    // Moving `outer` into `inner` would create a cycle → rejected.
    assert!(!s.move_into_group(outer, inner));
}

#[test]
fn remove_group_drops_subtree() {
    let mut s = LayerStack::new();
    let a = s.add_raster("a", 8, 8).unwrap();
    let g = s.add_group("g").unwrap();
    s.move_into_group(a, g);
    assert_eq!(s.len(), 2);
    s.remove(g);
    assert_eq!(s.len(), 0);
    assert_eq!(s.active(), None);
}

#[test]
fn remove_active_repoints_active_to_root() {
    let mut s = LayerStack::new();
    let a = s.add_raster("a", 8, 8).unwrap();
    let b = s.add_raster("b", 8, 8).unwrap();
    s.set_active(b);
    s.remove(b);
    // active falls back to a root layer (a).
    assert_eq!(s.active(), Some(a));
}

#[test]
fn ids_are_never_reused() {
    let mut s = LayerStack::new();
    let a = s.add_raster("a", 8, 8).unwrap();
    s.remove(a);
    let b = s.add_raster("b", 8, 8).unwrap();
    assert_ne!(a, b, "freed id must not be reused");
}

#[test]
fn add_mask_binds_to_raster_and_is_out_of_zorder() {
    // T3.5: a mask attaches to its raster parent via `mask`, NOT into root.
    let mut s = LayerStack::new();
    let r = s.add_raster("r", 8, 8).unwrap();
    let m = s.add_mask(r).unwrap();
    assert_eq!(s.get(r).unwrap().mask, Some(m), "parent points at the mask");
    assert!(matches!(s.get(m).unwrap().kind, LayerKind::Mask(_)));
    assert!(!s.root().contains(&m), "mask is not in the z-order");
    assert_eq!(s.len(), 2, "both live in the arena");
}

#[test]
fn add_mask_rejects_group_and_double_mask() {
    let mut s = LayerStack::new();
    let g = s.add_group("g").unwrap();
    assert!(s.add_mask(g).is_none(), "a group cannot have a mask");
    let r = s.add_raster("r", 8, 8).unwrap();
    assert!(s.add_mask(r).is_some());
    assert!(s.add_mask(r).is_none(), "no second mask on one raster");
}

#[test]
fn remove_parent_also_removes_its_mask() {
    // T3.5 closes the old collect_subtree TODO: removing the owner must drop
    // its owner-attached mask too (else it leaks in the tool's images map).
    let mut s = LayerStack::new();
    let parent = s.add_raster("p", 4, 4).unwrap();
    let mask = s.add_mask(parent).unwrap();
    assert_eq!(s.len(), 2);
    s.remove(parent);
    assert_eq!(s.len(), 0, "parent + mask both removed");
    assert!(s.get(mask).is_none(), "mask did not leak");
}

#[test]
fn reference_layer_is_exclusive() {
    // §2.9: only one reference layer per canvas — setting a new one clears
    // the previous.
    let mut s = LayerStack::new();
    let a = s.add_raster("a", 4, 4).unwrap();
    let b = s.add_raster("b", 4, 4).unwrap();
    s.set_reference(a, true);
    assert!(s.get(a).unwrap().is_reference);
    s.set_reference(b, true);
    assert!(s.get(b).unwrap().is_reference);
    assert!(
        !s.get(a).unwrap().is_reference,
        "previous reference cleared"
    );
    s.set_reference(b, false);
    assert!(!s.get(b).unwrap().is_reference, "toggled off");
}

#[test]
fn alpha_lock_and_clipping_flags_round_trip() {
    let mut s = LayerStack::new();
    let r = s.add_raster("r", 4, 4).unwrap();
    s.set_alpha_locked(r, true);
    s.set_clipping(r, true);
    assert!(s.get(r).unwrap().alpha_locked);
    assert!(s.get(r).unwrap().clipping);
    s.set_alpha_locked(r, false);
    assert!(!s.get(r).unwrap().alpha_locked);
}

#[test]
fn move_to_sibling_reorders_and_pins_base() {
    let mut s = LayerStack::new();
    let a = s.add_raster("a", 8, 8).unwrap(); // root=[a]   (a = base, bottom)
    let b = s.add_raster("b", 8, 8).unwrap(); // [b, a]
    let c = s.add_raster("c", 8, 8).unwrap(); // [c, b, a]
    assert!(s.move_to_sibling_of(c, b, true));
    assert_eq!(s.root(), &[b, c, a], "c after b");
    assert!(s.move_to_sibling_of(c, b, false));
    assert_eq!(s.root(), &[c, b, a], "c before b");
    // Dropping after the base clamps to above it — base stays pinned bottom.
    assert!(s.move_to_sibling_of(c, a, true));
    assert_eq!(s.root().last(), Some(&a), "base stays at root bottom");
}

#[test]
fn move_to_sibling_into_group_via_targets_parent() {
    let mut s = LayerStack::new();
    let _base = s.add_raster("base", 8, 8).unwrap();
    let g = s.add_group("g").unwrap();
    let inner = s.add_raster("inner", 8, 8).unwrap();
    assert!(s.move_into_group(inner, g));
    let x = s.add_raster("x", 8, 8).unwrap();
    // Drop x before `inner` (inside g) → x joins the group.
    assert!(s.move_to_sibling_of(x, inner, false));
    assert_eq!(s.depth(x), 1, "x moved into the group (target's parent)");
}

#[test]
fn base_sprite_never_moves() {
    let mut s = LayerStack::new();
    let base = s.add_raster("base", 8, 8).unwrap();
    let x = s.add_raster("x", 8, 8).unwrap();
    assert!(
        !s.move_to_sibling_of(base, x, false),
        "base can't be reordered"
    );
    assert!(
        !s.move_to_root_bottom_above_base(base),
        "base can't move to bottom"
    );
}

#[test]
fn move_to_root_bottom_lands_above_base() {
    let mut s = LayerStack::new();
    let base = s.add_raster("base", 8, 8).unwrap();
    let g = s.add_group("g").unwrap();
    let inner = s.add_raster("inner", 8, 8).unwrap();
    s.move_into_group(inner, g);
    // Pull `inner` out of the group to root bottom (above the base).
    assert!(s.move_to_root_bottom_above_base(inner));
    assert_eq!(s.depth(inner), 0, "inner back at root level");
    assert_eq!(s.root().last(), Some(&base), "base still pinned bottom");
    assert_eq!(
        s.root().iter().rev().nth(1),
        Some(&inner),
        "inner just above base"
    );
}

#[test]
fn remove_mask_alone_scrubs_parent_ref() {
    let mut s = LayerStack::new();
    let parent = s.add_raster("p", 4, 4).unwrap();
    let mask = s.add_mask(parent).unwrap();
    s.remove(mask);
    assert!(s.get(parent).is_some(), "parent survives mask removal");
    assert_eq!(
        s.get(parent).unwrap().mask,
        None,
        "dangling mask ref scrubbed"
    );
}
