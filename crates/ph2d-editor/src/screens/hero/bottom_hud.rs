//! BottomHUD painter — pill StatusBar centered horizontally,
//! exposing the canonical runtime statistics that 2D game engines
//! track per frame. Values are placeholders until the pilot project
//! wires real metrics from the renderer / ECS / physics.
//!
//! Standard 2D engine HUD segments (left → right):
//! 1. Mode chip (EDIT / PLAY / PAUSE) with a status dot.
//! 2. Frame rate + frame time (`60 fps · 16.7 ms`).
//! 3. Draw calls (`42 draws`).
//! 4. Sprites / quads rendered this frame (`1.2K sprites`).
//! 5. Entities total / active (`256 / 1024 ent`).
//! 6. Rigidbodies + active collision pairs (`32 bodies · 18 coll`).
//! 7. Memory footprint (`128 MB`).
//! 8. Camera zoom (`100%`).
//! 9. Loaded scene name (`default-scene`, muted).

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
        "Editor statistics",
        vec![
            StatusSegment::new("EDIT")
                .dot(true)
                .tone(SegmentTone::Neutral),
            StatusSegment::new("60 fps \u{00b7} 16.7 ms"),
            StatusSegment::new("42 draws"),
            StatusSegment::new("1.2K sprites").tone(SegmentTone::Accent),
            StatusSegment::new("256 / 1024 ent"),
            StatusSegment::new("32 bodies \u{00b7} 18 coll"),
            StatusSegment::new("128 MB"),
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
