//! Cluster-level painter for the hero TopBar. Extracted from
//! `topbar/mod.rs` in Wave 2 PR 11.7c — file was 727 LOC, the cluster
//! painter accounted for ~250 LOC of self-contained match-arm
//! geometry. Splitting it here brings `mod.rs` under the HR-18 cap of
//! 600 LOC without changing painter behavior.
//!
//! `paint_top_bar_cluster` is the one entry point — called from
//! `mod.rs::paint_top_bar` for each [`fixture::TopBarCluster`].
//! `cluster_width` is consulted by the same orchestrator to compute
//! right-aligned layout. Neither is re-exported beyond `super`.
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Circle, Fill, Point, VectorScene};

use crate::icons::IconId;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect};
use crate::widget::{ButtonState, PILL_PADDING_PX};
use crate::zones::Rect;

use super::super::fixture;
use super::super::ids;
use super::super::style::icon_button_fg;

pub(super) fn cluster_width(cluster: &fixture::TopBarCluster) -> f32 {
    use fixture::TopBarCluster;
    match cluster {
        TopBarCluster::Theme { .. } => 132.0,
        TopBarCluster::Single { .. } => 40.0 + PILL_PADDING_PX * 2.0,
        TopBarCluster::Project { .. } => 156.0,
        // Play cluster now holds 3 controls (Play 32 + Pause 24 +
        // Reset 24) plus inner spacings. ~120px fits comfortably.
        TopBarCluster::Play => 120.0,
        TopBarCluster::Right => 132.0,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_top_bar_cluster(
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
        TopBarCluster::Theme { label: _ } => {
            // Register the whole cluster as a clickable hit so the
            // dispatch can open the ThemeSelector context menu on
            // Primary Down. Other clusters (Single, Project, ...)
            // are hit-registered in their own arms below; we add it
            // here for Theme.
            hit_index.register(id, rect);
            // Display "PH2D" as the chip label — the cluster acts as
            // the engine's identity chip + theme picker entry-point.
            // (Theme's display name is still surfaced in the menu's
            // own items.)
            let label = "PH2D";
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
            // Project chip is the entry-point for the SceneList
            // popover (search + scene rows). Register the whole
            // cluster as a clickable hit; dispatch opens the menu
            // on Primary Down.
            hit_index.register(id, rect);
            // Leading glyph: the Blender `scene_data` icon ("scene")
            // — was an Avatar('E') initial; replaced because the
            // chip now reads as "current scene", not "project owner".
            let icon_size = 22.0_f32;
            let icon_rect = Rect::new(
                rect.x + pad_x,
                rect.y + (rect.h - icon_size) * 0.5,
                icon_size,
                icon_size,
            );
            paint_icon(
                scene,
                IconId::Scene,
                icon_rect,
                resolve(ColorToken::Text2, theme),
                1.5,
            );
            // Prefer the store's current scene name (mutated by
            // SceneList row clicks); fall back to the fixture name.
            let store_name = store.current_scene_name();
            let display = if store_name.is_empty() {
                name.as_str()
            } else {
                store_name
            };
            // Reserve room on the right for the dropdown chevron so
            // the name doesn't overlap it.
            let chev_size = 14.0_f32;
            let chev_rect = Rect::new(
                rect.x + rect.w - pad_x - chev_size,
                rect.y + (rect.h - chev_size) * 0.5,
                chev_size,
                chev_size,
            );
            let name_x = icon_rect.x + icon_size + Spacing::Md.px();
            let name_y = rect.y + (rect.h - font) * 0.5;
            let name_w = (chev_rect.x - name_x - Spacing::Sm.px()).max(0.0);
            paint_text(
                text_system,
                scene,
                display,
                name_x,
                name_y,
                font,
                name_w,
                resolve(ColorToken::Text1, theme),
            );
            paint_icon(
                scene,
                IconId::ChevronDown,
                chev_rect,
                resolve(ColorToken::Text3, theme),
                1.5,
            );
        }
        TopBarCluster::Play => {
            // Play / Pause / Reset transport. Play gets the
            // Accent-tinted pill (primary action); Pause and Reset
            // are flat icon buttons.
            let pill_d = 32.0_f32;
            let plain_d = 24.0_f32;
            // Play (leftmost, primary pill).
            let play_rect = Rect::new(
                rect.x + pad_x,
                rect.y + (rect.h - pill_d) * 0.5,
                pill_d,
                pill_d,
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
            // Pause (middle, plain icon button).
            let pause_rect = Rect::new(
                play_rect.x + pill_d + Spacing::Sm.px(),
                rect.y + (rect.h - plain_d) * 0.5,
                plain_d,
                plain_d,
            );
            hit_index.register(ids::TOPBAR_PAUSE, pause_rect);
            let pause_state = store
                .button_state(ids::TOPBAR_PAUSE)
                .unwrap_or(ButtonState::Normal);
            paint_icon(
                scene,
                IconId::Pause,
                pause_rect,
                resolve(icon_button_fg(pause_state), theme),
                1.5,
            );
            // Reset (rightmost, plain icon button).
            let reset_rect = Rect::new(
                pause_rect.x + plain_d + Spacing::Sm.px(),
                rect.y + (rect.h - plain_d) * 0.5,
                plain_d,
                plain_d,
            );
            hit_index.register(ids::TOPBAR_RESET, reset_rect);
            let reset_state = store
                .button_state(ids::TOPBAR_RESET)
                .unwrap_or(ButtonState::Normal);
            paint_icon(
                scene,
                IconId::Reset,
                reset_rect,
                resolve(icon_button_fg(reset_state), theme),
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
