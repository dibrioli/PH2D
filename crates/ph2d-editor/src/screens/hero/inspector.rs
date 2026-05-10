//! Inspector painter — title + sub + sections + per-field rows.

use super::HeroLayout;
use super::HeroSelection;
use super::fixture;
use super::ids;
use super::style::{
    FIELD_GAP, FIELD_ROW_H, PANEL_HEAD_PAD, SECTION_GAP, SECTION_HEAD_H, paint_panel_surface,
};
use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{
    fill_rounded_rect, paint_icon, paint_text, paint_text_centered, rect_to_vello, resolve,
    stroke_rounded_rect,
};
use crate::widget::{
    ButtonState, ChannelMode, Checkbox, CheckboxState, CheckboxValue, ColorSwatch, DropdownState,
    InterpolationMode, NumberInput, SectionHeader, Slider, SliderOrientation, SliderState,
    SwatchSize, TabItem, Tabs, TabsVariant, TextInputState, Toggle, ToggleState, paint_checkbox,
    paint_color_swatch, paint_section_header, paint_slider, paint_tabs, paint_toggle,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ColorValue, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Circle, Fill, Point, VectorScene};

/// Register every Inspector widget into the [`WidgetStore`]:
/// section tabs, sliders, linked NumberInputs, dropdowns, toggles,
/// checkbox, swatch, and the BlenderColorPicker (with sub-rect
/// hit shims for the wheel + value slider).
pub fn populate(store: &mut WidgetStore) {
    // Inspector tabs (Properties / Layers / Materials).
    for id in [
        ids::INSP_TAB_PROPS,
        ids::INSP_TAB_LAYERS,
        ids::INSP_TAB_MATERIALS,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    if let Some(InteractiveState::Button { state }) = store.get_mut(ids::INSP_TAB_PROPS) {
        *state = ButtonState::Pressed;
    }

    // Field sliders (normalized 0..=1).
    for (id, value) in [
        (ids::INSP_MOVE_SPEED, 0.62),
        (ids::INSP_JUMP_HEIGHT, 0.30),
        (ids::INSP_FRICTION, 0.08),
        (ids::INSP_DAMPING, 0.48),
        (ids::INSP_CAM_YAW, 0.57),
        (ids::INSP_CAM_PITCH, 0.0),
    ] {
        store.register(
            id,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value,
                orientation: SliderOrientation::Horizontal,
            },
        );
    }

    // Debug select dropdown.
    store.register(
        ids::INSP_DEBUG_SELECT,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(0),
        },
    );

    // NumberInput chips linked to the field sliders. Initial values
    // mirror the slider's value scaled to display range — the
    // dispatcher keeps them in sync as the user drags.
    for (id, value) in [
        (ids::INSP_NUM_MOVE_SPEED, 160.0_f64),
        (ids::INSP_NUM_JUMP_HEIGHT, 200.0),
        (ids::INSP_NUM_FRICTION, 0.0010),
        (ids::INSP_NUM_DAMPING, 0.70),
        (ids::INSP_NUM_CAM_YAW, 0.57),
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: crate::interaction::format_number(value),
                caret: 0,
                last_committed: value,
            },
        );
    }

    // Slider × NumberInput two-way bindings (Phase 2).
    for (slider_id, number_id) in [
        (ids::INSP_MOVE_SPEED, ids::INSP_NUM_MOVE_SPEED),
        (ids::INSP_JUMP_HEIGHT, ids::INSP_NUM_JUMP_HEIGHT),
        (ids::INSP_FRICTION, ids::INSP_NUM_FRICTION),
        (ids::INSP_DAMPING, ids::INSP_NUM_DAMPING),
        (ids::INSP_CAM_YAW, ids::INSP_NUM_CAM_YAW),
    ] {
        store.link_slider_number(slider_id, number_id);
    }

    // Hot-reload checkbox + Snap-grid toggle.
    store.register(
        ids::INSP_HOT_RELOAD_CHECK,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Checked,
        },
    );
    store.register(
        ids::INSP_SNAP_GRID_TOGGLE,
        InteractiveState::Toggle {
            state: ToggleState::Normal,
            on: true,
        },
    );

    // Tint swatch — clicking selects it (Plain) and the
    // BlenderColorPicker in the demo region reads its rgba.
    store.register(ids::INSP_TINT_SWATCH, InteractiveState::Plain);

    // BlenderColorPicker retained state — wheel + value-slider hit
    // shims route their picks back into the parent picker.
    store.register(
        ids::INSP_BLENDER_PICKER,
        InteractiveState::BlenderPicker {
            value: ColorValue::from_rgba8(231, 231, 231, 255),
            channel_mode: ChannelMode::Rgb,
            interpolation: InterpolationMode::Perceptual,
            active_palette: 0,
        },
    );
    store.register(
        ids::BLENDER_WHEEL,
        InteractiveState::BlenderHit {
            parent: ids::INSP_BLENDER_PICKER,
            kind: crate::interaction::BlenderHitKind::Wheel,
        },
    );
    store.register(
        ids::BLENDER_VALUE_SLIDER,
        InteractiveState::BlenderHit {
            parent: ids::INSP_BLENDER_PICKER,
            kind: crate::interaction::BlenderHitKind::ValueSlider,
        },
    );

    // Channel slider shims (4 rows: index 0 = R/H, 1 = G/S, 2 = B/V, 3 = A).
    for (id, idx) in [
        (ids::BLENDER_CHANNEL_0, 0u8),
        (ids::BLENDER_CHANNEL_1, 1),
        (ids::BLENDER_CHANNEL_2, 2),
        (ids::BLENDER_CHANNEL_3, 3),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::INSP_BLENDER_PICKER,
                kind: crate::interaction::BlenderHitKind::ChannelSlider(idx),
            },
        );
    }

    // Hex field as TextInput — the buffer is pre-allocated with the
    // initial hex string matching the picker's default value.
    store.register(
        ids::BLENDER_HEX,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: "#E7E7E7FF".to_string(),
            caret: 9,
        },
    );

    // Segmented toggle shims.
    store.register(
        ids::BLENDER_INTERP_LINEAR,
        InteractiveState::BlenderHit {
            parent: ids::INSP_BLENDER_PICKER,
            kind: crate::interaction::BlenderHitKind::InterpolationLinear,
        },
    );
    store.register(
        ids::BLENDER_INTERP_PERCEPTUAL,
        InteractiveState::BlenderHit {
            parent: ids::INSP_BLENDER_PICKER,
            kind: crate::interaction::BlenderHitKind::InterpolationPerceptual,
        },
    );
    store.register(
        ids::BLENDER_CHANNEL_RGB,
        InteractiveState::BlenderHit {
            parent: ids::INSP_BLENDER_PICKER,
            kind: crate::interaction::BlenderHitKind::ChannelRgb,
        },
    );
    store.register(
        ids::BLENDER_CHANNEL_HSV,
        InteractiveState::BlenderHit {
            parent: ids::INSP_BLENDER_PICKER,
            kind: crate::interaction::BlenderHitKind::ChannelHsv,
        },
    );

    // Palette swatch shims — default_palette has 12 swatches (0..11).
    for (id, swatch_idx) in [
        (ids::BLENDER_SWATCH_0, 0u8),
        (ids::BLENDER_SWATCH_1, 1),
        (ids::BLENDER_SWATCH_2, 2),
        (ids::BLENDER_SWATCH_3, 3),
        (ids::BLENDER_SWATCH_4, 4),
        (ids::BLENDER_SWATCH_5, 5),
        (ids::BLENDER_SWATCH_6, 6),
        (ids::BLENDER_SWATCH_7, 7),
        (ids::BLENDER_SWATCH_8, 8),
        (ids::BLENDER_SWATCH_9, 9),
        (ids::BLENDER_SWATCH_10, 10),
        (ids::BLENDER_SWATCH_11, 11),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::INSP_BLENDER_PICKER,
                kind: crate::interaction::BlenderHitKind::PaletteSwatch(swatch_idx),
            },
        );
    }
}

