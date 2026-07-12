//! The Text param row (Motion Nodes M1.P3 / doc 33) — a free-text field editing a **text
//! param** (a `motion.expression` formula) via the shared single-line `TextInput` widget
//! (never improvising chrome), the string sibling of the Angle/Seed number rows. All
//! keyboard/caret/selection/focus is the store-global dispatch's job; this only
//! registers, mirrors the doc value, and paints.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::{TextInput, TextInputState, paint_text_input_with_buffer};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Register (if absent) + mirror the live doc `value` into the store's `TextInput` when
/// unfocused — the app-standard per-frame sync (mold of `mirror_number`). A focused field
/// owns its buffer (the user is typing), so it is left alone.
pub(crate) fn mirror_text(store: &mut WidgetStore, id: NodeId, value: &str) {
    let _ = store.register_if_absent(
        id,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: value.to_string(),
            caret: value.len(),
            selection_anchor: None,
        },
    );
    if store.focus_id() != Some(id)
        && let Some(InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            ..
        }) = store.get_mut(id)
    {
        if text != value {
            text.clear();
            text.push_str(value);
        }
        *caret = (*caret).min(text.len());
        *selection_anchor = None;
    }
}

/// The field's current text (what a commit reports).
pub(crate) fn text_value(store: &WidgetStore, id: NodeId) -> String {
    store.text(id).unwrap_or_default().to_string()
}

/// `true` while the text field is focused (the user is typing) — the undo bracket holds.
pub(crate) fn text_is_typing(store: &WidgetStore, id: NodeId) -> bool {
    matches!(
        store.get(id),
        Some(InteractiveState::TextInput {
            state: TextInputState::Focused,
            ..
        })
    )
}

/// `label` line, then a full-width formula field below it (a formula needs the room, so
/// this stacks like the Enum row rather than the label-column Angle/Seed rows). Returns
/// the total height consumed.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_text_row(
    rect: Rect,
    label: &str,
    placeholder: &str,
    id: NodeId,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    let label_font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        label,
        rect.x,
        rect.y,
        label_font,
        rect.w,
        resolve(ColorToken::Text2, theme),
    );
    let fy = rect.y + label_font + Spacing::Xs.px();
    let field = Rect::new(rect.x, fy, rect.w, ROW_H_PX);

    let (state, text, caret, anchor) = match store.get(id) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, text.clone(), *caret, *selection_anchor),
        _ => (TextInputState::Normal, String::new(), 0, None),
    };
    let input = TextInput::new(id, "").placeholder(placeholder).state(state);
    paint_text_input_with_buffer(
        &input,
        Some(&text),
        Some(caret),
        anchor,
        field,
        scene,
        text_system,
        theme,
    );
    hit_index.register(id, field);
    label_font + Spacing::Xs.px() + ROW_H_PX
}
