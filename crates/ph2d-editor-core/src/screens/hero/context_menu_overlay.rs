//! Right-click context-menu overlay.
//!
//! Painted by the hero orchestrator after every panel painter so the
//! floating menu lands above everything. Reads the open-menu state
//! from [`crate::interaction::WidgetStore::context_menu`] and walks a
//! per-kind option list (one match arm per [`ContextMenuKind`]).

use super::fixture;
use super::ids;
use crate::icons::IconId;
use crate::interaction::{ContextMenuKind, HitIndex, InteractiveState, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect};
use crate::widget::{TextInput, paint_text_input_with_buffer};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{
    ColorToken, ROW_H_PX, Radius, SECTION_GAP_PX, Spacing, StrokeToken, Theme, TypeToken,
};
use ph2d_vector::{Color as VelloColor, VectorScene};

/// Standard context-menu width. Exported so cascade-opening handlers
/// in `chrome/` can decide whether the submenu fits to the right of
/// its parent row or needs to flip to the left.
pub const MENU_W: f32 = 200.0; // LITERAL-PX-OK: context menu fixed width (chrome-specific)
pub(super) const ROW_H: f32 = ROW_H_PX;
pub(super) const PAD_Y: f32 = Spacing::Sm.px();
const EDGE_INSET: f32 = 4.0; // LITERAL-PX-OK: keep the menu 4px away from the viewport edge

/// Clamp a menu rect anchored at `(anchor_x, anchor_y)` so it stays
/// inside `viewport`. Used by both the simple-row menu and the scene
/// list — cascading submenus (Settings → PPM) anchored near the
/// right edge would otherwise render off-screen.
fn clamp_to_viewport(anchor_x: f32, anchor_y: f32, w: f32, h: f32, viewport: Rect) -> Rect {
    let max_x = (viewport.x + viewport.w - w - EDGE_INSET).max(viewport.x + EDGE_INSET);
    let max_y = (viewport.y + viewport.h - h - EDGE_INSET).max(viewport.y + EDGE_INSET);
    let x = anchor_x.min(max_x).max(viewport.x + EDGE_INSET);
    let y = anchor_y.min(max_y).max(viewport.y + EDGE_INSET);
    Rect::new(x, y, w, h)
}

/// Five common highlighter colors used for note backgrounds and
/// section outlines. Wave 8 Phase 2.A re-export from
/// `ph2d-editor-core::widget::panel_chrome::HIGHLIGHTER_RGBA` so panel
/// crates don't need to reach into `ph2d_editor::screens::hero` to
/// paint user-placed notes.
pub use crate::widget::panel_chrome::HIGHLIGHTER_RGBA;

