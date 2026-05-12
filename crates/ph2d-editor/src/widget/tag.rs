//! [`Tag`] (a.k.a. Chip) — small pill carrying a label and an
//! optional close affordance.
//!
//! Used for filters, attached labels, and active-search tokens.
//! `Removable` tags expose `Action::Click` and a Close icon at the
//! right edge; non-removable tags read as `Role::Label`.

use crate::icons::IconId;
use crate::paint::{
    fill_rounded_rect, paint_icon, paint_text_centered, resolve, stroke_rounded_rect,
};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TagState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TagTone {
    /// Default chip — `Bg2` background.
    #[default]
    Neutral,
    /// `AccentSoft` background — for active filters.
    Accent,
    /// `SuccessSoft` — for "ok" tokens.
    Success,
    /// `WarnSoft` — for "warning" tokens.
    Warn,
    /// `DangerSoft` — for "blocked / error" tokens.
    Danger,
}

#[derive(Clone, Debug)]
pub struct Tag {
    pub id: NodeId,
    pub label: String,
    pub state: TagState,
    pub tone: TagTone,
    pub removable: bool,
}

impl Tag {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            state: TagState::Normal,
            tone: TagTone::Neutral,
            removable: false,
        }
    }

    pub fn state(mut self, state: TagState) -> Self {
        self.state = state;
        self
    }

    pub fn tone(mut self, tone: TagTone) -> Self {
        self.tone = tone;
        self
    }

    pub fn removable(mut self, yes: bool) -> Self {
        self.removable = yes;
        self
    }

    /// Build the AccessKit node. Removable tags expose Click; static
    /// tags are pure `Role::Label`.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let mut builder = NodeBuilder::new(Role::Label)
            .label(&self.label)
            .bounds(x, y, w, h);
        if self.removable {
            builder = builder
                .focusable(self.state != TagState::Disabled)
                .action(Action::Click);
        }
        builder.build()
    }

    /// Rect of the close `X` icon inside `host` (or `None` for
    /// non-removable tags). Hosts use this to register a dedicated
    /// hit zone for the close action so a click on the X removes
    /// the tag rather than activating the whole pill.
    pub fn close_rect(&self, host: Rect) -> Option<Rect> {
        if !self.removable {
            return None;
        }
        let pad_x = (host.h * 0.5).max(8.0);
        let close_size = (host.h * 0.7).clamp(10.0, 16.0);
        Some(Rect::new(
            host.x + host.w - pad_x - close_size,
            host.y + (host.h - close_size) * 0.5,
            close_size,
            close_size,
        ))
    }

    fn bg_token(&self) -> ColorToken {
        if self.state == TagState::Disabled {
            return ColorToken::Bg2;
        }
        match self.tone {
            TagTone::Neutral => ColorToken::Bg2,
            TagTone::Accent => ColorToken::AccentSoft,
            TagTone::Success => ColorToken::SuccessSoft,
            TagTone::Warn => ColorToken::WarnSoft,
            TagTone::Danger => ColorToken::DangerSoft,
        }
    }

    fn fg_token(&self) -> ColorToken {
        if self.state == TagState::Disabled {
            return ColorToken::TextDisabled;
        }
        match self.tone {
            TagTone::Neutral => ColorToken::Text2,
            TagTone::Accent => ColorToken::Accent,
            TagTone::Success => ColorToken::Success,
            TagTone::Warn => ColorToken::Warn,
            TagTone::Danger => ColorToken::Danger,
        }
    }
}

/// Pill body + label + optional close icon. Label is centered when
/// `!removable`; left-aligned with the close icon at right when
/// removable.
pub fn paint_tag(
    tag: &Tag,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Full.px();
    let bg = resolve(tag.bg_token(), theme);
    let fg = resolve(tag.fg_token(), theme);
    fill_rounded_rect(scene, rect, radius, bg);
    if tag.state == TagState::Hovered {
        stroke_rounded_rect(scene, rect, radius, 1.0, fg);
    }

    let pad_x = (rect.h * 0.5).max(8.0);
    if let Some(close_rect) = tag.close_rect(rect) {
        let label_rect = Rect::new(
            rect.x + pad_x,
            rect.y,
            (close_rect.x - rect.x - pad_x * 1.5).max(0.0),
            rect.h,
        );
        paint_text_centered(
            text_system,
            scene,
            &tag.label,
            label_rect,
            TypeToken::Xs.px(),
            fg,
        );
        paint_icon(scene, IconId::Close, close_rect, fg, 1.5);
    } else {
        paint_text_centered(text_system, scene, &tag.label, rect, TypeToken::Xs.px(), fg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let t = Tag::new(NodeId(1), "filter");
        assert_eq!(t.state, TagState::Normal);
        assert_eq!(t.tone, TagTone::Neutral);
        assert!(!t.removable);
    }

    #[test]
    fn a11y_static_tag_has_label_role() {
        let node = Tag::new(NodeId(1), "x").build_a11y(0.0, 0.0, 60.0, 24.0);
        assert_eq!(node.role(), Role::Label);
    }

    #[test]
    fn close_rect_only_for_removable() {
        let host = Rect::new(0.0, 0.0, 80.0, 22.0);
        assert!(Tag::new(NodeId(1), "x").close_rect(host).is_none());
        let r = Tag::new(NodeId(1), "x")
            .removable(true)
            .close_rect(host)
            .unwrap();
        // close X must sit at the right edge of the pill.
        assert!(r.x > host.x + host.w * 0.5);
    }

    #[test]
    fn a11y_removable_tag_supports_click() {
        let node = Tag::new(NodeId(1), "x")
            .removable(true)
            .build_a11y(0.0, 0.0, 60.0, 24.0);
        assert!(node.supports_action(Action::Click));
    }

    fn smoke(tag: Tag, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_tag(
            &tag,
            Rect::new(0.0, 0.0, 80.0, 22.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_neutral() {
        smoke(Tag::new(NodeId(1), "x"), Theme::Forge);
    }

    #[test]
    fn paint_smoke_accent_removable() {
        smoke(
            Tag::new(NodeId(1), "x")
                .tone(TagTone::Accent)
                .removable(true),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_warn_hovered() {
        smoke(
            Tag::new(NodeId(1), "x")
                .tone(TagTone::Warn)
                .state(TagState::Hovered),
            Theme::Blueprint,
        );
    }

    #[test]
    fn paint_smoke_danger_pressed() {
        smoke(
            Tag::new(NodeId(1), "x")
                .tone(TagTone::Danger)
                .state(TagState::Pressed),
            Theme::Workshop,
        );
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(
            Tag::new(NodeId(1), "x").state(TagState::Disabled),
            Theme::Forge,
        );
    }
}
