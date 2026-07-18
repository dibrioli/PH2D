//! [`TextInput`] — single-line text field.
//!
//! v1 paints layout-only: caret stays static at `caret_pos`, no IME
//! composing, no selection range. Real input handling lands when the
//! shell wires `winit::Event::KeyboardInput` into the editor (post
//! M13). The data shape here exists so widgets that compose
//! TextInput (NumberInput, Combobox) have a stable contract.

use crate::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Width of the caret bar. The viewport reserves it on the right so a caret at
/// the very end of a scrolled line stays inside the clip instead of landing on
/// its boundary — where it would be trimmed away exactly while you are typing.
const CARET_W: f32 = 1.0; // LITERAL-PX-OK: a hairline caret

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TextInputState {
    #[default]
    Normal,
    Hovered,
    Focused,
    Disabled,
    /// Validation failed; border switches to `Danger`.
    Error,
}

#[derive(Clone, Debug)]
pub struct TextInput {
    pub id: NodeId,
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub state: TextInputState,
    /// Byte offset of the caret within `value`. Out-of-range values
    /// are clamped at paint time. v1 draws the caret only when
    /// `state == Focused`.
    pub caret_byte: usize,
}

impl TextInput {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            value: String::new(),
            placeholder: String::new(),
            state: TextInputState::Normal,
            caret_byte: 0,
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.caret_byte = self.value.len();
        self
    }

    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }

    pub fn state(mut self, state: TextInputState) -> Self {
        self.state = state;
        self
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::TextInput)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != TextInputState::Disabled)
            .action(Action::Focus)
            .build()
    }
}

/// Border tokens chosen by state. Promoted to a free function so
/// `text_area`/`number_input` can reuse the same palette.
pub(crate) fn border_token(state: TextInputState) -> ColorToken {
    match state {
        TextInputState::Disabled => ColorToken::Border,
        TextInputState::Hovered => ColorToken::BorderEmph,
        TextInputState::Focused => ColorToken::Accent,
        TextInputState::Error => ColorToken::Danger,
        TextInputState::Normal => ColorToken::Border,
    }
}

pub(crate) fn fill_token(state: TextInputState) -> ColorToken {
    match state {
        TextInputState::Disabled => ColorToken::Bg2,
        _ => ColorToken::Bg1,
    }
}

pub fn paint_text_input(
    input: &TextInput,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_text_input_with_buffer(input, None, None, None, rect, scene, text_system, theme)
}

