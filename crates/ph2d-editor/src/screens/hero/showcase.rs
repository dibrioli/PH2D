//! Components Showcase — a floating panel anchored at the bottom-
//! left of the canvas that demonstrates every widget the rest of
//! the hero couldn't host inline. Lets a developer (or Enio
//! reviewing a build) see all 32 widgets in one place without
//! scrolling between mockups.
//!
//! Static demo data: every widget paints with sample state. A few
//! (TextInput, NumberInput, Combobox, BlenderColorPicker) read live
//! state from the [`WidgetStore`] when the consumer wires them.

use super::HeroLayout;
use super::ids;
use super::style::paint_panel_surface;
use crate::icons::IconId;
use crate::interaction::{HitIndex, WidgetEvent, WidgetStore};

/// Register every Showcase widget into the [`WidgetStore`] so that
/// hit-testing and state-driven painting work. Called once at screen
/// construction time (never during the hot-path paint).
pub fn populate(store: &mut WidgetStore) {
    use crate::interaction::InteractiveState;
    use crate::widget::{
        ButtonState, CheckboxState, CheckboxValue, ComboboxState, DropdownState, SliderOrientation,
        SliderState, TagState, TextInputState,
    };

    // Showcase panel + its drag handle — same drag mechanism as the
    // BlenderColorPicker (panel-agnostic on the parent `NodeId`).
    store.register(ids::SHOWCASE_PANEL, InteractiveState::Plain);
    store.register(
        ids::SHOWCASE_DRAG_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::SHOWCASE_PANEL,
            kind: crate::interaction::BlenderHitKind::DragHandle,
        },
    );

    // TextInput "Name" — initial text mirrors the painted placeholder.
    store.register(
        ids::SHOWCASE_TEXT_INPUT_NAME,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: "Player_01".to_string(),
            caret: 9,
            selection_anchor: None,
        },
    );

    // TextArea "Notes" — uses the TextInput variant (no separate
    // TextArea variant; the painter reads text the same way).
    store.register(
        ids::SHOWCASE_TEXT_AREA_NOTES,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: "Brush prefab — hot reloads on save.\nCollider via sprite alpha.".to_string(),
            caret: 0,
            selection_anchor: None,
        },
    );

    // Combobox "Asset" — query pre-filled to match the painted state.
    store.register(
        ids::SHOWCASE_COMBOBOX_ASSET,
        InteractiveState::Combobox {
            state: ComboboxState::Normal,
            open: false,
            query: "sp".to_string(),
            caret: 2,
            selection_anchor: None,
        },
    );

    // Checkbox "Lock" — indeterminate initial value.
    store.register(
        ids::SHOWCASE_CHECKBOX_LOCK,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Indeterminate,
        },
    );

    // Dropdown "View" — "Front" pre-selected (index 0).
    store.register(
        ids::SHOWCASE_DROPDOWN_VIEW,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(0),
        },
    );

    // RadioGroup "Mode" — "Shaded" pre-selected (index 0).
    store.register(
        ids::SHOWCASE_RADIO_MODE,
        InteractiveState::Radio {
            state: ButtonState::Normal,
            selected_index: 0,
        },
    );

    // Vertical slider — 65 % pre-filled, hovered visual for demo.
    store.register(
        ids::SHOWCASE_SLIDER_VERTICAL,
        InteractiveState::Slider {
            state: SliderState::Hovered,
            value: 0.65,
            orientation: SliderOrientation::Vertical,
        },
    );

    // Tags — DRAFT is non-removable; DONE is removable.
    store.register(
        ids::SHOWCASE_TAG_DRAFT,
        InteractiveState::Tag {
            state: TagState::Normal,
        },
    );
    store.register(
        ids::SHOWCASE_TAG_DONE,
        InteractiveState::Tag {
            state: TagState::Normal,
        },
    );

    // Vector3Editor NumberInputs (X / Y / Z positional fields).
    for (id, value) in [
        (ids::SHOWCASE_V3_X, 1.0_f64),
        (ids::SHOWCASE_V3_Y, 2.0),
        (ids::SHOWCASE_V3_Z, 3.0),
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: crate::interaction::format_number(value),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
    }

    // SectionHeader "Advanced" — plain focusable.
    store.register(ids::SHOWCASE_SECTION_ADVANCED, InteractiveState::Plain);

    // Modal cancel + confirm buttons.
    store.register(
        ids::SHOWCASE_MODAL_CANCEL,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    store.register(
        ids::SHOWCASE_MODAL_CONFIRM,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );

    // Card + list items + their divider.
    store.register(ids::SHOWCASE_CARD_QUICK_ACTIONS, InteractiveState::Plain);
    store.register(
        ids::SHOWCASE_LIST_OPEN,
        InteractiveState::ListItem {
            state: crate::widget::ListItemState::Normal,
            selected: false,
        },
    );
    store.register(
        ids::SHOWCASE_LIST_SAVE,
        InteractiveState::ListItem {
            state: crate::widget::ListItemState::Normal,
            selected: false,
        },
    );
    store.register(
        ids::SHOWCASE_LIST_EXPORT,
        InteractiveState::ListItem {
            state: crate::widget::ListItemState::Normal,
            selected: false,
        },
    );

    // ContextMenu + items.
    store.register(ids::SHOWCASE_CTX_MENU, InteractiveState::Plain);
    store.register(
        ids::SHOWCASE_CTX_ITEM_CUT,
        InteractiveState::ListItem {
            state: crate::widget::ListItemState::Normal,
            selected: false,
        },
    );
    store.register(
        ids::SHOWCASE_CTX_ITEM_COPY,
        InteractiveState::ListItem {
            state: crate::widget::ListItemState::Normal,
            selected: false,
        },
    );
    store.register(
        ids::SHOWCASE_CTX_ITEM_DELETE,
        InteractiveState::ListItem {
            state: crate::widget::ListItemState::Normal,
            selected: false,
        },
    );

    // Popover surface — plain (no interactive state needed).
    store.register(ids::SHOWCASE_POPOVER, InteractiveState::Plain);

    // Non-interactive decorative widgets — registered as Plain so the
    // a11y tree has nodes for them and the dispatcher never panics on
    // an unexpected id.
    for id in [
        ids::SHOWCASE_PROGRESS_DET,
        ids::SHOWCASE_PROGRESS_IND,
        ids::SHOWCASE_SPINNER,
        ids::SHOWCASE_AVATAR_CIRCLE,
        ids::SHOWCASE_AVATAR_SQUARE,
    ] {
        store.register(id, InteractiveState::Plain);
    }
}

