//! [`SectionHeader`] — single-line uppercase header for inspector
//! sections.
//!
//! Layout (left → right): optional collapse chevron · accent dot ·
//! label uppercase · optional count chip on the far right.
//! Used by the editor Inspector to break a body into "Params (12)",
//! "Advanced (7)", "Inputs (24)" etc.

use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, paint_text, paint_text_centered, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Circle, Fill, Point, VectorScene};

#[derive(Clone, Debug)]
pub struct SectionHeader {
    pub id: NodeId,
    pub label: String,
    pub count: Option<u32>,
    /// When `Some(open)`, paints a chevron that flips on open/closed.
    /// `None` means the section is non-collapsible.
    pub collapsible: Option<bool>,
}

impl SectionHeader {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            count: None,
            collapsible: None,
        }
    }

    pub fn count(mut self, n: u32) -> Self {
        self.count = Some(n);
        self
    }

    pub fn collapsible(mut self, open: bool) -> Self {
        self.collapsible = Some(open);
        self
    }

    pub fn is_open(&self) -> bool {
        self.collapsible.unwrap_or(true)
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let mut builder = NodeBuilder::new(Role::Label)
            .label(&self.label)
            .bounds(x, y, w, h);
        if self.collapsible.is_some() {
            builder = builder.focusable(true).action(Action::Click);
        }
        builder.build()
    }
}

pub fn paint_section_header(
    header: &SectionHeader,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let pad_x = Spacing::Md.px();
    let mut cursor_x = rect.x + pad_x;
    let icon_w = (rect.h * 0.6).clamp(10.0, 16.0);

    // Collapse chevron.
    if let Some(open) = header.collapsible {
        let chev_rect = Rect::new(cursor_x, rect.y + (rect.h - icon_w) * 0.5, icon_w, icon_w);
        let icon = if open {
            IconId::ChevronDown
        } else {
            IconId::ChevronRight
        };
        paint_icon(
            scene,
            icon,
            chev_rect,
            resolve(ColorToken::Text3, theme),
            1.5,
        );
        cursor_x += icon_w + Spacing::Xs.px();
    }

    // Accent dot — 6 px circle aligned baseline.
    let dot_r = 3.0;
    let dot_cx = cursor_x + dot_r;
    let dot_cy = rect.y + rect.h * 0.5;
    let dot = Circle::new(Point::new(dot_cx as f64, dot_cy as f64), dot_r as f64);
    scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(resolve(ColorToken::Accent, theme)),
        None,
        &dot,
    );
    cursor_x += dot_r * 2.0 + Spacing::Md.px();

    // Label — uppercase happens visually; we hand parley the source
    // string and rely on font + size to read as a section title.
    let font = TypeToken::Xs.px();
    let label_y = rect.y + (rect.h - font) * 0.5;
    let label_w = if header.count.is_some() {
        (rect.x + rect.w - cursor_x - 48.0 - pad_x).max(0.0)
    } else {
        (rect.x + rect.w - cursor_x - pad_x).max(0.0)
    };
    paint_text(
        text_system,
        scene,
        &header.label,
        cursor_x,
        label_y,
        font,
        label_w,
        resolve(ColorToken::Text2, theme),
    );

    // Count chip — right-aligned mono pill.
    if let Some(n) = header.count {
        let chip_w = 36.0_f32;
        let chip_h = (rect.h - 4.0).max(14.0);
        let chip_rect = Rect::new(
            rect.x + rect.w - pad_x - chip_w,
            rect.y + (rect.h - chip_h) * 0.5,
            chip_w,
            chip_h,
        );
        fill_rounded_rect(
            scene,
            chip_rect,
            Radius::Xs.px(),
            resolve(ColorToken::Bg3, theme),
        );
        let text = n.to_string();
        paint_text_centered(
            text_system,
            scene,
            &text,
            chip_rect,
            font,
            resolve(ColorToken::Text3, theme),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_no_count_no_collapsible() {
        let h = SectionHeader::new(NodeId(1), "Params");
        assert!(h.count.is_none());
        assert!(h.collapsible.is_none());
        assert!(h.is_open(), "non-collapsible defaults to open");
    }

    #[test]
    fn count_setter_round_trips() {
        let h = SectionHeader::new(NodeId(1), "x").count(12);
        assert_eq!(h.count, Some(12));
    }

    #[test]
    fn collapsible_open_false() {
        let h = SectionHeader::new(NodeId(1), "x").collapsible(false);
        assert!(!h.is_open());
    }

    #[test]
    fn a11y_role_is_label() {
        let node = SectionHeader::new(NodeId(1), "Params").build_a11y(0.0, 0.0, 280.0, 24.0);
        assert_eq!(node.role(), Role::Label);
    }

    #[test]
    fn a11y_collapsible_supports_click() {
        let node = SectionHeader::new(NodeId(1), "x")
            .collapsible(false)
            .build_a11y(0.0, 0.0, 280.0, 24.0);
        assert!(node.supports_action(Action::Click));
    }

    fn smoke(h: SectionHeader, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_section_header(
            &h,
            Rect::new(0.0, 0.0, 280.0, 24.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_plain() {
        smoke(SectionHeader::new(NodeId(1), "Params"), Theme::ForgeSdf);
    }

    #[test]
    fn paint_smoke_with_count() {
        smoke(
            SectionHeader::new(NodeId(1), "Params").count(12),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_collapsible_closed() {
        smoke(
            SectionHeader::new(NodeId(1), "Advanced")
                .count(7)
                .collapsible(false),
            Theme::Blueprint,
        );
    }

    #[test]
    fn paint_smoke_collapsible_open() {
        smoke(
            SectionHeader::new(NodeId(1), "Inputs")
                .count(24)
                .collapsible(true),
            Theme::PaintStudio,
        );
    }
}
