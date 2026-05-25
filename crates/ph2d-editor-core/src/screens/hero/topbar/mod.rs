//! TopBar painter — 5 pill clusters + centered wordmark.

use super::HeroLayout;
use super::fixture;
use super::ids;
use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::widget::{
    ButtonState, IconButtonStyle, IconGlyph, PILL_PADDING_PX, Tooltip, paint_icon_button,
    paint_tooltip,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

/// Register every TopBar widget into the [`WidgetStore`]. Called
/// once by `HeroScreen::pre_populate_store`.
pub fn populate(store: &mut WidgetStore) {
    for id in [
        ids::TOPBAR_THEME,
        ids::TOPBAR_SAVE,
        ids::TOPBAR_SAVE_AS,
        ids::TOPBAR_OPEN,
        ids::TOPBAR_IMAGE_TOOLS,
        ids::TOPBAR_WIDGET_GALLERY,
        ids::TOPBAR_GRID_SETTINGS,
        ids::TOPBAR_SETTINGS,
        ids::TOPBAR_PROJECT,
        ids::TOPBAR_PLAY_BUTTON,
        ids::TOPBAR_PAUSE,
        ids::TOPBAR_RESET,
        ids::TOPBAR_RIGHT_LAYERS,
        ids::TOPBAR_RIGHT_ASSETS,
        ids::TOPBAR_RIGHT_SCRIPT,
        ids::IMAGE_ACTION_TRIM,
        ids::IMAGE_ACTION_MAKE_SQUARE,
        ids::IMAGE_ACTION_BGREMOVAL,
        ids::IMAGE_ACTION_REAL_SIZE,
        ids::IMAGE_ACTION_PADDING,
        ids::IMAGE_ACTION_COLOR_EQUALIZATION,
        ids::IMAGE_ACTION_EQUALIZE_SIZES,
        ids::IMAGE_ACTION_RASTERIZE,
        ids::IMAGE_ACTION_UPSCALE,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // Search input for the project-chip Scene List popover. Lives
    // here so opening the menu doesn't have to allocate state on
    // demand — its buffer is just filtered against the scene list
    // at paint time.
    store.register(
        ids::CTX_SCENE_SEARCH,
        InteractiveState::TextInput {
            state: crate::widget::TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    // Seed the generic tooltip side-table. Previously these strings
    // lived only in `tooltip_for(id)` and the hover painter matched
    // ids directly; now every widget can register its own tooltip
    // via `store.set_tooltip(id, text)` — keeps screens cohesive
    // with no boilerplate per-id lookup.
    for (id, text) in [
        // ASCII shortcuts — the macOS Command glyph U+2318 (⌘) and
        // Return glyph U+21B5 (↵) aren't in our parley font fallback
        // chain and rendered as tofu boxes. `Cmd+S` / `Cmd+Enter` are
        // legible on every theme without a special font.
        (ids::TOPBAR_SAVE, "Save \u{00b7} Cmd+S"),
        (ids::TOPBAR_SAVE_AS, "Save As\u{2026} \u{00b7} Cmd+Shift+S"),
        (ids::TOPBAR_OPEN, "Open \u{00b7} Cmd+O"),
        (ids::TOPBAR_IMAGE_TOOLS, "Image Tools"),
        (
            ids::TOPBAR_WIDGET_GALLERY,
            "Widget Gallery \u{00b7} reference",
        ),
        (ids::TOPBAR_GRID_SETTINGS, "Grid Settings"),
        (
            ids::IMAGE_ACTION_TRIM,
            ph2d_i18n::tr("tool.trim_transparency.tooltip"),
        ),
        (
            ids::IMAGE_ACTION_MAKE_SQUARE,
            ph2d_i18n::tr("tool.make_square.tooltip"),
        ),
        (
            ids::IMAGE_ACTION_BGREMOVAL,
            ph2d_i18n::tr("tool.bgremoval.tooltip"),
        ),
        (
            ids::IMAGE_ACTION_REAL_SIZE,
            ph2d_i18n::tr("tool.real_size.tooltip"),
        ),
        (
            ids::IMAGE_ACTION_PADDING,
            ph2d_i18n::tr("tool.padding.tooltip"),
        ),
        (
            ids::IMAGE_ACTION_COLOR_EQUALIZATION,
            ph2d_i18n::tr("tool.color_equalization.tooltip"),
        ),
        (
            ids::IMAGE_ACTION_EQUALIZE_SIZES,
            ph2d_i18n::tr("tool.equalize_sizes.tooltip"),
        ),
        (
            ids::IMAGE_ACTION_RASTERIZE,
            ph2d_i18n::tr("tool.rasterize.tooltip"),
        ),
        (
            ids::IMAGE_ACTION_UPSCALE,
            ph2d_i18n::tr("tool.upscale.tooltip"),
        ),
        (ids::TOPBAR_SETTINGS, "Project settings"),
        (ids::TOPBAR_PROJECT, "Project"),
        (ids::TOPBAR_PLAY_BUTTON, "Run \u{00b7} \u{2318}\u{21b5}"),
        (ids::TOPBAR_PAUSE, "Pause"),
        (ids::TOPBAR_RESET, "Reset"),
        (ids::TOPBAR_RIGHT_LAYERS, "Layers"),
        (ids::TOPBAR_RIGHT_ASSETS, "Asset library"),
        (ids::TOPBAR_RIGHT_SCRIPT, "Code \u{00b7} Luau"),
        (ids::TOOL_TRANSLATE, "Translate \u{00b7} G"),
        (ids::TOOL_ROTATE, "Rotate \u{00b7} R"),
        (ids::TOOL_SCALE, "Scale \u{00b7} S"),
        (ids::TOOL_PIVOT, "Pivot"),
        (ids::TOOL_UNDO, "Undo"),
        (ids::TOOL_REDO, "Redo"),
        (ids::HIERARCHY_ADD, "Add entity"),
    ] {
        store.set_tooltip(id, text);
    }
}

/// Apply a [`WidgetEvent`] against TopBar widgets. Returns true iff
/// the event was consumed. Stub for now — TopBar buttons currently
/// only carry hover/press visual feedback; project-level reactions
/// (Save flush, Play start, etc.) land in follow-up PRs.
pub fn apply_event(_store: &mut WidgetStore, _event: WidgetEvent) -> bool {
    false
}

/// Paint a Tooltip floating just above the currently hovered widget.
/// Called by the hero orchestrator after every chrome painter has
/// run so the tooltip lands on top. Tooltip text is read from the
/// store's generic tooltip side-table — populate it via
/// `WidgetStore::set_tooltip(id, text)` from any painter / populate
/// pass.
pub fn paint_hover_tooltip(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &HitIndex,
    store: &WidgetStore,
) {
    let Some(id) = store.hot_id() else {
        return;
    };
    // Suppress the tooltip when the hot widget is an open Dropdown
    // (or any widget whose popover paints directly below the hit
    // rect — they share the exact pill geometry the tooltip wants
    // and would otherwise paint OVER the first option). See
    // `docs/UI_Bugs/README.md` §9.13.
    if matches!(
        store.get(id),
        Some(crate::interaction::InteractiveState::Dropdown { open: true, .. })
            | Some(crate::interaction::InteractiveState::Combobox { open: true, .. })
    ) {
        return;
    }
    let Some(text) = store.tooltip_for(id) else {
        return;
    };
    let Some(target_rect) = hit_index.rect_for(id) else {
        return;
    };
    // Real text measurement instead of `chars × 6.5` — for
    // proportional fonts the approximation is off by 10-30 % and
    // the pill ends up clipped or oversized. See
    // `docs/UI_Bugs/README.md` §3.3.
    let font_size = ph2d_tokens::TypeToken::Sm.px();
    let measured_w = text_system.layout(text, font_size, f32::INFINITY).width();
    let pill_w = (measured_w + Spacing::Xl.px()).max(60.0); // LITERAL-PX-OK: tooltip pill min width (chrome-specific)
    let pill_h = (font_size + 10.0).max(22.0); // LITERAL-PX-OK: tooltip pill height composite + min (chrome-specific)
    // Center over the target; if that would clip the right edge of
    // the viewport, fall back to right-aligning to the target.
    let tip_x = target_rect.x + (target_rect.w - pill_w) * 0.5;
    let tip_rect = Rect::new(
        tip_x,
        target_rect.y + target_rect.h + Spacing::Sm.px(),
        pill_w,
        pill_h,
    );
    let tip = Tooltip::new(NodeId(0), text);
    paint_tooltip(&tip, tip_rect, scene, text_system, theme);
}

#[allow(clippy::too_many_arguments)]
pub fn paint_top_bar(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    image_tools_mode: bool,
) {
    let clusters = fixture::topbar_clusters();
    let row_h = layout.top_bar.h;
    let mut x = layout.top_bar.x;
    let gap = Spacing::Md.px();
    // Left half now holds 5 clusters: Theme, Project (Level), Save,
    // Open, Image Tools (Project moved here 2026-05-24).
    let split = 5.min(clusters.len());
    // Group backdrop (2026-05-24, user: "Tudo à esquerda inclusive
    // seletor de themes e levels num fundo, todos à direita, outro
    // fundo"). Paint a RailBg-colored rounded plate spanning the left
    // cluster group BEFORE individual clusters — mirrors the side
    // rail's frosted-glass backing.
    {
        let mut left_w = 0.0_f32;
        for (i, (_, c)) in clusters[..split].iter().enumerate() {
            if i > 0 {
                left_w += gap;
            }
            left_w += cluster_width(c);
        }
        if left_w > 0.0 {
            paint_topbar_group_backdrop(
                scene,
                theme,
                Rect::new(layout.top_bar.x, layout.top_bar.y, left_w, row_h),
                layout.viewport.y,
            );
        }
    }
    // Left half is always painted — the Image Tools mode keeps the
    // identity / Save / Open / ImageTools cluster visible so the user
    // can exit the mode by clicking ImageTools again.
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
        // Active-state ring on the ImageTools pill when the mode is on
        // — uses the theme's Accent token so it reads on every theme.
        if image_tools_mode && *id == ids::TOPBAR_IMAGE_TOOLS {
            stroke_rounded_rect(
                scene,
                rect,
                Radius::Xl.px(),
                2.0,
                resolve(ColorToken::Accent, theme),
            );
        }
        x = rect.x + rect.w + gap;
    }
    // The wordmark "PH2D · EDITOR" that used to fill the middle gap
    // is intentionally absent now — the engine's identity is carried
    // by the leftmost theme chip (also labelled "PH2D"). Leaving the
    // gap transparent also keeps the topbar's bg fully see-through.
    let _ = x; // silence unused-after-removal

    if image_tools_mode {
        // Mode on — replace the right half with the image-action row.
        paint_image_action_row(layout, scene, theme, hit_index, store, gap);
        return;
    }

    // Default mode — paint the right clusters (Project / Play / Right /
    // Settings) right-aligned to the bar.
    let right_clusters = &clusters[split..];
    let mut right_w = 0.0_f32;
    for (i, (_, c)) in right_clusters.iter().enumerate() {
        if i > 0 {
            right_w += gap;
        }
        right_w += cluster_width(c);
    }
    let right_x = layout.top_bar.x + layout.top_bar.w - right_w;
    // Right-group backdrop (2026-05-24 redesign — sibling of the left
    // group). Painted before the clusters so they layer on top.
    if right_w > 0.0 {
        paint_topbar_group_backdrop(
            scene,
            theme,
            Rect::new(right_x, layout.top_bar.y, right_w, row_h),
            layout.viewport.y,
        );
    }
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

/// Paint a frosted-glass backing for a topbar cluster group
/// (2026-05-24 redesign — user request to visually group the chips
/// like the side rail's tray). Inset is `Sm` horizontally + a tiny
/// vertical bleed so per-cluster strokes don't sit flush with the
/// backdrop edge.
fn paint_topbar_group_backdrop(
    scene: &mut VectorScene,
    theme: Theme,
    group_rect: Rect,
    viewport_top: f32,
) {
    let pad_h = Spacing::Sm.px();
    let pad_v = Spacing::Xxs.px();
    // Top edge extends up to viewport top — user 2026-05-24: "o fundo
    // deve se expandir até o topo da tela". Bottom edge keeps the
    // small symmetric Xxs inset.
    let top_y = viewport_top.min(group_rect.y - pad_v);
    let bottom_y = group_rect.y + group_rect.h + pad_v;
    let bg = Rect::new(
        group_rect.x - pad_h,
        top_y,
        group_rect.w + pad_h * 2.0,
        bottom_y - top_y,
    );
    fill_rounded_rect(scene, bg, Radius::Lg.px(), resolve(ColorToken::RailBg, theme));
}

/// Paint the image-action row that occupies the right half of the
/// TopBar when [`HeroScreen::image_tools_mode`] is `true`. Each pill
/// is registered in the hit index so dispatch can route clicks; tooltips
/// are seeded by [`populate`].
///
/// Wave 2 PR 11.4: pill list is now derived from the
/// [`crate::installed_registry`] when present (one entry per manifest
/// in the `image_tools` cluster); falls back to the legacy hardcoded
/// list for tests that don't install a registry.
fn paint_image_action_row(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    gap: f32,
) {
    let row_h = layout.top_bar.h;
    let pill_w = 40.0 + PILL_PADDING_PX * 2.0; // LITERAL-PX-OK: TopBar action pill base width (chrome dim, matches single-cluster width)
    let pills = image_action_pills();
    let total_w = pill_w * pills.len() as f32 + gap * pills.len().saturating_sub(1) as f32;
    let start_x = layout.top_bar.x + layout.top_bar.w - total_w;
    // Group backdrop for the image-tools action row (2026-05-24
    // redesign — siblings of the left/right group backdrops in
    // default mode).
    if total_w > 0.0 {
        paint_topbar_group_backdrop(
            scene,
            theme,
            Rect::new(start_x, layout.top_bar.y, total_w, row_h),
            layout.viewport.y,
        );
    }
    let mut rx = start_x;
    for pill in &pills {
        let rect = Rect::new(rx, layout.top_bar.y, pill_w, row_h);
        hit_index.register(pill.id, rect);
        let state = store.button_state(pill.id).unwrap_or(ButtonState::Normal);
        // Canonical framed icon-chip — single source of truth (was a
        // hand-rolled fill+stroke+icon here). Manifest pills carry a
        // `BezPath` glyph, which `ButtonKind` can't hold, so the glyph is
        // a paint-time `IconGlyph`.
        let glyph = match &pill.icon {
            PillIcon::FromManifest(path) => IconGlyph::Path(path),
            PillIcon::Legacy(icon) => IconGlyph::Builtin(*icon),
        };
        paint_icon_button(rect, glyph, IconButtonStyle::Chip, state, scene, theme);
        rx = rect.x + rect.w + gap;
    }
}

/// Drawing source for one Image Tools action pill's glyph. When the
/// pill came from a manifest the manifest's `icon_fn` already produced
/// a 24×24 [`ph2d_vector::BezPath`]; when the pill came from the
/// legacy fallback the editor's `IconId` table supplies the path.
enum PillIcon {
    /// Manifest-derived (Wave 2 PR 11.4). `icon_fn` returned this path.
    FromManifest(ph2d_vector::BezPath),
    /// Legacy fallback for tests / pre-registry boot.
    Legacy(IconId),
}

/// One pill row entry. Tuple form previously; refactored into a struct
/// in PR 11.4 because the icon source now has two flavors (manifest
/// BezPath vs legacy IconId).
struct ImageActionPill {
    id: NodeId,
    icon: PillIcon,
    label_key: &'static str,
}

/// Build the Image Tools action pill list. Wave 2 PR 11.4: derives
/// from the runtime [`crate::installed_registry`] when present.
/// Manifests register the cluster id `"image_tools"`; the registry
/// returns them sorted by `(order, id)` so paint order matches design
/// intent (trim 40 → make_square 50 → bgremoval 60).
///
/// Falls back to the legacy hardcoded triple for tests / pre-registry
/// boot. Both paths produce the same `NodeId`s because the chrome
/// consts in [`crate::screens::hero::ids`] hash the SAME slug as the
/// matching manifest's `id` field (PR 11.4 contract — pinned by the
/// `chrome_manifest_coverage` integration test).
fn image_action_pills() -> Vec<ImageActionPill> {
    use ph2d_tool_registry::hash_node_id;
    if let Some(reg) = crate::installed_registry() {
        return reg
            .cluster("image_tools")
            .iter()
            .map(|m| ImageActionPill {
                id: hash_node_id(m.id),
                icon: PillIcon::FromManifest((m.icon_fn)()),
                label_key: m.label_key,
            })
            .collect();
    }
    vec![
        ImageActionPill {
            id: ids::IMAGE_ACTION_TRIM,
            icon: PillIcon::Legacy(IconId::TrimTransparency),
            label_key: "tool.trim_transparency.label",
        },
        ImageActionPill {
            id: ids::IMAGE_ACTION_MAKE_SQUARE,
            icon: PillIcon::Legacy(IconId::MakeSquare),
            label_key: "tool.make_square.label",
        },
        ImageActionPill {
            id: ids::IMAGE_ACTION_BGREMOVAL,
            icon: PillIcon::Legacy(IconId::BgRemoval),
            label_key: "tool.bgremoval.label",
        },
    ]
}

/// Geometry of the Image Tools action pill row for a given layout.
/// Shared between [`paint_image_action_row`] (paints + hit-registers)
/// and [`image_action_a11y_nodes`] (publishes AccessKit nodes) so the
/// two surfaces can't drift.
///
/// The returned tuples expose only `(NodeId, Rect)` — the icon source
/// (manifest BezPath vs legacy IconId) is an implementation detail
/// hidden from callers since neither downstream cares about it
/// geometrically.
pub(crate) fn image_action_pill_rects(layout: &HeroLayout, gap: f32) -> Vec<(NodeId, Rect)> {
    let row_h = layout.top_bar.h;
    let pill_w = 40.0 + PILL_PADDING_PX * 2.0; // LITERAL-PX-OK: TopBar action pill base width (chrome dim)
    let pills = image_action_pills();
    let total_w = pill_w * pills.len() as f32 + gap * pills.len().saturating_sub(1) as f32;
    let start_x = layout.top_bar.x + layout.top_bar.w - total_w;
    let mut rx = start_x;
    let mut out = Vec::with_capacity(pills.len());
    for pill in &pills {
        let rect = Rect::new(rx, layout.top_bar.y, pill_w, row_h);
        out.push((pill.id, rect));
        rx = rect.x + rect.w + gap;
    }
    out
}

/// AccessKit nodes for the Image Tools action pills (HR-12). Returns
/// one `Node` per visible action: `Role::Button` + i18n label +
/// bounds + `Action::Click`. Mirrors the canonical shape from
/// [`crate::widget::Button::build_a11y`].
///
/// The desktop shell hasn't wired the AccessKit `TreeUpdate` pipeline
/// yet (M14.x scope), so this is currently a structural surface that
/// tests assert against. When the shell wires AccessKit, it inserts
/// these as children of the root Window node returned by
/// [`super::HeroScreen::build_a11y`].
pub fn image_action_a11y_nodes(
    layout: &HeroLayout,
    image_tools_mode: bool,
    gap: f32,
) -> Vec<(NodeId, ph2d_a11y::Node)> {
    use ph2d_a11y::{Action, NodeBuilder, Role};
    if !image_tools_mode {
        return Vec::new();
    }
    let pills = image_action_pills();
    let rects = image_action_pill_rects(layout, gap);
    rects
        .into_iter()
        .zip(pills.iter())
        .map(|((id, rect), pill)| {
            let node = NodeBuilder::new(Role::Button)
                .label(ph2d_i18n::tr(pill.label_key))
                .bounds(rect.x as f64, rect.y as f64, rect.w as f64, rect.h as f64)
                .focusable(true)
                .action(Action::Click)
                .build();
            (id, node)
        })
        .collect()
}

/// Cluster-level painter + width helper. Extracted to a sibling
/// file in Wave 2 PR 11.7c so this `mod.rs` stays under the HR-18
/// 600-LOC cap. Same private surface as before; both functions
/// remain `pub(super)`-callable from `paint_top_bar`.
mod cluster_painter;
use cluster_painter::{cluster_width, paint_top_bar_cluster};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screens::hero::style::{EDGE_PAD, TOPBAR_GAP, TOPBAR_H};
    use crate::zones::Rect;

    fn sample_layout() -> HeroLayout {
        // Minimal HeroLayout fixture — only `top_bar` is consulted by
        // the image-action pill geometry, the rest can stay default.
        let viewport = Rect::new(0.0, 0.0, 1366.0, 1024.0);
        HeroLayout::for_viewport(viewport)
    }

    /// HR-12 enforcement for the Image Tools row. Both Trim and Make
    /// Square pills must expose `Role::Button` + non-empty label +
    /// `Action::Click` + bounds matching the painted rect. Locked-in so
    /// future actions added to the row inherit the same a11y contract.
    #[test]
    fn image_action_a11y_nodes_match_paint_rects() {
        let layout = sample_layout();
        let gap = TOPBAR_GAP; // matches paint_top_bar's gap
        let _ = (EDGE_PAD, TOPBAR_H); // silence unused-import lints if absent
        let rects = image_action_pill_rects(&layout, gap);
        let nodes = image_action_a11y_nodes(&layout, true, gap);

        // Same length, same NodeIds in the same order.
        assert_eq!(rects.len(), nodes.len());
        for ((rect_id, rect), (node_id, node)) in rects.iter().zip(nodes.iter()) {
            assert_eq!(rect_id, node_id);
            // Label is non-empty (i18n stub round-tripped through tr()).
            assert!(
                !node.label().unwrap_or("").is_empty(),
                "node for {rect_id:?} has empty label"
            );
            // Bounds match the painted rect.
            let b = node.bounds().expect("button node must carry bounds");
            assert!(
                (b.x0 - rect.x as f64).abs() < 1e-3
                    && (b.y0 - rect.y as f64).abs() < 1e-3
                    && (b.x1 - (rect.x + rect.w) as f64).abs() < 1e-3
                    && (b.y1 - (rect.y + rect.h) as f64).abs() < 1e-3,
                "bounds mismatch for {rect_id:?}: a11y={b:?} paint={rect:?}",
            );
        }
    }

    /// When image_tools_mode is off the row isn't painted, so no a11y
    /// nodes should be published either.
    #[test]
    fn image_action_a11y_empty_when_mode_off() {
        let layout = sample_layout();
        assert!(image_action_a11y_nodes(&layout, false, 16.0).is_empty());
    }
}
