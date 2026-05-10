//! TopBar painter — 5 pill clusters + centered wordmark.

use super::HeroLayout;
use super::fixture;
use super::ids;
use super::style::icon_button_fg;
use crate::icons::IconId;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{
    fill_rounded_rect, paint_icon, paint_text, paint_text_centered, resolve, stroke_rounded_rect,
};
use crate::widget::{ButtonState, PILL_PADDING_PX};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Circle, Fill, Point, VectorScene};

pub fn paint_top_bar(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let clusters = fixture::topbar_clusters();
    let row_h = layout.top_bar.h;
    let mut x = layout.top_bar.x;
    let gap = Spacing::Md.px();
    let split = 4.min(clusters.len());
    for (id, cluster) in &clusters[..split] {
        let rect = Rect::new(x, layout.top_bar.y, cluster_width(cluster), row_h);
        paint_top_bar_cluster(
            *id,
            cluster,
            rect,
            scene,
            text_system,
            theme,
            hit_index,
            store,
        );
        x = rect.x + rect.w + gap;
    }
    let right_clusters = &clusters[split..];
    let mut right_w = 0.0_f32;
    for (_, c) in right_clusters {
        right_w += cluster_width(c) + gap;
    }
    let right_x = layout.top_bar.x + layout.top_bar.w - right_w + gap.max(0.0);
    let wordmark_rect = Rect::new(x, layout.top_bar.y, (right_x - x).max(0.0), row_h);
    paint_text_centered(
        text_system,
        scene,
        "PH2D \u{00b7} EDITOR",
        wordmark_rect,
        TypeToken::Sm.px(),
        resolve(ColorToken::Text3, theme),
    );
    let mut rx = right_x;
    for (id, cluster) in right_clusters {
        let rect = Rect::new(rx, layout.top_bar.y, cluster_width(cluster), row_h);
        paint_top_bar_cluster(
            *id,
            cluster,
            rect,
            scene,
            text_system,
            theme,
            hit_index,
            store,
        );
        rx = rect.x + rect.w + gap;
    }
}

fn cluster_width(cluster: &fixture::TopBarCluster) -> f32 {
    use fixture::TopBarCluster;
    match cluster {
        TopBarCluster::Theme { .. } => 132.0,
        TopBarCluster::Single { .. } => 40.0 + PILL_PADDING_PX * 2.0,
        TopBarCluster::Project { .. } => 156.0,
        TopBarCluster::Play => 92.0,
        TopBarCluster::Right => 132.0,
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_top_bar_cluster(
    id: NodeId,
    cluster: &fixture::TopBarCluster,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    use fixture::TopBarCluster;
    let radius = Radius::Xl.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
    let pad_x = Spacing::Md.px();
    let icon_w = 18.0;
    let font = TypeToken::Sm.px();
    match cluster {
        TopBarCluster::Theme { label } => {
            let mut cx = rect.x + pad_x + 4.0;
            let cy = rect.y + rect.h * 0.5;
            for (i, token) in [ColorToken::Accent, ColorToken::AccentSoft]
                .iter()
                .enumerate()
            {
                let dot = Circle::new(Point::new(cx as f64, cy as f64), 4.0);
                scene.inner_mut().fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(resolve(*token, theme)),
                    None,
                    &dot,
                );
                cx += if i == 0 { 10.0 } else { 0.0 };
            }
            let label_x = rect.x + pad_x + 28.0;
            let label_y = rect.y + (rect.h - font) * 0.5;
            paint_text(
                text_system,
                scene,
                label,
                label_x,
                label_y,
                font,
                rect.w - (label_x - rect.x) - 24.0,
                resolve(ColorToken::Text1, theme),
            );
            let chev_rect = Rect::new(
                rect.x + rect.w - pad_x - icon_w,
                rect.y + (rect.h - icon_w) * 0.5,
                icon_w,
                icon_w,
            );
            paint_icon(
                scene,
                IconId::ChevronDown,
                chev_rect,
                resolve(ColorToken::Text3, theme),
                1.5,
            );
        }
        TopBarCluster::Single { icon, .. } => {
            hit_index.register(id, rect);
            let state = store.button_state(id).unwrap_or(ButtonState::Normal);
            let chip = Rect::new(
                rect.x + (rect.w - 32.0) * 0.5,
                rect.y + (rect.h - 32.0) * 0.5,
                32.0,
                32.0,
            );
            paint_icon(
                scene,
                *icon,
                chip,
                resolve(icon_button_fg(state), theme),
                1.5,
            );
        }
        TopBarCluster::Project { name } => {
            let icon_rect = Rect::new(
                rect.x + pad_x,
                rect.y + (rect.h - icon_w) * 0.5,
                icon_w,
                icon_w,
            );
            paint_icon(
                scene,
                IconId::Folder,
                icon_rect,
                resolve(ColorToken::Text2, theme),
                1.5,
            );
            let name_x = icon_rect.x + icon_w + Spacing::Md.px();
            let name_y = rect.y + (rect.h - font) * 0.5;
            paint_text(
                text_system,
                scene,
                name,
                name_x,
                name_y,
                font,
                rect.w - (name_x - rect.x) - pad_x,
                resolve(ColorToken::Text1, theme),
            );
        }
        TopBarCluster::Play => {
            let toggle_rect = Rect::new(rect.x + pad_x, rect.y + (rect.h - 22.0) * 0.5, 22.0, 22.0);
            hit_index.register(ids::TOPBAR_PLAY_TOGGLE, toggle_rect);
            let toggle_state = store
                .button_state(ids::TOPBAR_PLAY_TOGGLE)
                .unwrap_or(ButtonState::Normal);
            paint_icon(
                scene,
                IconId::Light,
                toggle_rect,
                resolve(icon_button_fg(toggle_state), theme),
                1.5,
            );
            let play_rect = Rect::new(
                rect.x + rect.w - pad_x - 32.0,
                rect.y + (rect.h - 32.0) * 0.5,
                32.0,
                32.0,
            );
            hit_index.register(id, play_rect);
            let play_state = store.button_state(id).unwrap_or(ButtonState::Normal);
            let bg = match play_state {
                ButtonState::Pressed => ColorToken::AccentPress,
                ButtonState::Hovered => ColorToken::AccentSoft,
                _ => ColorToken::Danger,
            };
            fill_rounded_rect(scene, play_rect, Radius::Lg.px(), resolve(bg, theme));
            paint_icon(
                scene,
                IconId::Play,
                play_rect,
                resolve(ColorToken::AccentFg, theme),
                1.5,
            );
        }
        TopBarCluster::Right => {
            let icons = [
                (ids::TOPBAR_RIGHT_LAYERS, IconId::Layers),
                (ids::TOPBAR_RIGHT_ASSETS, IconId::Asset),
                (ids::TOPBAR_RIGHT_SCRIPT, IconId::Script),
            ];
            let chip = 32.0_f32;
            for (i, (icon_id, icon)) in icons.iter().enumerate() {
                let cx = rect.x + pad_x + (chip + 4.0) * i as f32;
                let chip_rect = Rect::new(cx, rect.y + (rect.h - chip) * 0.5, chip, chip);
                hit_index.register(*icon_id, chip_rect);
                let state = store.button_state(*icon_id).unwrap_or(ButtonState::Normal);
                paint_icon(
                    scene,
                    *icon,
                    chip_rect,
                    resolve(icon_button_fg(state), theme),
                    1.5,
                );
            }
        }
    }
}