/// Does `id` correspond to the currently-active choice for any of
/// the single-choice menus? The row paint draws an accent bullet
/// next to the row whose id matches, mirroring SceneList's "current
/// scene" indicator (2026-05-24 menu standardization).
///
/// One function covers every menu because IDs are unique across all
/// menu kinds — checking `id == active_theme_id || id == active_radius
/// _id || ...` is unambiguous and avoids threading `kind` through the
/// row loop.
fn id_is_currently_selected(
    id: NodeId,
    theme: Theme,
    store: &WidgetStore,
    project: &crate::project::ProjectSettings,
) -> bool {
    use crate::project::{DisplayUnit, ImageFilterMode};
    use crate::widget::RailButtonSize;
    let theme_id = match theme {
        Theme::Forge => ids::CTX_MENU_THEME_FORGE,
        Theme::Workshop => ids::CTX_MENU_THEME_PAINT,
        Theme::Sunstone => ids::CTX_MENU_THEME_SUNSTONE,
        Theme::Blueprint => ids::CTX_MENU_THEME_BLUEPRINT,
    };
    if id == theme_id {
        return true;
    }
    const RADIUS_ROUND_THRESH: f32 = 1.3; // LITERAL-PX-OK: midpoint between Default(1.0) and Round(1.5) radius preset
    let radius_id = if store.radius_scale() < 0.5 {
        ids::CTX_MENU_RADIUS_SHARP
    } else if store.radius_scale() > RADIUS_ROUND_THRESH {
        ids::CTX_MENU_RADIUS_ROUND
    } else {
        ids::CTX_MENU_RADIUS_DEFAULT
    };
    if id == radius_id {
        return true;
    }
    let rail_id = match store.rail_button_size() {
        RailButtonSize::Small => ids::CTX_MENU_RAIL_SIZE_SMALL,
        RailButtonSize::Medium => ids::CTX_MENU_RAIL_SIZE_MEDIUM,
        RailButtonSize::Large => ids::CTX_MENU_RAIL_SIZE_LARGE,
    };
    if id == rail_id {
        return true;
    }
    let ppm_id = match project.pixels_per_meter as i32 {
        16 => Some(ids::CTX_MENU_PPM_16),
        32 => Some(ids::CTX_MENU_PPM_32),
        100 => Some(ids::CTX_MENU_PPM_100),
        256 => Some(ids::CTX_MENU_PPM_256),
        1024 => Some(ids::CTX_MENU_PPM_1024),
        _ => None,
    };
    if ppm_id == Some(id) {
        return true;
    }
    let unit_id = match project.display_unit {
        DisplayUnit::Meters => ids::CTX_MENU_UNIT_METERS,
        DisplayUnit::Pixels => ids::CTX_MENU_UNIT_PIXELS,
    };
    if id == unit_id {
        return true;
    }
    let filter_id = match project.image_filter {
        ImageFilterMode::PixelArt => ids::CTX_MENU_FILTER_PIXELART,
        ImageFilterMode::Smooth => ids::CTX_MENU_FILTER_SMOOTH,
    };
    if id == filter_id {
        return true;
    }
    // Text rendering — value lives on the `paint::text_rendering`
    // thread-local (published per-frame from `HeroScreen.text_rendering`),
    // so we can read it here without threading another param through.
    let text_id = match crate::paint::text_rendering() {
        ph2d_tokens::TextRendering::Default => ids::CTX_MENU_TEXT_DEFAULT,
        ph2d_tokens::TextRendering::CrispHeavy => ids::CTX_MENU_TEXT_CRISP_HEAVY,
        ph2d_tokens::TextRendering::CrispHeavyPlus => ids::CTX_MENU_TEXT_CRISP_HEAVY_PLUS,
    };
    if id == text_id {
        return true;
    }
    // Display submenu (VSync / Immediate) — store mirrors the last
    // value `settings_present::apply` published; default `true` matches
    // the shell's `Fifo` baseline.
    let display_id = if store.present_vsync() {
        ids::CTX_MENU_DISPLAY_VSYNC
    } else {
        ids::CTX_MENU_DISPLAY_IMMEDIATE
    };
    if id == display_id {
        return true;
    }
    false
}

