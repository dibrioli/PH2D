//! Dead-code Widget Gallery showcase painters.
//!
//! Extracted from [`super`] (Track C5). These are the demo / reference
//! section painters that powered the original Inspector body before
//! the M14 active sections (Name / Visibility / Transform / Render
//! Source) replaced them. They now live in the floating Widget Gallery
//! panel (toggled via the palette pill in the TopBar) and remain the
//! canonical visual reference for every primitive widget — peripheral
//! agents copy section layout / state-machine wiring from here.
//!
//! The `#![allow(dead_code)]` at the inspector module level covers
//! these.

use super::super::style::{
    PANEL_HEAD_PAD, paint_panel_corner_dot, paint_panel_surface, panel_drag_handle_rect,
    panel_resize_handle_rect,
};
use super::ids;
use super::state::{
    LAST_BODY_TOP_SCREEN_Y, LAST_SECTION_TOPS_Y, push_section_top_y, set_last_gallery_content_h,
    set_last_gallery_visible_h, set_pending_dropdown_chip, take_pending_dropdown_chip,
};
use super::{
    SECTION_IDS, TREE_LEAF_IDS, active_index, paint_one_note, paint_section_separator,
    read_combobox, read_number_input, read_text_input,
};
use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, NoteData, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text, paint_text_title, rect_to_vello, resolve};
use crate::widget::Dropdown;
use crate::widget::DropdownOption;
use crate::widget::{
    Avatar, AvatarShape, Button, ButtonKind, ButtonState, Card, Checkbox, CheckboxState,
    CheckboxValue, ColorSwatch, Combobox, ComboboxOption, DropdownState, ListItem, ListItemState,
    NumberInput, ProgressBar, RadioGroup, RadioOption, RadioOrientation, SectionHeader,
    SliderState, Spinner, SwatchSize, TabItem, Tabs, TabsVariant, Tag, TagState, TagTone, TextArea,
    TextInput, Toggle, ToggleState, TreeNode, TreeView, Vector3Editor, paint_avatar, paint_button,
    paint_card, paint_checkbox, paint_color_swatch, paint_combobox_with_state, paint_list_item,
    paint_number_input_with_buffer, paint_progress_bar, paint_radio_group_with_labels,
    paint_section_header, paint_slider_with_chip, paint_spinner, paint_tabs, paint_tag,
    paint_text_area_with_state, paint_text_input_with_buffer, paint_toggle, paint_tree_view,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{
    ColorToken, Density, ICON_BTN_SIZE_PX, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken,
};
use ph2d_vector::VectorScene;

const BODY_PAD: f32 = 10.0; // LITERAL-PX-OK: showcase body inset (between Spacing::Md and Lg; chrome-specific)
const ROW_GAP: f32 = Spacing::Sm.px();
const SECTION_HEAD_H: f32 = ROW_H_PX;
const FIELD_H: f32 = Spacing::Xl3.px();

pub(in crate::screens::hero) fn paint_showcase_body(
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    paint_panel_surface(rect, scene, theme);
    // Drag pill + resize gripper hit zones. Visuals are inside
    // `paint_panel_surface` / `paint_panel_corner_dot`; we register
    // the hits here against the gallery's own NodeIds so the
    // BlenderHit dispatch (`DragHandle` / `ResizeHandle`) drives
    // `GAL_PANEL` independently of the Inspector.
    let drag_handle_rect = panel_drag_handle_rect(rect);
    let resize_handle_rect = panel_resize_handle_rect(rect);
    hit_index.register(ids::GAL_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::GAL_RESIZE_HANDLE, resize_handle_rect);

    // Header: title + subtitle + divider, matching the reference
    // snapshot's Inspector header style.
    let title_y = rect.y + 18.0; // LITERAL-PX-OK: panel title baseline (matches PANEL_HEAD_PAD chrome composite)
    paint_text_title(
        text_system,
        scene,
        "Widget Gallery",
        rect.x + PANEL_HEAD_PAD,
        title_y,
        TypeToken::Md.px(),
        rect.w - PANEL_HEAD_PAD * 2.0 - 40.0, // LITERAL-PX-OK: reserve for header Close button (chrome dim ~ICON_BTN_SIZE)
        resolve(ColorToken::Text1, theme),
    );
    paint_text(
        text_system,
        scene,
        "Canonical widget showcase \u{00b7} reference for peripheral agents",
        rect.x + PANEL_HEAD_PAD,
        title_y + TypeToken::Md.px() + Spacing::Xs.px(),
        TypeToken::Xs.px() - 1.0,
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text3, theme),
    );
    // Close (X) at top-right of the header strip.
    let close_size = Spacing::Xl2.px();
    let close_rect = Rect::new(
        rect.x + rect.w - PANEL_HEAD_PAD - close_size,
        title_y - 2.0,
        close_size,
        close_size,
    );
    hit_index.register(ids::GAL_CLOSE, close_rect);
    crate::paint::paint_icon(
        scene,
        IconId::Close,
        close_rect,
        resolve(ColorToken::Text2, theme),
        StrokeToken::Default.px(),
    );

    let div_y = title_y + TypeToken::Md.px() + TypeToken::Xs.px() + Spacing::Xl.px();
    let div = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        div_y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        1.0,
    );
    scene.fill_rect(rect_to_vello(div), resolve(ColorToken::Border, theme));

    // Body clipped to panel rect with wheel-driven scroll offset
    // routed through `GAL_PANEL` (independent of `INSP_PANEL`).
    // Reserve room for the scrollbar even when it isn't visible so
    // the section content width is stable.
    let content_top = div_y + Spacing::Sm.px();
    let content_bottom = rect.y + rect.h - Spacing::Xs.px();
    let scroll_y = store.panel_scroll(ids::GAL_PANEL).max(0.0);
    let clip = ph2d_vector::Rect::new(
        rect.x as f64,
        content_top as f64,
        (rect.x + rect.w) as f64,
        content_bottom as f64,
    );
    scene.push_clip(&clip);

    let inner_x = rect.x + BODY_PAD;
    let scrollbar_reserve = crate::widget::SCROLLBAR_W + Spacing::Sm.px();
    let inner_w = (rect.w - BODY_PAD * 2.0 - scrollbar_reserve).max(0.0);
    let body_top_y = content_top - scroll_y + Spacing::Xs.px();
    let mut y = body_top_y;
    // Publish the body's screen-Y origin so the right-click dispatch
    // can convert screen-y → body-y when computing `before_section`
    // for a new note (`section_index_below_body_y`). Inspector's live
    // paint also writes this thread-local, but the gallery paints
    // AFTER inspector in `paint_hero_screen`, so the gallery's value
    // wins for the next dispatch tick — correct for clicks on the
    // gallery body.
    LAST_BODY_TOP_SCREEN_Y.with(|c| c.set(content_top + Spacing::Xs.px()));

    // Notes — read once and partition by `before_section`. Notes
    // tagged with `Some(i)` paint immediately above `SECTION_IDS[i]`;
    // notes with `None` paint at the tail after the last section.
    let all_notes = store.notes_for_panel(ids::GAL_PANEL).to_vec();
    let mut notes_per_section: [Vec<(usize, NoteData)>; 10] = Default::default();
    let mut trailing_notes: Vec<(usize, NoteData)> = Vec::new();
    for (idx, note) in all_notes.into_iter().enumerate() {
        match note.before_section {
            Some(i) if (i as usize) < notes_per_section.len() => {
                notes_per_section[i as usize].push((idx, note));
            }
            _ => trailing_notes.push((idx, note)),
        }
    }

    // Body-relative top-Y of each section header — captured so the
    // right-click dispatch can map a click to "which section the
    // user is targeting" for note insertion.
    let mut section_tops_y: Vec<f32> = Vec::with_capacity(SECTION_IDS.len());
    let mut section_idx: usize = 0;
    macro_rules! paint_pending_notes {
        () => {
            for (slot, note) in &notes_per_section[section_idx] {
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
        };
    }
    // Section macro: paints any notes anchored here, then the
    // section, then the colored outline (if the user picked one via
    // right-click → "Section outline"), then a separator. Each
    // iteration also records the section's body-relative top y so
    // `section_index_below_body_y` works.
    macro_rules! section {
        ($f:ident, $section_id:expr) => {
            paint_pending_notes!();
            let y_before = y;
            push_section_top_y(&mut section_tops_y, y_before - body_top_y);
            let new_y = $f(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                inner_x,
                inner_w,
                y,
            );
            if let Some(color_idx) = store.section_outline_color($section_id) {
                let rgba = crate::screens::hero::context_menu_overlay::HIGHLIGHTER_RGBA
                    [color_idx.min(4) as usize];
                let pad = Spacing::Xs.px();
                let block = Rect::new(
                    inner_x - pad,
                    y_before - pad,
                    inner_w + pad * 2.0,
                    (new_y - y_before + pad * 2.0).max(0.0),
                );
                let outline_color =
                    ph2d_vector::Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]); // LITERAL-COLOR-OK: user-color — showcase preview outline from user-stored ColorValue
                crate::paint::stroke_rounded_rect(
                    scene,
                    block,
                    Radius::Md.px(),
                    StrokeToken::Thick.px(),
                    outline_color,
                );
            }
            y = paint_section_separator(scene, theme, inner_x, inner_w, new_y);
            #[allow(unused_assignments)]
            {
                section_idx += 1;
            }
        };
    }
    section!(paint_inputs_section, ids::INSP_SECTION_INPUTS);
    section!(paint_slider_section, ids::INSP_SECTION_SLIDER);
    section!(paint_switches_section, ids::INSP_SECTION_SWITCHES);
    section!(paint_lists_section, ids::INSP_SECTION_LISTS);
    section!(paint_vector_section, ids::INSP_SECTION_VECTOR);
    section!(paint_status_section, ids::INSP_SECTION_STATUS);
    section!(paint_color_section, ids::INSP_SECTION_COLOR);
    section!(paint_actions_section, ids::INSP_SECTION_ACTIONS);
    section!(paint_identity_section, ids::INSP_SECTION_IDENTITY);
    section!(paint_card_section, ids::INSP_SECTION_CARD);
    // Trailing notes (anchor = None or out-of-range section index)
    // paint at the bottom after all sections.
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
    LAST_SECTION_TOPS_Y.with(|t| *t.borrow_mut() = section_tops_y);
    // Publish content + visible heights so the host can clamp the
    // wheel-scroll bound and so the scrollbar's thumb sizes itself
    // correctly. Mirror of the live Inspector's `set_last_inspector_*`
    // pair — kept separate so the two panels can scroll independently.
    let content_h = (y - body_top_y).max(0.0);
    let visible_h = (content_bottom - content_top).max(0.0);
    set_last_gallery_content_h(content_h);
    set_last_gallery_visible_h(visible_h);

    // Scrollbar — same widget as Inspector / Hierarchy, but routed
    // via `GALLERY_SCROLLBAR_ID` so `dispatch::scrollbar_panel_for_id`
    // sends drag-thumb moves to `GAL_PANEL`.
    if crate::widget::scrollbar_is_needed(content_h, visible_h) {
        let body = Rect::new(rect.x, content_top, rect.w, visible_h);
        let track = crate::widget::scrollbar_track_rect(body);
        let thumb = crate::widget::scrollbar_thumb_rect(track, scroll_y, content_h, visible_h);
        let is_active = matches!(store.scrollbar_drag(), Some(d) if d.panel == ids::GAL_PANEL);
        crate::widget::paint_scrollbar(
            body, scroll_y, content_h, visible_h, is_active, scene, theme,
        );
        hit_index.register(crate::widget::GALLERY_SCROLLBAR_ID, thumb);
    }

    // Late-paint phase: open Dropdown popover sits on top of every
    // section that ran before it. `take_pending_dropdown_chip` is a
    // thread_local owned by the showcase; the live Inspector never
    // paints dropdowns so there's no contention.
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
        for (i, opt) in dd.options.iter().enumerate() {
            hit_index.register(opt.id, dd.option_rect(chip, i));
        }
    }

    scene.pop_layer();
    paint_panel_corner_dot(rect, scene, theme);
    hit_index.register(ids::GAL_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::GAL_RESIZE_HANDLE, resize_handle_rect);
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Paint a collapsible section header at `(x, y)` and register its
/// click hit. Returns `(next_y, is_open)` where `is_open` controls
/// whether the caller paints the section body or skips ahead.
///
/// Single source of truth for every section in the Inspector — the
/// chevron direction, hit rect, and collapsed-flag read all live
/// here so individual section painters stay focused on their content.
#[allow(clippy::too_many_arguments)]
fn paint_collapsible_header(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    id: NodeId,
    color_id: NodeId,
    label: &str,
    _count: u32,
) -> (f32, bool) {
    let r = Rect::new(x, y, w, SECTION_HEAD_H);
    let is_collapsed = store.is_collapsed(id);
    hit_index.register(id, r);
    // Color circle replaces the legacy count chip. Default to a
    // neutral mid-gray when the user hasn't picked a color yet.
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xFF]);
    let header = SectionHeader::new(id, label)
        .color(rgba)
        .collapsible(!is_collapsed);
    paint_section_header(&header, r, scene, text_system, theme);
    // Register the circle hit zone AFTER the header rect so the
    // circle "wins" the back-to-front HitIndex lookup. Clicking the
    // circle opens the picker for this section; clicking elsewhere
    // on the header toggles collapse.
    if let Some(circle_rect) = crate::widget::color_circle_hit_rect(&header, r) {
        hit_index.register(color_id, circle_rect);
    }
    (y + SECTION_HEAD_H + Spacing::Xs.px(), !is_collapsed)
}

