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
use crate::paint::{
    fill_rounded_rect, paint_icon, paint_icon_path, paint_text, resolve, stroke_rounded_rect,
};
use crate::widget::{ButtonState, IconButtonStyle, IconGlyph, PILL_PADDING_PX, paint_icon_button};
use crate::zones::Rect;

use super::super::fixture;
use super::super::ids;
use super::super::style::icon_button_fg;

/// Width allocated to one rail-style chip column. Chip = 36 px
/// (Small `RailButtonSize`) + 4 px margin each side = 44 px column.
/// This mirrors the side rail's chip-to-column ratio so the chip's
/// 1 px Border visually merges with the backdrop edge (the "frameless"
/// rail look). Longer labels truncate via the clip layer in
/// `paint_topbar_rail_chip`.
pub(super) const TOPBAR_RAIL_CHIP_W: f32 = 44.0; // LITERAL-PX-OK: chip column width = chip_px(36) + 4 px margin × 2

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
        TopBarCluster::Play => TOPBAR_RAIL_CHIP_W * 3.0, // LITERAL-PX-OK: 3 chip columns (Play/Pause/Reset)
        TopBarCluster::Right => TOPBAR_RAIL_CHIP_W * 3.0, // LITERAL-PX-OK: 3 chip columns (Layers/Assets/Script)
    }
}

