//! O popover da lista de cenas (Scene List): a busca em cima e até oito cenas filtradas; saiu do
//! [`super::context_menu_overlay`] no corte de LOC da integração de 13/11.

use super::context_menu_overlay::{ROW_H, clamp_to_viewport, pad_y};
use super::fixture;
use crate::ids;
use crate::interaction::{HitIndex, InteractiveState, WidgetStore};
use crate::paint::paint_text;
use crate::widget::lista_flutuante::Superficie;
use crate::widget::{TextInput, paint_text_input_with_buffer};
use crate::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Paint the Scene List popover.
///
/// Layout: TextInput search field at the top + up to 8 filtered
/// scene rows below. The search uses the regular TextInput state at
/// `CTX_SCENE_SEARCH` (its buffer is the filter query). Each visible
/// result row registers a unique `CTX_SCENE_ROWS[i]` hit so the
/// dispatch / `apply_event` chain can route the selection.
pub(super) fn paint_scene_list(
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
    let total_h = pad_y() * 2.0 + search_h + Spacing::Xs.px() + row_count as f32 * row_h;
    let rect = clamp_to_viewport(req.x, req.y, menu_w, total_h, viewport);

    let superficie = Superficie::de(theme);
    superficie.moldura(scene, rect);

    // Search input row.
    let inner_x = rect.x + Spacing::Xs.px();
    let inner_w = rect.w - Spacing::Xs.px() * 2.0;
    let search_rect = Rect::new(inner_x, rect.y + pad_y(), inner_w, search_h);
    hit_index.register(ids::CTX_SCENE_SEARCH, search_rect);
    let ti = TextInput::new(ids::CTX_SCENE_SEARCH, "")
        .placeholder(tr("chrome.menu.search_scenes"))
        .visual((ti_state, store.hover_live(ids::CTX_SCENE_SEARCH)));
    let pinta_busca = |scene: &mut VectorScene, text_system: &mut TextSystem| {
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
    };
    match superficie.tinta() {
        Some(ink) => crate::widget::card_ink::com(ink, || pinta_busca(scene, text_system)),
        None => pinta_busca(scene, text_system),
    }

    // Result rows.
    let rows_y0 = search_rect.y + search_rect.h + Spacing::Xs.px();
    let pad_x = Spacing::Md.px();
    let font = TypeToken::Sm.px();
    if filtered.is_empty() {
        let r = Rect::new(inner_x, rows_y0, inner_w, row_h);
        paint_text(
            text_system,
            scene,
            ph2d_i18n::tr("chrome.no_matches"),
            r.x + pad_x,
            r.y + (r.h - font) * 0.5,
            font,
            r.w - pad_x * 2.0,
            superficie.cor(ColorToken::Text3),
        );
    } else {
        for (i, name) in filtered.iter().enumerate() {
            let row_id = ids::CTX_SCENE_ROWS[i];
            let r = Rect::new(inner_x, rows_y0 + i as f32 * row_h, inner_w, row_h);
            hit_index.register(row_id, r);
            if Some(row_id) == store.hot_id() {
                superficie.realce(scene, r);
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
                    &ph2d_vector::Brush::Solid(superficie.cor(ColorToken::Accent)),
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
                superficie.cor(if is_current {
                    ColorToken::Text1
                } else {
                    ColorToken::Text2
                }),
            );
        }
    }
}