/// Apply a [`WidgetEvent`] against Inspector widgets. Returns true
/// iff the event was consumed. Stub for now — tab switching, field
/// commits to the simulation, etc. land in follow-up PRs.
pub fn apply_event(_store: &mut WidgetStore, _event: WidgetEvent) -> bool {
    false
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

    let title = selection
        .map(|s| s.label.as_str())
        .unwrap_or("(no selection)");
    let sub = "prefab.player.idle";

    let title_y = rect.y + 18.0;
    paint_text(
        text_system,
        scene,
        title,
        rect.x + PANEL_HEAD_PAD,
        title_y,
        TypeToken::Md.px(),
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text1, theme),
    );
    paint_text(
        text_system,
        scene,
        sub,
        rect.x + PANEL_HEAD_PAD,
        title_y + TypeToken::Md.px() + 4.0,
        TypeToken::Xs.px(),
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text3, theme),
    );

    let div_y = title_y + TypeToken::Md.px() + TypeToken::Xs.px() + 16.0;
    let div = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        div_y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        1.0,
    );
    scene.fill_rect(rect_to_vello(div), resolve(ColorToken::Border, theme));

    // Inspector Tabs (Phase 1 polish): Properties / Layers / Materials.
    let tabs_h = 28.0_f32;
    let tabs_y = div_y + Spacing::Md.px();
    let tabs_rect = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        tabs_y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        tabs_h,
    );
    paint_inspector_tabs(tabs_rect, scene, text_system, theme, hit_index, store);

    let mut y = tabs_y + tabs_h + Spacing::Md.px();
    let body_pad = 10.0_f32;
    for section in fixture::inspector_sections() {
        let header_rect = Rect::new(
            rect.x + body_pad,
            y,
            rect.w - body_pad * 2.0,
            SECTION_HEAD_H,
        );
        let mut header = SectionHeader::new(NodeId(0), section.label.clone()).count(section.count);
        if let Some(open) = section.collapsible {
            header = header.collapsible(open);
        }
        paint_section_header(&header, header_rect, scene, text_system, theme);
        y += SECTION_HEAD_H;
        if matches!(section.collapsible, Some(false)) {
            y += SECTION_GAP;
            continue;
        }
        for field in &section.fields {
            if y + FIELD_ROW_H * 2.0 > rect.y + rect.h {
                return;
            }
            let field_id = ids::inspector_field_id(&field.label);
            paint_inspector_field(
                field,
                field_id,
                rect.x + body_pad,
                rect.w - body_pad * 2.0,
                y,
                scene,
                text_system,
                theme,
                hit_index,
                store,
            );
            y += FIELD_ROW_H * 2.0 + FIELD_GAP;
        }
        y += SECTION_GAP;
    }

    // Behavior section (Phase 1 polish): Checkbox + Toggle + ColorSwatch.
    if y + 100.0 < rect.y + rect.h {
        paint_inspector_behavior_section(
            rect.x + body_pad,
            rect.w - body_pad * 2.0,
            y,
            scene,
            text_system,
            theme,
            hit_index,
            store,
        );
    }
}

