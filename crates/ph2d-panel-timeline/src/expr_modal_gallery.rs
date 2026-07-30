//! **A coluna da GALERIA do card de expressões** — a lista de famílias, as receitas de
//! uma família, e as recusas com roteamento.
//!
//! Split de `expr_modal_columns.rs` quando as duas colunas de knob o levaram a 623 > 600
//! LOC. O corte é pela COLUNA, que é a divisão que o nome do arquivo original já anunciava:
//! *onde o artista ESCOLHE* contra *onde ele AJUSTA*.

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{TextInput, TextInputState, paint_text_input_with_buffer};
use ph2d_editor_core::zones::Rect;
use ph2d_expr_recipes::{CATALOG, Family, SearchHit, search};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};

use crate::expr_modal::{ExprModal, GalleryPage};
use crate::expr_modal_columns::expr_button;
use crate::expr_modal_paint::GALLERY_W;
use crate::ids;

/// The left column: the search field, then either the families or one family's
/// recipes (plus any refusal cards a query surfaced).
pub(crate) fn paint_gallery(
    m: &ExprModal,
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    body_slots: usize,
) {
    let font = TypeToken::Sm.px();
    let mut cy = y;

    // Search field — a TextInput seeded ONCE (re-seeding every frame would stomp
    // the artist's typing, the lesson `expr_edit` already paid for).
    let field = Rect::new(x, cy, GALLERY_W, ROW_H_PX);
    ctx.host.store_mut().register_if_absent(
        ids::EXPR_MODAL_SEARCH,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    let (ti_state, text, caret, anchor) = match ctx.host.store().get(ids::EXPR_MODAL_SEARCH) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, text.clone(), *caret, *selection_anchor),
        _ => (TextInputState::Normal, String::new(), 0, None),
    };
    let input = TextInput::new(
        ids::EXPR_MODAL_SEARCH,
        ph2d_i18n::tr("panel.timeline.expr_search"),
    )
    .state(ti_state);
    paint_text_input_with_buffer(
        &input,
        Some(text.as_str()),
        Some(caret),
        anchor,
        field,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host
        .hit_index_mut()
        .register(ids::EXPR_MODAL_SEARCH, field);
    cy += ROW_H_PX;

    let slots = body_slots - 1;
    let query = text.trim().to_string();

    if !query.is_empty() {
        // A query flattens the gallery: recipes first, then the refusal cards
        // that route to where a refused idea actually lives.
        let hits = search(&query);
        let shown = hits.len().min(slots);
        for h in hits.iter().take(shown) {
            match h {
                SearchHit::Recipe(r) => {
                    let id = ids::expr_gallery_id(r.id);
                    expr_button(
                        ctx,
                        theme,
                        id,
                        r.label,
                        Rect::new(x, cy, GALLERY_W, ROW_H_PX),
                    );
                }
                SearchHit::Refusal(rf) => {
                    let id = ids::expr_refusal_id(rf.key);
                    let label = format!("{} -> {}", rf.title, rf.to.label());
                    expr_button(
                        ctx,
                        theme,
                        id,
                        &label,
                        Rect::new(x, cy, GALLERY_W, ROW_H_PX),
                    );
                }
            }
            cy += ROW_H_PX;
        }
        if hits.len() > shown {
            // ⚠️ Named, never silent: a list that quietly stops reads as "there is
            // nothing else", which is the one thing it must not say.
            paint_text(
                ctx.text_system,
                ctx.scene,
                &format!("+{} more", hits.len() - shown),
                x + Spacing::Sm.px(),
                cy + (ROW_H_PX - font) * 0.5,
                font,
                GALLERY_W,
                resolve(ColorToken::Text2, theme),
            );
        }
        return;
    }

    match m.page {
        GalleryPage::Families => {
            for f in Family::ALL {
                let n = CATALOG.iter().filter(|r| r.family == f).count();
                let id = ids::expr_gallery_id(f.label());
                let label = format!("{}  ({n})", f.label());
                expr_button(
                    ctx,
                    theme,
                    id,
                    &label,
                    Rect::new(x, cy, GALLERY_W, ROW_H_PX),
                );
                cy += ROW_H_PX;
            }
        }
        GalleryPage::Family(f) => {
            let id = ids::expr_gallery_id("..");
            expr_button(
                ctx,
                theme,
                id,
                "< All",
                Rect::new(x, cy, GALLERY_W, ROW_H_PX),
            );
            cy += ROW_H_PX;
            for r in CATALOG.iter().filter(|r| r.family == f) {
                let id = ids::expr_gallery_id(r.id);
                expr_button(
                    ctx,
                    theme,
                    id,
                    r.label,
                    Rect::new(x, cy, GALLERY_W, ROW_H_PX),
                );
                cy += ROW_H_PX;
            }
        }
    }
}
