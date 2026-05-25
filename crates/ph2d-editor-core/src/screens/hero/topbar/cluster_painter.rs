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
use ph2d_tokens::{ColorToken, Radius, SECTION_GAP_PX, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Circle, Fill, Point, VectorScene};

use crate::icons::IconId;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect};
use crate::widget::{ButtonState, IconButtonStyle, IconGlyph, PILL_PADDING_PX, paint_icon_button};
use crate::zones::Rect;

use super::super::fixture;
use super::super::ids;
use super::super::style::icon_button_fg;

/// Width allocated to one rail-style chip + its label-above column
/// (2026-05-24 Stage-2 topbar redesign). Chip = `Spacing::Xl2`
/// (= 32 px); column gives ~28 px of breathing room around the chip
/// so 6–8 char labels read without clipping. Longer labels truncate
/// via the clip layer in `paint_topbar_rail_chip`.
const TOPBAR_RAIL_CHIP_W: f32 = 60.0; // LITERAL-PX-OK: chip column width (chrome dim)

pub(super) fn cluster_width(cluster: &fixture::TopBarCluster) -> f32 {
    use fixture::TopBarCluster;
    let _ = PILL_PADDING_PX; // keep import alive; old Single width used it
    match cluster {
        TopBarCluster::Theme { .. } => 100.0, // LITERAL-PX-OK: Theme chip — narrowed 132 → 100 (PH2D + dots + chev only, user 2026-05-24)
        // Single-icon cluster = one rail-style chip column.
        TopBarCluster::Single { .. } => TOPBAR_RAIL_CHIP_W,
        TopBarCluster::Project { .. } => 156.0, // LITERAL-PX-OK: Project chip width (chrome dim)
        // Play / Right multi-icon clusters host 3 chip columns each
        // (Play/Pause/Reset; Layers/Assets/Script) — per user 2026-05-24
        // each icon becomes its own labeled chip.
        TopBarCluster::Play => TOPBAR_RAIL_CHIP_W * 3.0,
        TopBarCluster::Right => TOPBAR_RAIL_CHIP_W * 3.0,
    }
}

