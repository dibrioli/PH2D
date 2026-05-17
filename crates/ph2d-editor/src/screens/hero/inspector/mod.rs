//! Inspector painter.
//!
//! Header: panel title "Inspector" + a sub-row showing the
//! currently-selected entity's name (or "(none)").
//!
//! Body: placeholder until the pilot project wires real per-component
//! editors. The 10-section widget showcase that used to live here
//! (Inputs / Slider / Switches / Lists / Vector / Status / Color /
//! Actions / Identity / Card + notes) now lives in the floating
//! Widget Gallery panel (toggle via the palette pill in the TopBar);
//! the section painters are reached from [`paint_showcase_body`].
//! The `#[allow(dead_code)]` at file scope covers helpers that
//! aren't directly called by the live `paint_inspector` body but
//! ARE used by `paint_showcase_body`.
//!
//! The floating [`crate::widget::BlenderColorPicker`] (handled by
//! [`super::color_picker_demo`]) is not duplicated here — its
//! retained state still lives on this panel under
//! [`ids::INSP_BLENDER_PICKER`], which is registered in
//! [`populate`].
#![allow(dead_code)]

mod notes;
mod populate;
mod sections;
mod showcase;
mod state;

use notes::paint_one_note;
pub use populate::populate;
pub(in crate::screens::hero) use sections::{
    paint_entity_name_row, paint_render_source_section, paint_transform_section,
    paint_visibility_row,
};
pub(in crate::screens::hero) use showcase::paint_showcase_body;
use state::{
    LAST_BODY_TOP_SCREEN_Y, LAST_SECTION_TOPS_Y, current_inspector_name_is_some,
    current_inspector_sprite, current_inspector_transform, current_inspector_visibility,
    set_last_inspector_content_h, set_last_inspector_visible_h, take_pending_dropdown_chip,
};
pub(in crate::screens::hero) use state::{
    last_body_top_screen_y, last_gallery_content_h, last_gallery_visible_h,
    last_inspector_content_h, last_inspector_visible_h, section_index_below_body_y,
    set_current_display_unit, set_current_inspector_name, set_current_inspector_sprite,
    set_current_inspector_transform, set_current_inspector_visibility,
};

use super::HeroLayout;
use super::HeroSelection;
use super::ids;
use super::style::{
    PANEL_HEAD_PAD, paint_panel_corner_dot, paint_panel_surface, panel_drag_handle_rect,
    panel_resize_handle_rect,
};
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text, paint_text_title, rect_to_vello, resolve};
use crate::widget::{ButtonState, ComboboxState, Dropdown, DropdownOption, TextInputState};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

const BODY_PAD: f32 = 10.0; // LITERAL-PX-OK: inspector body inset (between Spacing::Md and Lg; chrome-specific)
const ROW_GAP: f32 = Spacing::Sm.px();
const SECTION_HEAD_H: f32 = ROW_H_PX;
const FIELD_H: f32 = Spacing::Xl3.px();

/// Stable id list for every collapsible section header in the
/// Inspector. Order matches `paint_inspector` paint order so the
/// `apply_event` lookup and `populate` registration walk the same
/// sequence.
pub const SECTION_IDS: [ph2d_a11y::NodeId; 10] = [
    ids::INSP_SECTION_INPUTS,
    ids::INSP_SECTION_SLIDER,
    ids::INSP_SECTION_SWITCHES,
    ids::INSP_SECTION_LISTS,
    ids::INSP_SECTION_VECTOR,
    ids::INSP_SECTION_STATUS,
    ids::INSP_SECTION_COLOR,
    ids::INSP_SECTION_ACTIONS,
    ids::INSP_SECTION_IDENTITY,
    ids::INSP_SECTION_CARD,
];

/// Live Inspector section headers. Right-click on any of these opens
/// the SectionOutline context menu (same affordance as the Widget
/// Gallery's `SECTION_IDS`). The painter registers each editable block
/// (Name / Visibility / Transform / Render Source) against one of
/// these so the user can frame a section while reviewing.
pub const LIVE_SECTION_IDS: [ph2d_a11y::NodeId; 4] = [
    ids::INSP_LIVE_NAME_SECTION,
    ids::INSP_LIVE_VISIBILITY_SECTION,
    ids::INSP_LIVE_TRANSFORM_SECTION,
    ids::INSP_LIVE_RENDER_SECTION,
];