/// Discreet colored separator painted at the end of each section's
/// content (when expanded). 1 px tall, almost full-width (2 px
/// horizontal inset only — line spans the panel body edge to edge).
/// Balanced vertical padding (`SEPARATOR_PAD_Y` above + below) so
/// the line sits centered in its own gap, not lopsided against the
/// content above. Owns the entire inter-section spacing — callers
/// should NOT add `SECTION_GAP` on top.
#[allow(clippy::too_many_arguments)]
fn paint_left_label(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    label: &str,
    label_w: f32,
    y: f32,
    row_h: f32,
) {
    let label_y = y + (row_h - TypeToken::Xs.px()) * 0.5;
    paint_text(
        text_system,
        scene,
        label,
        x,
        label_y,
        TypeToken::Xs.px(),
        label_w,
        resolve(ColorToken::Text2, theme),
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_inputs_section(
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
        ids::INSP_SECTION_INPUTS,
        ids::INSP_SECTION_INPUTS_COLOR,
        "Inputs",
        4,
    );
    if !open {
        return y;
    }

    // TextInput.
    let r = Rect::new(x, y, w, FIELD_H);
    hit_index.register(ids::INSP_SAMPLE_TEXT, r);
    let (ti_state, ti_text, ti_caret, ti_anchor) = read_text_input(store, ids::INSP_SAMPLE_TEXT);
    let input = TextInput::new(ids::INSP_SAMPLE_TEXT, "Name")
        .placeholder("Entity name")
        .state(ti_state);
    paint_text_input_with_buffer(
        &input,
        Some(ti_text),
        Some(ti_caret),
        ti_anchor,
        r,
        scene,
        text_system,
        theme,
    );
    y += FIELD_H + ROW_GAP;

    // TextArea (3 lines tall).
    let area_h = 60.0_f32; // LITERAL-PX-OK: 3-line TextArea showcase height (chrome-specific demo dim)
    let r = Rect::new(x, y, w, area_h);
    hit_index.register(ids::INSP_SAMPLE_TEXTAREA, r);
    let (ta_state, ta_text, ta_caret, ta_anchor) =
        read_text_input(store, ids::INSP_SAMPLE_TEXTAREA);
    let mut ta = TextArea::new(ids::INSP_SAMPLE_TEXTAREA, "Notes")
        .placeholder("Notes…")
        .state(ta_state);
    ta.value = ta_text.to_string();
    paint_text_area_with_state(&ta, Some(ta_caret), ta_anchor, r, scene, text_system, theme);
    y += area_h + ROW_GAP;

    // Combobox.
    let r = Rect::new(x, y, w, FIELD_H);
    hit_index.register(ids::INSP_SAMPLE_COMBO, r);
    let (cb_state, cb_open, cb_query, cb_caret, cb_anchor) =
        read_combobox(store, ids::INSP_SAMPLE_COMBO);
    let combo = Combobox::new(
        ids::INSP_SAMPLE_COMBO,
        "Asset",
        vec![
            ComboboxOption::new(ids::INSP_SAMPLE_COMBO_OPT_A, "spike.gltf"),
            ComboboxOption::new(ids::INSP_SAMPLE_COMBO_OPT_B, "block.gltf"),
            ComboboxOption::new(ids::INSP_SAMPLE_COMBO_OPT_C, "decal.png"),
        ],
    )
    .query(cb_query)
    .open(cb_open)
    .state(cb_state);
    paint_combobox_with_state(&combo, cb_caret, cb_anchor, r, scene, text_system, theme);
    y += FIELD_H + ROW_GAP;

    // NumberInput.
    let label_w = 80.0_f32; // LITERAL-PX-OK: showcase label column width (demo geometry)
    let chip_w = (w - label_w - Spacing::Md.px()).max(40.0); // LITERAL-PX-OK: min chip width (demo)
    let r = Rect::new(x + label_w + Spacing::Md.px(), y, chip_w, FIELD_H);
    hit_index.register(ids::INSP_SAMPLE_NUMBER, r);
    paint_left_label(scene, text_system, theme, x, "Value", label_w, y, FIELD_H);
    let (n_state, n_value, n_buffer, n_caret, n_anchor) =
        read_number_input(store, ids::INSP_SAMPLE_NUMBER);
    let num = NumberInput::new(ids::INSP_SAMPLE_NUMBER, "Value", n_value).state(n_state);
    paint_number_input_with_buffer(
        &num,
        Some(n_buffer),
        n_caret,
        n_anchor,
        r,
        scene,
        text_system,
        theme,
    );
    y + FIELD_H
}

#[allow(clippy::too_many_arguments)]
fn paint_slider_section(
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
        ids::INSP_SECTION_SLIDER,
        ids::INSP_SECTION_SLIDER_COLOR,
        "Slider",
        1,
    );
    if !open {
        return y;
    }
    let (_, value) = store
        .slider(ids::INSP_SAMPLE_SLIDER)
        .unwrap_or((SliderState::Normal, 0.62)); // LITERAL-PX-OK: slider default ratio (showcase demo seed value)
    let r = Rect::new(x, y, w, FIELD_H);
    paint_slider_with_chip(
        r,
        "Speed",
        value,
        ids::INSP_SAMPLE_SLIDER,
        ids::INSP_SAMPLE_SLIDER_CHIP,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y += FIELD_H;
    y
}

#[allow(clippy::too_many_arguments)]
fn paint_switches_section(
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
        ids::INSP_SECTION_SWITCHES,
        ids::INSP_SECTION_SWITCHES_COLOR,
        "Switches",
        3,
    );
    if !open {
        return y;
    }

    // Checkbox.
    let r = Rect::new(x, y, w, Density::Compact.row_h_px());
    hit_index.register(ids::INSP_SAMPLE_CHECKBOX, r);
    let (cb_state, cb_value) = store
        .checkbox(ids::INSP_SAMPLE_CHECKBOX)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Checked));
    let cb = Checkbox::new(ids::INSP_SAMPLE_CHECKBOX, "Hot reload on save")
        .state(cb_state)
        .value(cb_value);
    paint_checkbox(&cb, r, scene, text_system, theme);
    y += Density::Compact.row_h_px() + ROW_GAP;

    // Toggle.
    let toggle_w = TypeToken::Xl3.px(); // LITERAL-PX-OK: toggle widget width (matches TypeToken::Xl3 = 44px by coincidence; chrome-specific)
    let row_h = Density::Compact.row_h_px();
    let tr = Rect::new(x + w - toggle_w, y, toggle_w, row_h);
    hit_index.register(ids::INSP_SAMPLE_TOGGLE, tr);
    let (tg_state, tg_on) = store
        .toggle(ids::INSP_SAMPLE_TOGGLE)
        .unwrap_or((ToggleState::Normal, true));
    let toggle = Toggle::new(ids::INSP_SAMPLE_TOGGLE, "Snap to grid")
        .state(tg_state)
        .on(tg_on);
    paint_toggle(&toggle, tr, scene, theme);
    paint_left_label(
        scene,
        text_system,
        theme,
        x,
        "Snap to grid",
        w - toggle_w - Spacing::Sm.px(),
        y,
        row_h,
    );
    y += row_h + ROW_GAP;

    // Segmented RadioGroup. We use the per-tab pressed-button trick
    // (mirrors the Inspector-tabs pattern in `paint_inspector_tabs`
    // pre-cleanup) so selection survives across frames without
    // adding a typed RadioGroup state to the store.
    let selected = active_index(
        store,
        &[
            ids::INSP_SAMPLE_RADIO_A,
            ids::INSP_SAMPLE_RADIO_B,
            ids::INSP_SAMPLE_RADIO_C,
        ],
    )
    .unwrap_or(0);
    let selected_label: &str = ["Low", "Mid", "High"][selected];
    let rg = RadioGroup::new(
        NodeId(0),
        "Quality",
        vec![
            RadioOption::new(ids::INSP_SAMPLE_RADIO_A, "Low".to_string(), "Low"),
            RadioOption::new(ids::INSP_SAMPLE_RADIO_B, "Mid".to_string(), "Mid"),
            RadioOption::new(ids::INSP_SAMPLE_RADIO_C, "High".to_string(), "High"),
        ],
    )
    .orientation(RadioOrientation::Segmented)
    .selected(selected_label.to_string());
    let r = Rect::new(x, y, w, ROW_H_PX);
    paint_radio_group_with_labels(&rg, r, scene, text_system, theme);
    // Register per-option hit rects.
    for (i, id) in [
        ids::INSP_SAMPLE_RADIO_A,
        ids::INSP_SAMPLE_RADIO_B,
        ids::INSP_SAMPLE_RADIO_C,
    ]
    .iter()
    .enumerate()
    {
        hit_index.register(*id, rg.option_rect(r, i));
    }
    y + ROW_H_PX
}