/// Paint a rail-style chip with a horizontal sub-label in the
/// backdrop's TOP GUTTER — horizontal counterpart of the side rail's
/// chip + vertical-rotated label arrangement.
///
/// MIRRORS `widget::tool_rail::paint_tool_rail` Icon entry EXACTLY for
/// the chip itself: same `Radius::Sm`, same BgElev fill, same Border
/// stroke (1 px Normal, Accent under press), same Text2/Accent icon
/// foreground modulated by `ButtonState`. The matrix is copied
/// verbatim — DO NOT diverge.
///
/// Geometry — the rail/topbar analogy:
///   - In the rail, the rotated sub-label hugs the LEFT edge of the
///     `RailBg` backdrop (the "left gutter") and the chip sits to its
///     right, with `LABEL_TO_CHIP_GAP_PX = 3` between them.
///   - Here, the label hugs the TOP edge of the topbar group backdrop
///     (the "top gutter") at `viewport_y + Xxs`, and the chip sits
///     vertically centered in the topbar row below it.
///   - Crucially, the chip and the label are visually SEPARATE — the
///     label is anchored to the backdrop, not to the chip — so the
///     chip's 1 px Border doesn't read as a frame wrapping the label
///     too. (User feedback 2026-05-24: "Parece que vc criou 2 botões,
///     1 por dentro do outro" — the previous layout stacked label +
///     chip tightly inside the same column, making the chip's border
///     look like it framed both.)
#[allow(clippy::too_many_arguments)]
fn paint_topbar_rail_chip(
    chip_id: NodeId,
    icon: IconId,
    label: &str,
    chip_col: Rect,
    viewport_y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    // Chip size mirrors the rail. Read from store so the Themes-menu
    // RailButtonSize preset (Small/Medium/Large) affects the topbar
    // too — they're meant to look identical.
    let chip_px = store.rail_button_size().chip_px();
    // Label font: same formula as `paint_tool_rail` (line 232):
    //   `(Xs.px() - 2.0).max(Md.px())` → 9 px under the default tokens.
    let sub_font = (TypeToken::Xs.px() - 2.0).max(Spacing::Md.px());
    // Label band height = rail's `LABEL_VISUAL_EXTENT_PX = 11.0`.
    let label_band_h = 11.0_f32; // LITERAL-PX-OK: mirror of rail's LABEL_VISUAL_EXTENT_PX
    // Chip centered vertically in the topbar row. The label is NOT
    // stacked above it inside the same column — it lives independently
    // in the backdrop's top gutter (see label_rect below). This is the
    // anti-"2-botões-aninhados" fix.
    let chip_x = chip_col.x + (chip_col.w - chip_px) * 0.5;
    let chip_y = chip_col.y + (chip_col.h - chip_px) * 0.5;
    let chip_rect = Rect::new(chip_x, chip_y, chip_px, chip_px);
    hit_index.register(chip_id, chip_rect);
    let state = store.button_state(chip_id).unwrap_or(ButtonState::Normal);
    // --- Mirror of paint_tool_rail Icon entry (tool_rail.rs:248-280) ---
    let radius = Radius::Sm.px();
    let is_active = state == ButtonState::Pressed;
    let bg = match state {
        ButtonState::Hovered | ButtonState::Focused => ColorToken::BgElev,
        ButtonState::Pressed => ColorToken::AccentSoft,
        _ if is_active => ColorToken::AccentSoft,
        _ => ColorToken::BgElev,
    };
    fill_rounded_rect(scene, chip_rect, radius, resolve(bg, theme));
    let (border, border_w) = match state {
        ButtonState::Hovered | ButtonState::Focused => (ColorToken::BorderEmph, 1.0),
        ButtonState::Pressed => (ColorToken::Accent, StrokeToken::Default.px()),
        _ if is_active => (ColorToken::Accent, StrokeToken::Default.px()),
        _ => (ColorToken::Border, 1.0),
    };
    stroke_rounded_rect(scene, chip_rect, radius, border_w, resolve(border, theme));
    let fg = match state {
        ButtonState::Hovered | ButtonState::Focused => ColorToken::Text1,
        ButtonState::Pressed => ColorToken::Accent,
        _ if is_active => ColorToken::Accent,
        _ => ColorToken::Text2,
    };
    paint_icon(scene, icon, chip_rect, resolve(fg, theme), StrokeToken::Default.px());
    // --- Label band: sits in the backdrop's TOP GUTTER, not above the
    // chip inside its column. The backdrop extends up to viewport_y;
    // anchoring the label at `viewport_y + Xxs` puts it "quase tocando
    // no topo" (user 2026-05-24) and visually divorces it from the
    // chip's border below.
    let label_y = viewport_y + Spacing::Xxs.px();
    let label_rect = Rect::new(chip_col.x, label_y, chip_col.w, label_band_h);
    let label_clip = ph2d_vector::Rect::new(
        label_rect.x as f64,
        label_rect.y as f64,
        (label_rect.x + label_rect.w) as f64,
        (label_rect.y + label_rect.h) as f64,
    );
    scene.push_clip(&label_clip);
    crate::paint::paint_text_centered(
        text_system,
        scene,
        label,
        label_rect,
        sub_font,
        resolve(ColorToken::Text2, theme),
    );
    scene.pop_layer();
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_top_bar_cluster(
    id: NodeId,
    cluster: &fixture::TopBarCluster,
    rect: Rect,
    viewport_y: f32,
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
    let icon_w = 18.0; // LITERAL-PX-OK: chev icon dim (chrome accent)
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
            let mut cx = rect.x + pad_x + Spacing::Xs.px();
            let cy = rect.y + rect.h * 0.5;
            for (i, token) in [ColorToken::Accent, ColorToken::AccentSoft]
                .iter()
                .enumerate()
            {
                let dot = Circle::new(Point::new(cx as f64, cy as f64), Spacing::Xs.px() as f64);
                scene.inner_mut().fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(resolve(*token, theme)),
                    None,
                    &dot,
                );
                cx += if i == 0 { 10.0 } else { 0.0 }; // LITERAL-PX-OK: inter-accent-dot spacing in PH2D logo (decorative)
            }
            let label_x = rect.x + pad_x + 28.0; // LITERAL-PX-OK: label inset after the PH2D accent-dot logo
            let label_y = rect.y + (rect.h - font) * 0.5;
            paint_text(
                text_system,
                scene,
                label,
                label_x,
                label_y,
                font,
                rect.w - (label_x - rect.x) - Spacing::Xl2.px(),
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
                StrokeToken::Default.px(),
            );
        }
        TopBarCluster::Single { label, icon } => {
            // 2026-05-24 Stage 2 redesign — rail-style chip + label
            // above (instead of bare centered icon). `label` was
            // already on the cluster; now surfaced visually.
            let _ = icon_button_fg(ButtonState::Normal); // keep import alive
            paint_topbar_rail_chip(
                id,
                *icon,
                label,
                rect,
                viewport_y,
                scene,
                text_system,
                theme,
                hit_index,
                store,
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
            let icon_size = 22.0_f32; // LITERAL-PX-OK: Project chip icon size (specific accent dim)
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
                StrokeToken::Default.px(),
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
            let chev_size = SECTION_GAP_PX;
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
                StrokeToken::Default.px(),
            );
        }
        TopBarCluster::Play => {
            // 2026-05-24 Stage 2 — each transport button becomes its
            // own rail-style chip with a label above (Play / Pause /
            // Reset). Replaces the hand-rolled 3-icon row.
            let _ = (paint_icon_button, IconGlyph::Builtin(IconId::Play),
                     IconButtonStyle::Plain); // keep imports alive
            let col_w = rect.w / 3.0;
            let entries = [
                (id, IconId::Play, "Play"),
                (ids::TOPBAR_PAUSE, IconId::Pause, "Pause"),
                (ids::TOPBAR_RESET, IconId::Reset, "Reset"),
            ];
            for (i, (chip_id, icon, label)) in entries.iter().enumerate() {
                let col = Rect::new(rect.x + col_w * i as f32, rect.y, col_w, rect.h);
                paint_topbar_rail_chip(
                    *chip_id, *icon, label, col, viewport_y, scene, text_system, theme, hit_index,
                    store,
                );
            }
        }
        TopBarCluster::Right => {
            // 2026-05-24 Stage 2 — each viewport mode becomes its own
            // rail-style chip with a label above.
            let col_w = rect.w / 3.0;
            let entries = [
                (ids::TOPBAR_RIGHT_LAYERS, IconId::Layers, "Layers"),
                (ids::TOPBAR_RIGHT_ASSETS, IconId::Asset, "Assets"),
                (ids::TOPBAR_RIGHT_SCRIPT, IconId::Script, "Script"),
            ];
            for (i, (chip_id, icon, label)) in entries.iter().enumerate() {
                let col = Rect::new(rect.x + col_w * i as f32, rect.y, col_w, rect.h);
                paint_topbar_rail_chip(
                    *chip_id, *icon, label, col, viewport_y, scene, text_system, theme, hit_index,
                    store,
                );
            }
        }
    }
}
