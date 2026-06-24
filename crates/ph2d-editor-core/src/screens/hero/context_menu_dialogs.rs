//! Centered, dialog-style context-menu popovers (split from [`super::context_menu_overlay`] for the
//! file-LOC cap): a title + hint + single `TextInput` + one accent CTA, centered in the viewport.
//! Shared by the Settings → API-key popover, the LLM vector-prompt dialog, and the color-picker
//! palette-rename modal. Centered (not anchored) because their openers sit at the far-right edge;
//! focus + keyboard + paste use the standard widget dispatch, and `dispatch::pointer`'s menu-close
//! guard keeps the popover open while the user types into the field.

use super::context_menu_overlay::{PAD_Y, ROW_H};
use super::ids;
use crate::interaction::{HitIndex, InteractiveState, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use crate::widget::{Button, ButtonState, TextInput, paint_button, paint_text_input_with_buffer};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Paint the Settings → Anthropic API-key popover (P4, ADR-0061).
pub(super) fn paint_api_key_submenu(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    viewport: Rect,
) {
    paint_centered_input_dialog(
        CenteredInputDialog {
            title: "Anthropic API Key",
            hint: "Paste your key (sk-ant-\u{2026}), then Save.",
            input_id: ids::CTX_MENU_API_KEY_INPUT,
            placeholder: "sk-ant-\u{2026}",
            button_id: ids::CTX_MENU_API_KEY_SAVE,
            button_label: "Save",
        },
        scene,
        text_system,
        theme,
        hit_index,
        store,
        viewport,
    );
}

/// Paint the LLM vector prompt dialog (P4, ADR-0061).
pub(super) fn paint_vector_prompt_dialog(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    viewport: Rect,
) {
    paint_centered_input_dialog(
        CenteredInputDialog {
            title: "Generate Vector Shape",
            hint: "Describe a shape, then Generate (e.g. a six-pointed star).",
            input_id: ids::CTX_MENU_VECTOR_PROMPT_INPUT,
            placeholder: "a six-pointed star\u{2026}",
            button_id: ids::CTX_MENU_VECTOR_PROMPT_GENERATE,
            button_label: "Generate",
        },
        scene,
        text_system,
        theme,
        hit_index,
        store,
        viewport,
    );
}

/// Paint the color-picker palette rename modal — the shared `BLENDER_PALETTE_NAME` field doubles
/// as the dialog input (so Enter routes through the existing rename handler).
pub(super) fn paint_palette_rename_dialog(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    viewport: Rect,
) {
    paint_centered_input_dialog(
        CenteredInputDialog {
            title: "Rename Palette",
            hint: "Enter a new name, then Rename (or press Enter).",
            input_id: ids::BLENDER_PALETTE_NAME,
            placeholder: "Palette name\u{2026}",
            button_id: ids::CTX_MENU_PALETTE_RENAME,
            button_label: "Rename",
        },
        scene,
        text_system,
        theme,
        hit_index,
        store,
        viewport,
    );
}

/// The text/ids that distinguish one centered input dialog from another.
struct CenteredInputDialog {
    title: &'static str,
    hint: &'static str,
    input_id: NodeId,
    placeholder: &'static str,
    button_id: NodeId,
    button_label: &'static str,
}

/// Paint a centered, dialog-style popover: a title + a hint + a single `TextInput` + one accent CTA
/// button. Shared by the API-key, vector-prompt and palette-rename popovers.
fn paint_centered_input_dialog(
    d: CenteredInputDialog,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    viewport: Rect,
) {
    let menu_w = 320.0_f32; // LITERAL-PX-OK: dialog width (long key / prompt string)
    let field_h = 30.0_f32; // LITERAL-PX-OK: input height (matches scene-list search)
    let title_h = ROW_H;
    let hint_h = ROW_H;
    let button_h = ROW_H;
    let gap = Spacing::Xs.px();

    // Current field buffer + caret/selection from the TextInput state.
    let (buffer, caret, anchor, ti_state) = match store.get(d.input_id) {
        Some(InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            state,
        }) => (text.as_str(), *caret, *selection_anchor, *state),
        _ => ("", 0, None, crate::widget::TextInputState::Normal),
    };

    let total_h = PAD_Y * 2.0 + title_h + hint_h + field_h + button_h + gap * 3.0; // LITERAL-PX-OK: 3 inter-row gaps between the 4 stacked dialog rows
    let rect_x = (viewport.x + (viewport.w - menu_w) * 0.5).max(viewport.x);
    let rect_y = (viewport.y + (viewport.h - total_h) * 0.5).max(viewport.y);
    let rect = Rect::new(rect_x, rect_y, menu_w, total_h);

    // Floating panel surface.
    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    let inner_x = rect.x + Spacing::Xs.px();
    let inner_w = rect.w - Spacing::Xs.px() * 2.0;
    let pad_x = Spacing::Md.px();
    let font = TypeToken::Sm.px();

    // Title.
    let mut y = rect.y + PAD_Y;
    paint_text(
        text_system,
        scene,
        d.title,
        inner_x + pad_x,
        y + (title_h - font) * 0.5,
        font,
        inner_w - pad_x * 2.0,
        resolve(ColorToken::Text1, theme),
    );
    y += title_h + gap;

    // Hint.
    paint_text(
        text_system,
        scene,
        d.hint,
        inner_x + pad_x,
        y + (hint_h - font) * 0.5,
        font,
        inner_w - pad_x * 2.0,
        resolve(ColorToken::Text3, theme),
    );
    y += hint_h + gap;

    // The input field. Clip its content to the field rect so a long key (or a
    // mistakenly-pasted multi-line blob) can't spill past the field bounds.
    let field_rect = Rect::new(inner_x, y, inner_w, field_h);
    hit_index.register(d.input_id, field_rect);
    let ti = TextInput::new(d.input_id, "")
        .placeholder(d.placeholder)
        .state(ti_state);
    let field_clip = ph2d_vector::Rect::new(
        field_rect.x as f64,
        field_rect.y as f64,
        (field_rect.x + field_rect.w) as f64,
        (field_rect.y + field_rect.h) as f64,
    );
    scene.push_clip(&field_clip);
    paint_text_input_with_buffer(
        &ti,
        Some(buffer),
        Some(caret),
        anchor,
        field_rect,
        scene,
        text_system,
        theme,
    );
    scene.pop_layer();
    y += field_h + gap;

    // The accent CTA via the canonical `paint_button` (token-styled, matches the app).
    let button_rect = Rect::new(inner_x, y, inner_w, button_h);
    hit_index.register(d.button_id, button_rect);
    let btn = Button::new(d.button_id, d.button_label)
        .accent()
        .state(button_state(store, d.button_id));
    paint_button(&btn, button_rect, scene, text_system, theme);
}

/// Hover/press [`ButtonState`] for a menu-row-id painted as a [`Button`], from
/// the store's hot/active tracking.
fn button_state(store: &WidgetStore, id: NodeId) -> ButtonState {
    if store.active_id() == Some(id) {
        ButtonState::Pressed
    } else if store.hot_id() == Some(id) {
        ButtonState::Hovered
    } else {
        ButtonState::Normal
    }
}
