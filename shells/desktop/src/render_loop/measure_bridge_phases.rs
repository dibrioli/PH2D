//! MEASUREMENT scaffold (not a gate): where the PANEL and CHROME phases of `painter_bridge::dispatch`
//! actually spend a frame, at three canvas sizes.
//!
//! Why this exists and the `tests/measure_painter_bridge_phases.rs` sibling was not enough: that one
//! timed the TOOL-SIDE getters (`painter.selection()`, `painter.brush_settings()`, …) and every one of
//! them measured **0.0000 ms** at 4096². That is a real negative result — the accessors are innocent —
//! but it leaves the 3,8 ms `panel` + 3,7 ms `chrome` unexplained, because the getters are only half of
//! each phase. The other half is the SHELL side: the twelve `draw_overlays` calls (which write into a
//! `VectorScene`) and the `ph2d_panel_painter_layers::set_current_*` publishes. This harness drives the
//! REAL `draw_overlays` — the same `pub(super)` function the bridge calls, with the same per-call perf
//! array — over a REAL fixture (a selected sprite in a `SimWorld`, a `HeroScreen`, a live camera).
//!
//! Fixture = the case Enio measured: one layer, no selection, no open shape, painter active, a sprite
//! selected and the cursor over the canvas (so the brush ring — the one overlay that always draws —
//! is on the expensive path, not early-returning on `panel_at`).
//!
//! Run:
//! ```text
//! cargo test -p ph2d-host-desktop --release measure_where_the_bridge_phases_spend_a_frame \
//!   -- --ignored --nocapture
//! ```

use ph2d_ecs::SimWorld;
use ph2d_editor::tool::RasterEditTool;
use ph2d_editor::{HeroScreen, NodeId};
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, Sprite};
use ph2d_tool_painter::PainterTool;
use ph2d_vector::VectorScene;

/// Median wall-clock of `n` calls, in ms.
///
/// ⚠️ The result is fed to `black_box`: a closure whose value is dropped is dead code the optimizer
/// may delete OUTRIGHT, and a deleted call measures a perfect 0.0000 ms — a zero that says nothing
/// about the code and everything about the harness. (First draft of this file made exactly that
/// mistake; the numbers did not move, which is the evidence that they are real.)
fn med<T>(n: usize, mut f: impl FnMut() -> T) -> f64 {
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        let t = std::time::Instant::now();
        let out = f();
        v.push(t.elapsed().as_secs_f64() * 1e3);
        std::hint::black_box(&out);
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

/// The fixture: a painter bound to a `size`² canvas, a sprite entity selected in the hero screen.
fn fixture(size: u32) -> (PainterTool, SimWorld, HeroScreen, Camera2d, WindowSize) {
    let mut painter = PainterTool::default();
    painter.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    painter.set_brush_size_px(40.0);

    let mut sim = SimWorld::new();
    let entity = sim
        .world_mut()
        .spawn((
            crate::Transform::default(),
            Sprite::atlas(0, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
        ))
        .id();

    let mut hero = HeroScreen::new(NodeId(1));
    hero.gizmo.replace_selection(Some(entity.to_bits()));

    let camera = Camera2d::new([0.0, 0.0], 4.0);
    let window = WindowSize::new(1920, 1080);
    (painter, sim, hero, camera, window)
}

#[test]
#[ignore = "measurement, not a gate"]
fn measure_where_the_bridge_phases_spend_a_frame() {
    for size in [1024u32, 2048, 4096] {
        let (painter, sim, hero, camera, window) = fixture(size);
        let mut scene = VectorScene::new();
        let mut text = ph2d_text::TextSystem::without_system_fonts();
        // Cursor over the canvas centre — NOT over a panel, so the brush ring takes its drawing path.
        let cursor = (960.0f32, 540.0f32);
        let mut perf = [0f32; super::paint_perf::CHROME_SUB];

        // Warm: first call builds whatever is lazy.
        super::painter_bridge_overlays::draw_overlays(
            &painter, &hero, &sim, &camera, window, &mut scene, &mut text, cursor, &mut perf, true,
        );

        println!("\n=== canvas {size}x{size} — median of 200 (ms) ===");

        // ── CHROME: the REAL draw_overlays, whole and split by call ──────────
        let whole = med(200, || {
            scene = VectorScene::new();
            super::painter_bridge_overlays::draw_overlays(
                &painter, &hero, &sim, &camera, window, &mut scene, &mut text, cursor, &mut perf,
                true,
            );
        });
        println!("CHROME draw_overlays TOTAL {whole:.4}");
        for (i, name) in super::paint_perf::CHROME_LABELS.iter().enumerate() {
            println!("  {name:<9} {:.4}", perf[i]);
        }

        // ── PANEL: the SHELL-side publishes (what the earlier harness skipped) ─
        println!("PANEL shell-side publishes:");
        #[cfg(feature = "panel-painter-layers")]
        {
            println!(
                "  set_current_layers   {:.4}",
                med(200, || ph2d_panel_painter_layers::set_current_layers(Some(
                    painter.layers().clone()
                )))
            );
            println!(
                "  set_current_selection {:.4}",
                med(200, || ph2d_panel_painter_layers::set_current_selection(
                    painter.selection()
                ))
            );
            let bs = painter.brush_settings();
            println!(
                "  set_current_brush    {:.4}",
                med(200, || ph2d_panel_painter_layers::set_current_brush(Some(
                    bs
                )))
            );
            println!(
                "  wet_tuning set_brush {:.4}",
                med(200, || ph2d_panel_wet_tuning::set_current_brush(Some(bs)))
            );
        }
        // The two store-touching calls the PANEL phase makes every frame.
        println!(
            "  store.panel_at()     {:.4}",
            med(200, || hero.store.panel_at(cursor.0, cursor.1))
        );
        // The env read the PANEL phase does EVERY frame (PH2D_PREVIEW_DIAG), before any sub-mark.
        println!(
            "  env::var_os(DIAG)    {:.4}",
            med(200, || std::env::var_os("PH2D_PREVIEW_DIAG"))
        );
        // ── The PANEL phase HEAD: reads that run ONLY when the trap is armed ──
        // These sit between `m_panel` and `ph_panel_sub[0]`, so they are inside the measured PANEL
        // window but OUTSIDE every sub-slot: if the trap is expensive it inflates the number it reports.
        println!("PANEL head (trap-only reads — armed only under PH2D_PAINT_PERF):");
        println!(
            "  preview_drain_diag   {:.4}",
            med(200, || painter.preview_drain_diag())
        );
        println!(
            "  preview_is_trivial   {:.4}",
            med(200, || painter.preview_is_trivial_stack())
        );
        println!(
            "  mask_view_grayscale  {:.4}",
            med(200, || painter.mask_view_grayscale())
        );
        println!(
            "  source_size          {:.4}",
            med(200, || painter.source_size())
        );
    }
}
