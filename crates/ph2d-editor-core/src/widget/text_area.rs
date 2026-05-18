//! [`TextArea`] — multiline text field.
//!
//! Same border/focus/error palette as [`super::text_input::TextInput`]
//! but reserves at least 3 rows of vertical space and renders each
//! hard-wrapped (`\n`) line separately.
//!
//! Caret + selection rendering uses the same real-measurement pattern
//! as [`super::text_input::paint_text_input_with_buffer`] (see
//! `docs/UI_Bugs/README.md` §3.3) — char-count approximations land
//! between glyphs on proportional fonts.

use crate::paint::{fill_rounded_rect, paint_text, rect_to_vello, resolve, stroke_rounded_rect};
use crate::widget::text_input::{TextInputState, border_token, fill_token};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Clone, Debug)]
pub struct TextArea {
    pub id: NodeId,
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub state: TextInputState,
}

impl TextArea {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            value: String::new(),
            placeholder: String::new(),
            state: TextInputState::Normal,
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
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
        NodeBuilder::new(Role::MultilineTextInput)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != TextInputState::Disabled)
            .action(Action::Focus)
            .build()
    }
}

/// Suggested minimum height = 3 rows at body font size.
pub fn min_height(font_size: f32) -> f32 {
    font_size * 3.0 + Spacing::Md.px() * 2.0 // LITERAL-PX-OK: 3 rows of body text (row count)
}

pub fn paint_text_area(
    area: &TextArea,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_text_area_with_state(area, None, None, rect, scene, text_system, theme);
}