/// Apply a [`WidgetEvent`] against Showcase widgets. Returns true
/// iff the event was consumed. Stub.
pub fn apply_event(_store: &mut WidgetStore, _event: WidgetEvent) -> bool {
    false
}
use crate::paint::{paint_text, paint_text_centered, resolve};
use crate::widget::{
    Avatar, AvatarShape, BlenderColorPicker, Button, ButtonKind, Card, ChannelMode, Checkbox,
    CheckboxState, CheckboxValue, Combobox, ComboboxOption, ComboboxState, ContextMenu,
    ContextMenuEntry, Divider, DividerOrientation, Dropdown, DropdownOption, DropdownState,
    ListItem, ListItemState, Modal, NumberInput, Popover, ProgressBar, RadioGroup, RadioOption,
    RadioOrientation, SectionHeader, Slider, SliderState, Spinner, Tag, TagTone, TextArea,
    TextInput, TextInputState, TreeNode, TreeView, Vector3Editor, paint_avatar, paint_card,
    paint_checkbox, paint_combobox, paint_context_menu, paint_divider, paint_dropdown,
    paint_list_item, paint_modal, paint_popover, paint_progress_bar, paint_radio_group,
    paint_section_header, paint_slider, paint_spinner, paint_tag, paint_text_area,
    paint_text_input, paint_vector3_editor,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Width and height of the showcase panel. Anchored to the bottom-
/// left of the canvas, with EDGE_PAD inset.
pub const SHOWCASE_W: f32 = 360.0;
pub const SHOWCASE_H: f32 = 720.0;

#[allow(clippy::too_many_arguments)]
pub fn paint_components_showcase(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    if layout.canvas.w < SHOWCASE_W + 40.0 || layout.canvas.h < SHOWCASE_H + 40.0 {
        return; // viewport too small — skip showcase.
    }
    let (dx, dy) = store.blender_picker_offset(ids::SHOWCASE_PANEL);
    let base_x = layout.canvas.x + 12.0;
    let base_y = layout.canvas.y + layout.canvas.h - SHOWCASE_H - 12.0;
    let min_x = layout.canvas.x + 8.0;
    let max_x = layout.canvas.x + layout.canvas.w - SHOWCASE_W - 8.0;
    let min_y = layout.canvas.y + 8.0;
    let max_y = layout.canvas.y + layout.canvas.h - SHOWCASE_H - 8.0;
    let rect = Rect::new(
        (base_x + dx).clamp(min_x, max_x),
        (base_y + dy).clamp(min_y, max_y),
        SHOWCASE_W,
        SHOWCASE_H,
    );
    paint_panel_surface(rect, scene, theme);

    let pad = Spacing::Lg.px();
    let inner_x = rect.x + pad;
    let inner_w = rect.w - pad * 2.0;
    let mut y = rect.y + pad + 4.0;

    // Drag handle bar at the top — same visual + same dispatch path
    // as the BlenderColorPicker handle (3 dots; click+drag updates
    // `panel_offset` keyed by `SHOWCASE_PANEL`).
    let drag_h = 14.0_f32;
    let drag_rect = Rect::new(inner_x, y, inner_w, drag_h);
    crate::paint::fill_rounded_rect(
        scene,
        drag_rect,
        ph2d_tokens::Radius::Sm.px(),
        crate::paint::resolve(ColorToken::Bg2, theme),
    );
    let dot_y = drag_rect.y + drag_rect.h * 0.5 - 1.5;
    let dot_color = crate::paint::resolve(ColorToken::Text3, theme);
    for i in 0..3i32 {
        let dot_x = drag_rect.x + drag_rect.w * 0.5 + (i - 1) as f32 * 6.0 - 1.5;
        let dot_rect = Rect::new(dot_x, dot_y, 3.0, 3.0);
        crate::paint::fill_rounded_rect(scene, dot_rect, 1.5, dot_color);
    }
    hit_index.register(ids::SHOWCASE_DRAG_HANDLE, drag_rect);
    y += drag_h + Spacing::Sm.px();

    // Title.
    paint_text(
        text_system,
        scene,
        "Components Showcase",
        inner_x,
        y,
        TypeToken::Md.px(),
        inner_w,
        resolve(ColorToken::Text1, theme),
    );
    y += TypeToken::Md.px() + Spacing::Md.px();

    // Subtitle.
    paint_text(
        text_system,
        scene,
        "Every widget the M13 library ships, in functional use.",
        inner_x,
        y,
        TypeToken::Xs.px(),
        inner_w,
        resolve(ColorToken::Text3, theme),
    );
    y += TypeToken::Xs.px() + Spacing::Lg.px();

    // 1. Card hosting a tiny title/sub + 3 ListItems + Divider.
    let card_h = 156.0;
    let card_rect = Rect::new(inner_x, y, inner_w, card_h);
    let card = Card::new(ids::SHOWCASE_CARD_QUICK_ACTIONS).title("Quick actions");
    hit_index.register(ids::SHOWCASE_CARD_QUICK_ACTIONS, card_rect);
    paint_card(&card, card_rect, scene, text_system, theme);
    let body = card.body_rect(card_rect);
    let row_h = 28.0;
    let items = [
        (
            ids::SHOWCASE_LIST_OPEN,
            "Open",
            IconId::Open,
            Some("\u{2318}O"),
        ),
        (
            ids::SHOWCASE_LIST_SAVE,
            "Save",
            IconId::Save,
            Some("\u{2318}S"),
        ),
        (ids::SHOWCASE_LIST_EXPORT, "Export", IconId::Export, None),
    ];
    for (i, (item_id, label, icon, shortcut)) in items.iter().enumerate() {
        let row = Rect::new(body.x, body.y + (row_h + 2.0) * i as f32, body.w, row_h);
        hit_index.register(*item_id, row);
        let mut li = ListItem::new(*item_id, *label).icon(*icon);
        if let Some(sc) = shortcut {
            li = li.value(*sc);
        }
        paint_list_item(&li, row, scene, text_system, theme);
    }
    // Divider after items.
    let div_y = body.y + (row_h + 2.0) * items.len() as f32 + 2.0;
    let div_rect = Rect::new(body.x, div_y, body.w, 8.0);
    let div = Divider::new(ids::SHOWCASE_CARD_DIVIDER).orientation(DividerOrientation::Horizontal);
    paint_divider(&div, div_rect, scene, theme);
    y += card_h + Spacing::Md.px();

    // 2. Vector3Editor (Position transform).
    paint_text(
        text_system,
        scene,
        "Transform",
        inner_x,
        y,
        TypeToken::Xs.px(),
        inner_w,
        resolve(ColorToken::Text2, theme),
    );
    y += TypeToken::Xs.px() + 4.0;
    let v3_rect = Rect::new(inner_x, y, inner_w, 32.0);
    // Read live values from store; fall back to hard-coded defaults if
    // the store hasn't been populated yet (e.g. smoke tests).
    let x_val = store.number_value(ids::SHOWCASE_V3_X).unwrap_or(1.0);
    let y_val = store.number_value(ids::SHOWCASE_V3_Y).unwrap_or(2.0);
    let z_val = store.number_value(ids::SHOWCASE_V3_Z).unwrap_or(3.0);
    hit_index.register(
        ids::SHOWCASE_V3_X,
        Rect::new(inner_x, y, inner_w / 3.0, 32.0),
    );
    hit_index.register(
        ids::SHOWCASE_V3_Y,
        Rect::new(inner_x + inner_w / 3.0, y, inner_w / 3.0, 32.0),
    );
    hit_index.register(
        ids::SHOWCASE_V3_Z,
        Rect::new(inner_x + inner_w * 2.0 / 3.0, y, inner_w / 3.0, 32.0),
    );
    let v3 = Vector3Editor::new(
        ids::SHOWCASE_V3_POS,
        "Position",
        NumberInput::new(ids::SHOWCASE_V3_X, "X", x_val),
        NumberInput::new(ids::SHOWCASE_V3_Y, "Y", y_val),
        NumberInput::new(ids::SHOWCASE_V3_Z, "Z", z_val),
    );
    paint_vector3_editor(&v3, v3_rect, scene, text_system, theme);
    y += 32.0 + Spacing::Md.px();

    // 3. ProgressBar — determinate + indeterminate side by side.
    paint_text(
        text_system,
        scene,
        "Progress",
        inner_x,
        y,
        TypeToken::Xs.px(),
        inner_w,
        resolve(ColorToken::Text2, theme),
    );
    y += TypeToken::Xs.px() + 4.0;
    let bar_h = 8.0;
    let bar_w = (inner_w - Spacing::Md.px()) / 2.0;
    let p_det = ProgressBar::new(ids::SHOWCASE_PROGRESS_DET, "Loading")
        .determinate(0.62)
        .show_percent(false);
    paint_progress_bar(
        &p_det,
        Rect::new(inner_x, y, bar_w, bar_h),
        scene,
        text_system,
        theme,
    );
    let p_ind = ProgressBar::new(ids::SHOWCASE_PROGRESS_IND, "Indeterminate").indeterminate();
    paint_progress_bar(
        &p_ind,
        Rect::new(inner_x + bar_w + Spacing::Md.px(), y, bar_w, bar_h),
        scene,
        text_system,
        theme,
    );
    y += bar_h + Spacing::Md.px();

    // 4. Spinner + Avatar (Square + Circle).
    let spinner_rect = Rect::new(inner_x, y, 24.0, 24.0);
    let spinner = Spinner::new(ids::SHOWCASE_SPINNER, "Working");
    paint_spinner(&spinner, spinner_rect, scene, theme);
    let av_circle = Rect::new(inner_x + 36.0, y, 24.0, 24.0);
    let av_square = Rect::new(inner_x + 72.0, y, 24.0, 24.0);
    let av_c = Avatar::new(ids::SHOWCASE_AVATAR_CIRCLE, "Anna", 'A').shape(AvatarShape::Circle);
    let av_s = Avatar::new(ids::SHOWCASE_AVATAR_SQUARE, "Bob", 'B').shape(AvatarShape::Square);
    paint_avatar(&av_c, av_circle, scene, text_system, theme);
    paint_avatar(&av_s, av_square, scene, text_system, theme);
    // Tags on the right — register hit rects so clicks reach them.
    let tag_x = inner_x + 108.0;
    let tag_w = 50.0_f32;
    let tag_rect_a = Rect::new(tag_x, y + 2.0, tag_w, 20.0);
    let tag_rect_b = Rect::new(tag_x + tag_w + 4.0, y + 2.0, tag_w, 20.0);
    hit_index.register(ids::SHOWCASE_TAG_DRAFT, tag_rect_a);
    hit_index.register(ids::SHOWCASE_TAG_DONE, tag_rect_b);
    let tag_a = Tag::new(ids::SHOWCASE_TAG_DRAFT, "DRAFT").tone(TagTone::Warn);
    let tag_b = Tag::new(ids::SHOWCASE_TAG_DONE, "DONE")
        .tone(TagTone::Success)
        .removable(true);
    paint_tag(&tag_a, tag_rect_a, scene, text_system, theme);
    paint_tag(&tag_b, tag_rect_b, scene, text_system, theme);
    y += 24.0 + Spacing::Md.px();

    // 5. RadioGroup (Segmented) + Dropdown.
    let radio_rect = Rect::new(inner_x, y, inner_w * 0.55, 28.0);
    hit_index.register(ids::SHOWCASE_RADIO_MODE, radio_rect);
    // Read selected index from store (default "shaded" = index 0).
    let radio_sel_idx = store
        .get(ids::SHOWCASE_RADIO_MODE)
        .and_then(|s| {
            if let crate::interaction::InteractiveState::Radio { selected_index, .. } = s {
                Some(*selected_index)
            } else {
                None
            }
        })
        .unwrap_or(0);
    let radio_options: [&'static str; 3] = ["shaded", "wire", "solid"];
    let radio_selected = radio_options
        .get(radio_sel_idx)
        .copied()
        .unwrap_or("shaded");
    let radio = RadioGroup::new(
        ids::SHOWCASE_RADIO_MODE,
        "Mode",
        vec![
            RadioOption::new(ids::SHOWCASE_RADIO_SHADED, "shaded", "Shaded"),
            RadioOption::new(ids::SHOWCASE_RADIO_WIRE, "wire", "Wire"),
            RadioOption::new(ids::SHOWCASE_RADIO_SOLID, "solid", "Solid"),
        ],
    )
    .orientation(RadioOrientation::Segmented)
    .selected(radio_selected);
    paint_radio_group(&radio, radio_rect, scene, theme);

    let dd_rect = Rect::new(
        inner_x + radio_rect.w + Spacing::Md.px(),
        y,
        inner_w - radio_rect.w - Spacing::Md.px(),
        28.0,
    );
    hit_index.register(ids::SHOWCASE_DROPDOWN_VIEW, dd_rect);
    let dd: Dropdown<&'static str> = Dropdown::new(
        ids::SHOWCASE_DROPDOWN_VIEW,
        "View",
        vec![
            DropdownOption::new(ids::SHOWCASE_DROPDOWN_OPT_FRONT, "front", "Front"),
            DropdownOption::new(ids::SHOWCASE_DROPDOWN_OPT_SIDE, "side", "Side"),
        ],
    )
    .placeholder("View")
    .selected("front");
    paint_dropdown(&dd, dd_rect, scene, text_system, theme);
    y += 28.0 + Spacing::Md.px();

    // 6. Combobox + Checkbox.
    let cb_rect = Rect::new(inner_x, y, inner_w * 0.6, 28.0);
    hit_index.register(ids::SHOWCASE_COMBOBOX_ASSET, cb_rect);
    // Read live query + open state from the store.
    let (combo_query, combo_open, combo_state) = store
        .get(ids::SHOWCASE_COMBOBOX_ASSET)
        .and_then(|s| {
            if let crate::interaction::InteractiveState::Combobox {
                query, open, state, ..
            } = s
            {
                Some((query.as_str(), *open, *state))
            } else {
                None
            }
        })
        .unwrap_or(("sp", false, ComboboxState::Normal));
    let combo = Combobox::new(
        ids::SHOWCASE_COMBOBOX_ASSET,
        "Asset",
        vec![
            ComboboxOption::new(ids::SHOWCASE_COMBOBOX_OPT_SPIKE, "spike.png"),
            ComboboxOption::new(ids::SHOWCASE_COMBOBOX_OPT_BLOCK, "block.png"),
        ],
    )
    .query(combo_query)
    .open(combo_open)
    .state(combo_state);
    paint_combobox(&combo, cb_rect, scene, text_system, theme);
    let chk_rect = Rect::new(
        inner_x + cb_rect.w + Spacing::Md.px(),
        y + 4.0,
        inner_w - cb_rect.w - Spacing::Md.px(),
        20.0,
    );
    hit_index.register(ids::SHOWCASE_CHECKBOX_LOCK, chk_rect);
    // Read live checkbox value from the store.
    let (chk_state, chk_value) = store
        .checkbox(ids::SHOWCASE_CHECKBOX_LOCK)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Indeterminate));
    let chk = Checkbox::new(ids::SHOWCASE_CHECKBOX_LOCK, "Lock")
        .state(chk_state)
        .value(chk_value);
    paint_checkbox(&chk, chk_rect, scene, text_system, theme);
    y += 28.0 + Spacing::Md.px();

    // 7. TextInput + TextArea.
    let ti_rect = Rect::new(inner_x, y, inner_w, 30.0);
    hit_index.register(ids::SHOWCASE_TEXT_INPUT_NAME, ti_rect);
    // Read live text + state from the store.
    let (ti_text, ti_state) = store
        .get(ids::SHOWCASE_TEXT_INPUT_NAME)
        .and_then(|s| {
            if let crate::interaction::InteractiveState::TextInput { text, state, .. } = s {
                Some((text.as_str(), *state))
            } else {
                None
            }
        })
        .unwrap_or(("Player_01", TextInputState::Normal));
    let ti = TextInput::new(ids::SHOWCASE_TEXT_INPUT_NAME, "Name")
        .value(ti_text)
        .state(ti_state);
    paint_text_input(&ti, ti_rect, scene, text_system, theme);
    y += 30.0 + 4.0;
    let ta_rect = Rect::new(inner_x, y, inner_w, 50.0);
    hit_index.register(ids::SHOWCASE_TEXT_AREA_NOTES, ta_rect);
    // Read live text + state from the store (uses TextInput variant).
    let (ta_text, ta_state) = store
        .get(ids::SHOWCASE_TEXT_AREA_NOTES)
        .and_then(|s| {
            if let crate::interaction::InteractiveState::TextInput { text, state, .. } = s {
                Some((text.as_str(), *state))
            } else {
                None
            }
        })
        .unwrap_or((
            "Brush prefab — hot reloads on save.\nCollider via sprite alpha.",
            TextInputState::Normal,
        ));
    let ta = TextArea::new(ids::SHOWCASE_TEXT_AREA_NOTES, "Notes")
        .value(ta_text)
        .state(ta_state);
    paint_text_area(&ta, ta_rect, scene, text_system, theme);
    y += 50.0 + Spacing::Md.px();

    // 8. ContextMenu (static demo) + Popover (small wrap).
    let menu_rect = Rect::new(inner_x, y, inner_w * 0.5, 110.0);
    hit_index.register(ids::SHOWCASE_CTX_MENU, menu_rect);
    let menu = ContextMenu::new(
        ids::SHOWCASE_CTX_MENU,
        "File",
        vec![
            ContextMenuEntry::Item(
                ListItem::new(ids::SHOWCASE_CTX_ITEM_CUT, "Cut").value("\u{2318}X"),
            ),
            ContextMenuEntry::Item(
                ListItem::new(ids::SHOWCASE_CTX_ITEM_COPY, "Copy").value("\u{2318}C"),
            ),
            ContextMenuEntry::Separator(Divider::new(ids::SHOWCASE_CTX_DIVIDER)),
            ContextMenuEntry::Item(
                ListItem::new(ids::SHOWCASE_CTX_ITEM_DELETE, "Delete").value("Del"),
            ),
        ],
    );
    paint_context_menu(&menu, menu_rect, scene, text_system, theme, 26.0);

    let pop_rect = Rect::new(
        inner_x + menu_rect.w + Spacing::Md.px(),
        y,
        inner_w - menu_rect.w - Spacing::Md.px(),
        110.0,
    );
    hit_index.register(ids::SHOWCASE_POPOVER, pop_rect);
    let pop = Popover::new(ids::SHOWCASE_POPOVER);
    paint_popover(&pop, pop_rect, scene, theme);
    paint_text_centered(
        text_system,
        scene,
        "Popover\nsurface",
        pop_rect,
        TypeToken::Xs.px(),
        resolve(ColorToken::Text2, theme),
    );
    y += 110.0 + Spacing::Md.px();

    // 9. SectionHeader collapsible standalone + Slider vertical.
    let sh_rect = Rect::new(inner_x, y, inner_w * 0.55, 24.0);
    hit_index.register(ids::SHOWCASE_SECTION_ADVANCED, sh_rect);
    let sh = SectionHeader::new(ids::SHOWCASE_SECTION_ADVANCED, "Advanced")
        .count(7)
        .collapsible(false);
    paint_section_header(&sh, sh_rect, scene, text_system, theme);
    let vs_rect = Rect::new(inner_x + sh_rect.w + Spacing::Md.px(), y, 24.0, 96.0);
    hit_index.register(ids::SHOWCASE_SLIDER_VERTICAL, vs_rect);
    // Read live slider value + state from the store.
    let (vs_state, vs_value) = store
        .slider(ids::SHOWCASE_SLIDER_VERTICAL)
        .unwrap_or((SliderState::Hovered, 0.65));
    let mut vs = Slider::new(ids::SHOWCASE_SLIDER_VERTICAL, "Vertical")
        .accent(true)
        .orientation(crate::widget::SliderOrientation::Vertical);
    vs.set_value(vs_value);
    vs.state = vs_state;
    paint_slider(&vs, vs_rect, scene, theme);
    y += 96.0 + Spacing::Md.px();

    // 10. Modal mini paint at the showcase footer.
    let modal_h = (rect.y + rect.h - y - pad).max(0.0);
    if modal_h > 110.0 {
        let modal_rect = Rect::new(inner_x, y, inner_w, modal_h.min(170.0));
        hit_index.register(ids::SHOWCASE_MODAL_CANCEL, modal_rect);
        hit_index.register(ids::SHOWCASE_MODAL_CONFIRM, modal_rect);
        let modal = Modal::new(
            ids::SHOWCASE_MODAL_CANCEL, // id identifies the modal via its cancel button
            "Confirm delete",
            Button::new(ids::SHOWCASE_MODAL_CANCEL, "Cancel"),
            Button::new(ids::SHOWCASE_MODAL_CONFIRM, "Delete").kind(ButtonKind::Danger),
        );
        // Use a clipping-friendly viewport equal to the showcase
        // surface so the scrim doesn't paint outside.
        paint_modal(&modal, rect, modal_rect, scene, text_system, theme);
    }

    // Keep these imports alive — BlenderColorPicker + TreeView paint in
    // a dedicated extra region (bottom-right canvas), not inside the
    // main showcase panel. The let-binding suppresses dead-code lints
    // while those regions are still stubs.
    let _ = (
        TreeView::new(NodeId(0), "x", vec![TreeNode::new(NodeId(0), "x")]),
        BlenderColorPicker::new(NodeId(0), "x").channel_mode(ChannelMode::Hsv),
        ListItemState::Normal,
        DropdownState::Normal,
    );
    // BlenderColorPicker + TreeView paint in the dedicated demo
    // region attached to the canvas (bottom-right, see
    // `paint_blender_picker_demo`).
    paint_tree_view_demo(rect, scene, text_system, theme);
    paint_blender_picker_demo(layout, scene, text_system, theme, hit_index, store);
}

