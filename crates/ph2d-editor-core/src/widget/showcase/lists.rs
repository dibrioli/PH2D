//! Showcase `paint_lists_section` painter.
//!
//! Extracted from monolithic `showcase.rs` in Wave 6+7 Phase 1.C.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_lists_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (mut y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_LISTS,
        ids::INSP_SECTION_LISTS_COLOR,
        "Lists",
        4,
    );
    if !open {
        return y;
    }

    // Dropdown.
    let r = Rect::new(x, y, w, FIELD_H);
    hit_index.register(ids::INSP_SAMPLE_DROPDOWN, r);
    let (dd_state, dd_open, dd_sel) = match store.get(ids::INSP_SAMPLE_DROPDOWN) {
        Some(InteractiveState::Dropdown {
            state,
            open,
            selected_index,
        }) => (*state, *open, *selected_index),
        _ => (DropdownState::Normal, false, Some(0)),
    };
    let labels = ["Front", "Side", "Top"];
    let selected_label = dd_sel
        .and_then(|i| labels.get(i).copied())
        .unwrap_or("Front");
    let dd = Dropdown::new(
        ids::INSP_SAMPLE_DROPDOWN,
        "View",
        vec![
            DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_A, "front", "Front"),
            DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_B, "side", "Side"),
            DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_C, "top", "Top"),
        ],
    )
    .selected(selected_label)
    .open(dd_open)
    .state(dd_state);
    // Chip first; popover is painted at the END of paint_inspector
    // (after every other section) so it lands ABOVE every other
    // widget. Without this, sections painted later in the loop
    // (Vector/Status/Color) covered the open list. Stash the open
    // dropdown + its chip rect via a thread-local for the second
    // pass — paint_inspector reads it after the section loop and
    // before pop_layer.
    crate::widget::paint_dropdown_chip(&dd, r, scene, text_system, theme);
    if dd_open {
        set_pending_dropdown_chip(Some((dd_sel.unwrap_or(0), r)));
    }
    y += FIELD_H + ROW_GAP;

    // Tabs (segmented).
    let r = Rect::new(x, y, w, ROW_H_PX);
    let selected = active_index(
        store,
        &[
            ids::INSP_SAMPLE_TAB_A,
            ids::INSP_SAMPLE_TAB_B,
            ids::INSP_SAMPLE_TAB_C,
        ],
    )
    .unwrap_or(0);
    let tabs = Tabs::new(
        NodeId(0),
        "Mode",
        vec![
            TabItem::new(ids::INSP_SAMPLE_TAB_A, "Edit"),
            TabItem::new(ids::INSP_SAMPLE_TAB_B, "Play"),
            TabItem::new(ids::INSP_SAMPLE_TAB_C, "Debug"),
        ],
    )
    .variant(TabsVariant::Segmented)
    .selected(selected);
    paint_tabs(&tabs, r, scene, text_system, theme);
    for (i, item) in tabs.items.iter().enumerate() {
        hit_index.register(item.id, tabs.tab_rect(r, i));
    }
    y += ROW_H_PX + Spacing::Xs.px();

    // Tab body — distinct sample per selected tab so the user can
    // see the tab actually swapping content (vs. just visual
    // emphasis on the segmented control). Each body is painted in a
    // `BgElev` rounded panel for visual grouping with the tabs.
    let body_h = ICON_BTN_SIZE_PX;
    let body_rect = Rect::new(x, y, w, body_h);
    fill_rounded_rect(
        scene,
        body_rect,
        Radius::Sm.px(),
        resolve(ColorToken::Bg2, theme),
    );
    let (caption, tone) = match selected {
        0 => ("Editing scene · pencil tool", ColorToken::Text1),
        1 => ("Running simulation · 60 fps", ColorToken::Success),
        _ => ("Logging · 124 events captured", ColorToken::Warn),
    };
    paint_text(
        text_system,
        scene,
        caption,
        body_rect.x + Spacing::Md.px(),
        body_rect.y + (body_rect.h - TypeToken::Xs.px()) * 0.5,
        TypeToken::Xs.px(),
        body_rect.w - Spacing::Md.px() * 2.0,
        resolve(tone, theme),
    );
    y += body_h + ROW_GAP;

    // TreeView (1 root + 2 leaves). Expand-state lives on the
    // store's `collapsed` side-table (same one that drives section
    // collapse) so clicking the root row truly hides/shows its
    // children. Selected leaf comes from the per-leaf `Pressed`
    // button state — `pin_button_selection` keeps exactly one
    // leaf Pressed across frames.
    let expanded = !store.is_collapsed(ids::INSP_SAMPLE_TREE_ROOT);
    let visible_rows = if expanded { 3.0 } else { 1.0 }; // LITERAL-PX-OK: row count constants for tree height calc
    let tree_h = Density::Cozy.row_h_px() * visible_rows;
    let r = Rect::new(x, y, w, tree_h);
    let mut tree = TreeView::new(
        NodeId(0),
        "Tree",
        vec![
            TreeNode::new(ids::INSP_SAMPLE_TREE_ROOT, "Group")
                .icon(IconId::Folder)
                .children(vec![
                    TreeNode::new(ids::INSP_SAMPLE_TREE_LEAF_A, "Item A").icon(IconId::Sprite),
                    TreeNode::new(ids::INSP_SAMPLE_TREE_LEAF_B, "Item B").icon(IconId::Sprite),
                ]),
        ],
    );
    if expanded {
        tree.expand(ids::INSP_SAMPLE_TREE_ROOT);
    }
    let selected_leaf = active_index(store, &TREE_LEAF_IDS).unwrap_or(0);
    tree.select(TREE_LEAF_IDS[selected_leaf]);
    paint_tree_view(
        &tree,
        r,
        scene,
        text_system,
        theme,
        Density::Cozy.row_h_px(),
    );
    // Register hit rects for each visible row.
    for (i, (_depth, node)) in tree.visible_rows().iter().enumerate() {
        hit_index.register(node.id, tree.row_rect(r, i, Density::Cozy.row_h_px()));
    }
    y += tree_h + ROW_GAP;

    // ListItem.
    let li_h = Density::Cozy.row_h_px();
    let r = Rect::new(x, y, w, li_h);
    hit_index.register(ids::INSP_SAMPLE_LIST_ITEM, r);
    let li_state = match store.get(ids::INSP_SAMPLE_LIST_ITEM) {
        Some(InteractiveState::ListItem { state, .. }) => *state,
        _ => ListItemState::Normal,
    };
    let item = ListItem::new(ids::INSP_SAMPLE_LIST_ITEM, "Open file")
        .icon(IconId::Open)
        // ASCII shortcut — the Command glyph U+2318 (⌘) isn't in the
        // editor's font fallback chain (parley GenericFamily::SansSerif
        // resolves to system fonts that don't include the Unicode
        // technical-symbol block). Showing `Cmd+O` instead of the
        // tofu `□O` for the keyboard hint.
        .value("Cmd+O")
        .chevron(true)
        .state(li_state);
    paint_list_item(&item, r, scene, text_system, theme);
    y + li_h
}
