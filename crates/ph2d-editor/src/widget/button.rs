//! [`Button`] — text/icon CTA, four kinds × six states.
//!
//! Same pattern as [`crate::widget::ColorSwatch`]: data + state enum +
//! token-resolved colors + AccessKit `Role::Button` node + colocated
//! [`paint_button`]. Per ADR-0023 §11 the accent ladder collapses
//! Hover→AccentSoft, Pressed→AccentPress on a single hue; Danger
//! follows the same ladder rotated to the danger hue.

use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, paint_text_centered, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{Color as TokenColor, ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ButtonState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
    /// Awaiting an async result. Body grays out, label is replaced
    /// with a spinner glyph (rendered by [`paint_button`]).
    Loading,
}

/// Visual variant. The geometry is identical across kinds — only the
/// token palette changes — except for [`ButtonKind::IconOnly`] which
/// renders a square chip with no label and an icon centered.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ButtonKind {
    /// Ghost / text-only on the panel surface. Default for secondary
    /// actions (Cancel, Reset).
    #[default]
    Default,
    /// Primary CTA. Filled `Accent` background.
    Accent,
    /// Destructive CTA. Filled `Danger` background.
    Danger,
    /// Square 36x36 chip with only an icon. Used in tool palettes
    /// and dense toolbars.
    IconOnly { icon: IconId },
}

#[derive(Clone, Debug)]
pub struct Button {
    pub id: NodeId,
    pub label: String,
    pub state: ButtonState,
    pub kind: ButtonKind,
}

impl Button {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            state: ButtonState::Normal,
            kind: ButtonKind::Default,
        }
    }

    /// Convenience: filled accent CTA.
    pub fn accent(mut self) -> Self {
        self.kind = ButtonKind::Accent;
        self
    }

    /// Convenience: filled destructive CTA.
    pub fn danger(mut self) -> Self {
        self.kind = ButtonKind::Danger;
        self
    }

    /// Convenience: 36x36 icon-only chip. Label still required for
    /// AccessKit (screen readers narrate it).
    pub fn icon_only(mut self, icon: IconId) -> Self {
        self.kind = ButtonKind::IconOnly { icon };
        self
    }

    pub fn kind(mut self, kind: ButtonKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn state(mut self, state: ButtonState) -> Self {
        self.state = state;
        self
    }

    /// Resolve the foreground (text + icon) token for the current
    /// state and kind.
    pub fn fg_color(&self, theme: Theme) -> TokenColor {
        if self.state == ButtonState::Disabled {
            return ColorToken::TextDisabled.resolve(theme);
        }
        match self.kind {
            ButtonKind::Default | ButtonKind::IconOnly { .. } => ColorToken::Text1.resolve(theme),
            ButtonKind::Accent | ButtonKind::Danger => ColorToken::AccentFg.resolve(theme),
        }
    }

    /// Resolve the background token. Returns `None` for ghost
    /// (Default + IconOnly Normal); the rect stays transparent and
    /// the label/icon paint over the panel surface beneath.
    pub fn bg_color(&self, theme: Theme) -> Option<TokenColor> {
        let token = match (self.kind, self.state) {
            (_, ButtonState::Disabled) => match self.kind {
                ButtonKind::Default | ButtonKind::IconOnly { .. } => return None,
                _ => ColorToken::Border,
            },
            (ButtonKind::Default, ButtonState::Hovered | ButtonState::Focused) => {
                ColorToken::BgElev
            }
            (ButtonKind::Default, ButtonState::Pressed) => ColorToken::AccentSoft,
            (ButtonKind::Default, _) => return None,
            (ButtonKind::IconOnly { .. }, ButtonState::Hovered | ButtonState::Focused) => {
                ColorToken::BgElev
            }
            (ButtonKind::IconOnly { .. }, ButtonState::Pressed) => ColorToken::AccentSoft,
            (ButtonKind::IconOnly { .. }, _) => return None,
            (ButtonKind::Accent, ButtonState::Pressed) => ColorToken::AccentPress,
            (ButtonKind::Accent, ButtonState::Hovered) => ColorToken::AccentSoft,
            (ButtonKind::Accent, _) => ColorToken::Accent,
            (ButtonKind::Danger, ButtonState::Pressed | ButtonState::Hovered) => {
                ColorToken::DangerSoft
            }
            (ButtonKind::Danger, _) => ColorToken::Danger,
        };
        Some(token.resolve(theme))
    }

    /// Show a focus ring? True only when focused (per WCAG 2.4.7).
    pub fn focus_ring(&self) -> bool {
        self.state == ButtonState::Focused
    }

    pub fn font_size(&self) -> f32 {
        TypeToken::Base.px()
    }

    pub fn padding(&self) -> f32 {
        Spacing::Lg.px()
    }

    pub fn radius(&self) -> f32 {
        Radius::Md.px()
    }

    /// Build the AccessKit node. Per ADR-0023 §10: every interactive
    /// widget exposes role + label + clickable action.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Button)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != ButtonState::Disabled)
            .action(Action::Click)
            .build()
    }
}

/// Suggested square edge for [`ButtonKind::IconOnly`].
pub const ICON_BUTTON_SIZE_PX: f32 = 36.0;

