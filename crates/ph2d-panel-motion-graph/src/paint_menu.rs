//! The add-node popup's DRAW (Motion Nodes M1.E7) — sibling of `paint` (panel LOC
//! cap), and the counterpart of `interact_menu`, which owns its gestures.

use super::{
    MENU_DOT_R, MENU_DOT_X, MENU_HEADER_PAD_X, MENU_HEADER_PAD_Y, MENU_HEADER_SIZE, MENU_RADIUS,
    MENU_ROW_SIZE, MENU_ROW_TEXT_INSET_R, MENU_ROW_TEXT_X, MENU_ROW_TEXT_Y, cat_token,
};
use crate::geom;
use crate::snapshot::current_catalog;
use crate::state::AddMenu;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text_title, rect_to_vello, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};

/// The add-node popup: a `Bg2` panel with a header + one row per catalog entry,
/// each a category-tinted dot + the English display name. Hit-tested in
/// `interact` against `Background` gestures via `geom::add_menu_row`.
pub(super) fn draw_add_menu(ctx: &mut PaintCtx, menu: &AddMenu, canvas: Rect, theme: Theme) {
    let catalog = current_catalog();
    let panel = geom::add_menu_panel(menu, catalog.len(), canvas);
    fill_rounded_rect(
        ctx.scene,
        panel,
        MENU_RADIUS,
        resolve(ColorToken::Bg2, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        panel,
        MENU_RADIUS,
        1.0,
        resolve(ColorToken::Border, theme),
    );
    paint_text_title(
        ctx.text_system,
        ctx.scene,
        "Add Node",
        panel.x + geom::MENU_PAD + MENU_HEADER_PAD_X,
        panel.y + MENU_HEADER_PAD_Y,
        MENU_HEADER_SIZE,
        panel.w - 2.0 * geom::MENU_PAD,
        resolve(ColorToken::Text2, theme),
    );
    // The list SCROLLS inside the panel (86 node types do not fit on a screen). It is clipped to
    // its own band, so a row scrolled half-way out is drawn half — and hit-tested half, against
    // the same rect (`geom::add_menu_list`), because the row you can see is the row you can click.
    let list = geom::add_menu_list(panel);
    ctx.scene.push_clip(&rect_to_vello(list));
    for (i, c) in catalog.iter().enumerate() {
        let row = geom::add_menu_row(panel, i, menu.scroll);
        // Rows entirely outside the band are not drawn at all: with 86 of them, most of the menu
        // is off-list at any moment, and Vello charges per draw object (doc 53).
        if row.y + row.h < list.y || row.y > list.y + list.h {
            continue;
        }
        fill_circle(
            ctx.scene,
            row.x + MENU_DOT_X,
            row.y + row.h * 0.5,
            MENU_DOT_R,
            resolve(cat_token(c.category), theme),
        );
        paint_text_title(
            ctx.text_system,
            ctx.scene,
            c.display,
            row.x + MENU_ROW_TEXT_X,
            row.y + MENU_ROW_TEXT_Y,
            MENU_ROW_SIZE,
            row.w - MENU_ROW_TEXT_INSET_R,
            resolve(ColorToken::Text1, theme),
        );
    }
    ctx.scene.pop_layer();

    // The scrollbar, and only when the list can actually scroll: a scrollbar on a list that fits
    // is a control that lies about there being more.
    if let (Some(track), Some(thumb)) = (
        geom::add_menu_track(panel, catalog.len()),
        geom::add_menu_thumb(panel, catalog.len(), menu.scroll),
    ) {
        fill_rounded_rect(
            ctx.scene,
            track,
            geom::MENU_BAR_W * 0.5,
            resolve(ColorToken::Bg3, theme),
        );
        fill_rounded_rect(
            ctx.scene,
            thumb,
            geom::MENU_BAR_W * 0.5,
            resolve(ColorToken::BorderStrong, theme),
        );
    }
}
