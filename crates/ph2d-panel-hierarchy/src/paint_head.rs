//! **O cabeçalho da Hierarquia** — o título, a contagem de entidades e componentes, o botão Add e o
//! campo de busca —, irmão de `paint.rs` pelo tecto de 200 LOC por função
//! (`architecture_panel_loc_cap`).
//!
//! Corte mecânico: o bloco saiu inteiro e verbatim do `paint_hierarchy_body`, e devolve as três
//! coisas que o corpo lê dele — o recuo do corpo, o rect da busca e o texto que se busca.
//!
//! ⚠️ O `placeholder` da busca mudou-se com o bloco, e a entrada dele no `PANEL_BASELINE` do
//! `hr15_no_hardcoded_ui_strings` mudou de endereço no mesmo commit: a dívida é a mesma.

use super::*;

/// Pinta o cabeçalho e devolve `(body_pad, search_rect, search_text)`.
pub(super) fn paint_hierarchy_head(
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) -> (f32, Rect, String) {
    // Canonical panel title (single source of truth — `panel_chrome`).
    // Reserve ≈ICON_BTN_SIZE on the right for the header Add-button.
    let title_y = rect.y + PANEL_TITLE_BASELINE;
    let title_size = paint_panel_title(rect, "Hierarchy", 40.0, scene, text_system, theme); // LITERAL-PX-OK: Add-button reserve
    let (entities, components) = if let Some(live) = current_live_entries() {
        let entity_count = live.len() as u32;
        let comp_count = current_component_count();
        (entity_count, comp_count)
    } else {
        fixture::hierarchy_counts()
    };
    let counts = if components > 0 {
        format!("{entities} entities \u{00b7} {components} components")
    } else {
        format!("{entities} entities")
    };
    paint_text(
        text_system,
        scene,
        &counts,
        rect.x + PANEL_HEAD_PAD,
        title_y + title_size + Spacing::Xs.px(),
        TypeToken::Xs.px() - 1.0,
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text3, theme),
    );

    let add_size = 30.0_f32; // LITERAL-PX-OK: Add button square size (chrome-specific, distinct from ROW_H_PX)
    let add_rect = Rect::new(
        rect.x + rect.w - PANEL_HEAD_PAD - add_size,
        title_y - 2.0,
        add_size,
        add_size,
    );
    hit_index.register(ids::HIERARCHY_ADD, add_rect);
    // Canonical icon button (same ghost-icon look as every other icon
    // button — e.g. panel Close). Was a one-off accent-circle that read
    // as "disabled" (AccentSoft) under the cursor.
    let add_state = store.button_visual(ids::HIERARCHY_ADD);
    let add_btn = widget::Button::new(ids::HIERARCHY_ADD, "Add")
        .icon_only(IconId::Add)
        .visual(add_state);
    widget::paint_button(&add_btn, add_rect, scene, text_system, theme);

    let header_bottom = title_y + TypeToken::Md.px() + TypeToken::Xs.px() + 18.0; // LITERAL-PX-OK: header baseline composite
    // Body padding reduced (Md → Xs) 2026-05-24 per user feedback
    // ("nomes mais próximos da borda esquerda do painel (quase sem
    // padding)") — Godot's Scene panel sits text flush with the
    // panel chrome. Search field + rows now share this tighter inset.
    let body_pad = Spacing::Xs.px();

    let search_h = ROW_H_PX;
    let search_rect = Rect::new(
        rect.x + body_pad,
        header_bottom,
        (rect.w - body_pad * 2.0).max(0.0),
        search_h,
    );
    hit_index.register(crate::ids::HIER_SEARCH, search_rect);
    let (search_state, search_text, search_caret, search_anchor) =
        match store.get(crate::ids::HIER_SEARCH) {
            Some(InteractiveState::TextInput {
                state,
                text,
                caret,
                selection_anchor,
            }) => (*state, text.clone(), *caret, *selection_anchor),
            _ => (TextInputState::Normal, String::new(), 0, None),
        };
    let search_input = TextInput::new(crate::ids::HIER_SEARCH, "")
        .placeholder("Search\u{2026}")
        .visual((search_state, store.hover_live(crate::ids::HIER_SEARCH)));
    paint_text_input_with_buffer(
        &search_input,
        Some(search_text.as_str()),
        Some(search_caret),
        search_anchor,
        search_rect,
        scene,
        text_system,
        theme,
    );
    (body_pad, search_rect, search_text)
}