/// Paint the open context menu (if any) and register hit rects for
/// each item. Called last in the hero paint pipeline so the menu
/// always sits on top.
///
/// `viewport` is used to clamp the menu rect so a cascade submenu
/// anchored near the right/bottom edge (e.g. Settings → PPM opened
/// from the topbar gear) doesn't render off-screen.
pub fn paint_context_menu_overlay(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    project: &crate::project::ProjectSettings,
    viewport: Rect,
) {
    let Some(req) = store.context_menu() else {
        return;
    };
    let items: &[(NodeId, &str, Option<[u8; 4]>)] = match req.kind {
        ContextMenuKind::CreateNote { .. } => &[(ids::CTX_MENU_CREATE_NOTE, "Create note", None)],
        ContextMenuKind::SectionOutline { .. } => &[
            (ids::CTX_MENU_OUTLINE_NONE, "No outline", None),
            (ids::CTX_MENU_OUTLINE_0, "Yellow", Some(HIGHLIGHTER_RGBA[0])),
            (ids::CTX_MENU_OUTLINE_1, "Pink", Some(HIGHLIGHTER_RGBA[1])),
            (ids::CTX_MENU_OUTLINE_2, "Green", Some(HIGHLIGHTER_RGBA[2])),
            (ids::CTX_MENU_OUTLINE_3, "Blue", Some(HIGHLIGHTER_RGBA[3])),
            (ids::CTX_MENU_OUTLINE_4, "Orange", Some(HIGHLIGHTER_RGBA[4])),
        ],
        // Right-clicked on a note: 5 background-color options
        // (reuses the outline color slot ids; apply_event branches
        // on `last_context_menu.kind` to decide whether to set the
        // section outline or the note bg).
        ContextMenuKind::NoteBackground { .. } => &[
            (ids::CTX_MENU_OUTLINE_0, "Yellow", Some(HIGHLIGHTER_RGBA[0])),
            (ids::CTX_MENU_OUTLINE_1, "Pink", Some(HIGHLIGHTER_RGBA[1])),
            (ids::CTX_MENU_OUTLINE_2, "Green", Some(HIGHLIGHTER_RGBA[2])),
            (ids::CTX_MENU_OUTLINE_3, "Blue", Some(HIGHLIGHTER_RGBA[3])),
            (ids::CTX_MENU_OUTLINE_4, "Orange", Some(HIGHLIGHTER_RGBA[4])),
        ],
        // Topbar theme cluster click: 4 themes + 3 radius presets,
        // separated visually. Theme entries get a small accent
        // swatch tinted with each theme's flavor so the user can
        // recognize them at a glance.
        ContextMenuKind::ThemeSelector => &[
            (
                ids::CTX_MENU_THEME_FORGE,
                "Forge (dark)",
                Some([0xc8, 0x4b, 0xa0, 0xFF]),
            ),
            (
                ids::CTX_MENU_THEME_PAINT,
                "Workshop (dark)",
                Some([0x4b, 0xa0, 0xc8, 0xFF]),
            ),
            (
                ids::CTX_MENU_THEME_SUNSTONE,
                "Sunstone (light)",
                Some([0xf0, 0xc0, 0x4f, 0xFF]),
            ),
            (
                ids::CTX_MENU_THEME_BLUEPRINT,
                "Blueprint (light)",
                Some([0x6c, 0x8e, 0xc8, 0xFF]),
            ),
            (ids::CTX_MENU_RADIUS_SHARP, "— Corners: Sharp", None),
            (ids::CTX_MENU_RADIUS_DEFAULT, "— Corners: Default", None),
            (ids::CTX_MENU_RADIUS_ROUND, "— Corners: Round", None),
            (ids::CTX_MENU_RAIL_SIZE_SMALL, "— Rail Buttons: Small", None),
            (
                ids::CTX_MENU_RAIL_SIZE_MEDIUM,
                "— Rail Buttons: Medium",
                None,
            ),
            (ids::CTX_MENU_RAIL_SIZE_LARGE, "— Rail Buttons: Large", None),
            (ids::CTX_MENU_MIRROR_UI, "— Mirror UI", None),
            (ids::CTX_MENU_SHOW_STATS, "— Show Statistics", None),
            // "Show Grid" removed — Grid Settings panel now owns the
            // grid visibility toggle (Display section "Show grid").
        ],
        ContextMenuKind::SaveMenu => &[
            (ids::CTX_MENU_SAVE, "Save \u{00b7} Cmd+S", None),
            (
                ids::CTX_MENU_SAVE_AS,
                "Save As\u{2026} \u{00b7} Cmd+Shift+S",
                None,
            ),
        ],
        ContextMenuKind::OpenMenu => &[
            (
                ids::CTX_MENU_OPEN_PROJECT,
                "Open Project\u{2026} \u{00b7} Cmd+O",
                None,
            ),
            (
                ids::CTX_MENU_IMPORT,
                "Import\u{2026} \u{00b7} Cmd+Shift+I",
                None,
            ),
        ],
        // Settings cluster (gear) — TOP-LEVEL categories. Each entry
        // gets a real `IconId::ChevronRight` painted at the right edge
        // by the row loop below (matched by `kind == SettingsMenu`),
        // not a Unicode glyph inside the label string. Enio 2026-05-25:
        // "menu e submenus de settings ainda fora do padrão, usando
        // emojis e não ícones".
        ContextMenuKind::SettingsMenu => &[
            (ids::CTX_MENU_SETTINGS_PPM, "Pixels per meter", None),
            (ids::CTX_MENU_SETTINGS_UNIT, "Display unit", None),
            (ids::CTX_MENU_SETTINGS_FILTER, "Image filter", None),
            (ids::CTX_MENU_SETTINGS_DISPLAY, "Display", None),
            (ids::CTX_MENU_SETTINGS_TEXT, "Text rendering", None),
            (
                ids::CTX_MENU_SETTINGS_API_KEY,
                "Anthropic API Key\u{2026}",
                None,
            ),
        ],
        // M14.7 polish (6.3): Pixels-per-meter submenu — same 5
        // presets as before the cascade, just one click deeper.
        //
        // 16   — classic retro tile (Stardew, 16-bit-era)
        // 32   — Unity 2D default (a 32-px tile = 1 world meter)
        // 100  — Godot default (project-level scale that suits
        //        mixed-resolution indie work)
        // 256  — Hand-drawn HD 2D (Cuphead-tier)
        // 1024 — 4K reference sprites (1 quad fits on a 1×1 unit)
        ContextMenuKind::SettingsPpmSubmenu => &[
            (ids::CTX_MENU_PPM_16, "16 (retro tile)", None),
            (ids::CTX_MENU_PPM_32, "32 (Unity 2D)", None),
            (ids::CTX_MENU_PPM_100, "100 (Godot)", None),
            (ids::CTX_MENU_PPM_256, "256 (HD 2D)", None),
            (ids::CTX_MENU_PPM_1024, "1024 (4K ref)", None),
        ],
        ContextMenuKind::SettingsUnitSubmenu => &[
            (ids::CTX_MENU_UNIT_METERS, "Meters", None),
            (ids::CTX_MENU_UNIT_PIXELS, "Pixels", None),
        ],
        // Image-filter submenu — the single global sampling mode
        // applied to every sprite/texture + the Vello preview.
        ContextMenuKind::SettingsFilterSubmenu => &[
            (ids::CTX_MENU_FILTER_PIXELART, "Pixel Art (crisp)", None),
            (ids::CTX_MENU_FILTER_SMOOTH, "Smooth (bilinear)", None),
        ],
        // Display submenu — runtime swap-chain present mode. VSync is
        // perfectly smooth; Immediate is non-blocking (no mouse-stutter)
        // at the cost of vsync-pacing.
        ContextMenuKind::SettingsDisplaySubmenu => &[
            (ids::CTX_MENU_DISPLAY_VSYNC, "VSync (smooth)", None),
            (
                ids::CTX_MENU_DISPLAY_IMMEDIATE,
                "Immediate (no stutter)",
                None,
            ),
        ],
        // Text rendering submenu — 4 presets, monotonic in
        // aggressiveness: Default (historic) → Crisp Light
        // (boost 30/20/10 + snap-X) → Crisp (60/40/20) →
        // Crisp Heavy (100/70/40).
        ContextMenuKind::SettingsTextSubmenu => &[
            (ids::CTX_MENU_TEXT_DEFAULT, "Default", None),
            (ids::CTX_MENU_TEXT_CRISP_HEAVY, "Crisp Heavy", None),
            (ids::CTX_MENU_TEXT_CRISP_HEAVY_PLUS, "Crisp Heavy +", None),
        ],
        // The SceneList kind is rendered by its dedicated branch
        // below — `items` stays empty so the simple-row loop is
        // skipped.
        ContextMenuKind::SceneList => &[],
        // Likewise the API-key submenu + the vector-prompt dialog paint a
        // TextInput + button in their own branches (P4, ADR-0061).
        ContextMenuKind::SettingsApiKeySubmenu => &[],
        ContextMenuKind::VectorPromptDialog => &[],
        // The palette-rename modal paints a TextInput + Rename button in its own branch below.
        ContextMenuKind::RenamePaletteDialog => &[],
        // M14.6 F + M14.7: per-row Hierarchy actions. Order follows
        // the Unity / Godot / Blender convention: Rename first (the
        // most common edit), then additive ops (Duplicate, Add
        // Child), then the milder revert (Reset Transform), with
        // Delete last as the destructive endpoint.
        ContextMenuKind::HierarchyRow { .. } => &[
            (ids::CTX_MENU_HIER_RENAME, "Rename\u{2026}", None),
            (ids::CTX_MENU_HIER_DUPLICATE, "Duplicate", None),
            (ids::CTX_MENU_HIER_ADD_CHILD, "Add Child", None),
            (ids::CTX_MENU_HIER_MERGE_SPRITES, "Merge Sprites", None),
            (ids::CTX_MENU_HIER_RESET_TRANSFORM, "Reset Transform", None),
            (ids::CTX_MENU_HIER_DELETE, "Delete", None),
        ],
        // Vector Direct-Select vertex continuity (Illustrator/Affinity order;
        // no swatch — the Direct overlay glyph IS the affordance, not a menu icon).
        ContextMenuKind::VectorPointType => &[
            (ids::CTX_MENU_POINT_TYPE_CORNER, "Corner", None),
            (ids::CTX_MENU_POINT_TYPE_SMOOTH, "Smooth", None),
            (ids::CTX_MENU_POINT_TYPE_ASYMMETRIC, "Asymmetric", None),
            (ids::CTX_MENU_POINT_TYPE_AUTO, "Auto", None),
        ],
        // Painter Falloff curve point handle (Blender per-point handle types).
        ContextMenuKind::FalloffPointHandle => &[
            (ids::CTX_MENU_FALLOFF_HANDLE_VECTOR, "Vector", None),
            (ids::CTX_MENU_FALLOFF_HANDLE_AUTO, "Auto", None),
        ],
    };

    if matches!(req.kind, ContextMenuKind::SceneList) {
        paint_scene_list(req, scene, text_system, theme, hit_index, store, viewport);
        return;
    }
    if matches!(req.kind, ContextMenuKind::SettingsApiKeySubmenu) {
        super::context_menu_dialogs::paint_api_key_submenu(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            viewport,
        );
        return;
    }
    if matches!(req.kind, ContextMenuKind::VectorPromptDialog) {
        super::context_menu_dialogs::paint_vector_prompt_dialog(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            viewport,
        );
        return;
    }
    if matches!(req.kind, ContextMenuKind::RenamePaletteDialog) {
        super::context_menu_dialogs::paint_palette_rename_dialog(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            viewport,
        );
        return;
    }
    let total_h = ROW_H * items.len() as f32 + PAD_Y * 2.0;
    let rect = clamp_to_viewport(req.x, req.y, MENU_W, total_h, viewport);

    // Floating panel: BgElev fill + Border stroke + Md radius.
    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    // Rows.
    let row_x = rect.x + Spacing::Xs.px();
    let row_w = rect.w - Spacing::Xs.px() * 2.0;
    // Bullet column width — same gap SceneList uses (bullet x +
    // 10 px → text x). Painted on every row so labels align whether
    // the row is current or not.
    let bullet_col_w: f32 = 10.0; // LITERAL-PX-OK: chrome-specific bullet→text gap
    for (i, (id, label, swatch)) in items.iter().enumerate() {
        let r = Rect::new(row_x, rect.y + PAD_Y + ROW_H * i as f32, row_w, ROW_H);
        hit_index.register(*id, r);
        if Some(*id) == store.hot_id() {
            fill_rounded_rect(scene, r, Radius::Sm.px(), resolve(ColorToken::Bg2, theme));
        }
        let pad_x = Spacing::Md.px();
        let icon_size = SECTION_GAP_PX;
        let icon_y = r.y + (r.h - icon_size) * 0.5;
        // Bullet for currently-selected item (SceneList parity,
        // 2026-05-24 menu standardization).
        let is_current = id_is_currently_selected(*id, theme, store, project);
        let bullet_x = r.x + pad_x;
        if is_current {
            let dot = ph2d_vector::Circle::new(
                ph2d_vector::Point::new(bullet_x as f64, (r.y + r.h * 0.5) as f64),
                3.0, // LITERAL-PX-OK: bullet dot radius (chrome accent, matches SceneList)
            );
            scene.inner_mut().fill(
                ph2d_vector::Fill::NonZero,
                ph2d_vector::Affine::IDENTITY,
                &ph2d_vector::Brush::Solid(resolve(ColorToken::Accent, theme)),
                None,
                &dot,
            );
        }
        let glyph_x = bullet_x + bullet_col_w;
        // Leading visual: color swatch for outline picks, "+" icon
        // for create-note. Painted to the right of the bullet column
        // so the bullet always lines up flush-left.
        let has_glyph = swatch.is_some() || matches!(req.kind, ContextMenuKind::CreateNote { .. });
        if let Some(rgba) = swatch {
            let sw = Rect::new(glyph_x, icon_y, icon_size, icon_size);
            fill_rounded_rect(
                scene,
                sw,
                3.0, // LITERAL-PX-OK: context-menu swatch radius (chrome-specific accent)
                VelloColor::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]), // LITERAL-COLOR-OK: user-color — swatch shows an outline-pick rgba, not a theme token
            );
            stroke_rounded_rect(scene, sw, 3.0, 1.0, resolve(ColorToken::Border, theme)); // LITERAL-PX-OK: context-menu swatch radius (chrome-specific accent)
        } else if matches!(req.kind, ContextMenuKind::CreateNote { .. }) {
            paint_icon(
                scene,
                IconId::Add,
                Rect::new(glyph_x, icon_y, icon_size, icon_size),
                resolve(ColorToken::Text2, theme),
                StrokeToken::Default.px(),
            );
        }
        let text_x = if has_glyph {
            glyph_x + icon_size + Spacing::Sm.px()
        } else {
            glyph_x
        };
        let text_y = r.y + (r.h - TypeToken::Sm.px()) * 0.5;
        paint_text(
            text_system,
            scene,
            label,
            text_x,
            text_y,
            TypeToken::Sm.px(),
            (r.x + r.w - text_x - pad_x).max(0.0),
            resolve(
                if is_current {
                    ColorToken::Text1
                } else {
                    ColorToken::Text2
                },
                theme,
            ),
        );
        // Cascade indicator on the SettingsMenu rows — real
        // `IconId::ChevronRight` glyph at the right edge, replacing
        // the Unicode U+25B6 / U+203A trailing character. Enio
        // 2026-05-25: "usando emojis e não ícones".
        if matches!(req.kind, ContextMenuKind::SettingsMenu) {
            let chev = Rect::new(r.x + r.w - pad_x - icon_size, icon_y, icon_size, icon_size);
            paint_icon(
                scene,
                IconId::ChevronRight,
                chev,
                resolve(ColorToken::Text3, theme),
                StrokeToken::Default.px(),
            );
        }
    }
}

