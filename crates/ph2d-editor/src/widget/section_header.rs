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
use ph2d_vector::{Affine, Brush, Circle, Color as VelloColor, Fill, Point, VectorScene};

#[derive(Clone, Debug)]
pub struct SectionHeader {
    pub id: NodeId,
    pub label: String,
    pub count: Option<u32>,
    /// When `Some(open)`, paints a chevron that flips on open/closed.
    /// `None` means the section is non-collapsible.
    pub collapsible: Option<bool>,
    /// Right-edge color circle (RGBA bytes). When `Some`, replaces
    /// the count chip — the user can click it to open the global
    /// color picker for the section. When `None`, falls back to
    /// the count chip (or nothing).
    pub color: Option<[u8; 4]>,
}

impl SectionHeader {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            count: None,
            collapsible: None,
            color: None,
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

    pub fn color(mut self, rgba: [u8; 4]) -> Self {
        self.color = Some(rgba);
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

/// Canonical section-header chrome (Inspector + every collapsible
/// panel uses this). Layout (left → right): collapse chevron + label
/// in UPPERCASE + optional count pill on the far right. The previous
/// accent-dot ornament was retired — every section is collapsible by
/// design, so the chevron itself is the only "anchor" glyph.
pub fn paint_section_header(
    header: &SectionHeader,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    // Collapsed sections get a darker background plate so the
    // visual hierarchy (open = transparent on panel; closed =
    // tinted) reads as "this block is folded". Centralized here so
    // every panel using `SectionHeader` gets the same affordance.
    if matches!(header.collapsible, Some(false)) {
        fill_rounded_rect(scene, rect, Radius::Sm.px(), resolve(ColorToken::Bg3, theme));
    }
    let pad_x = Spacing::Md.px();
    let mut cursor_x = rect.x + pad_x;
    let icon_w = (rect.h * 0.7).clamp(12.0, 18.0);

    // Collapse chevron — always painted now; non-collapsible headers
    // simply default to "open" via `is_open()`. Single visual anchor
    // for the "this is a section title, click me to fold" affordance.
    let chev_rect = Rect::new(cursor_x, rect.y + (rect.h - icon_w) * 0.5, icon_w, icon_w);
    let icon = if header.is_open() {
        IconId::ChevronDown
    } else {
        IconId::ChevronRight
    };
    paint_icon(scene, icon, chev_rect, resolve(ColorToken::Text2, theme), 1.5);
    cursor_x += icon_w + Spacing::Sm.px();

    // Bigger, bolder label in UPPERCASE — the §x.x design refresh
    // ditched the tiny lowercase title in favor of a clear section
    // banner. Parley has no weight-700 font in the editor's stack
    // (text/lib.rs: "no italics, no weight"), so visual emphasis is
    // built from font_size (Sm instead of Xs) + Text1 contrast +
    // case folding.
    let font = TypeToken::Sm.px();
    let label_y = rect.y + (rect.h - font) * 0.5;
    let label_w = if header.count.is_some() {
        (rect.x + rect.w - cursor_x - 48.0 - pad_x).max(0.0)
    } else {
        (rect.x + rect.w - cursor_x - pad_x).max(0.0)
    };
    let upper = header.label.to_uppercase();
    paint_text(
        text_system,
        scene,
        &upper,
        cursor_x,
        label_y,
        font,
        label_w,
        resolve(ColorToken::Text1, theme),
    );

    // Right-edge ornament. Priority: color circle > count chip.
    // The color circle is the user-clickable "open the picker for
    // this section" affordance; the count chip is legacy and only
    // shown when no color is configured.
    if let Some(rgba) = header.color {
        let radius_px = 7.0_f32;
        let cx = rect.x + rect.w - pad_x - radius_px;
        let cy = rect.y + rect.h * 0.5;
        let circle = Circle::new(Point::new(cx as f64, cy as f64), radius_px as f64);
        let fill = VelloColor::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]);
        scene.inner_mut().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(fill),
            None,
            &circle,
        );
        // 1-px ring so the circle still reads against same-colored
        // backgrounds (e.g. a light yellow circle on light theme).
        let ring = Circle::new(
            Point::new(cx as f64, cy as f64),
            (radius_px + 0.5) as f64,
        );
        scene.inner_mut().stroke(
            &ph2d_vector::Stroke::new(1.0),
            Affine::IDENTITY,
            &Brush::Solid(resolve(ColorToken::Border, theme)),
            None,
            &ring,
        );
    } else if let Some(n) = header.count {
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
            TypeToken::Xs.px(),
            resolve(ColorToken::Text3, theme),
        );
    }
}

/// Rect of the color-circle hit zone in screen coordinates. Hosts
/// register this rect for hit-testing the "open picker for this
/// section" gesture. Returns `None` for headers that have no color
/// circle (the painter falls back to the count chip / no ornament).
pub fn color_circle_hit_rect(header: &SectionHeader, host: Rect) -> Option<Rect> {
    header.color?;
    let pad_x = Spacing::Md.px();
    let radius_px = 7.0_f32;
    let size = radius_px * 2.0 + 4.0;
    Some(Rect::new(
        host.x + host.w - pad_x - size,
        host.y + (host.h - size) * 0.5,
        size,
        size,
    ))
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
    fn paint_emits_no_panic_for_uppercase_label_and_chevron() {
        // Smoke: header with all the new chrome bits (chevron +
        // uppercase + count chip) renders without geometry asserts.
        let h = SectionHeader::new(NodeId(1), "vector").count(3).collapsible(true);
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_section_header(
            &h,
            Rect::new(0.0, 0.0, 280.0, 28.0),
            &mut scene,
            &mut text,
            Theme::ForgeSdf,
        );
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
