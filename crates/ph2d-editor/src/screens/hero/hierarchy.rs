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
use crate::widget::{
    ButtonState, Tag, TagState, TagTone, TextInput, TextInputState, paint_tag,
    paint_text_input_with_buffer,
};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Theme, TypeToken};
use ph2d_vector::{Color as VelloColor, VectorScene};

/// Register the hierarchy header `+` button + every entity row's hit
/// id. Entity rows are `Plain` (focusable; no per-state visual
/// transitions — selection is driven by `apply_event` below).
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

    let title_y = rect.y + 18.0;
    paint_text_title(
        text_system,
        scene,
        "Hierarchy",
        rect.x + PANEL_HEAD_PAD,
        title_y,
        TypeToken::Md.px(),
        rect.w - PANEL_HEAD_PAD * 2.0 - 40.0,
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
        title_y + TypeToken::Md.px() + 4.0,
        TypeToken::Xs.px() - 1.0,
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text3, theme),
    );

    let add_size = 30.0_f32;
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
    fill_rounded_rect(scene, add_rect, 999.0, resolve(add_bg, theme));
    stroke_rounded_rect(
        scene,
        add_rect,
        999.0,
        1.0,
        resolve(ColorToken::Accent, theme),
    );
    let add_fg = if add_state == ButtonState::Pressed {
        ColorToken::AccentFg
    } else {
        ColorToken::Accent
    };
    paint_icon(scene, IconId::Add, add_rect, resolve(add_fg, theme), 1.5);

    let header_bottom = title_y + TypeToken::Md.px() + TypeToken::Xs.px() + 18.0;
    let body_pad = 8.0_f32;

    // M14.6 E: search/filter TextInput. Sits between the header and the
    // entity rows; its current buffer drives a case-insensitive name
    // filter with ancestor-path preservation (see `match_filter`).
    let search_h = 28.0_f32;
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

    let body_top = search_rect.y + search_rect.h + 6.0;
    // Scrollable content area below the header. Clip layer + wheel
    // offset so the entity list can grow past the panel bottom.
    let content_bottom = rect.y + rect.h - 4.0;
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
    let scrollbar_reserve = crate::widget::SCROLLBAR_W + 6.0;
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
    const INDENT_PX: f32 = 16.0;
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
            (row_w - indent).max(80.0),
            HIER_ROW_H,
        );
        if let Some(ref sel_label) = selected_label {
            entity.selected = entity.name == *sel_label;
        }
        // Dim the row currently being dragged so the user sees
        // "this is what's moving".
        entity.muted = entity.muted || dragging.map(|d| d.dragged == *id).unwrap_or(false);
        hit_index.register(*id, row_rect);
        paint_hierarchy_row(
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
        row_rects.push((*id, row_rect));
        if is_collapsed {
            collapsed_gate = Some(depth);
        }
        y += HIER_ROW_H + 2.0;
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
            let inside_top = top + rrect.h * 0.3;
            let inside_bot = top + rrect.h * 0.7;
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
                    6.0,
                    2.0,
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

#[allow(clippy::too_many_arguments)]
fn paint_hierarchy_row(
    entity: &fixture::HierarchyEntity,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    row_id: Option<ph2d_a11y::NodeId>,
    mut hit_index: Option<&mut HitIndex>,
    has_children: bool,
    is_collapsed: bool,
    direct_match: bool,
) {
    if entity.selected {
        fill_rounded_rect(
            scene,
            rect,
            Radius::Sm.px(),
            resolve(ColorToken::AccentSoft, theme),
        );
        stroke_rounded_rect(
            scene,
            rect,
            Radius::Sm.px(),
            1.0,
            resolve(ColorToken::Accent, theme),
        );
    }
    let indent_w = 16.0 * entity.indent as f32;
    let pad = 10.0_f32;
    // M14.6C: chevron column. Always reserves 16 px so child rows
    // align with their parents' entity-icon column. Painted only when
    // `has_children` (otherwise the slot is empty whitespace).
    let chev_w = 12.0_f32;
    let chev_pad = 4.0_f32;
    let chev_x = rect.x + pad + indent_w;
    if has_children {
        let chev_rect = Rect::new(chev_x, rect.y + (rect.h - chev_w) * 0.5, chev_w, chev_w);
        let chev_icon = if is_collapsed {
            IconId::ChevronRight
        } else {
            IconId::ChevronDown
        };
        paint_icon(
            scene,
            chev_icon,
            chev_rect,
            resolve(ColorToken::Text2, theme),
            1.5,
        );
        if let (Some(row_id), Some(idx)) = (row_id, hit_index.as_mut()) {
            // Hit-rect: chevron glyph + padding for 24×24 click target.
            let hit_rect = Rect::new(
                chev_rect.x - 6.0,
                chev_rect.y - 6.0,
                chev_w + 12.0,
                chev_w + 12.0,
            );
            idx.register(
                crate::screens::hero::ids::hier_expand_companion(row_id),
                hit_rect,
            );
        }
    }
    let icon_w = 16.0_f32;
    let icon_x = chev_x + chev_w + chev_pad;
    let icon_rect = Rect::new(icon_x, rect.y + (rect.h - icon_w) * 0.5, icon_w, icon_w);
    let icon_color = if entity.selected {
        ColorToken::Accent
    } else if entity.muted {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text3
    };
    paint_icon(
        scene,
        entity.icon,
        icon_rect,
        resolve(icon_color, theme),
        1.5,
    );

    let mut right_x = rect.x + rect.w - pad;
    // M14.6A: clickable eye icon replaces the legacy visibility dot.
    // Open eye = visible (Text2), closed eye = hidden (TextDisabled).
    let eye_icon = if entity.visible {
        IconId::Eye
    } else {
        IconId::EyeClosed
    };
    let eye_color = if entity.visible {
        ColorToken::Text2
    } else {
        ColorToken::TextDisabled
    };
    let eye_size = 16.0_f32;
    let eye_rect = Rect::new(
        right_x - eye_size,
        rect.y + (rect.h - eye_size) * 0.5,
        eye_size,
        eye_size,
    );
    paint_icon(scene, eye_icon, eye_rect, resolve(eye_color, theme), 1.5);
    if let (Some(row_id), Some(idx)) = (row_id, hit_index.as_mut()) {
        let hit_pad = 4.0_f32;
        let hit_rect = Rect::new(
            eye_rect.x - hit_pad,
            eye_rect.y - hit_pad,
            eye_rect.w + hit_pad * 2.0,
            eye_rect.h + hit_pad * 2.0,
        );
        idx.register(
            crate::screens::hero::ids::hier_eye_companion(row_id),
            hit_rect,
        );
    }
    right_x -= eye_size + 6.0;
    if let Some(swatch) = entity.swatch {
        let sw = 14.0_f32;
        let sw_rect = Rect::new(right_x - sw, rect.y + (rect.h - sw) * 0.5, sw, sw);
        let [r, g, b, a] = swatch;
        fill_rounded_rect(scene, sw_rect, 4.0, VelloColor::from_rgba8(r, g, b, a));
        stroke_rounded_rect(scene, sw_rect, 4.0, 1.0, resolve(ColorToken::Border, theme));
        right_x -= sw + 6.0;
    }
    if let Some(badge) = &entity.badge {
        // Phase 2 polish: render the kind badge as a `Tag` widget
        // tinted by kind (PRF=Accent, UNI=Neutral, CAM=Success,
        // OUT=Warn, etc). Functional in the sense that it shares
        // identity with the rest of the editor's chrome.
        let badge_w = 36.0_f32;
        let badge_h = 18.0_f32;
        let badge_rect = Rect::new(
            right_x - badge_w,
            rect.y + (rect.h - badge_h) * 0.5,
            badge_w,
            badge_h,
        );
        let tone = match badge.as_str() {
            "PRF" => TagTone::Accent,
            "UNI" => TagTone::Neutral,
            "OUT" => TagTone::Warn,
            "CAM" => TagTone::Success,
            "TIL" => TagTone::Neutral,
            "TRG" => TagTone::Danger,
            "LGT" => TagTone::Warn,
            "SPR" => TagTone::Accent,
            _ => TagTone::Neutral,
        };
        let tag = Tag::new(ph2d_a11y::NodeId(0), badge)
            .tone(tone)
            .state(if entity.muted {
                TagState::Disabled
            } else {
                TagState::Normal
            });
        paint_tag(&tag, badge_rect, scene, text_system, theme);
        right_x -= badge_w + 6.0;
    }

    let name_x = icon_rect.x + icon_w + 8.0;
    let name_color = if entity.muted {
        ColorToken::TextDisabled
    } else if direct_match {
        // M14.6 E: rows whose name literally matched the search query
        // get the Accent color so the eye locks onto hits even when
        // ancestors are painted alongside them for context.
        ColorToken::Accent
    } else {
        ColorToken::Text1
    };
    paint_text(
        text_system,
        scene,
        &entity.name,
        name_x,
        rect.y + (rect.h - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        (right_x - name_x).max(0.0),
        resolve(name_color, theme),
    );
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