/// Paint the Scene List popover.
///
/// Layout: TextInput search field at the top + up to 8 filtered
/// scene rows below. The search uses the regular TextInput state at
/// `CTX_SCENE_SEARCH` (its buffer is the filter query). Each visible
/// result row registers a unique `CTX_SCENE_ROWS[i]` hit so the
/// dispatch / `apply_event` chain can route the selection.
fn paint_scene_list(
    req: crate::interaction::ContextMenuRequest,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    viewport: Rect,
) {
    let menu_w = 260.0_f32; // LITERAL-PX-OK: scene-list popover width (chrome-specific)
    let search_h = 30.0_f32; // LITERAL-PX-OK: scene-list search field height (chrome-specific)
    let row_h = ROW_H;
    let max_rows = ids::CTX_SCENE_ROWS.len();
    // Read the current search query directly from the TextInput
    // state at CTX_SCENE_SEARCH. Empty string when the user hasn't
    // typed anything yet.
    let (query, caret, anchor, ti_state) = match store.get(ids::CTX_SCENE_SEARCH) {
        Some(InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            state,
        }) => (text.as_str(), *caret, *selection_anchor, *state),
        _ => ("", 0, None, crate::widget::TextInputState::Normal),
    };
    // Filter scenes case-insensitively. Empty query matches all.
    let lower_q = query.to_lowercase();
    let filtered: Vec<&'static str> = fixture::scenes()
        .iter()
        .copied()
        .filter(|s| lower_q.is_empty() || s.to_lowercase().contains(&lower_q))
        .take(max_rows)
        .collect();

    let row_count = filtered.len().max(1); // reserve at least one row for "No matches"
    let total_h = PAD_Y * 2.0 + search_h + Spacing::Xs.px() + row_count as f32 * row_h;
    let rect = clamp_to_viewport(req.x, req.y, menu_w, total_h, viewport);

    // Floating panel surface.
    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    // Search input row.
    let inner_x = rect.x + Spacing::Xs.px();
    let inner_w = rect.w - Spacing::Xs.px() * 2.0;
    let search_rect = Rect::new(inner_x, rect.y + PAD_Y, inner_w, search_h);
    hit_index.register(ids::CTX_SCENE_SEARCH, search_rect);
    let ti = TextInput::new(ids::CTX_SCENE_SEARCH, "")
        .placeholder("Search scenes\u{2026}")
        .state(ti_state);
    paint_text_input_with_buffer(
        &ti,
        Some(query),
        Some(caret),
        anchor,
        search_rect,
        scene,
        text_system,
        theme,
    );

    // Result rows.
    let rows_y0 = search_rect.y + search_rect.h + Spacing::Xs.px();
    let pad_x = Spacing::Md.px();
    let font = TypeToken::Sm.px();
    if filtered.is_empty() {
        let r = Rect::new(inner_x, rows_y0, inner_w, row_h);
        paint_text(
            text_system,
            scene,
            "No matches",
            r.x + pad_x,
            r.y + (r.h - font) * 0.5,
            font,
            r.w - pad_x * 2.0,
            resolve(ColorToken::Text3, theme),
        );
    } else {
        for (i, name) in filtered.iter().enumerate() {
            let row_id = ids::CTX_SCENE_ROWS[i];
            let r = Rect::new(inner_x, rows_y0 + i as f32 * row_h, inner_w, row_h);
            hit_index.register(row_id, r);
            if Some(row_id) == store.hot_id() {
                fill_rounded_rect(scene, r, Radius::Sm.px(), resolve(ColorToken::Bg2, theme));
            }
            // Highlight the currently-active scene with an accent
            // bullet so the user sees "this is what's loaded".
            let is_current = *name == store.current_scene_name();
            let bullet_x = r.x + pad_x;
            let bullet_y = r.y + r.h * 0.5;
            if is_current {
                let dot = ph2d_vector::Circle::new(
                    ph2d_vector::Point::new(bullet_x as f64, bullet_y as f64),
                    3.0, // LITERAL-PX-OK: scene-list bullet dot radius (chrome-specific accent)
                );
                scene.inner_mut().fill(
                    ph2d_vector::Fill::NonZero,
                    ph2d_vector::Affine::IDENTITY,
                    &ph2d_vector::Brush::Solid(resolve(ColorToken::Accent, theme)),
                    None,
                    &dot,
                );
            }
            let text_x = bullet_x + 10.0; // LITERAL-PX-OK: bullet → text gap (chrome-specific)
            let text_y = r.y + (r.h - font) * 0.5;
            paint_text(
                text_system,
                scene,
                name,
                text_x,
                text_y,
                font,
                r.w - (text_x - r.x) - pad_x,
                resolve(
                    if is_current {
                        ColorToken::Text1
                    } else {
                        ColorToken::Text2
                    },
                    theme,
                ),
            );
            let _ = VelloColor::from_rgba8(0, 0, 0, 0); // LITERAL-COLOR-OK: import-keepalive no-op (keeps the VelloColor import live for conditional paths)
            let _ = IconId::Search; // keep IconId import in scope
        }
    }
}

// The centered dialog-style popovers (API-key / vector-prompt / palette-rename) live in the sibling
// `super::context_menu_dialogs` (file-LOC split); this module dispatches to them above.