/// Paint a rail-style chip with a horizontal sub-label IMEDIATAMENTE
/// acima do chip (compact stack, no breathing space above the label —
/// Enio 2026-05-25: "caiba apenas os botões e as labels praticamente
/// sem espaços").
///
/// Chip itself MIRRORS `widget::tool_rail::paint_tool_rail` Icon
/// entry EXACTLY: same `Radius::Sm`, BgElev fill, Border stroke
/// (1 px Normal, Accent under press), Text2/Accent icon foreground
/// per `ButtonState`. Matrix copied verbatim — DO NOT diverge.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_topbar_rail_chip(
    chip_id: NodeId,
    glyph: IconGlyph<'_>,
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
    let label_to_chip_gap = 3.0_f32; // LITERAL-PX-OK: mirror of rail's LABEL_TO_CHIP_GAP_PX
    // Stack starts at the top of the backdrop (which is glued to the
    // viewport top via `paint_topbar_group_backdrop`). `Xxs` inner pad
    // mirrors what the backdrop reserves on top.
    let stack_y = viewport_y + Spacing::Xxs.px();
    let chip_x = chip_col.x + (chip_col.w - chip_px) * 0.5;
    let chip_y = stack_y + label_band_h + label_to_chip_gap;
    let chip_rect = Rect::new(chip_x, chip_y, chip_px, chip_px);
    hit_index.register(chip_id, chip_rect);
    let state = store.button_state(chip_id).unwrap_or(ButtonState::Normal);
    // Matriz EXATA do rail (tool_rail.rs:248-280). With the narrower
    // 44 px column, the chip's 1 px Border now sits within 4 px of the
    // backdrop edge — the same chip/backdrop ratio the rail has,
    // making the border read as part of the chrome instead of a
    // "moldura intermediária".
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
    match glyph {
        IconGlyph::Builtin(id) => paint_icon(
            scene,
            id,
            chip_rect,
            resolve(fg, theme),
            StrokeToken::Default.px(),
        ),
        IconGlyph::Path(p) => paint_icon_path(
            scene,
            p,
            chip_rect,
            resolve(fg, theme),
            StrokeToken::Default.px(),
        ),
    }
    // --- Label band: sits directly above the chip (gap mirrors the
    // rail's `LABEL_TO_CHIP_GAP_PX`).
    let label_rect = Rect::new(chip_col.x, stack_y, chip_col.w, label_band_h);
    let label_clip = ph2d_vector::Rect::new(
        label_rect.x as f64,
        label_rect.y as f64,
        (label_rect.x + label_rect.w) as f64,
        (label_rect.y + label_rect.h) as f64,
    );
    scene.push_clip(&label_clip);
    // UPPERCASE + truncate to ≤5 chars (Enio 2026-05-25: "contraia
    // todos os nomes ou transforma em em siglas para caber em até 5
    // letras"). Fixture labels longer than 5 chars must abbreviate
    // themselves (e.g. "Image Tools" → "IMG"); we hard-cap here so
    // any drift through i18n still fits.
    let short: String = label.chars().take(5).collect::<String>().to_uppercase();
    crate::paint::paint_text_centered(
        text_system,
        scene,
        &short,
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
    let pad_x = Spacing::Md.px();
    let icon_w = 18.0; // LITERAL-PX-OK: chev icon dim (chrome accent)
    let font = TypeToken::Sm.px();
    // Theme + Project are "wide chips" — share the chip-row Y of the
    // rail-style chips (Save/Open/IMG sit below their label, both
    // anchored in the backdrop top via viewport_y). Width is the
    // full cluster width (set per cluster in `cluster_width`), height
    // = chip_px so the BgElev fill + Border stroke read identically
    // to the icon chips, just wider (Enio 2026-05-25: "aparência
    // similar aos botões embora mais largos").
    let chip_px = store.rail_button_size().chip_px();
    let label_band_h = 11.0_f32; // LITERAL-PX-OK: mirror of rail's LABEL_VISUAL_EXTENT_PX
    let label_to_chip_gap = 3.0_f32; // LITERAL-PX-OK: mirror of rail's LABEL_TO_CHIP_GAP_PX
    let inner_y = viewport_y + Spacing::Xxs.px() + label_band_h + label_to_chip_gap;
    let inner = Rect::new(rect.x, inner_y, rect.w, chip_px);
    // Wide-chip surface (fill BgElev + border Border 1 px Radius::Sm)
    // ONLY for Theme + Project. Single/Play/Right delegate to
    // `paint_topbar_rail_chip`, which paints its own per-chip surface.
    let paint_wide_chip = |scene: &mut VectorScene| {
        fill_rounded_rect(scene, inner, Radius::Sm.px(), resolve(ColorToken::BgElev, theme));
        stroke_rounded_rect(
            scene,
            inner,
            Radius::Sm.px(),
            1.0,
            resolve(ColorToken::Border, theme),
        );
    };
    match cluster {
        TopBarCluster::Theme { label: _ } => {
            paint_wide_chip(scene);
            hit_index.register(id, inner);
            let label = "PH2D";
            let mut cx = inner.x + pad_x + Spacing::Xs.px();
            let cy = inner.y + inner.h * 0.5;
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
            let label_x = inner.x + pad_x + 28.0; // LITERAL-PX-OK: label inset after the PH2D accent-dot logo
            let label_y = inner.y + (inner.h - font) * 0.5;
            paint_text(
                text_system,
                scene,
                label,
                label_x,
                label_y,
                font,
                inner.w - (label_x - inner.x) - Spacing::Xl2.px(),
                resolve(ColorToken::Text1, theme),
            );
            let chev_rect = Rect::new(
                inner.x + inner.w - pad_x - icon_w,
                inner.y + (inner.h - icon_w) * 0.5,
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
                IconGlyph::Builtin(*icon),
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
            paint_wide_chip(scene);
            hit_index.register(id, inner);
            let icon_size = 22.0_f32; // LITERAL-PX-OK: Project chip icon size (specific accent dim)
            let icon_rect = Rect::new(
                inner.x + pad_x,
                inner.y + (inner.h - icon_size) * 0.5,
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
            let store_name = store.current_scene_name();
            let display = if store_name.is_empty() {
                name.as_str()
            } else {
                store_name
            };
            let chev_size = SECTION_GAP_PX;
            let chev_rect = Rect::new(
                inner.x + inner.w - pad_x - chev_size,
                inner.y + (inner.h - chev_size) * 0.5,
                chev_size,
                chev_size,
            );
            let name_x = icon_rect.x + icon_size + Spacing::Md.px();
            let name_y = inner.y + (inner.h - font) * 0.5;
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
            let _ = (
                paint_icon_button,
                IconGlyph::Builtin(IconId::Play),
                IconButtonStyle::Plain,
            ); // keep imports alive
            let col_w = rect.w / 3.0; // LITERAL-PX-OK: 3 chip columns in this cluster
            let entries = [
                (id, IconId::Play, "Play"),
                (ids::TOPBAR_PAUSE, IconId::Pause, "Pause"),
                (ids::TOPBAR_RESET, IconId::Reset, "Reset"),
            ];
            for (i, (chip_id, icon, label)) in entries.iter().enumerate() {
                let col = Rect::new(rect.x + col_w * i as f32, rect.y, col_w, rect.h);
                paint_topbar_rail_chip(
                    *chip_id,
                    IconGlyph::Builtin(*icon),
                    label,
                    col,
                    viewport_y,
                    scene,
                    text_system,
                    theme,
                    hit_index,
                    store,
                );
            }
        }
        TopBarCluster::Right => {
            // 2026-05-24 Stage 2 — each viewport mode becomes its own
            // rail-style chip with a label above.
            let col_w = rect.w / 3.0; // LITERAL-PX-OK: 3 chip columns in this cluster
            let entries = [
                (ids::TOPBAR_RIGHT_LAYERS, IconId::Layers, "Layers"),
                (ids::TOPBAR_RIGHT_ASSETS, IconId::Asset, "Assets"),
                (ids::TOPBAR_RIGHT_SCRIPT, IconId::Script, "Script"),
            ];
            for (i, (chip_id, icon, label)) in entries.iter().enumerate() {
                let col = Rect::new(rect.x + col_w * i as f32, rect.y, col_w, rect.h);
                paint_topbar_rail_chip(
                    *chip_id,
                    IconGlyph::Builtin(*icon),
                    label,
                    col,
                    viewport_y,
                    scene,
                    text_system,
                    theme,
                    hit_index,
                    store,
                );
            }
        }
    }
}