/// Like [`paint_text_input`] but draws an override `buffer` and
/// caret offset when the caller has a live
/// [`crate::interaction::WidgetStore`] entry for the input. Reading
/// from the store avoids per-frame allocations that would happen if
/// the caller copied `store.text(id)` into `TextInput.value`.
/// `selection_anchor` is the other end of an active selection (for
/// double-click "select all" + Shift+Arrow); when None, no selection
/// is drawn.
#[allow(clippy::too_many_arguments)]
pub fn paint_text_input_with_buffer(
    input: &TextInput,
    buffer: Option<&str>,
    caret: Option<usize>,
    selection_anchor: Option<usize>,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Sm.px();
    fill_rounded_rect(scene, rect, radius, resolve(fill_token(input.state), theme));
    let stroke_w = if input.state == TextInputState::Focused {
        2.0
    } else {
        1.0
    };
    stroke_rounded_rect(
        scene,
        rect,
        radius,
        stroke_w,
        resolve(border_token(input.state), theme),
    );

    let pad_x = Spacing::Lg.px();
    let pad_y = Spacing::Md.px();
    let font_size = TypeToken::Base.px();
    let inner_x = rect.x + pad_x;
    let inner_y = rect.y + (rect.h - font_size) * 0.5 - pad_y * 0.0;
    let inner_w = (rect.w - pad_x * 2.0).max(0.0);

    let displayed: &str = buffer.unwrap_or(input.value.as_str());
    let displayed_caret = caret.unwrap_or(input.caret_byte);

    // **A text field is one LINE, so it SCROLLS — it does not wrap.** `paint_text`
    // breaks the run at `max_width`, so a name longer than the box grew a second
    // line and spilled out of the field, over whatever sat below it (Enio,
    // 2026-07-16). The line is laid out unbounded (`f32::INFINITY` — the same "do
    // not wrap" value `prefix_width` itself passes), clipped to the inner box, and
    // slid left just far enough to keep the caret inside: the viewport every text
    // field in every toolkit has.
    //
    // Only a FOCUSED field scrolls. With no caret to follow there is nothing to
    // chase, and a reader looking at an unfocused field wants the BEGINNING of the
    // name — scrolling it to the end would hide the part that identifies it.
    let focused = input.state == TextInputState::Focused;
    let caret_w = focused.then(|| {
        text_system.prefix_width(
            &displayed[..displayed_caret.min(displayed.len())],
            font_size,
        )
    });
    let text_x = inner_x - caret_scroll(focused, inner_w, caret_w.unwrap_or(0.0));
    scene.push_clip(&crate::paint::rect_to_vello(Rect::new(
        inner_x, rect.y, inner_w, rect.h,
    )));

    if input.state == TextInputState::Focused
        && let Some(anchor) = selection_anchor
        && anchor != displayed_caret
    {
        let (sel_start, sel_end) = if anchor < displayed_caret {
            (anchor, displayed_caret)
        } else {
            (displayed_caret, anchor)
        };
        let sel_start = sel_start.min(displayed.len());
        let sel_end = sel_end.min(displayed.len());
        let prefix_w = text_system.prefix_width(&displayed[..sel_start], font_size);
        let mid_w = if sel_start == sel_end {
            0.0
        } else {
            text_system.prefix_width(&displayed[sel_start..sel_end], font_size)
        };
        // The highlight rides the same viewport as the glyphs it covers; the clip
        // trims whatever runs past the box, so it needs no clamp of its own.
        let sel_x = text_x + prefix_w;
        let sel_w = mid_w;
        if sel_w > 0.0 {
            let sel_rect = Rect::new(sel_x, rect.y + pad_y, sel_w, rect.h - pad_y * 2.0);
            fill_rounded_rect(scene, sel_rect, 1.0, resolve(ColorToken::AccentSoft, theme));
        }
    }

    if displayed.is_empty() && !input.placeholder.is_empty() {
        paint_text(
            text_system,
            scene,
            &input.placeholder,
            inner_x,
            inner_y,
            font_size,
            f32::INFINITY,
            resolve(ColorToken::Text3, theme),
        );
    } else if !displayed.is_empty() {
        let color = if input.state == TextInputState::Disabled {
            ColorToken::TextDisabled
        } else {
            ColorToken::Text1
        };
        paint_text(
            text_system,
            scene,
            displayed,
            text_x,
            inner_y,
            font_size,
            f32::INFINITY,
            resolve(color, theme),
        );
    }

    if let Some(caret_w) = caret_w {
        let caret_rect = Rect::new(
            text_x + caret_w,
            rect.y + pad_y,
            CARET_W,
            rect.h - pad_y * 2.0,
        );
        scene.fill_rect(
            crate::paint::rect_to_vello(caret_rect),
            resolve(ColorToken::Accent, theme),
        );
    }
    scene.pop_layer();
}