/// Map a slider's NodeId to its sibling NumberInput NodeId
/// (Phase 1 polish: sliders carry a linked numeric chip).
fn sibling_number_id(slider_id: Option<NodeId>) -> Option<NodeId> {
    let slider_id = slider_id?;
    Some(match slider_id {
        x if x == ids::INSP_MOVE_SPEED => ids::INSP_NUM_MOVE_SPEED,
        x if x == ids::INSP_JUMP_HEIGHT => ids::INSP_NUM_JUMP_HEIGHT,
        x if x == ids::INSP_FRICTION => ids::INSP_NUM_FRICTION,
        x if x == ids::INSP_DAMPING => ids::INSP_NUM_DAMPING,
        x if x == ids::INSP_CAM_YAW => ids::INSP_NUM_CAM_YAW,
        _ => return None,
    })
}

fn paint_inspector_tabs(
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let tabs = Tabs::new(
        ids::INSP_TAB_PROPS,
        "Inspector tabs",
        vec![
            TabItem::new(ids::INSP_TAB_PROPS, "Properties"),
            TabItem::new(ids::INSP_TAB_LAYERS, "Layers"),
            TabItem::new(ids::INSP_TAB_MATERIALS, "Materials"),
        ],
    )
    .variant(TabsVariant::Segmented)
    .selected(active_inspector_tab_index(store));
    paint_tabs(&tabs, rect, scene, text_system, theme);
    // Register per-tab hit rects (3 evenly split).
    let count = tabs.items.len() as f32;
    let tab_w = rect.w / count;
    for (i, item) in tabs.items.iter().enumerate() {
        let r = Rect::new(rect.x + tab_w * i as f32, rect.y, tab_w, rect.h);
        hit_index.register(item.id, r);
    }
}

/// Determine which tab id is currently the active one by reading the
/// per-tab Button state from the store. Falls back to Properties.
fn active_inspector_tab_index(store: &WidgetStore) -> usize {
    use crate::widget::ButtonState;
    let candidates = [
        ids::INSP_TAB_PROPS,
        ids::INSP_TAB_LAYERS,
        ids::INSP_TAB_MATERIALS,
    ];
    for (i, id) in candidates.iter().enumerate() {
        if matches!(store.button_state(*id), Some(ButtonState::Pressed)) {
            return i;
        }
    }
    0
}

