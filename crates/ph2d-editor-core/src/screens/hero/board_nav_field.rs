//! O campo de texto do painel de navegação do quadro. Irmão de `board_nav.rs` pelo tecto de LOC.

use super::*;

/// O campo de texto `id` em `r`, com o mesmo pintor dos outros campos.
pub(crate) fn paint_field(
    id: NodeId,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    hit: &mut HitIndex,
    store: &WidgetStore,
) {
    let Some(InteractiveState::TextInput {
        text,
        caret,
        selection_anchor,
        state,
    }) = store.get(id)
    else {
        return;
    };
    hit.register(id, r);
    let field = crate::widget::TextInput::new(id, "").visual((*state, store.hover_live(id)));
    scene.push_clip(&crate::paint::rect_to_vello(r));
    crate::widget::paint_text_input_with_buffer(
        &field,
        Some(text.as_str()),
        Some(*caret),
        *selection_anchor,
        r,
        scene,
        ts,
        theme,
    );
    scene.pop_layer();
}

/// Abre o campo `id` com `text` todo seleccionado e o teclado nele.
pub(crate) fn open_field(store: &mut WidgetStore, id: NodeId, text: String) {
    let caret = text.len();
    store.register(
        id,
        InteractiveState::TextInput {
            state: crate::widget::TextInputState::Focused,
            text,
            caret,
            selection_anchor: Some(0),
        },
    );
    store.set_focus(Some(id));
    store.mark_cancel_on_escape(id);
}

/// O texto do campo `id`.
pub(crate) fn field_text(store: &WidgetStore, id: NodeId) -> String {
    match store.get(id) {
        Some(InteractiveState::TextInput { text, .. }) => text.clone(),
        _ => String::new(),
    }
}

/// O texto de um campo (os testes da busca).
#[cfg(test)]
pub(crate) fn field_text_for_tests(hero: &HeroScreen, id: NodeId) -> String {
    field_text(&hero.store, id)
}
