//! Hierarchy panel painter — header + add button + entity rows.

use super::HeroLayout;
use super::HeroSelection;
use super::fixture;
use super::ids;
use super::style::{HIER_ROW_H, PANEL_HEAD_PAD, paint_panel_surface};
use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect};
use crate::widget::{ButtonState, Tag, TagState, TagTone, paint_tag};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Circle, Color as VelloColor, Fill, Point, VectorScene};

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
    for id in [
        ids::HIER_PLAYER,
        ids::HIER_SPRITE_IDLE,
        ids::HIER_COLLIDER_BOX,
        ids::HIER_SCRIPT_PLAYER,
        ids::HIER_RIGIDBODY,
        ids::HIER_TILEMAP_GROUND,
        ids::HIER_TILEMAP_DECOR,
        ids::HIER_SLIME_01,
        ids::HIER_SLIME_02,
        ids::HIER_TRIGGER_ZONE_A,
        ids::HIER_AMBIENT_LIGHT,
        ids::HIER_MAIN_CAMERA,
    ] {
        store.register(id, InteractiveState::Plain);
    }
}

/// Apply a [`WidgetEvent`] against hierarchy widgets. A click on an
/// entity row updates `selection`; everything else is ignored.
/// Returns true iff the event was consumed.
pub fn apply_event(
    _store: &mut WidgetStore,
    selection: &mut Option<HeroSelection>,
    event: WidgetEvent,
) -> bool {
    if let WidgetEvent::Click(id) = event
        && let Some(label) = ids::hierarchy_label_for_id(id)
    {
        *selection = Some(HeroSelection {
            label: label.into(),
            kind: ids::hierarchy_kind_for_label(label).into(),
            world_pos: (0.0, 0.0),
        });
        return true;
    }
    false
}

pub fn paint_hierarchy(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let rect = layout.hierarchy;
    paint_panel_surface(rect, scene, theme);

    let title_y = rect.y + 18.0;
    paint_text(
        text_system,
        scene,
        "Hierarchy",
        rect.x + PANEL_HEAD_PAD,
        title_y,
        TypeToken::Md.px(),
        rect.w - PANEL_HEAD_PAD * 2.0 - 40.0,
        resolve(ColorToken::Text1, theme),
    );
    let (entities, components) = fixture::hierarchy_counts();
    let counts = format!("{entities} entities \u{00b7} {components} components");
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

    let body_top = title_y + TypeToken::Md.px() + TypeToken::Xs.px() + 18.0;
    let body_pad = 8.0_f32;
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
    for mut entity in fixture::hierarchy() {
        let row_rect = Rect::new(rect.x + body_pad, y, row_w, HIER_ROW_H);
        if let Some(ref sel_label) = selected_label {
            entity.selected = entity.name == *sel_label;
        }
        if let Some(id) = ids::hierarchy_id(&entity.name) {
            hit_index.register(id, row_rect);
        }
        paint_hierarchy_row(&entity, row_rect, scene, text_system, theme);
        y += HIER_ROW_H + 2.0;
    }
    scene.pop_layer();
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

fn paint_hierarchy_row(
    entity: &fixture::HierarchyEntity,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
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
    let icon_w = 16.0_f32;
    let icon_x = rect.x + pad + indent_w;
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
    let visibility_color = if entity.visible {
        ColorToken::Success
    } else {
        ColorToken::Border
    };
    let vis_r = 5.0_f32;
    let vis_cx = right_x - vis_r;
    let vis_cy = rect.y + rect.h * 0.5;
    let vis_dot = Circle::new(Point::new(vis_cx as f64, vis_cy as f64), vis_r as f64);
    scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(resolve(visibility_color, theme)),
        None,
        &vis_dot,
    );
    right_x -= vis_r * 2.0 + 8.0;
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
