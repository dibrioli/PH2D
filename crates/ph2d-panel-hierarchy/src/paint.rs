//! Hierarchy panel paint — ADR-0029 Phase C.2 port of
//! `ph2d_editor_core::screens::hero::hierarchy::{paint_thunk,
//! paint_hierarchy}`.
//!
//! `paint` is the typed entry point invoked by the `Panel` trait. It
//! gates on visibility (via [`PanelHostInternal::panel_visible`]),
//! paints the panel chrome + entity rows, and publishes scroll
//! bounds + the per-frame row set back to the store.

use crate::HierarchyPanel;
use crate::row::paint_hierarchy_row;
use crate::search::compute_match_filter;
use crate::state::{
    self, current_component_count, current_live_entries, set_last_hierarchy_content_h,
};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::screens::hero::fixture;
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_TITLE_BASELINE, paint_panel_corner_dot, paint_panel_surface,
    paint_panel_title, panel_drag_handle_rect, panel_resize_handle_rect,
};
use ph2d_editor_core::widget::{
    self, ButtonState, HIERARCHY_SCROLLBAR_ID, SCROLLBAR_W, TextInput, TextInputState,
    paint_text_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, HIER_ROW_H_PX, ROW_H_PX, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

const HIER_ROW_H: f32 = HIER_ROW_H_PX;

pub(crate) fn paint(state: &mut state::HierarchyState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(HierarchyPanel::ID) {
        return;
    }
    let theme = ctx.host.theme();
    let selection_label = ctx.host.selection().map(|s| s.label.clone());
    let rename_target = state.rename_target_row;
    let row_set = {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        paint_hierarchy_body(
            ctx.layout,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            selection_label.as_deref(),
            rename_target,
        )
    };
    let content_h = state::last_hierarchy_content_h();
    let store = ctx.host.store_mut();
    store.set_hierarchy_row_ids(row_set);
    store.set_panel_content_h(ids::HIER_PANEL, content_h);
    let visible_h = (ctx.layout.hierarchy.h - 60.0).max(0.0); // LITERAL-PX-OK: header+scrollbar reserve composite (chrome dim)
    let max_scroll = (content_h - visible_h).max(0.0);
    let cur = store.panel_scroll(ids::HIER_PANEL);
    if cur > max_scroll {
        store.set_panel_scroll(ids::HIER_PANEL, max_scroll);
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_hierarchy_body(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    selection_label: Option<&str>,
    rename_target: Option<ph2d_a11y::NodeId>,
) -> std::collections::BTreeSet<ph2d_a11y::NodeId> {
    let rect = layout.hierarchy;
    paint_panel_surface(rect, scene, theme);
    let drag_handle_rect = panel_drag_handle_rect(
        rect,
        ph2d_editor_core::widget::panel_chrome::PANEL_HEADER_H_DEFAULT,
        // Hierarchy header has the Add (+) icon on the right; reserve
        // wider so drag doesn't shadow the add hit.
        ph2d_editor_core::widget::panel_chrome::PANEL_HEADER_ADD_RESERVE,
    );
    let resize_handle_rect = panel_resize_handle_rect(rect);
    let resize_handle_bl_rect =
        ph2d_editor_core::widget::panel_chrome::panel_resize_handle_rect_bl(rect);
    hit_index.register(ids::HIER_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::HIER_RESIZE_HANDLE, resize_handle_rect);
    hit_index.register(ids::HIER_RESIZE_HANDLE_BL, resize_handle_bl_rect);

    // Canonical panel title (single source of truth — `panel_chrome`).
    // Reserve ≈ICON_BTN_SIZE on the right for the header Add-button.
    let title_y = rect.y + PANEL_TITLE_BASELINE;
    let title_size = paint_panel_title(rect, "Hierarchy", 40.0, scene, text_system, theme); // LITERAL-PX-OK: Add-button reserve
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
        title_y + title_size + Spacing::Xs.px(),
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
    // Canonical icon button (same ghost-icon look as every other icon
    // button — e.g. panel Close). Was a one-off accent-circle that read
    // as "disabled" (AccentSoft) under the cursor.
    let add_state = store
        .button_state(ids::HIERARCHY_ADD)
        .unwrap_or(ButtonState::Normal);
    let add_btn = widget::Button::new(ids::HIERARCHY_ADD, "Add")
        .icon_only(IconId::Add)
        .state(add_state);
    widget::paint_button(&add_btn, add_rect, scene, text_system, theme);

    let header_bottom = title_y + TypeToken::Md.px() + TypeToken::Xs.px() + 18.0; // LITERAL-PX-OK: header baseline composite
    // Body padding reduced (Md → Xs) 2026-05-24 per user feedback
    // ("nomes mais próximos da borda esquerda do painel (quase sem
    // padding)") — Godot's Scene panel sits text flush with the
    // panel chrome. Search field + rows now share this tighter inset.
    let body_pad = Spacing::Xs.px();

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
    let scrollbar_reserve = SCROLLBAR_W + Spacing::Sm.px();
    let row_w = (rect.w - body_pad * 2.0 - scrollbar_reserve).max(0.0);
    let entities_by_id: std::collections::BTreeMap<ph2d_a11y::NodeId, fixture::HierarchyEntity> =
        if let Some(live) = current_live_entries() {
            live
        } else {
            fixture::hierarchy()
                .into_iter()
                .filter_map(|e| ids::hierarchy_id(&e.name).map(|id| (id, e)))
                .collect()
        };
    let order: Vec<ph2d_a11y::NodeId> = store.hierarchy_order().to_vec();
    let dragging = store.hierarchy_drag().filter(|d| d.active);

    const INDENT_PX: f32 = Spacing::Xl.px();
    let mut row_rects: Vec<(ph2d_a11y::NodeId, Rect)> = Vec::with_capacity(order.len());
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
        if let Some(gate) = collapsed_gate
            && depth <= gate
        {
            collapsed_gate = None;
        }
        if collapsed_gate.is_some() {
            continue;
        }
        if search_active && !display_mask.get(i).copied().unwrap_or(false) {
            continue;
        }
        let has_children = depths.get(i + 1).is_some_and(|&d| d > depth);
        let is_collapsed = has_children && !search_active && store.is_hierarchy_collapsed(*id);
        let mut entity = entity_template.clone();
        let indent = (depth as f32) * INDENT_PX;
        let direct_match = search_active && direct_match_mask.get(i).copied().unwrap_or(false);
        let row_rect = Rect::new(
            rect.x + body_pad + indent,
            y,
            (row_w - indent).max(80.0), // LITERAL-PX-OK: minimum row width when deeply indented (chrome-specific min)
            HIER_ROW_H,
        );
        // Fase 0 hotfix: respect any `selected` flag set by the shell
        // when building hierarchy entries (multi-row highlight comes
        // pre-marked). Fall back to label match only for entries that
        // weren't marked — covers the fixture/demo path where there
        // is no live bridge.
        if !entity.selected
            && let Some(sel_label) = selection_label
        {
            entity.selected = entity.name == sel_label;
        }
        entity.muted = entity.muted || dragging.map(|d| d.dragged == *id).unwrap_or(false);
        hit_index.register(*id, row_rect);
        let is_renaming = rename_target == Some(*id);
        if is_renaming {
            entity.name = String::new();
        }
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
        // Godot-style tree lines connecting parents to children. For
        // each ancestor depth column c < depth, draw a vertical line
        // (full row height) if a future row at depth ≥ c exists and
        // the column is still "alive" (no closer-up ancestor closes
        // it). The deepest column (c == depth - 1) gets an L-stub:
        // vertical down to mid-row, then horizontal to the chevron.
        if depth > 0 {
            // Internal row pad matches `row.rs::49` exactly.
            let row_inner_pad = 2.0_f32; // LITERAL-PX-OK: row inset (sync with row.rs)
            let chev_col_w = Spacing::Lg.px(); // sync with row.rs chev_w
            let half_chev = chev_col_w * 0.5;
            // Tree lines: bumped from `Border` → `Text3` 2026-05-24
            // per user: "linhas com mais contraste pois quase não
            // aparecem no fundo". Text3 reads as a soft chrome dim
            // (≈ same as the panel chevrons) so the relationship is
            // clear without competing with the row content.
            let line_color = resolve(ColorToken::Text3, theme);
            for c in 0..(depth as usize) {
                let col_chev_x = rect.x + body_pad + c as f32 * INDENT_PX + row_inner_pad;
                let line_x = col_chev_x + half_chev;
                let is_my_column = c == (depth as usize) - 1;
                if is_my_column {
                    let vert = Rect::new(
                        line_x - 0.5, // LITERAL-PX-OK: 1-px line centered
                        row_rect.y,
                        1.0,
                        row_rect.h * 0.5,
                    );
                    ph2d_editor_core::paint::fill_rounded_rect(scene, vert, 0.0, line_color);
                    let next_col_chev =
                        rect.x + body_pad + (c + 1) as f32 * INDENT_PX + row_inner_pad;
                    let h_stub = Rect::new(
                        line_x,
                        row_rect.y + row_rect.h * 0.5 - 0.5,
                        (next_col_chev - line_x).max(0.0),
                        1.0,
                    );
                    ph2d_editor_core::paint::fill_rounded_rect(scene, h_stub, 0.0, line_color);
                } else {
                    // Continues past me iff some row j > i has depth
                    // == c+1 before any row drops below c+1.
                    let target = c as u32 + 1;
                    let mut continues = false;
                    for &d in &depths[(i + 1)..] {
                        if d < target {
                            break;
                        }
                        if d == target {
                            continues = true;
                            break;
                        }
                    }
                    if continues {
                        let vert = Rect::new(line_x - 0.5, row_rect.y, 1.0, row_rect.h);
                        ph2d_editor_core::paint::fill_rounded_rect(scene, vert, 0.0, line_color);
                    }
                }
            }
        }
        if is_renaming {
            let icon_x_local = rect.x
                + Spacing::Md.px()
                + (depth as f32) * INDENT_PX
                + Spacing::Lg.px()
                + Spacing::Xs.px();
            let name_x = icon_x_local + Spacing::Xl.px() + Spacing::Md.px();
            let name_right = row_rect.x + row_rect.w - 10.0 - Spacing::Xl.px() - Spacing::Sm.px(); // LITERAL-PX-OK: 10 = chrome inset matching row pad
            let input_rect = Rect::new(
                name_x - Spacing::Xs.px(),
                row_rect.y + 1.0,
                (name_right - name_x + Spacing::Md.px()).max(80.0), // LITERAL-PX-OK: min rename input width
                row_rect.h - 2.0,
            );
            hit_index.register(ids::HIER_RENAME_INPUT, input_rect);
            let (state, text, caret, anchor) = match store.get(ids::HIER_RENAME_INPUT) {
                Some(InteractiveState::TextInput {
                    state,
                    text,
                    caret,
                    selection_anchor,
                }) => (*state, text.clone(), *caret, *selection_anchor),
                _ => (TextInputState::Focused, String::new(), 0, None),
            };
            let input = TextInput::new(ids::HIER_RENAME_INPUT, "").state(state);
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
                fill_rounded_rect(scene, indicator, 1.0, resolve(ColorToken::Accent, theme));
                drew = true;
                break;
            } else if d.cursor_y < inside_bot {
                stroke_rounded_rect(
                    scene,
                    *rrect,
                    Spacing::Sm.px(),
                    StrokeToken::Thick.px(),
                    resolve(ColorToken::Accent, theme),
                );
                drew = true;
                break;
            } else {
                let indicator = Rect::new(rrect.x, rrect.y + rrect.h - 1.0, rrect.w, 2.0);
                fill_rounded_rect(scene, indicator, 1.0, resolve(ColorToken::Accent, theme));
                drew = true;
                break;
            }
        }
        if !drew {
            let indicator = Rect::new(rect.x + body_pad, y - 1.0, row_w, 2.0);
            fill_rounded_rect(scene, indicator, 1.0, resolve(ColorToken::Accent, theme));
        }
    }
    scene.pop_layer();
    paint_panel_corner_dot(rect, scene, theme);
    ph2d_editor_core::widget::panel_chrome::paint_panel_corner_dot_bl(rect, scene, theme);
    hit_index.register(ids::HIER_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::HIER_RESIZE_HANDLE, resize_handle_rect);
    hit_index.register(ids::HIER_RESIZE_HANDLE_BL, resize_handle_bl_rect);
    let content_h = (y - start_y).max(0.0);
    set_last_hierarchy_content_h(content_h);

    let visible_h = (content_bottom - body_top).max(0.0);
    if widget::scrollbar_is_needed(content_h, visible_h) {
        let body = Rect::new(rect.x, body_top, rect.w, visible_h);
        let track = widget::scrollbar_track_rect(body);
        let thumb = widget::scrollbar_thumb_rect(track, scroll_y, content_h, visible_h);
        let is_active = matches!(store.scrollbar_drag(), Some(d) if d.panel == ids::HIER_PANEL);
        widget::paint_scrollbar(
            body, scroll_y, content_h, visible_h, is_active, scene, theme,
        );
        hit_index.register(HIERARCHY_SCROLLBAR_ID, thumb);
    }
    order.iter().copied().collect()
}
