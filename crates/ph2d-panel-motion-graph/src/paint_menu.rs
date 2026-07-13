//! The add-node popup's DRAW (Motion Nodes M1.E7) — sibling of `paint` (panel LOC
//! cap), and the counterpart of `interact_menu`, which owns its gestures.

use super::{
    MENU_DOT_R, MENU_DOT_X, MENU_HEADER_PAD_X, MENU_HEADER_PAD_Y, MENU_HEADER_SIZE, MENU_RADIUS,
    MENU_ROW_SIZE, MENU_ROW_TEXT_INSET_R, MENU_ROW_TEXT_X, MENU_ROW_TEXT_Y, cat_token,
};
use crate::geom;
use crate::hits::menu_search_id;
use crate::snapshot::{GraphViewSnapshot, menu_rows};
use crate::state::{Menu, MenuBody};
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text_title, rect_to_vello, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{TextInput, TextInputState, paint_text_input_with_buffer};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};

/// What the empty search field says. English (app UI, HR-15).
const SEARCH_PLACEHOLDER: &str = "Search nodes...";

/// The popup: a `Bg2` panel with a header, and one row per entry — a category-tinted dot
/// beside an English label. Hit-tested in `interact` against `Background` gestures via
/// `geom::menu_row`, **against the same [`menu_rows`] this draws** — the paint and the
/// click must enumerate one list, or a row means one thing on screen and another under the
/// cursor.
pub(super) fn draw_menu(
    ctx: &mut PaintCtx,
    menu: &Menu,
    snap: &GraphViewSnapshot,
    canvas: Rect,
    theme: Theme,
) {
    let rows = menu_rows(snap, menu);
    // The header names the question being asked. "Add Node" for the library; for a wire
    // dropped on a collapsed card, the question is which port INSIDE it the wire lands on.
    let header = match menu.body {
        MenuBody::Library { .. } => "Add Node",
        MenuBody::CardPorts { .. } => "Connect Inside Group",
    };
    let panel = geom::menu_panel(menu, rows.len(), canvas);
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
        header,
        panel.x + geom::MENU_PAD + MENU_HEADER_PAD_X,
        panel.y + MENU_HEADER_PAD_Y,
        MENU_HEADER_SIZE,
        panel.w - 2.0 * geom::MENU_PAD,
        resolve(ColorToken::Text2, theme),
    );
    // **The search field** (Enio, smoke 2026-07-13: *"não encontrei value.lfo"*). It is the
    // first thing under the header and it OWNS THE KEYBOARD as soon as the menu opens, so the
    // gesture is: press A, type. The store owns the buffer; `interact` mirrors it into
    // `menu.query` and every row derives from that.
    let field = geom::menu_search_rect(panel);
    let (fstate, text, caret, anchor) = match ctx.host.store().get(menu_search_id()) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, text.clone(), *caret, *selection_anchor),
        _ => (TextInputState::Normal, String::new(), 0, None),
    };
    let input = TextInput::new(menu_search_id(), "")
        .placeholder(SEARCH_PLACEHOLDER)
        .state(fstate);
    paint_text_input_with_buffer(
        &input,
        Some(&text),
        Some(caret),
        anchor,
        field,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host.hit_index_mut().register(menu_search_id(), field);

    // The list SCROLLS inside the panel (86 node types do not fit on a screen). It is clipped to
    // its own band, so a row scrolled half-way out is drawn half — and hit-tested half, against
    // the same rect (`geom::menu_list`), because the row you can see is the row you can click.
    let list = geom::menu_list(panel);
    ctx.scene.push_clip(&rect_to_vello(list));
    for (i, c) in rows.iter().enumerate() {
        let row = geom::menu_row(panel, i, menu.scroll);
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
            c.label,
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
        geom::menu_track(panel, rows.len()),
        geom::menu_thumb(panel, rows.len(), menu.scroll),
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
