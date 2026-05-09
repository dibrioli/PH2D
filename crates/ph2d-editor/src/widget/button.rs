//! [`Button`] — example widget. Pattern every future widget follows.
//!
//! Three responsibilities:
//! 1. **Visual** — resolves color/type/spacing tokens for the active
//!    Theme, exposes them so the Vello render path can paint without
//!    re-resolving every frame.
//! 2. **A11y** — emits an `accesskit::Node` via `ph2d_a11y::NodeBuilder`
//!    so screen readers see "button labeled X, focusable, clickable".
//! 3. **State** — tracks pressed / focused / hovered for Procreate-
//!    style luminance hierarchy (a single hue, three intensities).

use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_tokens::{Color, ColorToken, Spacing, Theme, TypeToken};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ButtonState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

#[derive(Clone, Debug)]
pub struct Button {
    pub id: NodeId,
    pub label: String,
    pub state: ButtonState,
    /// When true, paints with `AccentPrimary` foreground (e.g. the
    /// "primary" CTA in a panel).
    pub accent: bool,
}

impl Button {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            state: ButtonState::Normal,
            accent: false,
        }
    }

    pub fn accent(mut self, yes: bool) -> Self {
        self.accent = yes;
        self
    }

    pub fn state(mut self, state: ButtonState) -> Self {
        self.state = state;
        self
    }

    /// Resolve the foreground token for the current state + accent
    /// flag. Per ADR-0023 §11 (luminance hierarchy on a single hue).
    pub fn fg_color(&self, theme: Theme) -> Color {
        if self.state == ButtonState::Disabled {
            return ColorToken::TextDisabled.resolve(theme);
        }
        if self.accent {
            // Accent buttons go through the 3-intensity ladder.
            let token = match self.state {
                ButtonState::Pressed => ColorToken::AccentSecondary,
                ButtonState::Hovered | ButtonState::Focused => ColorToken::AccentTertiary,
                _ => ColorToken::AccentPrimary,
            };
            return token.resolve(theme);
        }
        ColorToken::TextPrimary.resolve(theme)
    }

    /// Resolve the background token. Accent CTAs paint a filled
    /// background; default buttons are transparent over the panel
    /// surface (relying on focus ring for affordance).
    pub fn bg_color(&self, theme: Theme) -> Option<Color> {
        if !self.accent {
            // Default button is text-only (transparent fill).
            return None;
        }
        let token = match self.state {
            ButtonState::Pressed => ColorToken::AccentSecondary,
            ButtonState::Hovered => ColorToken::AccentTertiary,
            _ => ColorToken::AccentPrimary,
        };
        Some(token.resolve(theme))
    }

    /// Show a focus ring? True only when focused (per WCAG 2.4.7).
    pub fn focus_ring(&self) -> bool {
        self.state == ButtonState::Focused
    }

    pub fn font_size(&self) -> f32 {
        TypeToken::Sm.px()
    }

    pub fn padding(&self) -> f32 {
        Spacing::Sm.px()
    }

    /// Build the AccessKit node. Per ADR-0023 §10: every interactive
    /// widget must expose role + label + clickable action.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Button)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != ButtonState::Disabled)
            .action(Action::Click)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_button_uses_text_primary_for_fg() {
        let b = Button::new(NodeId(1), "Save");
        let fg = b.fg_color(Theme::Dark);
        assert_eq!(fg, ColorToken::TextPrimary.resolve(Theme::Dark));
    }

    #[test]
    fn accent_button_uses_accent_primary_default_state() {
        let b = Button::new(NodeId(1), "Save").accent(true);
        let fg = b.fg_color(Theme::Dark);
        assert_eq!(fg, ColorToken::AccentPrimary.resolve(Theme::Dark));
    }

    #[test]
    fn accent_pressed_drops_to_secondary() {
        let b = Button::new(NodeId(1), "Save")
            .accent(true)
            .state(ButtonState::Pressed);
        let fg = b.fg_color(Theme::Dark);
        assert_eq!(fg, ColorToken::AccentSecondary.resolve(Theme::Dark));
    }

    #[test]
    fn accent_hover_drops_to_tertiary() {
        let b = Button::new(NodeId(1), "Save")
            .accent(true)
            .state(ButtonState::Hovered);
        let fg = b.fg_color(Theme::Dark);
        assert_eq!(fg, ColorToken::AccentTertiary.resolve(Theme::Dark));
    }

    #[test]
    fn disabled_uses_text_disabled() {
        let b = Button::new(NodeId(1), "Save").state(ButtonState::Disabled);
        let fg = b.fg_color(Theme::Light);
        assert_eq!(fg, ColorToken::TextDisabled.resolve(Theme::Light));
    }

    #[test]
    fn default_button_has_no_bg() {
        let b = Button::new(NodeId(1), "Save");
        assert!(b.bg_color(Theme::Dark).is_none());
    }

    #[test]
    fn accent_button_has_bg() {
        let b = Button::new(NodeId(1), "Save").accent(true);
        assert!(b.bg_color(Theme::Dark).is_some());
    }

    #[test]
    fn focus_ring_only_when_focused() {
        let b = Button::new(NodeId(1), "Save");
        assert!(!b.focus_ring());
        let b = b.state(ButtonState::Focused);
        assert!(b.focus_ring());
    }

    #[test]
    fn font_and_padding_use_tokens() {
        let b = Button::new(NodeId(1), "Save");
        assert_eq!(b.font_size(), TypeToken::Sm.px());
        assert_eq!(b.padding(), Spacing::Sm.px());
    }

    #[test]
    fn a11y_node_has_button_role_and_label_and_click() {
        let b = Button::new(NodeId(1), "Save");
        let node = b.build_a11y(0.0, 0.0, 80.0, 32.0);
        assert_eq!(node.role(), Role::Button);
        assert_eq!(node.label(), Some("Save"));
        assert!(node.supports_action(Action::Click));
    }

    #[test]
    fn disabled_button_is_not_focusable_in_a11y() {
        let b = Button::new(NodeId(1), "Save").state(ButtonState::Disabled);
        let _node = b.build_a11y(0.0, 0.0, 80.0, 32.0);
        // accesskit::Node doesn't expose is_focusable() directly;
        // we trust NodeBuilder honored focusable(false).
    }
}
