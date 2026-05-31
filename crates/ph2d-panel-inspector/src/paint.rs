//! Inspector panel paint — ADR-0029 Phase C.1 port of
//! `ph2d_editor_core::screens::hero::inspector::{paint_thunk,
//! paint_inspector}`.
//!
//! `paint` is the typed entry point invoked by the `Panel` trait. It
//! gates on visibility (via [`PanelHostInternal::panel_visible`]),
//! runs the snapshot sync, paints the live Inspector body, and
//! publishes scroll bounds back to the store.

use crate::state::{
    self, current_display_unit, current_inspector_name_is_some, current_inspector_ordering,
    current_inspector_sampling, current_inspector_sprite, current_inspector_transform,
    current_inspector_visibility, current_inspector_visibility_section, current_pixels_per_meter,
    last_inspector_content_h, last_inspector_visible_h, set_last_inspector_content_h,
    set_last_inspector_visible_h,
};
use crate::sync::sync_inspector_from_snapshots;
use crate::{InspectorPanel, sections};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, NoteData, WidgetStore};
use ph2d_editor_core::paint::{paint_text, rect_to_vello, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::screens::{HeroLayout, HeroSelection};
use ph2d_editor_core::widget::panel_chrome::{
    HIGHLIGHTER_RGBA, PANEL_HEAD_PAD, PANEL_TITLE_BASELINE, paint_panel_corner_dot,
    paint_panel_surface, paint_panel_title, panel_drag_handle_rect, panel_resize_handle_rect,
};
use ph2d_editor_core::widget::showcase::{LAST_BODY_TOP_SCREEN_Y, LAST_SECTION_TOPS_Y};
use ph2d_editor_core::widget::showcase::{
    paint_one_note, paint_section_separator, push_section_top_y, take_pending_dropdown_chip,
};
use ph2d_editor_core::widget::{
    self, Dropdown, DropdownOption, INSPECTOR_SCROLLBAR_ID, SCROLLBAR_W,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

const BODY_PAD: f32 = 10.0; // LITERAL-PX-OK: inspector body inset
const SECTION_HEAD_H: f32 = ROW_H_PX;

pub(crate) fn paint(inspector_state: &mut state::InspectorState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(InspectorPanel::ID) {
        return;
    }
    sync_inspector_from_snapshots(inspector_state, ctx.host);
    let display_unit = ctx.host.project().display_unit;
    let ppm = ctx.host.project().pixels_per_meter;
    state::set_current_display_unit(display_unit, ppm);
    let theme = ctx.host.theme();
    // Splitting the borrows: paint_inspector wants &mut hit_index,
    // &WidgetStore, &mut Scene, &mut TextSystem — all from disjoint
    // refs on the host. Reborrow store via `store()` then `store_mut()`
    // sequentially below for the post-paint scroll publish.
    {
        // Build a transient owning copy of selection to avoid holding an
        // immutable borrow of host while also borrowing host's
        // hit_index mutably; selection is small + Clone. The combined
        // `store_and_hit_index_mut` accessor avoids the dyn-trait
        // aliasing dance for the &WidgetStore + &mut HitIndex pair.
        let selection_clone: Option<HeroSelection> = ctx.host.selection().cloned();
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        paint_inspector(
            ctx.layout,
            selection_clone.as_ref(),
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
        );
    }
    state::set_current_display_unit(display_unit, ppm); // keep symmetric with legacy
    // Publish content_h + clamp scroll right after paint so
    // `dispatch_wheel` sees the new bounds on the very next event.
    let content_h = last_inspector_content_h();
    let visible_h = last_inspector_visible_h();
    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::INSP_PANEL, content_h);
    store.set_panel_visible_h(ids::INSP_PANEL, visible_h);
    let max_scroll = (content_h - visible_h).max(0.0);
    let cur = store.panel_scroll(ids::INSP_PANEL);
    if cur > max_scroll {
        store.set_panel_scroll(ids::INSP_PANEL, max_scroll);
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_inspector(
    layout: &HeroLayout,
    selection: Option<&HeroSelection>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let rect = layout.inspector;
    paint_panel_surface(rect, scene, theme);
    let drag_handle_rect = panel_drag_handle_rect(
        rect,
        ph2d_editor_core::widget::panel_chrome::PANEL_HEADER_H_DEFAULT,
        // Inspector has no close button — its visibility is governed
        // by the TopBar toggle. Reserve nothing on the right so the
        // drag area spans the full width.
        0.0,
    );
    let resize_handle_rect = panel_resize_handle_rect(rect);
    let resize_handle_bl_rect =
        ph2d_editor_core::widget::panel_chrome::panel_resize_handle_rect_bl(rect);
    hit_index.register(ids::INSP_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::INSP_RESIZE_HANDLE, resize_handle_rect);
    hit_index.register(ids::INSP_RESIZE_HANDLE_BL, resize_handle_bl_rect);

    // Canonical panel title — reserve right slot for the X close
    // button (UI canon post-2026-05-24: every panel except Hierarchy
    // carries close X; vide `docs/UI_Padrao/components/panel_chrome.md`).
    let title_y = rect.y + PANEL_TITLE_BASELINE;
    let title_size = paint_panel_title(
        rect,
        "Inspector",
        ph2d_editor_core::widget::panel_chrome::PANEL_HEADER_CLOSE_RESERVE,
        scene,
        text_system,
        theme,
    );
    // X close → toggles inspector visibility (apply_event handler).
    ph2d_editor_core::widget::panel_chrome::paint_panel_close_button(
        rect,
        ids::INSP_CLOSE,
        hit_index,
        scene,
        theme,
    );
    let sprite_for_header = current_inspector_sprite();
    let subtitle_owned;
    let subtitle: &str = match sprite_for_header.as_ref() {
        Some(info) => {
            let unit = current_display_unit();
            let ppm = current_pixels_per_meter();
            let w = unit.from_meters(info.world_size[0], ppm);
            let h = unit.from_meters(info.world_size[1], ppm);
            subtitle_owned = match unit {
                ph2d_editor_core::project::DisplayUnit::Meters => {
                    format!("{:.3} × {:.3} {}", w, h, unit.suffix())
                }
                ph2d_editor_core::project::DisplayUnit::Pixels => {
                    format!("{:.0} × {:.0} {}", w, h, unit.suffix())
                }
            };
            subtitle_owned.as_str()
        }
        None => "",
    };
    paint_text(
        text_system,
        scene,
        subtitle,
        rect.x + PANEL_HEAD_PAD,
        title_y + title_size + Spacing::Xs.px(),
        TypeToken::Sm.px(),
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text3, theme),
    );
    let div_y = title_y + TypeToken::Md.px() + TypeToken::Sm.px() + Spacing::Xl.px();
    let div = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        div_y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        1.0,
    );
    scene.fill_rect(rect_to_vello(div), resolve(ColorToken::Border, theme));

    let content_top = div_y + Spacing::Sm.px();
    let content_bottom = rect.y + rect.h - Spacing::Xs.px();
    let scroll_y = store.panel_scroll(ids::INSP_PANEL).max(0.0);
    let clip = ph2d_vector::Rect::new(
        rect.x as f64,
        content_top as f64,
        (rect.x + rect.w) as f64,
        content_bottom as f64,
    );
    scene.push_clip(&clip);

    let inner_x = rect.x + BODY_PAD;
    let scrollbar_reserve = SCROLLBAR_W + Spacing::Sm.px();
    let inner_w = (rect.w - BODY_PAD * 2.0 - scrollbar_reserve).max(0.0);
    let body_top_y = content_top - scroll_y + Spacing::Xs.px();
    let mut section_tops_y: Vec<f32> = Vec::with_capacity(4);
    LAST_BODY_TOP_SCREEN_Y.with(|c| c.set(content_top + Spacing::Xs.px()));
    let transform_info = current_inspector_transform();
    let sprite_info = current_inspector_sprite();
    let visibility_info = current_inspector_visibility();
    let ordering_info = current_inspector_ordering();
    let sampling_info = current_inspector_sampling();
    let name_present = current_inspector_name_is_some();
    let any_section = transform_info.is_some()
        || sprite_info.is_some()
        || visibility_info.is_some()
        || ordering_info.is_some()
        || sampling_info.is_some()
        || name_present;
    let mut y = body_top_y + Spacing::Xs.px();

    let all_notes = store.notes_for_panel(ids::INSP_PANEL).to_vec();
    let mut notes_per_section: [Vec<(usize, NoteData)>; 8] = Default::default();
    let mut trailing_notes: Vec<(usize, NoteData)> = Vec::new();
    for (idx, note) in all_notes.into_iter().enumerate() {
        match note.before_section {
            Some(i) if (i as usize) < notes_per_section.len() => {
                notes_per_section[i as usize].push((idx, note));
            }
            _ => trailing_notes.push((idx, note)),
        }
    }
    // Section macro: paints the section, then the outline (if any),
    // then notes anchored to THIS section (at the END, before the
    // separator the caller adds next). UI canon post-2026-05-24:
    // notes belong VISUALLY to the section the user right-clicked,
    // grouped inside it. Pre-canon notes painted ABOVE the header.
    macro_rules! live_section {
        ($section_id:expr, $section_idx:expr, $header_h:expr, $body:block) => {{
            let y_before = y;
            push_section_top_y(&mut section_tops_y, y_before - body_top_y);
            hit_index.register(
                $section_id,
                Rect::new(inner_x, y_before, inner_w, $header_h),
            );
            let mut new_y: f32 = $body;
            if let Some(color_idx) = store.section_outline_color($section_id) {
                let rgba = HIGHLIGHTER_RGBA[color_idx.min(4) as usize];
                let pad = Spacing::Xs.px();
                let block = Rect::new(
                    inner_x - pad,
                    y_before - pad,
                    inner_w + pad * 2.0,
                    (new_y - y_before + pad * 2.0).max(0.0),
                );
                let outline_color =
                    ph2d_vector::Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]); // LITERAL-COLOR-OK: HIGHLIGHTER_RGBA palette
                stroke_rounded_rect(
                    scene,
                    block,
                    Radius::Md.px(),
                    StrokeToken::Thick.px(),
                    outline_color,
                );
            }
            for (slot, note) in &notes_per_section[$section_idx] {
                paint_one_note(
                    scene,
                    text_system,
                    hit_index,
                    store,
                    inner_x,
                    inner_w,
                    &mut new_y,
                    note,
                    *slot,
                );
            }
            new_y
        }};
    }

    if name_present {
        y = live_section!(ids::INSP_LIVE_NAME_SECTION, 0, ROW_H_PX, {
            sections::paint_entity_name_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                inner_x,
                inner_w,
                y,
            )
        });
        y = paint_section_separator(scene, theme, inner_x, inner_w, y);
    }
    if visibility_info.is_some() {
        y = live_section!(ids::INSP_LIVE_VISIBILITY_SECTION, 1, ROW_H_PX, {
            let mut yy = sections::paint_visibility_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                inner_x,
                inner_w,
                y,
            );
            // W3 §8: the optional-component controls (layer mask / clip /
            // mask / on-screen) sit directly below the Visible toggle.
            if let Some(vis) = current_inspector_visibility_section() {
                yy = sections::paint_visibility_section(
                    scene,
                    text_system,
                    theme,
                    hit_index,
                    store,
                    inner_x,
                    inner_w,
                    yy,
                    &vis,
                );
            }
            yy
        });
        y = paint_section_separator(scene, theme, inner_x, inner_w, y);
    }
    if transform_info.is_some() {
        y = live_section!(ids::INSP_LIVE_TRANSFORM_SECTION, 2, SECTION_HEAD_H, {
            sections::paint_transform_section(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                inner_x,
                inner_w,
                y,
            )
        });
        y = paint_section_separator(scene, theme, inner_x, inner_w, y);
    }
    if let Some(info) = sprite_info.as_ref() {
        y = live_section!(ids::INSP_LIVE_RENDER_SECTION, 3, SECTION_HEAD_H, {
            sections::paint_render_source_section(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                inner_x,
                inner_w,
                y,
                info,
            )
        });
        y = paint_section_separator(scene, theme, inner_x, inner_w, y);
        y = live_section!(ids::INSP_LIVE_COLOR_SECTION, 4, SECTION_HEAD_H, {
            sections::paint_color_tint_section(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                inner_x,
                inner_w,
                y,
            )
        });
        y = paint_section_separator(scene, theme, inner_x, inner_w, y);
        y = live_section!(ids::INSP_LIVE_SHEET_SECTION, 5, SECTION_HEAD_H, {
            sections::paint_sprite_sheet_section(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                inner_x,
                inner_w,
                y,
            )
        });
    }
    // W3 §7 Ordering / Sorting — applies to any Transform-bearing entity
    // (not sprite-only), so it sits outside the `sprite_info` block.
    if let Some(ord) = ordering_info.as_ref() {
        y = paint_section_separator(scene, theme, inner_x, inner_w, y);
        y = live_section!(ids::INSP_LIVE_ORDERING_SECTION, 6, SECTION_HEAD_H, {
            sections::paint_ordering_section(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                inner_x,
                inner_w,
                y,
                ord,
            )
        });
    }
    // W3 §9 Sampling — Transform-bearing entity, sibling of §7.
    if let Some(samp) = sampling_info.as_ref() {
        y = paint_section_separator(scene, theme, inner_x, inner_w, y);
        y = live_section!(ids::INSP_LIVE_SAMPLING_SECTION, 7, SECTION_HEAD_H, {
            sections::paint_sampling_section(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                inner_x,
                inner_w,
                y,
                samp,
            )
        });
    }
    if any_section {
        for (slot, note) in &trailing_notes {
            paint_one_note(
                scene,
                text_system,
                hit_index,
                store,
                inner_x,
                inner_w,
                &mut y,
                note,
                *slot,
            );
        }
    }
    if !any_section {
        let placeholder = if selection.is_some() {
            "No properties yet for the selected entity."
        } else {
            "Select an entity in the Hierarchy to inspect its properties."
        };
        let line_h = TypeToken::Sm.px() + Spacing::Xs.px();
        let center_y = content_top + (content_bottom - content_top) * 0.5 - line_h * 0.5;
        paint_text(
            text_system,
            scene,
            placeholder,
            inner_x + Spacing::Md.px(),
            center_y,
            TypeToken::Sm.px(),
            (inner_w - Spacing::Xl.px()).max(80.0), // LITERAL-PX-OK: minimum placeholder text width
            resolve(ColorToken::Text3, theme),
        );
    }

    let content_h = (y - body_top_y).max(0.0);
    let visible_h = (content_bottom - content_top).max(0.0);
    set_last_inspector_content_h(content_h);
    set_last_inspector_visible_h(visible_h);
    LAST_SECTION_TOPS_Y.with(|t| *t.borrow_mut() = section_tops_y);

    if widget::scrollbar_is_needed(content_h, visible_h) {
        let body = Rect::new(rect.x, content_top, rect.w, visible_h);
        let track = widget::scrollbar_track_rect(body);
        let thumb = widget::scrollbar_thumb_rect(track, scroll_y, content_h, visible_h);
        let is_active = matches!(store.scrollbar_drag(), Some(d) if d.panel == ids::INSP_PANEL);
        widget::paint_scrollbar(
            body, scroll_y, content_h, visible_h, is_active, scene, theme,
        );
        hit_index.register(INSPECTOR_SCROLLBAR_ID, thumb);
    }

    if let Some((sel_idx, chip)) = take_pending_dropdown_chip() {
        let labels = ["Front", "Side", "Top"];
        let selected_label = labels.get(sel_idx).copied().unwrap_or("Front");
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
        .open(true);
        widget::paint_dropdown_popover(&dd, chip, scene, text_system, theme);
        for (i, opt) in dd.options.iter().enumerate() {
            hit_index.register(opt.id, dd.option_rect(chip, i));
        }
    }
    // W3 §7 Sorting Layer dropdown popover — same deferred-paint pass,
    // panel-local pending slot so it never collides with the sample dd.
    if let Some((sel_idx, chip)) = state::take_pending_ordering_dd() {
        let label = sections::ordering::LAYER_LABELS
            .get(sel_idx)
            .copied()
            .unwrap_or("Default");
        let dd = Dropdown::new(
            ids::INSP_ORDER_SORTING_LAYER,
            "",
            sections::ordering::layer_options(),
        )
        .selected(label)
        .open(true);
        widget::paint_dropdown_popover(&dd, chip, scene, text_system, theme);
        for (i, opt) in dd.options.iter().enumerate() {
            hit_index.register(opt.id, dd.option_rect(chip, i));
        }
    }

    scene.pop_layer();

    paint_panel_corner_dot(rect, scene, theme);
    ph2d_editor_core::widget::panel_chrome::paint_panel_corner_dot_bl(rect, scene, theme);
    hit_index.register(ids::INSP_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::INSP_RESIZE_HANDLE, resize_handle_rect);
    hit_index.register(ids::INSP_RESIZE_HANDLE_BL, resize_handle_bl_rect);
    // Re-register close AFTER drag so scrolled body widgets behind
    // the title can't shadow it (bug pattern reported 2026-05-24
    // for Widget Gallery; same surface here).
    hit_index.register(
        ids::INSP_CLOSE,
        ph2d_editor_core::widget::panel_chrome::panel_close_button_rect(rect),
    );
}