/// Color-circle hit ids — one per section header, in the same
/// order as `SECTION_IDS`. Clicking any of these opens the global
/// color picker editing `widget_colors[circle_id]`.
pub(super) const SECTION_COLOR_IDS: [ph2d_a11y::NodeId; 10] = [
    ids::INSP_SECTION_INPUTS_COLOR,
    ids::INSP_SECTION_SLIDER_COLOR,
    ids::INSP_SECTION_SWITCHES_COLOR,
    ids::INSP_SECTION_LISTS_COLOR,
    ids::INSP_SECTION_VECTOR_COLOR,
    ids::INSP_SECTION_STATUS_COLOR,
    ids::INSP_SECTION_COLOR_COLOR,
    ids::INSP_SECTION_ACTIONS_COLOR,
    ids::INSP_SECTION_IDENTITY_COLOR,
    ids::INSP_SECTION_CARD_COLOR,
];

/// Ids of the three Radio buttons that form the Switches sample's
/// segmented "Low / Mid / High" group. Same trick used for the
/// "Edit / Play / Debug" tabs — exactly one button is `Pressed` at
/// a time, the painter reads the active index from that flag.
pub(super) const RADIO_GROUP_IDS: [ph2d_a11y::NodeId; 3] = [
    ids::INSP_SAMPLE_RADIO_A,
    ids::INSP_SAMPLE_RADIO_B,
    ids::INSP_SAMPLE_RADIO_C,
];
pub(super) const TAB_GROUP_IDS: [ph2d_a11y::NodeId; 3] = [
    ids::INSP_SAMPLE_TAB_A,
    ids::INSP_SAMPLE_TAB_B,
    ids::INSP_SAMPLE_TAB_C,
];
pub(super) const TREE_LEAF_IDS: [ph2d_a11y::NodeId; 2] =
    [ids::INSP_SAMPLE_TREE_LEAF_A, ids::INSP_SAMPLE_TREE_LEAF_B];
/// Hit-slot ids for the 12 possible notes per panel. The painter
/// assigns slots by position (slot 0 = first painted note, etc.).
pub(super) const NOTE_SLOT_IDS: [ph2d_a11y::NodeId; 12] = [
    ids::INSP_NOTE_SLOT_0,
    ids::INSP_NOTE_SLOT_1,
    ids::INSP_NOTE_SLOT_2,
    ids::INSP_NOTE_SLOT_3,
    ids::INSP_NOTE_SLOT_4,
    ids::INSP_NOTE_SLOT_5,
    ids::INSP_NOTE_SLOT_6,
    ids::INSP_NOTE_SLOT_7,
    ids::INSP_NOTE_SLOT_8,
    ids::INSP_NOTE_SLOT_9,
    ids::INSP_NOTE_SLOT_10,
    ids::INSP_NOTE_SLOT_11,
];
pub(super) const NOTE_TITLE_IDS: [ph2d_a11y::NodeId; 12] = [
    ids::INSP_NOTE_TITLE_0,
    ids::INSP_NOTE_TITLE_1,
    ids::INSP_NOTE_TITLE_2,
    ids::INSP_NOTE_TITLE_3,
    ids::INSP_NOTE_TITLE_4,
    ids::INSP_NOTE_TITLE_5,
    ids::INSP_NOTE_TITLE_6,
    ids::INSP_NOTE_TITLE_7,
    ids::INSP_NOTE_TITLE_8,
    ids::INSP_NOTE_TITLE_9,
    ids::INSP_NOTE_TITLE_10,
    ids::INSP_NOTE_TITLE_11,
];
pub(super) const NOTE_BODY_IDS: [ph2d_a11y::NodeId; 12] = [
    ids::INSP_NOTE_BODY_0,
    ids::INSP_NOTE_BODY_1,
    ids::INSP_NOTE_BODY_2,
    ids::INSP_NOTE_BODY_3,
    ids::INSP_NOTE_BODY_4,
    ids::INSP_NOTE_BODY_5,
    ids::INSP_NOTE_BODY_6,
    ids::INSP_NOTE_BODY_7,
    ids::INSP_NOTE_BODY_8,
    ids::INSP_NOTE_BODY_9,
    ids::INSP_NOTE_BODY_10,
    ids::INSP_NOTE_BODY_11,
];

