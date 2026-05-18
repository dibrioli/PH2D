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

/// Per-frame render statistics surfaced in the bottom HUD.
/// Phase A (M14.4g): fps, frame_ms, draws, sprite_count, entity_count
/// are real (host writes via [`crate::HeroScreen::set_stats`]); the
/// memory / physics / network slots stay stub strings until those
/// subsystems land. Phase B will add GPU timestamp queries from
/// `wgpu::QuerySet` for per-pass breakdown.
#[derive(Copy, Clone, Debug, Default)]
pub struct BottomHudStats {
    /// Frames per second (derived from `frame_ms`).
    pub fps: f32,
    /// Frame time in milliseconds, EWMA-smoothed by the host.
    pub frame_ms: f32,
    /// GPU draw-calls per frame. For the M14.4d sprite pipeline this
    /// is 1 (one instanced draw per atlas). Tonemap, Vello chrome,
    /// and compositor are fullscreen blits and don't count as draws
    /// in the sprite-batching sense.
    pub draws: u32,
    /// Sprite instances submitted to the GPU this frame.
    pub sprite_count: u32,
    /// Total entity count in the live `SimWorld` (only meaningful in
    /// `PH2D_HERO_LIVE=1` mode; fixture demo reports the populator's
    /// constant).
    pub entity_count: u32,
    /// M14.7 polish (10.1): "raw" fps derived from the CPU-side
    /// frame work time (start of `render_frame` → end of
    /// `queue.submit`), with the vsync wait excluded. Useful as a
    /// hardware-capacity gauge — Unity / Godot expose the same
    /// reading. Status bar renders it alongside the synced fps so
    /// the user can see "60 fps synced · 2400 raw" and gauge how
    /// much headroom each new sprite costs.
    pub raw_fps: f32,
}

pub fn paint_bottom_hud(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    stats: BottomHudStats,
) {
    // Format `entity_count` with a `K` suffix above 999 so the
    // segment width stays predictable when the scene scales up.
    let sprite_label = if stats.sprite_count >= 1_000 {
        format!("{:.1}K sprites", stats.sprite_count as f32 / 1_000.0) // LITERAL-PX-OK: 1K divisor (display thousands)
    } else {
        format!("{} sprites", stats.sprite_count)
    };
    let draws_label = if stats.draws <= 1 {
        format!("1 draw · {} inst", stats.sprite_count)
    } else {
        format!("{} draws", stats.draws)
    };
    let bar = StatusBar::new(
        NodeId(300),
        "Editor statistics",
        vec![
            StatusSegment::new("EDIT")
                .dot(true)
                .tone(SegmentTone::Neutral),
            StatusSegment::new(format!(
                "{} fps \u{00b7} {:.1} ms \u{00b7} {} raw",
                stats.fps as u32, stats.frame_ms, stats.raw_fps as u32
            )),
            StatusSegment::new(draws_label),
            StatusSegment::new(sprite_label).tone(SegmentTone::Accent),
            StatusSegment::new(format!("{} ent", stats.entity_count)),
            // Physics / memory / network slots stay placeholder until
            // those subsystems land (M14.2 Luau + M10 Rapier are the
            // next major milestones; memory needs a `wgpu::Device`
            // global-memory query that's adapter-specific).
            StatusSegment::new("\u{2013} bodies"),
            StatusSegment::new("\u{2013} MB"),
            StatusSegment::new("100%"),
            StatusSegment::new("default-scene").tone(SegmentTone::Muted),
        ],
    );
    let pref_w = bar.preferred_width().min(layout.viewport.w - 40.0); // LITERAL-PX-OK: HUD viewport margin (chrome-specific)
    let rect = Rect::new(
        layout.viewport.x + (layout.viewport.w - pref_w) * 0.5,
        layout.bottom_hud.y,
        pref_w,
        layout.bottom_hud.h,
    );
    paint_status_bar(&bar, rect, scene, text_system, theme);
}
