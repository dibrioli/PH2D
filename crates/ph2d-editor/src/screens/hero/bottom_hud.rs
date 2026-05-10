//! BottomHUD painter — pill StatusBar centered horizontally.

use super::HeroLayout;
use crate::widget::{SegmentTone, StatusBar, StatusSegment, paint_status_bar};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

pub fn paint_bottom_hud(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let bar = StatusBar::new(
        NodeId(300),
        "Editor HUD",
        vec![
            StatusSegment::new("EDIT")
                .dot(true)
                .tone(SegmentTone::Neutral),
            StatusSegment::new("60 fps"),
            StatusSegment::new("13101 / 16660").tone(SegmentTone::Accent),
            StatusSegment::new("21n"),
            StatusSegment::new("100%"),
            StatusSegment::new("default-scene").tone(SegmentTone::Muted),
        ],
    );
    let pref_w = bar.preferred_width().min(layout.viewport.w - 40.0);
    let rect = Rect::new(
        layout.viewport.x + (layout.viewport.w - pref_w) * 0.5,
        layout.bottom_hud.y,
        pref_w,
        layout.bottom_hud.h,
    );
    paint_status_bar(&bar, rect, scene, text_system, theme);
}