#[allow(clippy::too_many_arguments)]
fn paint_lists_section(
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

#[allow(clippy::too_many_arguments)]
fn paint_vector_section(
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
        ids::INSP_SECTION_VECTOR,
        ids::INSP_SECTION_VECTOR_COLOR,
        "Vector",
        3,
    );
    if !open {
        return y;
    }
    let r = Rect::new(x, y, w, FIELD_H);
    // Pull each axis' live state from the store so focus/typing
    // actually drives the chip. Previously this rendered via the
    // static `paint_vector3_editor` which never reads the in-progress
    // edit buffer — clicking a chip "focused" it visually (border
    // accent) but every keystroke vanished because the painter kept
    // showing `input.value` regardless. See `docs/UI_Bugs/README.md`
    // §9.14.
    let (sx, vx, bx, cx, ax) = read_number_input(store, ids::INSP_SAMPLE_V3_X);
    let (sy, vy, by, cy, ay) = read_number_input(store, ids::INSP_SAMPLE_V3_Y);
    let (sz, vz, bz, cz, az) = read_number_input(store, ids::INSP_SAMPLE_V3_Z);
    let nx = NumberInput::new(ids::INSP_SAMPLE_V3_X, "X", vx).state(sx);
    let ny = NumberInput::new(ids::INSP_SAMPLE_V3_Y, "Y", vy).state(sy);
    let nz = NumberInput::new(ids::INSP_SAMPLE_V3_Z, "Z", vz).state(sz);
    let v3 = Vector3Editor::new(NodeId(0), "Position", nx, ny, nz);
    crate::widget::paint_vector3_editor_with_state(
        &v3,
        [Some(bx), Some(by), Some(bz)],
        [cx, cy, cz],
        [ax, ay, az],
        r,
        scene,
        text_system,
        theme,
    );
    let rects = v3.field_rects(r);
    for (id, fr) in [
        (ids::INSP_SAMPLE_V3_X, rects[0]),
        (ids::INSP_SAMPLE_V3_Y, rects[1]),
        (ids::INSP_SAMPLE_V3_Z, rects[2]),
    ] {
        hit_index.register(id, fr);
    }
    y += FIELD_H;
    y
}

