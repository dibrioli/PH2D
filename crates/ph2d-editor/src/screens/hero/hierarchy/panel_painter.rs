//! `paint_hierarchy` — the hierarchy panel painter. Extracted from
//! `hierarchy/mod.rs` in Wave 2 PR 11.7b to bring the parent module
//! under the HR-18 600-LOC cap. Logic unchanged; the painter reads
//! thread-local state from `super` (selection / live entries / rename
//! target / component count) and calls `super::row_painter::paint_hierarchy_row`
//! for each visible row.

use super::*;

pub fn paint_hierarchy(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &mut WidgetStore,
) {
    let rect = layout.hierarchy;
    paint_panel_surface(rect, scene, theme);
    // Standard panel chrome hit zones — visual is in
    // `paint_panel_surface`. Re-registered after the body to outrank
    // any scrolled row that drifted into the chrome area.
    let drag_handle_rect = panel_drag_handle_rect(rect);
    let resize_handle_rect = panel_resize_handle_rect(rect);
    hit_index.register(ids::HIER_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::HIER_RESIZE_HANDLE, resize_handle_rect);

    let title_y = rect.y + 18.0; // LITERAL-PX-OK: panel title baseline (chrome-specific, matches PANEL_HEAD_PAD)
    paint_text_title(
        text_system,
        scene,
        "Hierarchy",
        rect.x + PANEL_HEAD_PAD,
        title_y,
        TypeToken::Md.px(),
        rect.w - PANEL_HEAD_PAD * 2.0 - 40.0, // LITERAL-PX-OK: reserve for header Add-button (≈ICON_BTN_SIZE chrome)
        resolve(ColorToken::Text1, theme),
    );
    // Live counts in live-ECS mode (real entries from the host
    // bridge); fall back to the fixture's placeholder pair otherwise.
    // Components count is derived from the bottom-HUD stats the host
    // already publishes — when stats are zero we hide the segment so
    // we don't lie with "0 components" during fixture mode.
    let (entities, components) = if let Some(live) = current_live_entries() {
        let entity_count = live.len() as u32;
        let comp_count = current_component_count();
        (entity_count, comp_count)
    } else {
        fixture::hierarchy_counts()
    };
    let counts = if components > 0 {
        format!("{entities} entities \u{00b7} {components} components")
    } else {
        format!("{entities} entities")
    };
    paint_text(
        text_system,
        scene,
        &counts,
        rect.x + PANEL_HEAD_PAD,
        title_y + TypeToken::Md.px() + Spacing::Xs.px(),
        TypeToken::Xs.px() - 1.0,
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text3, theme),
    );

    let add_size = 30.0_f32; // LITERAL-PX-OK: Add button square size (chrome-specific, distinct from ROW_H_PX)
    let add_rect = Rect::new(
        rect.x + rect.w - PANEL_HEAD_PAD - add_size,
        title_y - 2.0,
        add_size,
        add_size,
    );
    hit_index.register(ids::HIERARCHY_ADD, add_rect);
    let add_state = store
        .button_state(ids::HIERARCHY_ADD)
        .unwrap_or(ButtonState::Normal);
    let add_bg = match add_state {
        ButtonState::Pressed => ColorToken::Accent,
        ButtonState::Hovered => ColorToken::AccentSoft,
        _ => ColorToken::AccentSoft,
    };
    fill_rounded_rect(scene, add_rect, Radius::Full.px(), resolve(add_bg, theme));
    stroke_rounded_rect(
        scene,
        add_rect,
        Radius::Full.px(),
        1.0,
        resolve(ColorToken::Accent, theme),
    );
    let add_fg = if add_state == ButtonState::Pressed {
        ColorToken::AccentFg
    } else {
        ColorToken::Accent
    };
    paint_icon(
        scene,
        IconId::Add,
        add_rect,
        resolve(add_fg, theme),
        StrokeToken::Default.px(),
    );

    let header_bottom = title_y + TypeToken::Md.px() + TypeToken::Xs.px() + 18.0; // LITERAL-PX-OK: header baseline composite
    let body_pad = Spacing::Md.px();

    // M14.6 E: search/filter TextInput. Sits between the header and the
    // entity rows; its current buffer drives a case-insensitive name
    // filter with ancestor-path preservation (see `match_filter`).
    let search_h = ROW_H_PX;
    let search_rect = Rect::new(
        rect.x + body_pad,
        header_bottom,
        (rect.w - body_pad * 2.0).max(0.0),
        search_h,
    );
    hit_index.register(ids::HIER_SEARCH, search_rect);
    let (search_state, search_text, search_caret, search_anchor) = match store.get(ids::HIER_SEARCH)
    {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, text.clone(), *caret, *selection_anchor),
        _ => (TextInputState::Normal, String::new(), 0, None),
    };
    let search_input = TextInput::new(ids::HIER_SEARCH, "")
        .placeholder("Search\u{2026}")
        .state(search_state);
    paint_text_input_with_buffer(
        &search_input,
        Some(search_text.as_str()),
        Some(search_caret),
        search_anchor,
        search_rect,
        scene,
        text_system,
        theme,
    );

    let body_top = search_rect.y + search_rect.h + Spacing::Sm.px();
    // Scrollable content area below the header. Clip layer + wheel
    // offset so the entity list can grow past the panel bottom.
    let content_bottom = rect.y + rect.h - Spacing::Xs.px();
    let scroll_y = store.panel_scroll(ids::HIER_PANEL).max(0.0);
    let clip = ph2d_vector::Rect::new(
        rect.x as f64,
        body_top as f64,
        (rect.x + rect.w) as f64,
        content_bottom as f64,
    );
    scene.push_clip(&clip);
    let start_y = body_top - scroll_y;
    let mut y = start_y;
    // Reserve room for the scrollbar on the right (same convention
    // as the inspector — keeps row width stable regardless of
    // whether the scrollbar is currently visible).
    let scrollbar_reserve = crate::widget::SCROLLBAR_W + Spacing::Sm.px();
    let row_w = (rect.w - body_pad * 2.0 - scrollbar_reserve).max(0.0);
    let selected_label = current_selection_label();
    // Build NodeId → entity lookup so we can iterate by the store's
    // drag-and-drop order (which can differ from `fixture::hierarchy()`'s
    // default order after a reorder).
    //
    // Live mode (ADR-0025 M14.4a): when the host has called
    // `HeroScreen::sync_from_hierarchy`, the thread-local
    // [`current_live_entries`] holds the live ECS-derived entries —
    // those take precedence over `fixture::hierarchy()`.
    let entities_by_id: std::collections::BTreeMap<ph2d_a11y::NodeId, fixture::HierarchyEntity> =
        if let Some(live) = current_live_entries() {
            live
        } else {
            fixture::hierarchy()
                .into_iter()
                .filter_map(|e| ids::hierarchy_id(&e.name).map(|id| (id, e)))
                .collect()
        };
    // Copy the order into an owned Vec so the borrow on `store`
    // dies before the `set_hierarchy_row_ids` mutation at the end
    // of this function. Per-frame allocation cost is a single Vec
    // of NodeIds — negligible against the hierarchy panel's overall
    // paint budget.
    let order: Vec<ph2d_a11y::NodeId> = store.hierarchy_order().to_vec();
    let dragging = store.hierarchy_drag().filter(|d| d.active);
    // First pass: paint rows + register hit zones. Rows indent
    // horizontally by `depth × INDENT_PX` to make tree structure
    // visible after a drop-inside DnD.
    const INDENT_PX: f32 = Spacing::Xl.px();
    let mut row_rects: Vec<(ph2d_a11y::NodeId, Rect)> = Vec::with_capacity(order.len());
    // M14.6C: precompute depth per row + collapsed-gate. With DFS
    // order, `has_children` is just "next row's depth is greater";
    // a parent in `hierarchy_collapsed` skips ALL its descendants
    // until depth drops back to ≤ its own.
    let live_mode = current_live_entries().is_some();
    let depths: Vec<u32> = order
        .iter()
        .map(|id| {
            if live_mode {
                entities_by_id.get(id).map(|e| e.indent as u32).unwrap_or(0)
            } else {
                store.hierarchy_depth_of(*id)
            }
        })
        .collect();
    // M14.6 E: compute the search-filter visibility mask once per
    // frame. Empty query → every row stays visible; non-empty query
    // marks rows whose `name` matches (case-insensitive) AND every
    // ancestor of a match so the path to the hit stays painted.
    // While a non-empty query is active, collapse state is bypassed
    // — the user expects search to reveal the matching subtree even
    // if its parent was collapsed.
    let query = search_text.trim().to_lowercase();
    let search_active = !query.is_empty();
    let (display_mask, direct_match_mask): (Vec<bool>, Vec<bool>) = if search_active {
        compute_match_filter(&order, &depths, &entities_by_id, &query)
    } else {
        (Vec::new(), Vec::new())
    };
    let mut collapsed_gate: Option<u32> = None;
    for (i, id) in order.iter().enumerate() {
        let Some(entity_template) = entities_by_id.get(id) else {
            continue;
        };
        let depth = depths[i];
        // Exit a collapsed subtree when depth returns to ≤ the gate.
        if let Some(gate) = collapsed_gate
            && depth <= gate
        {
            collapsed_gate = None;
        }
        // Skip descendants while inside a collapsed subtree.
        if collapsed_gate.is_some() {
            continue;
        }
        // M14.6 E: drop rows the search query has filtered out. The
        // mask is computed in DFS order so per-index lookup is O(1).
        // Fallback to `false` (hide) on the off chance the mask is
        // shorter than `order` — under an active search, the safe
        // default is "show only what matched", not "show everything".
        if search_active && !display_mask.get(i).copied().unwrap_or(false) {
            continue;
        }
        let has_children = depths.get(i + 1).is_some_and(|&d| d > depth);
        // Search overrides collapse: while filtering, parents whose
        // descendants matched must reveal their subtree.
        let is_collapsed = has_children && !search_active && store.is_hierarchy_collapsed(*id);
        let mut entity = entity_template.clone();
        let indent = (depth as f32) * INDENT_PX;
        // Highlight rows whose name literally matched the query; the
        // ancestors retained for context render with the standard
        // Text1 color so the eye can follow the path.
        let direct_match = search_active && direct_match_mask.get(i).copied().unwrap_or(false);
        let row_rect = Rect::new(
            rect.x + body_pad + indent,
            y,
            (row_w - indent).max(80.0), // LITERAL-PX-OK: minimum row width when deeply indented (chrome-specific min)
            HIER_ROW_H,
        );
        if let Some(ref sel_label) = selected_label {
            entity.selected = entity.name == *sel_label;
        }
        // Dim the row currently being dragged so the user sees
        // "this is what's moving".
        entity.muted = entity.muted || dragging.map(|d| d.dragged == *id).unwrap_or(false);
        hit_index.register(*id, row_rect);
        let is_renaming = current_rename_target() == Some(*id);
        // Skip the row's name label when in rename mode — the
        // TextInput overlay below replaces it. Other row chrome
        // (chevron, icon, eye, badge) still paints normally.
        if is_renaming {
            entity.name = String::new();
        }
        super::row_painter::paint_hierarchy_row(
            &entity,
            row_rect,
            scene,
            text_system,
            theme,
            Some(*id),
            Some(hit_index),
            has_children,
            is_collapsed,
            direct_match,
        );
        if is_renaming {
            // Overlay TextInput at the row's name area. Width spans
            // from the row's name x to the right edge minus the
            // existing chrome (eye / badge / swatch reserved space).
            let icon_x_local = rect.x
                + Spacing::Md.px()
                + (depth as f32) * INDENT_PX
                + Spacing::Lg.px()
                + Spacing::Xs.px();
            let name_x = icon_x_local + Spacing::Xl.px() + Spacing::Md.px();
            let name_right = row_rect.x + row_rect.w - 10.0 - Spacing::Xl.px() - Spacing::Sm.px(); // LITERAL-PX-OK: 10 = chrome inset matching row_painter pad
            let input_rect = Rect::new(
                name_x - Spacing::Xs.px(),
                row_rect.y + 1.0,
                (name_right - name_x + Spacing::Md.px()).max(80.0), // LITERAL-PX-OK: min rename input width
                row_rect.h - 2.0,
            );
            hit_index.register(super::ids::HIER_RENAME_INPUT, input_rect);
            let (state, text, caret, anchor) = match store.get(super::ids::HIER_RENAME_INPUT) {
                Some(InteractiveState::TextInput {
                    state,
                    text,
                    caret,
                    selection_anchor,
                }) => (*state, text.clone(), *caret, *selection_anchor),
                _ => (TextInputState::Focused, String::new(), 0, None),
            };
            let input = TextInput::new(super::ids::HIER_RENAME_INPUT, "").state(state);
            paint_text_input_with_buffer(
                &input,
                Some(text.as_str()),
                Some(caret),
                anchor,
                input_rect,
                scene,
                text_system,
                theme,
            );
        }
        row_rects.push((*id, row_rect));
        if is_collapsed {
            collapsed_gate = Some(depth);
        }
        y += HIER_ROW_H + Spacing::Xxs.px();
    }
    // Second pass: drop indicator while dragging. Mirrors the
    // dispatch's `find_hierarchy_drop` exactly (y-only band split)
    // so the user sees the same outcome the drop will produce:
    //   - top 30% of y → 2px Accent line ABOVE the row (sibling)
    //   - middle 40% of y → Accent stroke around the row (child)
    //   - bottom 30% of y → falls through to the next row's "above"
    if let Some(d) = dragging {
        let mut drew = false;
        for (id, rrect) in &row_rects {
            if *id == d.dragged {
                continue;
            }
            let top = rrect.y;
            let bot = rrect.y + rrect.h;
            let inside_top = top + rrect.h * 0.3; // LITERAL-PX-OK: drop-zone partition ratio (30% sibling-above)
            let inside_bot = top + rrect.h * 0.7; // LITERAL-PX-OK: drop-zone partition ratio (70% boundary)
            if d.cursor_y < top || d.cursor_y >= bot {
                continue;
            }
            if d.cursor_y < inside_top {
                let indicator = Rect::new(rrect.x, rrect.y - 1.0, rrect.w, 2.0);
                crate::paint::fill_rounded_rect(
                    scene,
                    indicator,
                    1.0,
                    resolve(ColorToken::Accent, theme),
                );
                drew = true;
                break;
            } else if d.cursor_y < inside_bot {
                crate::paint::stroke_rounded_rect(
                    scene,
                    *rrect,
                    Spacing::Sm.px(),
                    StrokeToken::Thick.px(),
                    resolve(ColorToken::Accent, theme),
                );
                drew = true;
                break;
            } else {
                // M14.7 polish: bottom 30% → "After this row" sibling
                // insertion. Indicator is a thin Accent line at the
                // row's bottom edge — mirrors the Before indicator at
                // the top so the user sees a clear "drop slot" cue.
                let indicator = Rect::new(rrect.x, rrect.y + rrect.h - 1.0, rrect.w, 2.0);
                crate::paint::fill_rounded_rect(
                    scene,
                    indicator,
                    1.0,
                    resolve(ColorToken::Accent, theme),
                );
                drew = true;
                break;
            }
        }
        if !drew {
            // Past the last row — append at the end indicator.
            let indicator = Rect::new(rect.x + body_pad, y - 1.0, row_w, 2.0);
            crate::paint::fill_rounded_rect(
                scene,
                indicator,
                1.0,
                resolve(ColorToken::Accent, theme),
            );
        }
    }
    scene.pop_layer();
    // Standard panel chrome — corner dot painted on top of the
    // body, hit zones re-registered so they outrank scrolled rows.
    paint_panel_corner_dot(rect, scene, theme);
    hit_index.register(ids::HIER_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::HIER_RESIZE_HANDLE, resize_handle_rect);
    // Publish total content height for `dispatch_wheel` clamp.
    // `y` advances by full row + gap regardless of scroll offset
    // — the difference from `start_y` is the unscrolled content
    // height (same trick the inspector uses).
    let content_h = (y - start_y).max(0.0);
    set_last_hierarchy_content_h(content_h);

    // Scrollbar (right edge of the entity body region). Same
    // centralized widget as the inspector — single hit id reused
    // by the dispatch.
    let visible_h = (content_bottom - body_top).max(0.0);
    if crate::widget::scrollbar_is_needed(content_h, visible_h) {
        let body = Rect::new(rect.x, body_top, rect.w, visible_h);
        let track = crate::widget::scrollbar_track_rect(body);
        let thumb = crate::widget::scrollbar_thumb_rect(track, scroll_y, content_h, visible_h);
        let is_active = matches!(store.scrollbar_drag(), Some(d) if d.panel == ids::HIER_PANEL);
        crate::widget::paint_scrollbar(
            body, scroll_y, content_h, visible_h, is_active, scene, theme,
        );
        hit_index.register(crate::widget::HIERARCHY_SCROLLBAR_ID, thumb);
    }
    // M14.6B: publish the row set for the dispatcher. Both fixture
    // and live ids land in `order`; the dispatcher's
    // `is_hierarchy_row` check now resolves correctly in either
    // mode. Cleared and replaced wholesale every frame so stale
    // entries (e.g. a row that despawned) drop out automatically.
    let row_set: std::collections::BTreeSet<ph2d_a11y::NodeId> = order.iter().copied().collect();
    store.set_hierarchy_row_ids(row_set);
}