/// How far the single line is slid LEFT so the caret stays inside the box.
///
/// Pure, so the rule can be stated and tested without a scene. Exactly as far as
/// the caret overhangs, plus the caret's own width — a caret parked ON the right
/// boundary is trimmed by the clip precisely while you are typing at the end of
/// the name, which is the one moment you need to see it. An unfocused field never
/// scrolls: there is no caret to follow, and the reader wants the name's start.
fn caret_scroll(focused: bool, inner_w: f32, caret_w: f32) -> f32 {
    if focused {
        (caret_w - (inner_w - CARET_W)).max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> TextInput {
        TextInput::new(NodeId(1), "Project name")
    }

    #[test]
    fn defaults_match_spec() {
        let t = fixture();
        assert_eq!(t.value, "");
        assert_eq!(t.placeholder, "");
        assert_eq!(t.state, TextInputState::Normal);
        assert_eq!(t.caret_byte, 0);
    }

    #[test]
    fn value_seed_moves_caret_to_end() {
        let t = fixture().value("hello");
        assert_eq!(t.value, "hello");
        assert_eq!(t.caret_byte, 5);
    }

    #[test]
    fn a11y_role_is_text_input() {
        let node = fixture().build_a11y(0.0, 0.0, 200.0, 32.0);
        assert_eq!(node.role(), Role::TextInput);
    }

    fn smoke(t: TextInput, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_text_input(
            &t,
            Rect::new(0.0, 0.0, 240.0, 32.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    /// Enio, 2026-07-16: renaming a clip to a long name drew a SECOND line that ran
    /// out of the box and over the buttons below it. A single-line field clips —
    /// and it must CLOSE what it opens, because an unbalanced layer would corrupt
    /// everything painted after the field, not just the field.
    #[test]
    fn a_long_name_is_clipped_to_the_field_instead_of_spilling_out_of_it() {
        let long = "L2 ldldll ldllld dhdhdhhhhd jjdjjjd jdjfjjd";
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let t = fixture().state(TextInputState::Focused);
        paint_text_input_with_buffer(
            &t,
            Some(long),
            Some(long.len()),
            None,
            Rect::new(0.0, 0.0, 140.0, 32.0),
            &mut scene,
            &mut text,
            Theme::Forge,
        );
        let enc = scene.inner().encoding();
        assert!(enc.n_clips >= 1, "the field must clip its text to its box");
        assert_eq!(enc.n_open_clips, 0, "the clip must be closed");
    }

    #[test]
    fn the_line_scrolls_only_far_enough_to_keep_the_caret_in_view() {
        // A caret inside the box leaves the line where it is.
        assert!((caret_scroll(true, 100.0, 40.0)).abs() < f32::EPSILON);
        // Past the right edge it slides exactly as far as it overhangs, plus the
        // caret's own width.
        assert!((caret_scroll(true, 100.0, 140.0) - 41.0).abs() < f32::EPSILON);
        // Unfocused: no caret to chase, so the name reads from its start.
        assert!((caret_scroll(false, 100.0, 140.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn paint_smoke_empty_with_placeholder() {
        smoke(fixture().placeholder("Untitled"), Theme::Forge);
    }

    #[test]
    fn paint_smoke_filled_focused() {
        smoke(
            fixture()
                .value("hello world")
                .state(TextInputState::Focused),
            Theme::Forge,
        );
    }

    #[test]
    fn paint_smoke_hovered() {
        smoke(fixture().state(TextInputState::Hovered), Theme::Sunstone);
    }

    #[test]
    fn paint_smoke_error() {
        smoke(
            fixture().value("oops").state(TextInputState::Error),
            Theme::Blueprint,
        );
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(
            fixture().value("locked").state(TextInputState::Disabled),
            Theme::Workshop,
        );
    }

    #[test]
    fn paint_with_buffer_overrides_value() {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let t = fixture().value("stale").state(TextInputState::Focused);
        // Pretend the WidgetStore has a freshly typed buffer.
        paint_text_input_with_buffer(
            &t,
            Some("live edit"),
            Some(4),
            None,
            Rect::new(0.0, 0.0, 240.0, 32.0),
            &mut scene,
            &mut text,
            Theme::Forge,
        );
    }

    #[test]
    fn paint_with_buffer_handles_empty_caret_oob() {
        // Caret beyond buffer length should still paint without
        // panic (clamped at draw time).
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let t = fixture().state(TextInputState::Focused);
        paint_text_input_with_buffer(
            &t,
            Some(""),
            Some(99),
            None,
            Rect::new(0.0, 0.0, 240.0, 32.0),
            &mut scene,
            &mut text,
            Theme::Sunstone,
        );
    }
}
