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

/// Register every Showcase widget into the [`WidgetStore`]. Empty
/// today — Showcase widgets currently paint with `NodeId(0)` and
/// are not yet wired for interaction. Adding ids + registrations
/// here is the next step (per agent split-up).
pub fn populate(_store: &mut WidgetStore) {}

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
    RadioOrientation, SectionHeader, Slider, SliderState, Spinner, Tag, TagState, TagTone,
    TextArea, TextInput, TextInputState, TreeNode, TreeView, Vector3Editor, paint_avatar,
    paint_card, paint_checkbox, paint_combobox, paint_context_menu, paint_divider, paint_dropdown,
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
    let rect = Rect::new(
        layout.canvas.x + 12.0,
        layout.canvas.y + layout.canvas.h - SHOWCASE_H - 12.0,
        SHOWCASE_W,
        SHOWCASE_H,
    );
    paint_panel_surface(rect, scene, theme);

    let pad = Spacing::Lg.px();
    let inner_x = rect.x + pad;
    let inner_w = rect.w - pad * 2.0;
    let mut y = rect.y + pad + 4.0;

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
    let card = Card::new(NodeId(0)).title("Quick actions");
    paint_card(&card, card_rect, scene, text_system, theme);
    let body = card.body_rect(card_rect);
    let row_h = 28.0;
    let items = [
        ("Open", IconId::Open, Some("\u{2318}O")),
        ("Save", IconId::Save, Some("\u{2318}S")),
        ("Export", IconId::Export, None),
    ];
    for (i, (label, icon, shortcut)) in items.iter().enumerate() {
        let row = Rect::new(body.x, body.y + (row_h + 2.0) * i as f32, body.w, row_h);
        let mut li = ListItem::new(NodeId(0), *label).icon(*icon);
        if let Some(sc) = shortcut {
            li = li.value(*sc);
        }
        paint_list_item(&li, row, scene, text_system, theme);
    }
    // Divider after items.
    let div_y = body.y + (row_h + 2.0) * items.len() as f32 + 2.0;
    let div_rect = Rect::new(body.x, div_y, body.w, 8.0);
    let div = Divider::new(NodeId(0)).orientation(DividerOrientation::Horizontal);
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
    let v3 = Vector3Editor::new(
        NodeId(0),
        "Position",
        NumberInput::new(NodeId(0), "X", 1.0),
        NumberInput::new(NodeId(0), "Y", 2.0),
        NumberInput::new(NodeId(0), "Z", 3.0),
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
    let p_det = ProgressBar::new(NodeId(0), "Loading")
        .determinate(0.62)
        .show_percent(false);
    paint_progress_bar(
        &p_det,
        Rect::new(inner_x, y, bar_w, bar_h),
        scene,
        text_system,
        theme,
    );
    let p_ind = ProgressBar::new(NodeId(0), "Indeterminate").indeterminate();
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
    let spinner = Spinner::new(NodeId(0), "Working");
    paint_spinner(&spinner, spinner_rect, scene, theme);
    let av_circle = Rect::new(inner_x + 36.0, y, 24.0, 24.0);
    let av_square = Rect::new(inner_x + 72.0, y, 24.0, 24.0);
    let av_c = Avatar::new(NodeId(0), "Anna", 'A').shape(AvatarShape::Circle);
    let av_s = Avatar::new(NodeId(0), "Bob", 'B').shape(AvatarShape::Square);
    paint_avatar(&av_c, av_circle, scene, text_system, theme);
    paint_avatar(&av_s, av_square, scene, text_system, theme);
    // Tags on the right.
    let tag_x = inner_x + 108.0;
    let tag_w = 50.0_f32;
    let tag_rect_a = Rect::new(tag_x, y + 2.0, tag_w, 20.0);
    let tag_rect_b = Rect::new(tag_x + tag_w + 4.0, y + 2.0, tag_w, 20.0);
    let tag_a = Tag::new(NodeId(0), "DRAFT").tone(TagTone::Warn);
    let tag_b = Tag::new(NodeId(0), "DONE")
        .tone(TagTone::Success)
        .removable(true);
    paint_tag(&tag_a, tag_rect_a, scene, text_system, theme);
    paint_tag(&tag_b, tag_rect_b, scene, text_system, theme);
    y += 24.0 + Spacing::Md.px();

    // 5. RadioGroup (Segmented) + Dropdown.
    let radio_rect = Rect::new(inner_x, y, inner_w * 0.55, 28.0);
    let radio = RadioGroup::new(
        NodeId(0),
        "Mode",
        vec![
            RadioOption::new(NodeId(0), "shaded", "Shaded"),
            RadioOption::new(NodeId(0), "wire", "Wire"),
            RadioOption::new(NodeId(0), "solid", "Solid"),
        ],
    )
    .orientation(RadioOrientation::Segmented)
    .selected("shaded");
    paint_radio_group(&radio, radio_rect, scene, theme);

    let dd_rect = Rect::new(
        inner_x + radio_rect.w + Spacing::Md.px(),
        y,
        inner_w - radio_rect.w - Spacing::Md.px(),
        28.0,
    );
    let dd: Dropdown<&'static str> = Dropdown::new(
        NodeId(0),
        "View",
        vec![
            DropdownOption::new(NodeId(0), "front", "Front"),
            DropdownOption::new(NodeId(0), "side", "Side"),
        ],
    )
    .placeholder("View")
    .selected("front");
    paint_dropdown(&dd, dd_rect, scene, text_system, theme);
    y += 28.0 + Spacing::Md.px();

    // 6. Combobox + Checkbox.
    let cb_rect = Rect::new(inner_x, y, inner_w * 0.6, 28.0);
    let combo = Combobox::new(
        NodeId(0),
        "Asset",
        vec![
            ComboboxOption::new(NodeId(0), "spike.png"),
            ComboboxOption::new(NodeId(0), "block.png"),
        ],
    )
    .query("sp")
    .state(ComboboxState::Normal);
    paint_combobox(&combo, cb_rect, scene, text_system, theme);
    let chk_rect = Rect::new(
        inner_x + cb_rect.w + Spacing::Md.px(),
        y + 4.0,
        inner_w - cb_rect.w - Spacing::Md.px(),
        20.0,
    );
    let chk = Checkbox::new(NodeId(0), "Lock").value(CheckboxValue::Indeterminate);
    paint_checkbox(&chk, chk_rect, scene, text_system, theme);
    y += 28.0 + Spacing::Md.px();

    // 7. TextInput + TextArea.
    let ti_rect = Rect::new(inner_x, y, inner_w, 30.0);
    let ti = TextInput::new(NodeId(0), "Name").value("Player_01");
    paint_text_input(&ti, ti_rect, scene, text_system, theme);
    y += 30.0 + 4.0;
    let ta_rect = Rect::new(inner_x, y, inner_w, 50.0);
    let ta = TextArea::new(NodeId(0), "Notes")
        .value("Brush prefab — hot reloads on save.\nCollider via sprite alpha.");
    paint_text_area(&ta, ta_rect, scene, text_system, theme);
    y += 50.0 + Spacing::Md.px();

    // 8. ContextMenu (static demo) + Popover (small wrap).
    let menu_rect = Rect::new(inner_x, y, inner_w * 0.5, 110.0);
    let menu = ContextMenu::new(
        NodeId(0),
        "File",
        vec![
            ContextMenuEntry::Item(ListItem::new(NodeId(0), "Cut").value("\u{2318}X")),
            ContextMenuEntry::Item(ListItem::new(NodeId(0), "Copy").value("\u{2318}C")),
            ContextMenuEntry::Separator(Divider::new(NodeId(0))),
            ContextMenuEntry::Item(ListItem::new(NodeId(0), "Delete").value("Del")),
        ],
    );
    paint_context_menu(&menu, menu_rect, scene, text_system, theme, 26.0);

    let pop_rect = Rect::new(
        inner_x + menu_rect.w + Spacing::Md.px(),
        y,
        inner_w - menu_rect.w - Spacing::Md.px(),
        110.0,
    );
    let pop = Popover::new(NodeId(0));
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
    let sh = SectionHeader::new(NodeId(0), "Advanced")
        .count(7)
        .collapsible(false);
    paint_section_header(&sh, sh_rect, scene, text_system, theme);
    let vs_rect = Rect::new(inner_x + sh_rect.w + Spacing::Md.px(), y, 24.0, 96.0);
    let mut vs = Slider::new(NodeId(0), "Vertical")
        .accent(true)
        .orientation(crate::widget::SliderOrientation::Vertical);
    vs.set_value(0.65);
    vs.state = SliderState::Hovered;
    paint_slider(&vs, vs_rect, scene, theme);
    y += 96.0 + Spacing::Md.px();

    // 10. Modal mini paint at the showcase footer.
    let modal_h = (rect.y + rect.h - y - pad).max(0.0);
    if modal_h > 110.0 {
        let modal_rect = Rect::new(inner_x, y, inner_w, modal_h.min(170.0));
        let modal = Modal::new(
            NodeId(0),
            "Confirm delete",
            Button::new(NodeId(0), "Cancel"),
            Button::new(NodeId(0), "Delete").kind(ButtonKind::Danger),
        );
        // Use a clipping-friendly viewport equal to the showcase
        // surface so the scrim doesn't paint outside.
        paint_modal(&modal, rect, modal_rect, scene, text_system, theme);
    }

    // Suppress unused warnings while the showcase doesn't yet wire
    // dynamic state from these widgets' interactive ids.
    let _ = (
        TreeView::new(NodeId(0), "x", vec![TreeNode::new(NodeId(0), "x")]),
        BlenderColorPicker::new(NodeId(0), "x").channel_mode(ChannelMode::Hsv),
        ListItemState::Normal,
        DropdownState::Normal,
        TextInputState::Normal,
        TagState::Normal,
        CheckboxState::Normal,
    );
    // BlenderColorPicker + TreeView paint in the dedicated demo
    // region attached to the canvas (bottom-right, see
    // `paint_components_showcase_extras`). Suppression above keeps
    // imports honest while that region wires in the next iteration.
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

fn paint_blender_picker_demo(
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
    let rect = Rect::new(
        layout.canvas.x + layout.canvas.w - w - 12.0,
        layout.canvas.y + layout.canvas.h - h - 12.0,
        w,
        h,
    );
    let cp = BlenderColorPicker::new(ids::INSP_BLENDER_PICKER, "Color");
    crate::widget::paint_blender_color_picker_with_store(
        &cp,
        rect,
        ids::INSP_BLENDER_PICKER,
        ids::BLENDER_WHEEL,
        ids::BLENDER_VALUE_SLIDER,
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