#[allow(clippy::too_many_arguments)]
fn paint_status_section(
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
        ids::INSP_SECTION_STATUS,
        ids::INSP_SECTION_STATUS_COLOR,
        "Status",
        3,
    );
    if !open {
        return y;
    }

    // ProgressBar — determinate, 60 %.
    let bar = ProgressBar::new(NodeId(0), "Build")
        .determinate(0.6) // LITERAL-PX-OK: progress ratio (demo value)
        .show_percent(true);
    let bar_rect = Rect::new(x, y, w, Spacing::Lg.px());
    paint_progress_bar(&bar, bar_rect, scene, text_system, theme);
    y += Spacing::Lg.px() + ROW_GAP;

    // Spinner + caption.
    let spin_rect = Rect::new(x, y, Radius::Xl2.px(), Radius::Xl2.px());
    paint_spinner(&Spinner::new(NodeId(0), "Loading"), spin_rect, scene, theme);
    paint_left_label(
        scene,
        text_system,
        theme,
        x + 28.0, // LITERAL-PX-OK: spinner+caption offset (Spinner 20 + gap ~8 chrome composite)
        "Loading…",
        w - 28.0, // LITERAL-PX-OK: caption width budget
        y,
        Radius::Xl2.px(),
    );
    y += Radius::Xl2.px() + ROW_GAP;

    // Tag chips — Accent + Success + Warn.
    let chip_w = 50.0_f32; // LITERAL-PX-OK: tag chip width (chrome-specific)
    let chip_h = TypeToken::Lg.px();
    let gap = Spacing::Sm.px();
    for (i, (label, tone)) in [
        ("PRF", TagTone::Accent),
        ("OK", TagTone::Success),
        ("HOT", TagTone::Warn),
    ]
    .iter()
    .enumerate()
    {
        let cr = Rect::new(x + (chip_w + gap) * i as f32, y, chip_w, chip_h);
        let tag = Tag::new(NodeId(0), *label).tone(*tone);
        paint_tag(&tag, cr, scene, text_system, theme);
    }
    y + chip_h
}