fn paint_tree_view_demo(
    _host: Rect,
    _scene: &mut VectorScene,
    _text: &mut TextSystem,
    _theme: Theme,
) {
    // No-op stub: TreeView demo lands when the showcase region grows
    // a third column. Kept so the widget appears in the dependency
    // graph and doesn't get pruned by a future dead-code lint.
}

pub fn paint_blender_picker_demo(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let w = 280.0_f32;
    let h = 560.0_f32;
    if layout.canvas.w < w + SHOWCASE_W + 60.0 || layout.canvas.h < h + 40.0 {
        return;
    }
    // Default-anchored to the bottom-right of the canvas; the
    // user-controlled drag handle adds an offset stored on the
    // WidgetStore. Clamp the final rect inside the viewport so a
    // bad drag can't strand the picker off-screen.
    let (dx, dy) = store.blender_picker_offset(ids::INSP_BLENDER_PICKER);
    let base_x = layout.canvas.x + layout.canvas.w - w - 12.0;
    let base_y = layout.canvas.y + layout.canvas.h - h - 12.0;
    let min_x = layout.canvas.x + 8.0;
    let max_x = layout.canvas.x + layout.canvas.w - w - 8.0;
    let min_y = layout.canvas.y + 8.0;
    let max_y = layout.canvas.y + layout.canvas.h - h - 8.0;
    let rect = Rect::new(
        (base_x + dx).clamp(min_x, max_x),
        (base_y + dy).clamp(min_y, max_y),
        w,
        h,
    );
    let cp = BlenderColorPicker::new(ids::INSP_BLENDER_PICKER, "Color");
    let sub_ids = crate::widget::BlenderSubIds {
        parent: ids::INSP_BLENDER_PICKER,
        wheel: ids::BLENDER_WHEEL,
        value_slider: ids::BLENDER_VALUE_SLIDER,
        interp_linear: ids::BLENDER_INTERP_LINEAR,
        interp_perceptual: ids::BLENDER_INTERP_PERCEPTUAL,
        channel_rgb: ids::BLENDER_CHANNEL_RGB,
        channel_hsv: ids::BLENDER_CHANNEL_HSV,
        channels: [
            ids::BLENDER_CHANNEL_0,
            ids::BLENDER_CHANNEL_1,
            ids::BLENDER_CHANNEL_2,
            ids::BLENDER_CHANNEL_3,
        ],
        channels_num: [
            ids::BLENDER_NUM_0,
            ids::BLENDER_NUM_1,
            ids::BLENDER_NUM_2,
            ids::BLENDER_NUM_3,
        ],
        hex: ids::BLENDER_HEX,
        add_swatch: ids::BLENDER_ADD_SWATCH,
        eyedropper: ids::BLENDER_EYEDROPPER,
        drag_handle: ids::BLENDER_DRAG_HANDLE,
        swatches: [
            ids::BLENDER_SWATCH_0,
            ids::BLENDER_SWATCH_1,
            ids::BLENDER_SWATCH_2,
            ids::BLENDER_SWATCH_3,
            ids::BLENDER_SWATCH_4,
            ids::BLENDER_SWATCH_5,
            ids::BLENDER_SWATCH_6,
            ids::BLENDER_SWATCH_7,
            ids::BLENDER_SWATCH_8,
            ids::BLENDER_SWATCH_9,
            ids::BLENDER_SWATCH_10,
            ids::BLENDER_SWATCH_11,
            ids::BLENDER_SWATCH_12,
            ids::BLENDER_SWATCH_13,
            ids::BLENDER_SWATCH_14,
            ids::BLENDER_SWATCH_15,
            ids::BLENDER_SWATCH_16,
            ids::BLENDER_SWATCH_17,
            ids::BLENDER_SWATCH_18,
            ids::BLENDER_SWATCH_19,
            ids::BLENDER_SWATCH_20,
            ids::BLENDER_SWATCH_21,
            ids::BLENDER_SWATCH_22,
            ids::BLENDER_SWATCH_23,
            ids::BLENDER_SWATCH_24,
            ids::BLENDER_SWATCH_25,
            ids::BLENDER_SWATCH_26,
        ],
    };
    crate::widget::paint_blender_color_picker_with_store(
        &cp,
        rect,
        &sub_ids,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ipad12_layout() -> HeroLayout {
        HeroLayout::for_viewport(Rect::new(
            0.0,
            0.0,
            super::super::HERO_VIEWPORT_W,
            super::super::HERO_VIEWPORT_H,
        ))
    }

    #[test]
    fn paint_showcase_smoke_default_theme() {
        let layout = ipad12_layout();
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        let mut hits = HitIndex::new();
        let store = WidgetStore::with_capacity(8);
        paint_components_showcase(
            &layout,
            &mut scene,
            &mut text,
            Theme::ForgeSdf,
            &mut hits,
            &store,
        );
    }

    #[test]
    fn paint_showcase_smoke_alternate_themes() {
        let layout = ipad12_layout();
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        let mut hits = HitIndex::new();
        let store = WidgetStore::with_capacity(8);
        for theme in [Theme::Sunstone, Theme::Blueprint, Theme::PaintStudio] {
            paint_components_showcase(&layout, &mut scene, &mut text, theme, &mut hits, &store);
        }
    }

    #[test]
    fn showcase_skips_when_canvas_too_small() {
        // Synthesize a very small viewport so the canvas is below
        // the showcase's required dimensions.
        let layout = HeroLayout::for_viewport(Rect::new(0.0, 0.0, 600.0, 400.0));
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        let mut hits = HitIndex::new();
        let store = WidgetStore::with_capacity(4);
        // Should not panic; canvas too small triggers early return.
        paint_components_showcase(
            &layout,
            &mut scene,
            &mut text,
            Theme::ForgeSdf,
            &mut hits,
            &store,
        );
    }
}