#[allow(clippy::too_many_arguments)]
fn paint_inspector_behavior_section(
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let head_rect = Rect::new(x, y, w, FIELD_ROW_H);
    let header = SectionHeader::new(NodeId(0), "Behavior".to_string()).count(3);
    paint_section_header(&header, head_rect, scene, text_system, theme);
    let mut row_y = y + FIELD_ROW_H + 4.0;

    // Checkbox row.
    let cb_state = store
        .checkbox(ids::INSP_HOT_RELOAD_CHECK)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Checked));
    let cb_rect = Rect::new(x, row_y, w, 20.0);
    hit_index.register(ids::INSP_HOT_RELOAD_CHECK, cb_rect);
    let cb = Checkbox::new(ids::INSP_HOT_RELOAD_CHECK, "Hot reload on save")
        .state(cb_state.0)
        .value(cb_state.1);
    paint_checkbox(&cb, cb_rect, scene, text_system, theme);
    row_y += 24.0;

    // Toggle row.
    let (tg_state, tg_on) = store
        .toggle(ids::INSP_SNAP_GRID_TOGGLE)
        .unwrap_or((ToggleState::Normal, true));
    let toggle_w = 44.0;
    let toggle_rect = Rect::new(x + w - toggle_w, row_y, toggle_w, 20.0);
    hit_index.register(ids::INSP_SNAP_GRID_TOGGLE, toggle_rect);
    let toggle = Toggle::new(ids::INSP_SNAP_GRID_TOGGLE, "Snap to grid")
        .state(tg_state)
        .on(tg_on);
    paint_toggle(&toggle, toggle_rect, scene, theme);
    paint_text(
        text_system,
        scene,
        "Snap to grid",
        x,
        row_y + (20.0 - TypeToken::Xs.px()) * 0.5,
        TypeToken::Xs.px(),
        w - toggle_w - Spacing::Md.px(),
        resolve(ColorToken::Text2, theme),
    );
    row_y += 24.0;

    // ColorSwatch row.
    paint_text(
        text_system,
        scene,
        "Tint",
        x,
        row_y + (20.0 - TypeToken::Xs.px()) * 0.5,
        TypeToken::Xs.px(),
        80.0,
        resolve(ColorToken::Text2, theme),
    );
    let sw_rect = Rect::new(x + w - 32.0, row_y - 4.0, 32.0, 28.0);
    hit_index.register(ids::INSP_TINT_SWATCH, sw_rect);
    let mut tint = ColorSwatch::new(ids::INSP_TINT_SWATCH, "Tint", [120, 60, 200, 255]);
    tint.size = SwatchSize::Md;
    paint_color_swatch(&tint, sw_rect, scene, theme);
}

