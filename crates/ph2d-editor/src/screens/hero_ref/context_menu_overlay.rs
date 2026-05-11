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

use super::ids;
use crate::icons::IconId;
use crate::interaction::{ContextMenuKind, HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, resolve, stroke_rounded_rect};
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
                "Forge SDF (dark)",
                Some([0xc8, 0x4b, 0xa0, 0xFF]),
            ),
            (
                ids::CTX_MENU_THEME_PAINT,
                "Paint Studio (dark)",
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
        ],
    };
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