#[allow(clippy::too_many_arguments)]
fn paint_color_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_COLOR,
        ids::INSP_SECTION_COLOR_COLOR,
        "Color",
        1,
    );
    if !open {
        return y;
    }
    let sw_h = ROW_H_PX;
    let label_w = 80.0_f32; // LITERAL-PX-OK: showcase label column width
    paint_left_label(scene, text_system, theme, x, "Tint", label_w, y, sw_h);
    let sw_size = Spacing::Xl3.px();
    let sr = Rect::new(x + w - sw_size, y, sw_size, sw_h);
    hit_index.register(ids::INSP_SAMPLE_SWATCH, sr);
    // Read the swatch's live color from `widget_colors` so the
    // picker's edits propagate back through hero's per-frame
    // mirror. Fallback to the initial purple if nothing was seeded.
    let rgba = store
        .widget_color(ids::INSP_SAMPLE_SWATCH)
        .unwrap_or([120, 60, 200, 255]);
    let mut tint = ColorSwatch::new(ids::INSP_SAMPLE_SWATCH, "Tint", rgba);
    tint.size = SwatchSize::Md;
    paint_color_swatch(&tint, sr, scene, theme);
    y + sw_h
}

#[allow(clippy::too_many_arguments)]
fn paint_actions_section(
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
        ids::INSP_SECTION_ACTIONS,
        ids::INSP_SECTION_ACTIONS_COLOR,
        "Actions",
        4,
    );
    if !open {
        return y;
    }
    let btn_h = 30.0_f32; // LITERAL-PX-OK: action button height (distinct from ROW_H_PX)
    let gap = Spacing::Sm.px();
    // Three labelled buttons.
    let trio_w = (w - gap * 2.0) / 3.0; // LITERAL-PX-OK: button count divisor
    let trio = [
        (ids::INSP_SAMPLE_BTN_PRIMARY, "Save", ButtonKind::Accent),
        (
            ids::INSP_SAMPLE_BTN_SECONDARY,
            "Cancel",
            ButtonKind::Default,
        ),
        (ids::INSP_SAMPLE_BTN_DANGER, "Delete", ButtonKind::Danger),
    ];
    for (i, (id, label, kind)) in trio.iter().enumerate() {
        let r = Rect::new(x + (trio_w + gap) * i as f32, y, trio_w, btn_h);
        hit_index.register(*id, r);
        let state = store.button_state(*id).unwrap_or(ButtonState::Normal);
        let btn = Button::new(*id, *label).kind(*kind).state(state);
        paint_button(&btn, r, scene, text_system, theme);
    }
    y += btn_h + ROW_GAP;

    // Icon button + Tag (removable) on one row.
    let icon_size = Spacing::Xl3.px();
    let ir = Rect::new(x, y, icon_size, icon_size);
    hit_index.register(ids::INSP_SAMPLE_BTN_ICON, ir);
    let icon_state = store
        .button_state(ids::INSP_SAMPLE_BTN_ICON)
        .unwrap_or(ButtonState::Normal);
    let icon_btn = Button::new(ids::INSP_SAMPLE_BTN_ICON, "")
        .kind(ButtonKind::IconOnly {
            icon: IconId::Settings,
        })
        .state(icon_state);
    paint_button(&icon_btn, ir, scene, text_system, theme);

    let tag_w = 80.0_f32; // LITERAL-PX-OK: showcase tag chip width
    let tag_h = Density::Compact.row_h_px();
    let tr = Rect::new(
        x + icon_size + Spacing::Md.px(),
        y + (icon_size - tag_h) * 0.5,
        tag_w,
        tag_h,
    );
    let tag_state = match store.get(ids::INSP_SAMPLE_TAG_REMOVE) {
        Some(InteractiveState::Tag { state }) => *state,
        _ => TagState::Normal,
    };
    let tag = Tag::new(ids::INSP_SAMPLE_TAG_REMOVE, "filter")
        .tone(TagTone::Accent)
        .removable(true)
        .state(tag_state);
    paint_tag(&tag, tr, scene, text_system, theme);
    if let Some(close_r) = tag.close_rect(tr) {
        hit_index.register(ids::INSP_SAMPLE_TAG_REMOVE, close_r);
    }
    y + icon_size
}