#[allow(clippy::too_many_arguments)]
fn paint_inspector_field(
    field: &fixture::InspectorField,
    field_id: Option<NodeId>,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    use fixture::InspectorFieldKind;
    let head_rect = Rect::new(x, y, w, FIELD_ROW_H);
    let dot_r = 3.5;
    let dot_cx = x + dot_r + 4.0;
    let dot_cy = head_rect.y + head_rect.h * 0.5;
    let dot = Circle::new(Point::new(dot_cx as f64, dot_cy as f64), dot_r as f64);
    scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(resolve(ColorToken::Accent, theme)),
        None,
        &dot,
    );
    let label_x = dot_cx + dot_r + Spacing::Md.px();
    let label_y = head_rect.y + (head_rect.h - TypeToken::Xs.px()) * 0.5;
    paint_text(
        text_system,
        scene,
        &field.label,
        label_x,
        label_y,
        TypeToken::Xs.px(),
        head_rect.x + head_rect.w - label_x,
        resolve(ColorToken::Text2, theme),
    );

    let body_y = y + FIELD_ROW_H;
    let body_rect = Rect::new(x, body_y, w, FIELD_ROW_H);
    match &field.kind {
        InspectorFieldKind::Slider { value, display } => {
            let val_w = 64.0_f32;
            let val_rect = Rect::new(
                body_rect.x + body_rect.w - val_w,
                body_rect.y,
                val_w,
                body_rect.h,
            );
            let slider_rect = Rect::new(
                body_rect.x,
                body_rect.y,
                body_rect.w - val_w - Spacing::Md.px(),
                body_rect.h,
            );
            let id = field_id.unwrap_or(NodeId(0));
            let (live_state, live_value) = field_id
                .and_then(|i| store.slider(i))
                .unwrap_or((SliderState::Normal, *value));
            if let Some(i) = field_id {
                hit_index.register(i, slider_rect);
            }
            let mut s = Slider::new(id, &field.label).accent(true);
            s.set_value(live_value);
            s.state = live_state;
            paint_slider(&s, slider_rect, scene, theme);
            // NumberInput-style value chip linked to the slider via
            // a sibling NodeId. The dispatcher keeps both in sync;
            // for now we paint a NumberInput stub showing either the
            // store-backed numeric value (if a sibling id exists) or
            // the fixture's display string as a fallback.
            let num_id = sibling_number_id(field_id);
            let (num_state, num_value, num_buf, num_caret) = num_id
                .and_then(|i| {
                    store
                        .number_input(i)
                        .map(|(s, v, b, c)| (s, v, Some(b), Some(c)))
                })
                .unwrap_or_else(|| {
                    (
                        crate::widget::TextInputState::Normal,
                        display.parse::<f64>().unwrap_or(*value as f64),
                        None,
                        None,
                    )
                });
            if let Some(i) = num_id {
                hit_index.register(i, val_rect);
            }
            let mut ni = NumberInput::new(num_id.unwrap_or(NodeId(0)), &field.label, num_value);
            ni.state = num_state;
            crate::widget::paint_number_input_with_buffer(
                &ni,
                num_buf,
                num_caret.unwrap_or(0),
                val_rect,
                scene,
                text_system,
                theme,
            );
        }
        InspectorFieldKind::Select { current } => {
            let is_open = field_id
                .and_then(|i| match store.get(i) {
                    Some(InteractiveState::Dropdown { open, .. }) => Some(*open),
                    _ => None,
                })
                .unwrap_or(false);
            if let Some(i) = field_id {
                hit_index.register(i, body_rect);
            }
            let border = if is_open {
                ColorToken::Accent
            } else {
                ColorToken::Border
            };
            fill_rounded_rect(
                scene,
                body_rect,
                Radius::Sm.px(),
                resolve(ColorToken::Bg3, theme),
            );
            stroke_rounded_rect(
                scene,
                body_rect,
                Radius::Sm.px(),
                if is_open { 2.0 } else { 1.0 },
                resolve(border, theme),
            );
            paint_text(
                text_system,
                scene,
                current,
                body_rect.x + Spacing::Lg.px(),
                body_rect.y + (body_rect.h - TypeToken::Xs.px()) * 0.5,
                TypeToken::Xs.px(),
                body_rect.w - Spacing::Lg.px() * 2.0 - 24.0,
                resolve(ColorToken::Text1, theme),
            );
            let chev_rect = Rect::new(
                body_rect.x + body_rect.w - Spacing::Lg.px() - 16.0,
                body_rect.y + (body_rect.h - 16.0) * 0.5,
                16.0,
                16.0,
            );
            let chev = if is_open {
                IconId::ChevronUp
            } else {
                IconId::ChevronDown
            };
            paint_icon(
                scene,
                chev,
                chev_rect,
                resolve(ColorToken::Text3, theme),
                1.5,
            );
        }
        InspectorFieldKind::Linked { source } => {
            fill_rounded_rect(
                scene,
                body_rect,
                Radius::Sm.px(),
                resolve(ColorToken::AccentSoft, theme),
            );
            paint_text(
                text_system,
                scene,
                source,
                body_rect.x + Spacing::Lg.px(),
                body_rect.y + (body_rect.h - TypeToken::Xs.px()) * 0.5,
                TypeToken::Xs.px(),
                body_rect.w - Spacing::Lg.px() * 2.0,
                resolve(ColorToken::Accent, theme),
            );
        }
        InspectorFieldKind::LinkedSlider { value, display } => {
            let mut s = Slider::new(NodeId(0), &field.label)
                .accent(true)
                .state(SliderState::Normal);
            s.set_value(*value);
            let val_w = 56.0_f32;
            let slider_rect = Rect::new(
                body_rect.x,
                body_rect.y,
                body_rect.w - val_w - Spacing::Md.px(),
                body_rect.h,
            );
            paint_slider(&s, slider_rect, scene, theme);
            let val_rect = Rect::new(
                body_rect.x + body_rect.w - val_w,
                body_rect.y,
                val_w,
                body_rect.h,
            );
            fill_rounded_rect(
                scene,
                val_rect,
                Radius::Xs.px(),
                resolve(ColorToken::Bg3, theme),
            );
            if !display.is_empty() {
                paint_text_centered(
                    text_system,
                    scene,
                    display,
                    val_rect,
                    TypeToken::Xs.px(),
                    resolve(ColorToken::Text1, theme),
                );
            }
        }
    }
}