/// Apply a [`WidgetEvent`] against Inspector widgets. Currently
/// handles:
///   - **Section headers**: clicking toggles the section's collapsed
///     flag on the store.
///   - **Radio group + Tabs**: clicking one option pins it as the
///     `Pressed` Button state and resets the siblings to `Normal`.
///     Without this, the default Button-Up handler reverts every
///     option to `Hovered`/`Normal` after release, losing selection.
pub fn apply_event(store: &mut WidgetStore, event: WidgetEvent) -> bool {
    if let WidgetEvent::Click(id) = event {
        // Context menu items — route by id using the snapshot of
        // the most recently closed menu request (dispatch closes the
        // menu on the Down event; the Click arrives on Up after the
        // snapshot landed in `last_context_menu`). Returns true so
        // the orchestrator doesn't pass the Click to other panels.
        const OUTLINE_ITEMS: [(ph2d_a11y::NodeId, Option<u8>); 6] = [
            (ids::CTX_MENU_OUTLINE_NONE, None),
            (ids::CTX_MENU_OUTLINE_0, Some(0)),
            (ids::CTX_MENU_OUTLINE_1, Some(1)),
            (ids::CTX_MENU_OUTLINE_2, Some(2)),
            (ids::CTX_MENU_OUTLINE_3, Some(3)),
            (ids::CTX_MENU_OUTLINE_4, Some(4)),
        ];
        if id == ids::CTX_MENU_CREATE_NOTE {
            if let Some(req) = store.consume_last_context_menu()
                && let crate::interaction::ContextMenuKind::CreateNote { panel, .. } = req.kind
            {
                // Convert the click's screen y into body-relative y
                // and look up which section sits below it; the new
                // note slots in just above that section. Returns
                // `None` when the click was past the last section,
                // in which case the note appends at the bottom.
                let scroll_y = store.panel_scroll(panel);
                let body_top_screen = last_body_top_screen_y();
                let body_y = req.y - body_top_screen + scroll_y;
                let before = section_index_below_body_y(body_y);
                // Default to yellow (color_idx 0).
                let new_index = store.notes_for_panel(panel).len();
                store.notes_push(panel, 0, before);
                // Seed the editable title TextInput at the slot the
                // painter will assign to this note. The body starts
                // empty.
                if let Some(title_id) = NOTE_TITLE_IDS.get(new_index)
                    && let Some(InteractiveState::TextInput { text, caret, .. }) =
                        store.get_mut(*title_id)
                {
                    *text = format!("Note {}", new_index + 1);
                    *caret = text.len();
                }
                if let Some(body_id) = NOTE_BODY_IDS.get(new_index)
                    && let Some(InteractiveState::TextInput { text, caret, .. }) =
                        store.get_mut(*body_id)
                {
                    text.clear();
                    *caret = 0;
                }
            }
            return true;
        }
        if let Some((_, color_idx)) = OUTLINE_ITEMS.iter().find(|(item_id, _)| *item_id == id) {
            if let Some(req) = store.consume_last_context_menu() {
                match req.kind {
                    crate::interaction::ContextMenuKind::SectionOutline { section } => {
                        store.set_section_outline_color(section, *color_idx);
                    }
                    crate::interaction::ContextMenuKind::NoteBackground { panel, note_index } => {
                        if let Some(c) = color_idx {
                            store.note_set_color(panel, note_index as usize, *c);
                        }
                    }
                    _ => {}
                }
            }
            return true;
        }

        // Color circle on a section header OR the ColorSwatch
        // sample → open the global BlenderColorPicker editing this
        // widget's color. The picker's value seeds from the
        // widget's current color (defaulting to neutral gray for
        // unseeded targets).
        if SECTION_COLOR_IDS.contains(&id) || id == ids::INSP_SAMPLE_SWATCH {
            let seed = store.widget_color(id).unwrap_or([0x88, 0x88, 0x88, 0xFF]);
            store.set_widget_color(id, seed);
            store.set_picker_target(Some(id));
            // Seed the picker's retained color from the target so
            // the floating picker opens already showing the
            // current color.
            store.set_blender_value(
                ids::INSP_BLENDER_PICKER,
                ph2d_tokens::ColorValue::from_rgba8(seed[0], seed[1], seed[2], seed[3]),
            );
            return true;
        }

        // Section header → flip collapse.
        if SECTION_IDS.contains(&id) {
            store.toggle_collapsed(id);
            return true;
        }
        // TreeView root chevron → flip child visibility. Uses the
        // same `is_collapsed` side-table as sections; the painter
        // calls `tree.expand(...)` only when `!collapsed`.
        if id == ids::INSP_SAMPLE_TREE_ROOT {
            store.toggle_collapsed(id);
            return true;
        }
        // Tree leaf selection — same pin-pressed trick as radio/tabs.
        if TREE_LEAF_IDS.contains(&id) {
            pin_button_selection(store, id, &TREE_LEAF_IDS);
            return true;
        }
        // Radio group selection lock — pin clicked, clear siblings.
        if RADIO_GROUP_IDS.contains(&id) {
            pin_button_selection(store, id, &RADIO_GROUP_IDS);
            return true;
        }
        // Tabs sample — same shape, different ids.
        if TAB_GROUP_IDS.contains(&id) {
            pin_button_selection(store, id, &TAB_GROUP_IDS);
            return true;
        }
    }
    false
}