#[allow(clippy::too_many_arguments)]
fn paint_identity_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_IDENTITY,
        ids::INSP_SECTION_IDENTITY_COLOR,
        "Identity",
        2,
    );
    if !open {
        return y;
    }
    let size = ICON_BTN_SIZE_PX;
    let gap = Spacing::Md.px();
    let circle = Rect::new(x, y, size, size);
    paint_avatar(
        &Avatar::new(NodeId(0), "Enio", 'E'),
        circle,
        scene,
        text_system,
        theme,
    );
    let square = Rect::new(x + size + gap, y, size, size);
    paint_avatar(
        &Avatar::new(NodeId(0), "Player", 'P').shape(AvatarShape::Square),
        square,
        scene,
        text_system,
        theme,
    );
    // Caption to the right.
    paint_left_label(
        scene,
        text_system,
        theme,
        x + size * 2.0 + gap * 2.0,
        "Avatar · circle + square",
        (w - size * 2.0 - gap * 2.0).max(0.0),
        y,
        size,
    );
    let _ = w; // suppress unused
    y + size
}

#[allow(clippy::too_many_arguments)]
fn paint_card_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let (y, open) = paint_collapsible_header(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ids::INSP_SECTION_CARD,
        ids::INSP_SECTION_CARD_COLOR,
        "Card",
        1,
    );
    if !open {
        return y;
    }
    let card_h = 80.0_f32; // LITERAL-PX-OK: demo Card height (showcase-specific)
    let r = Rect::new(x, y, w, card_h);
    let card = Card::new(NodeId(0)).title("Quick actions");
    paint_card(&card, r, scene, text_system, theme);
    // Body caption — Card itself doesn't paint into body_rect, so we
    // write a single label line inside it as a static demo.
    let body = card.body_rect(r);
    paint_text(
        text_system,
        scene,
        "Card body slot — host content here.",
        body.x,
        body.y,
        TypeToken::Xs.px(),
        body.w,
        resolve(ColorToken::Text2, theme),
    );
    y + card_h
}

// ── Store accessor helpers ─────────────────────────────────────────────────