/// Paint a button at the given rect. Honors [`ButtonKind`] for
/// background, focus ring, label/icon swap, and Loading→spinner glyph.
pub fn paint_button(
    button: &Button,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = button.radius();
    if let Some(bg) = button.bg_color(theme) {
        fill_rounded_rect(
            scene,
            rect,
            radius,
            ph2d_vector::Color::from_rgba8(bg.r, bg.g, bg.b, bg.a),
        );
    }
    if button.focus_ring() {
        let ring = ColorToken::BorderEmph.resolve(theme);
        stroke_rounded_rect(
            scene,
            rect,
            radius,
            2.0,
            ph2d_vector::Color::from_rgba8(ring.r, ring.g, ring.b, ring.a),
        );
    }
    let fg_token = button.fg_color(theme);
    let fg = ph2d_vector::Color::from_rgba8(fg_token.r, fg_token.g, fg_token.b, fg_token.a);
    match button.kind {
        ButtonKind::IconOnly { icon } => {
            paint_icon(scene, icon, rect, fg, 1.5);
        }
        _ => {
            if button.state == ButtonState::Loading {
                paint_icon(scene, IconId::Spinner, rect, fg, 1.5);
            } else {
                paint_text_centered(
                    text_system,
                    scene,
                    &button.label,
                    rect,
                    button.font_size(),
                    fg,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Button {
        Button::new(NodeId(1), "Save")
    }

    #[test]
    fn default_button_uses_text_primary_for_fg() {
        let b = fixture();
        assert_eq!(
            b.fg_color(Theme::Forge),
            ColorToken::Text1.resolve(Theme::Forge)
        );
    }

    #[test]
    fn accent_button_paints_accent_fg() {
        let b = fixture().accent();
        assert_eq!(
            b.fg_color(Theme::Forge),
            ColorToken::AccentFg.resolve(Theme::Forge)
        );
    }

    #[test]
    fn danger_button_paints_danger_bg() {
        let b = fixture().danger();
        assert_eq!(
            b.bg_color(Theme::Forge),
            Some(ColorToken::Danger.resolve(Theme::Forge))
        );
    }

    #[test]
    fn danger_hover_softens() {
        let b = fixture().danger().state(ButtonState::Hovered);
        assert_eq!(
            b.bg_color(Theme::Forge),
            Some(ColorToken::DangerSoft.resolve(Theme::Forge))
        );
    }

    #[test]
    fn icon_only_uses_icon_kind() {
        let b = fixture().icon_only(IconId::Save);
        assert!(matches!(
            b.kind,
            ButtonKind::IconOnly { icon: IconId::Save }
        ));
    }

    #[test]
    fn disabled_overrides_fg() {
        let b = fixture().accent().state(ButtonState::Disabled);
        assert_eq!(
            b.fg_color(Theme::Sunstone),
            ColorToken::TextDisabled.resolve(Theme::Sunstone)
        );
    }

    #[test]
    fn default_normal_has_no_bg() {
        assert!(fixture().bg_color(Theme::Forge).is_none());
    }

    #[test]
    fn default_hover_lifts_to_bg_elev() {
        let b = fixture().state(ButtonState::Hovered);
        assert_eq!(
            b.bg_color(Theme::Forge),
            Some(ColorToken::BgElev.resolve(Theme::Forge))
        );
    }

    #[test]
    fn focus_ring_only_when_focused() {
        assert!(!fixture().focus_ring());
        assert!(fixture().state(ButtonState::Focused).focus_ring());
    }

    #[test]
    fn radius_uses_md_token() {
        assert_eq!(fixture().radius(), Radius::Md.px());
    }

    #[test]
    fn a11y_node_has_button_role_and_click() {
        let node = fixture().build_a11y(0.0, 0.0, 80.0, 32.0);
        assert_eq!(node.role(), Role::Button);
        assert_eq!(node.label(), Some("Save"));
        assert!(node.supports_action(Action::Click));
    }

    fn smoke(button: Button, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_button(
            &button,
            Rect::new(0.0, 0.0, 120.0, 32.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_normal() {
        smoke(fixture(), Theme::Forge);
    }

    #[test]
    fn paint_smoke_hovered() {
        smoke(fixture().state(ButtonState::Hovered), Theme::Forge);
    }

    #[test]
    fn paint_smoke_pressed() {
        smoke(fixture().state(ButtonState::Pressed), Theme::Forge);
    }

    #[test]
    fn paint_smoke_focused() {
        smoke(fixture().state(ButtonState::Focused), Theme::Forge);
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(fixture().state(ButtonState::Disabled), Theme::Forge);
    }

    #[test]
    fn paint_smoke_accent() {
        smoke(fixture().accent(), Theme::Sunstone);
    }

    #[test]
    fn paint_smoke_danger() {
        smoke(fixture().danger(), Theme::Sunstone);
    }

    #[test]
    fn paint_smoke_icon_only() {
        smoke(fixture().icon_only(IconId::Settings), Theme::Forge);
    }

    #[test]
    fn paint_smoke_loading_renders_spinner() {
        smoke(fixture().accent().state(ButtonState::Loading), Theme::Forge);
    }
}