/// Like [`paint_text_area`] but draws a caret line and (optionally) a
/// selection range when focused. Pass `caret = None` /
/// `selection_anchor = None` for the unfocused case.
///
/// Multi-line aware: the painter splits `value` on `\n`, walks each
/// line, and uses the real text-system layout width of the prefix
/// inside the caret's line to position the caret between glyphs (not
/// on top of them — see `docs/UI_Bugs/README.md` §3.3).
#[allow(clippy::too_many_arguments)]
pub fn paint_text_area_with_state(
    area: &TextArea,
    caret: Option<usize>,
    selection_anchor: Option<usize>,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Sm.px();
    fill_rounded_rect(scene, rect, radius, resolve(fill_token(area.state), theme));
    let stroke_w = if area.state == TextInputState::Focused {
        2.0
    } else {
        1.0
    };
    stroke_rounded_rect(
        scene,
        rect,
        radius,
        stroke_w,
        resolve(border_token(area.state), theme),
    );

    let pad_x = Spacing::Lg.px();
    let pad_y = Spacing::Md.px();
    let inner_x = rect.x + pad_x;
    let inner_y = rect.y + pad_y;
    let inner_w = (rect.w - pad_x * 2.0).max(0.0);
    let inner_right = inner_x + inner_w;
    let font_size = TypeToken::Base.px();
    let line_h = font_size + Spacing::Xs.px();

    let focused = area.state == TextInputState::Focused;
    let displayed = area.value.as_str();

    // Selection background per affected line — drawn before the
    // text so glyphs sit on top. For multi-line selections the rect
    // extends to the right edge on intermediate lines (standard
    // text-editor convention).
    if focused
        && !displayed.is_empty()
        && let Some(anchor) = selection_anchor
        && let Some(c) = caret
        && anchor != c
    {
        let (mut s, mut e) = if anchor < c { (anchor, c) } else { (c, anchor) };
        s = s.min(displayed.len());
        e = e.min(displayed.len());

        let mut line_start: usize = 0;
        for (line_idx, line) in displayed.split('\n').enumerate() {
            let line_end = line_start + line.len();
            if e >= line_start && s <= line_end {
                let local_s = s.saturating_sub(line_start).min(line.len());
                let local_e = (e - line_start).min(line.len());
                let prefix_w = text_system.prefix_width(&line[..local_s], font_size);
                let mid_w = if local_s == local_e {
                    if e > line_end {
                        inner_right - (inner_x + prefix_w)
                    } else {
                        0.0
                    }
                } else {
                    let measured = text_system.prefix_width(&line[local_s..local_e], font_size);
                    if e > line_end {
                        (inner_right - (inner_x + prefix_w)).max(measured)
                    } else {
                        measured
                    }
                };
                let sel_x = (inner_x + prefix_w).min(inner_right);
                let sel_w = mid_w.min(inner_right - sel_x);
                if sel_w > 0.0 {
                    let sel_y = inner_y + line_idx as f32 * line_h;
                    let sel_rect = Rect::new(sel_x, sel_y, sel_w, line_h);
                    fill_rounded_rect(scene, sel_rect, 1.0, resolve(ColorToken::AccentSoft, theme));
                }
            }
            line_start = line_end + 1; // +1 for the '\n'
        }
    }

    if displayed.is_empty() && !area.placeholder.is_empty() {
        paint_text(
            text_system,
            scene,
            &area.placeholder,
            inner_x,
            inner_y,
            font_size,
            inner_w,
            resolve(ColorToken::Text3, theme),
        );
    } else if !displayed.is_empty() {
        let color = if area.state == TextInputState::Disabled {
            ColorToken::TextDisabled
        } else {
            ColorToken::Text1
        };
        // Paint each line individually so the caret/selection math
        // (which uses `\n`-split offsets) stays in lockstep with the
        // visible glyph layout. Using a single paint_text call would
        // let parley insert its own line breaks for long lines,
        // desyncing the caret.
        for (i, line) in displayed.split('\n').enumerate() {
            paint_text(
                text_system,
                scene,
                line,
                inner_x,
                inner_y + i as f32 * line_h,
                font_size,
                inner_w,
                resolve(color, theme),
            );
        }
    }

    if focused {
        let caret_byte = caret.unwrap_or(0).min(displayed.len());
        let mut line_start: usize = 0;
        let mut line_idx: usize = 0;
        let mut line_text: &str = "";
        for line in displayed.split('\n') {
            let line_end = line_start + line.len();
            if caret_byte <= line_end {
                line_text = line;
                break;
            }
            line_start = line_end + 1;
            line_idx += 1;
        }
        let local = caret_byte.saturating_sub(line_start).min(line_text.len());
        let prefix = &line_text[..local];
        let prefix_w = if prefix.is_empty() {
            0.0
        } else {
            text_system.prefix_width(prefix, font_size)
        };
        let caret_x = (inner_x + prefix_w).min(inner_right);
        let caret_y = inner_y + line_idx as f32 * line_h;
        let caret_rect = Rect::new(
            caret_x,
            caret_y,
            StrokeToken::Default.px(),
            (line_h - 2.0).max(2.0),
        );
        scene.fill_rect(
            rect_to_vello(caret_rect),
            resolve(ColorToken::Accent, theme),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let a = TextArea::new(NodeId(1), "Notes");
        assert_eq!(a.value, "");
        assert_eq!(a.state, TextInputState::Normal);
    }

    #[test]
    fn min_height_is_3_rows_plus_padding() {
        let h = min_height(14.0);
        assert!(h >= 14.0 * 3.0);
    }

    #[test]
    fn a11y_role_is_multiline_text_input() {
        let node = TextArea::new(NodeId(1), "x").build_a11y(0.0, 0.0, 200.0, 80.0);
        assert_eq!(node.role(), Role::MultilineTextInput);
    }

    fn smoke(area: TextArea, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_text_area(
            &area,
            Rect::new(0.0, 0.0, 240.0, 96.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_empty_with_placeholder() {
        smoke(
            TextArea::new(NodeId(1), "x").placeholder("Notes…"),
            Theme::Forge,
        );
    }

    #[test]
    fn paint_smoke_filled_focused() {
        smoke(
            TextArea::new(NodeId(1), "x")
                .value("multi\nline\nvalue")
                .state(TextInputState::Focused),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(
            TextArea::new(NodeId(1), "x")
                .value("read-only")
                .state(TextInputState::Disabled),
            Theme::Blueprint,
        );
    }
}