/// Force `selected` into `ButtonState::Pressed` and every other id in
/// `group` into `ButtonState::Normal`. Used for "exactly one is
/// active" segmented controls (RadioGroup, Tabs) that model selection
/// as a single Pressed flag across N Button entries.
fn pin_button_selection(
    store: &mut WidgetStore,
    selected: ph2d_a11y::NodeId,
    group: &[ph2d_a11y::NodeId],
) {
    for id in group {
        if let Some(InteractiveState::Button { state }) = store.get_mut(*id) {
            *state = if *id == selected {
                ButtonState::Pressed
            } else {
                ButtonState::Normal
            };
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn paint_inspector(
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
    // Standard panel chrome hit zones — visual is in
    // `paint_panel_surface`; we just register hits against this
    // panel's NodeIds. Re-registered after the body to outrank any
    // scrolled widget that drifted into the chrome area.
    let drag_handle_rect = panel_drag_handle_rect(rect);
    let resize_handle_rect = panel_resize_handle_rect(rect);
    hit_index.register(ids::INSP_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::INSP_RESIZE_HANDLE, resize_handle_rect);

    // Header: panel title "Inspector" + a sub-field showing the
    // currently-selected entity's name (or "(none)" when no entity
    // is selected). The pilot project's selection wiring drives the
    // sub-field; the panel title is constant.
    let title_y = rect.y + 18.0; // LITERAL-PX-OK: panel title baseline (matches PANEL_HEAD_PAD composite)
    paint_text_title(
        text_system,
        scene,
        "Inspector",
        rect.x + PANEL_HEAD_PAD,
        title_y,
        TypeToken::Md.px(),
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text1, theme),
    );
    // M14.E: header subtitle is now the sprite's world size (when a
    // sprite is selected). The entity name moved to the editable
    // TextInput at the top of the body (see `paint_entity_name_row`).
    // For non-sprite entities the subtitle is empty — Transform's own
    // section + the editable name row carry the identification.
    let sprite_for_header = current_inspector_sprite();
    let subtitle_owned;
    let subtitle: &str = match sprite_for_header.as_ref() {
        Some(info) => {
            subtitle_owned = format!("{:.3} × {:.3} m", info.world_size[0], info.world_size[1]);
            subtitle_owned.as_str()
        }
        None => "",
    };
    paint_text(
        text_system,
        scene,
        subtitle,
        rect.x + PANEL_HEAD_PAD,
        title_y + TypeToken::Md.px() + Spacing::Xs.px(),
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

    // Scroll-aware body. The clip covers the entire body region so
    // overflow stays inside the panel; `scroll_y` shifts the content
    // cursor up by however much the wheel has been scrolled.
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
    // Reserve room on the right for the scrollbar's track even if
    // the bar isn't visible this frame — keeps the content width
    // stable so widgets don't reflow when notes/section toggles
    // push content past the viewport. `SCROLLBAR_W + 6` covers the
    // track (10) + the 2 px outer gap + a 4 px breathing margin.
    let scrollbar_reserve = crate::widget::SCROLLBAR_W + Spacing::Sm.px();
    let inner_w = (rect.w - BODY_PAD * 2.0 - scrollbar_reserve).max(0.0);
    let body_top_y = content_top - scroll_y + Spacing::Xs.px();
    // Body: placeholder until the pilot project wires real component
    // editors. The 10-section widget showcase + notes are now in
    // the floating Widget Gallery panel ([`paint_showcase_body`],
    // toggled via the palette pill in the TopBar). The working
    // Inspector here focuses on the actual editor's job — inspect
    // properties of the currently-selected entity. No selection →
    // instructional prompt.
    let section_tops_y: Vec<f32> = Vec::new();
    LAST_BODY_TOP_SCREEN_Y.with(|c| c.set(content_top + Spacing::Xs.px()));
    let transform_info = current_inspector_transform();
    let sprite_info = current_inspector_sprite();
    let visibility_info = current_inspector_visibility();
    let name_present = current_inspector_name_is_some();
    let any_section = transform_info.is_some()
        || sprite_info.is_some()
        || visibility_info.is_some()
        || name_present;
    let mut y = body_top_y + Spacing::Xs.px();
    // Wave 4.1 — restored section outline affordance in the live
    // Inspector. Mirrors the Widget Gallery showcase: right-click on
    // a section header opens the SectionOutline context menu (5
    // highlighter colors + "No outline"). The picked color paints a
    // colored frame around the section body until the user clears
    // it. Each section registers a header strip against its
    // `INSP_LIVE_*_SECTION` id BEFORE its body widgets so internal
    // hits (NumberInput chips, checkbox, etc.) win the back-to-front
    // hit_index walk for primary clicks while the header strip still
    // receives the secondary (right-click) at the title area.
    use crate::screens::hero::context_menu_overlay::HIGHLIGHTER_RGBA;
    macro_rules! live_section {
        ($section_id:expr, $header_h:expr, $body:block) => {{
            let y_before = y;
            // Register the header strip FIRST so child widgets
            // painted inside `$body` override the back-to-front hit
            // walk for primary clicks on their own rects.
            hit_index.register(
                $section_id,
                Rect::new(inner_x, y_before, inner_w, $header_h),
            );
            let new_y: f32 = $body;
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
                    ph2d_vector::Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]); // LITERAL-COLOR-OK: user-color — HIGHLIGHTER_RGBA palette
                crate::paint::stroke_rounded_rect(
                    scene,
                    block,
                    Radius::Md.px(),
                    ph2d_tokens::StrokeToken::Thick.px(),
                    outline_color,
                );
            }
            new_y
        }};
    }

    // ── Entity name (M14.E) — editable TextInput at the very top
    // of the body. Replaces the read-only name displays that used to
    // live in the header subtitle (now world size) and the Render
    // Source "Name" row (now removed).
    if name_present {
        y = live_section!(ids::INSP_LIVE_NAME_SECTION, ROW_H_PX, {
            paint_entity_name_row(
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
    // ── Visibility row (M14.D) — checkbox below the name field.
    // Drives the same `ph2d_ecs::Visibility` component as the
    // Hierarchy panel's eye toggle (M14.6 A) via
    // `EditorCommand::SetComponent`.
    if visibility_info.is_some() {
        y = live_section!(ids::INSP_LIVE_VISIBILITY_SECTION, ROW_H_PX, {
            paint_visibility_row(
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
    // ── Transform section (M14.A) — first SECTION (below the
    // Visibility row), since Transform is the most fundamental
    // component. Matches Unity / Godot / Blender conventions where
    // Transform sits above all other components.
    if transform_info.is_some() {
        y = live_section!(ids::INSP_LIVE_TRANSFORM_SECTION, SECTION_HEAD_H, {
            paint_transform_section(
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
    // ── Render Source section (M14.5) — below Transform.
    if let Some(info) = sprite_info.as_ref() {
        y = live_section!(ids::INSP_LIVE_RENDER_SECTION, SECTION_HEAD_H, {
            paint_render_source_section(
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
    }
    // ── Placeholder when nothing is selected ──
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

    // Publish the total content height + the EXACT visible body
    // height for the wheel dispatch + hero clamp. visible_h must
    // match the actual content viewport, not a rough panel.h - 60
    // heuristic — the latter overestimated by ~20-30 px and
    // prevented scrolling to reach the last note (user reported the
    // scroll bound didn't adapt to a newly-added note).
    let content_h = (y - body_top_y).max(0.0);
    let visible_h = (content_bottom - content_top).max(0.0);
    set_last_inspector_content_h(content_h);
    set_last_inspector_visible_h(visible_h);
    LAST_SECTION_TOPS_Y.with(|t| *t.borrow_mut() = section_tops_y);

    // Scrollbar (right edge of body). Hit-registered with the
    // canonical inspector scrollbar id; dispatch handles drag.
    if crate::widget::scrollbar_is_needed(content_h, visible_h) {
        let body = Rect::new(rect.x, content_top, rect.w, visible_h);
        let track = crate::widget::scrollbar_track_rect(body);
        let thumb = crate::widget::scrollbar_thumb_rect(track, scroll_y, content_h, visible_h);
        let is_active = matches!(store.scrollbar_drag(), Some(d) if d.panel == ids::INSP_PANEL);
        crate::widget::paint_scrollbar(
            body, scroll_y, content_h, visible_h, is_active, scene, theme,
        );
        // Register the thumb hit so Down on it starts a drag.
        hit_index.register(crate::widget::INSPECTOR_SCROLLBAR_ID, thumb);
    }

    // Second pass: open Dropdown popover paints on top of every
    // section that ran before it. We reconstruct the Dropdown
    // (cheap — fixed option list) and call the popover-only painter
    // at the chip rect captured during the Lists section.
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
        crate::widget::paint_dropdown_popover(&dd, chip, scene, text_system, theme);
        // Re-register the option hits AFTER the popover paint so
        // they sit on top of the section hits painted earlier.
        for (i, opt) in dd.options.iter().enumerate() {
            hit_index.register(opt.id, dd.option_rect(chip, i));
        }
    }

    scene.pop_layer();

    // Standard panel chrome — painted AFTER the body so the
    // corner dot sits on top of widgets that may overlap the
    // bottom-right area. Same re-registration pattern keeps the
    // drag pill + resize handle hit zones above any scrolled
    // widget's rect.
    paint_panel_corner_dot(rect, scene, theme);
    hit_index.register(ids::INSP_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::INSP_RESIZE_HANDLE, resize_handle_rect);
}

/// Paint the canonical widget showcase at `rect`. Designed to be
/// called from the floating Widget Gallery panel (see
/// [`super::widget_gallery`]). The live `paint_inspector` uses its
/// own minimalist body; this entry point keeps the 10-section
/// showcase reachable inside the working app so peripheral agents
/// have a single in-app source of truth for UI decoration.
///
/// Re-uses the `paint_*_section` painters preserved as `dead_code`
/// after the Inspector switched to the live entity-binding model.
/// Uses `ids::GAL_*` for panel chrome so the gallery's drag /
/// resize state is independent of the live Inspector at
/// `ids::INSP_*`.
///
/// `rect` is the panel rect in viewport pixels. Content is clipped
/// to that rect; v1 is non-scrollable, so callers should size the
/// panel tall enough to fit all 10 sections (~700 px at default
/// theme).
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_section_separator(
    scene: &mut VectorScene,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let pad_x = 2.0_f32;
    let pad_y = SEPARATOR_PAD_Y;
    let thickness = 1.0_f32;
    let line = Rect::new(x + pad_x, y + pad_y, (w - pad_x * 2.0).max(0.0), thickness);
    fill_rounded_rect(scene, line, 0.5, resolve(ColorToken::Accent, theme));
    y + pad_y + thickness + pad_y
}

/// Vertical breathing room above AND below the section separator
/// line — same on both sides so the colored pill reads as centered
/// between two sections rather than glued to the previous one.
const SEPARATOR_PAD_Y: f32 = Spacing::Md.px();

#[allow(clippy::too_many_arguments)]
pub(super) fn read_text_input(
    store: &WidgetStore,
    id: NodeId,
) -> (TextInputState, &str, usize, Option<usize>) {
    match store.get(id) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, text.as_str(), *caret, *selection_anchor),
        _ => (TextInputState::Normal, "", 0, None),
    }
}

pub(super) fn read_combobox(
    store: &WidgetStore,
    id: NodeId,
) -> (ComboboxState, bool, &str, usize, Option<usize>) {
    match store.get(id) {
        Some(InteractiveState::Combobox {
            state,
            open,
            query,
            caret,
            selection_anchor,
        }) => (*state, *open, query.as_str(), *caret, *selection_anchor),
        _ => (ComboboxState::Normal, false, "", 0, None),
    }
}

pub(super) fn read_number_input(
    store: &WidgetStore,
    id: NodeId,
) -> (TextInputState, f64, &str, usize, Option<usize>) {
    store
        .number_input(id)
        .unwrap_or((TextInputState::Normal, 0.0, "", 0, None))
}

/// Find which id in `ids` is the active (`Pressed`) one. Used for
/// segmented button groups (Tabs, RadioGroup) that we model as N
/// independent Button states with exactly one in the Pressed state.
pub(super) fn active_index(store: &WidgetStore, ids: &[NodeId]) -> Option<usize> {
    for (i, id) in ids.iter().enumerate() {
        if matches!(store.button_state(*id), Some(ButtonState::Pressed)) {
            return Some(i);
        }
    }
    None
}
