//! Inspector painter — title + sub + sections + per-field rows.

use super::HeroLayout;
use super::HeroSelection;
use super::fixture;
use super::ids;
use super::style::{
    FIELD_GAP, FIELD_ROW_H, PANEL_HEAD_PAD, SECTION_GAP, SECTION_HEAD_H, paint_panel_surface,
};
use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, WidgetStore};
use crate::paint::{
    fill_rounded_rect, paint_icon, paint_text, paint_text_centered, rect_to_vello, resolve,
    stroke_rounded_rect,
};
use crate::widget::{SectionHeader, Slider, SliderState, paint_section_header, paint_slider};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Circle, Fill, Point, VectorScene};

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

    let mut y = div_y + Spacing::Md.px();
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
            let val_w = 56.0_f32;
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
            fill_rounded_rect(
                scene,
                val_rect,
                Radius::Xs.px(),
                resolve(ColorToken::Bg3, theme),
            );
            paint_text_centered(
                text_system,
                scene,
                display,
                val_rect,
                TypeToken::Xs.px(),
                resolve(ColorToken::Text1, theme),
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
