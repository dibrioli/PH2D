//! [`StatusBar`] — pill-shaped horizontal HUD with N segments
//! separated by 1 px inset borders.
//!
//! Used in the editor BottomHUD to surface mode + frame stats:
//! `EDIT • 60 fps • 13101/16660 • 21n • 100% • default-scene`.
//! Each [`StatusSegment`] carries its own copy and an optional
//! emphasis token (Accent, Success, etc); a leading status dot is
//! supported for the `EDIT` segment.

use crate::paint::{fill_rounded_rect, paint_text, rect_to_vello, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Circle, Fill, Point, VectorScene};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum SegmentTone {
    #[default]
    Neutral,
    Accent,
    Success,
    Warn,
    Danger,
    Muted,
}

impl SegmentTone {
    fn fg(self) -> ColorToken {
        match self {
            Self::Neutral => ColorToken::Text2,
            Self::Accent => ColorToken::Accent,
            Self::Success => ColorToken::Success,
            Self::Warn => ColorToken::Warn,
            Self::Danger => ColorToken::Danger,
            Self::Muted => ColorToken::Text3,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StatusSegment {
    pub text: String,
    pub tone: SegmentTone,
    /// Show a leading 7 px circle in `Success` color (used by the
    /// EDIT segment as a "live" indicator).
    pub leading_dot: bool,
}

impl StatusSegment {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tone: SegmentTone::Neutral,
            leading_dot: false,
        }
    }

    pub fn tone(mut self, tone: SegmentTone) -> Self {
        self.tone = tone;
        self
    }

    pub fn dot(mut self, yes: bool) -> Self {
        self.leading_dot = yes;
        self
    }
}

#[derive(Clone, Debug)]
pub struct StatusBar {
    pub id: NodeId,
    pub label: String,
    pub segments: Vec<StatusSegment>,
}

impl StatusBar {
    pub fn new(id: NodeId, label: impl Into<String>, segments: Vec<StatusSegment>) -> Self {
        Self {
            id,
            label: label.into(),
            segments,
        }
    }

    /// Width estimate based on character count + padding per segment.
    /// Used by the hero composer to center the bar horizontally.
    pub fn preferred_width(&self) -> f32 {
        let pad = Spacing::Lg.px();
        let dot_extra = 12.0;
        let approx_advance = TypeToken::Sm.px() * 0.6;
        self.segments
            .iter()
            .map(|s| {
                let w = s.text.chars().count() as f32 * approx_advance + pad * 2.0;
                if s.leading_dot { w + dot_extra } else { w }
            })
            .sum()
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let label = if self.segments.is_empty() {
            self.label.clone()
        } else {
            self.segments
                .iter()
                .map(|s| s.text.as_str())
                .collect::<Vec<_>>()
                .join(" · ")
        };
        NodeBuilder::new(Role::Status)
            .label(label)
            .bounds(x, y, w, h)
            .build()
    }
}

pub fn paint_status_bar(
    bar: &StatusBar,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Full.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));

    if bar.segments.is_empty() {
        return;
    }
    let pad = Spacing::Lg.px();
    let dot_extra = 12.0;
    let font = TypeToken::Sm.px();
    // Real text measurement per segment — the previous `chars × 0.6`
    // heuristic miscounted proportional glyphs and made wide
    // segments (e.g. `13101 / 16660`) shrink while narrow ones
    // overflowed. See `docs/UI_Bugs/README.md` §3.3.
    let widths: Vec<f32> = bar
        .segments
        .iter()
        .map(|s| {
            let measured = text_system.layout(&s.text, font, f32::INFINITY).width();
            let mut w = measured + pad * 2.0;
            if s.leading_dot {
                w += dot_extra;
            }
            w
        })
        .collect();
    let total: f32 = widths.iter().sum();
    let scale = if total > 0.0 { rect.w / total } else { 1.0 };

    let mut x = rect.x;
    for (i, segment) in bar.segments.iter().enumerate() {
        let w = widths[i] * scale;
        let seg_rect = Rect::new(x, rect.y, w, rect.h);
        if i > 0 {
            // 1 px inner divider in Border color, full segment height
            // minus 6 px breathing room top/bottom.
            let div = Rect::new(x, rect.y + 6.0, 1.0, rect.h - 12.0);
            scene.fill_rect(rect_to_vello(div), resolve(ColorToken::Border, theme));
        }
        let fg_token = segment.tone.fg();
        let fg = resolve(fg_token, theme);
        let mut text_x = seg_rect.x + pad;
        if segment.leading_dot {
            let r = 3.5;
            let cx = text_x + r;
            let cy = seg_rect.y + seg_rect.h * 0.5;
            let dot = Circle::new(Point::new(cx as f64, cy as f64), r as f64);
            scene.inner_mut().fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(resolve(ColorToken::Success, theme)),
                None,
                &dot,
            );
            text_x += r * 2.0 + 6.0;
        }
        let text_y = seg_rect.y + (seg_rect.h - font) * 0.5;
        let text_w = (seg_rect.x + seg_rect.w - text_x - pad).max(0.0);
        paint_text(
            text_system,
            scene,
            &segment.text,
            text_x,
            text_y,
            font,
            text_w,
            fg,
        );
        x += w;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> StatusBar {
        StatusBar::new(
            NodeId(1),
            "Editor HUD",
            vec![
                StatusSegment::new("EDIT").dot(true),
                StatusSegment::new("60 fps"),
                StatusSegment::new("13101 / 16660").tone(SegmentTone::Accent),
                StatusSegment::new("21n"),
                StatusSegment::new("100%"),
                StatusSegment::new("default-scene").tone(SegmentTone::Muted),
            ],
        )
    }

    #[test]
    fn defaults_match_spec() {
        let s = fixture();
        assert_eq!(s.segments.len(), 6);
        assert!(s.segments[0].leading_dot);
    }

    #[test]
    fn preferred_width_grows_with_segments() {
        let one = StatusBar::new(NodeId(1), "x", vec![StatusSegment::new("EDIT")]);
        assert!(fixture().preferred_width() > one.preferred_width());
    }

    #[test]
    fn a11y_role_is_status() {
        let node = fixture().build_a11y(0.0, 0.0, 400.0, 32.0);
        assert_eq!(node.role(), Role::Status);
    }

    fn smoke(bar: StatusBar, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_status_bar(
            &bar,
            Rect::new(0.0, 0.0, bar.preferred_width(), 34.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_full_hud() {
        smoke(fixture(), Theme::Forge);
    }

    #[test]
    fn paint_smoke_single_segment() {
        smoke(
            StatusBar::new(NodeId(1), "x", vec![StatusSegment::new("EDIT").dot(true)]),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_empty_segments_just_pill() {
        smoke(StatusBar::new(NodeId(1), "x", vec![]), Theme::Blueprint);
    }
}
