//! Right-click context-menu overlay.
//!
//! Painted by the hero orchestrator after every panel painter so the
//! floating menu lands above everything. Reads the open-menu state
//! from [`crate::interaction::WidgetStore::context_menu`] and walks a
//! per-kind option list.
//!
//! Two kinds (so far):
//!   - [`ContextMenuKind::CreateNote`] — single "Create note" item.
//!   - [`ContextMenuKind::SectionOutline`] — 6 items: "No outline" +
//!     5 highlighter colors (yellow / pink / green / blue / orange).

use super::fixture;
use super::ids;
use crate::icons::IconId;
use crate::interaction::{ContextMenuKind, HitIndex, InteractiveState, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect};
use crate::widget::{TextInput, paint_text_input_with_buffer};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Color as VelloColor, VectorScene};

const MENU_W: f32 = 200.0;
const ROW_H: f32 = 28.0;
const PAD_Y: f32 = 6.0;

/// Five common highlighter colors. Matches the design brief for note
/// backgrounds and section outlines.
pub const HIGHLIGHTER_RGBA: [[u8; 4]; 5] = [
    [0xFF, 0xF5, 0x9D, 0xFF], // yellow
    [0xF8, 0xBB, 0xD0, 0xFF], // pink
    [0xC8, 0xE6, 0xC9, 0xFF], // green
    [0xBB, 0xDE, 0xFB, 0xFF], // blue
    [0xFF, 0xE0, 0xB2, 0xFF], // orange
];

/// Paint the open context menu (if any) and register hit rects for
/// each item. Called last in the hero paint pipeline so the menu
/// always sits on top.
pub fn paint_context_menu_overlay(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
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
        // ending in "\u{2003}\u{25b6}" (em-space + black right-
        // pointing triangle) opens its dedicated submenu (replaces
        // this menu). U+25B6 IS present in `InterVariable.ttf` —
        // U+25B8 (smaller triangle) is not, which was the earlier
        // tofu the user reported. Future categories (Snap, Theme,
        // etc.) line up here.
        ContextMenuKind::SettingsMenu => &[(
            ids::CTX_MENU_SETTINGS_PPM,
            "Pixels per meter\u{2003}\u{25b6}",
            None,
        )],
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
        // The SceneList kind is rendered by its dedicated branch
        // below — `items` stays empty so the simple-row loop is
        // skipped.
        ContextMenuKind::SceneList => &[],
        // M14.6 F + M14.7: per-row Hierarchy actions. Order follows
        // the Unity / Godot / Blender convention: Rename first (the
        // most common edit), then additive ops (Duplicate, Add
        // Child), then the milder revert (Reset Transform), with
        // Delete last as the destructive endpoint.
        ContextMenuKind::HierarchyRow { .. } => &[
            (ids::CTX_MENU_HIER_RENAME, "Rename\u{2026}", None),
            (ids::CTX_MENU_HIER_DUPLICATE, "Duplicate", None),
            (ids::CTX_MENU_HIER_ADD_CHILD, "Add Child", None),
            (ids::CTX_MENU_HIER_RESET_TRANSFORM, "Reset Transform", None),
            (ids::CTX_MENU_HIER_DELETE, "Delete", None),
        ],
    };

    if matches!(req.kind, ContextMenuKind::SceneList) {
        paint_scene_list(req, scene, text_system, theme, hit_index, store);
        return;
    }
    let total_h = ROW_H * items.len() as f32 + PAD_Y * 2.0;
    let rect = Rect::new(req.x, req.y, MENU_W, total_h);

    // Floating panel: BgElev fill + Border stroke + Md radius.
    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    // Rows.
    let row_x = rect.x + Spacing::Xs.px();
    let row_w = rect.w - Spacing::Xs.px() * 2.0;
    for (i, (id, label, swatch)) in items.iter().enumerate() {
        let r = Rect::new(row_x, rect.y + PAD_Y + ROW_H * i as f32, row_w, ROW_H);
        hit_index.register(*id, r);
        if Some(*id) == store.hot_id() {
            fill_rounded_rect(scene, r, Radius::Sm.px(), resolve(ColorToken::Bg2, theme));
        }
        let pad_x = Spacing::Md.px();
        let icon_size = 14.0_f32;
        let icon_y = r.y + (r.h - icon_size) * 0.5;
        let glyph_x = r.x + pad_x;
        // Leading visual: color swatch for outline picks, "+" icon
        // for create-note. Keeps the menu legible without text alone.
        if let Some(rgba) = swatch {
            let sw = Rect::new(glyph_x, icon_y, icon_size, icon_size);
            fill_rounded_rect(
                scene,
                sw,
                3.0,
                VelloColor::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]),
            );
            stroke_rounded_rect(scene, sw, 3.0, 1.0, resolve(ColorToken::Border, theme));
        } else if matches!(req.kind, ContextMenuKind::CreateNote { .. }) {
            paint_icon(
                scene,
                IconId::Add,
                Rect::new(glyph_x, icon_y, icon_size, icon_size),
                resolve(ColorToken::Text2, theme),
                1.5,
            );
        }
        let text_x = glyph_x + icon_size + Spacing::Sm.px();
        let text_y = r.y + (r.h - TypeToken::Sm.px()) * 0.5;
        paint_text(
            text_system,
            scene,
            label,
            text_x,
            text_y,
            TypeToken::Sm.px(),
            (r.x + r.w - text_x - pad_x).max(0.0),
            resolve(ColorToken::Text1, theme),
        );
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
) {
    let menu_w = 260.0_f32;
    let search_h = 30.0_f32;
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
    let rect = Rect::new(req.x, req.y, menu_w, total_h);

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
                    3.0,
                );
                scene.inner_mut().fill(
                    ph2d_vector::Fill::NonZero,
                    ph2d_vector::Affine::IDENTITY,
                    &ph2d_vector::Brush::Solid(resolve(ColorToken::Accent, theme)),
                    None,
                    &dot,
                );
            }
            let text_x = bullet_x + 10.0;
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
            let _ = VelloColor::from_rgba8(0, 0, 0, 0); // keep VelloColor import in scope
            let _ = IconId::Search; // keep IconId import in scope
        }
    }
}
