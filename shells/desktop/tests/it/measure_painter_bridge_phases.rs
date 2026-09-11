//! MEASUREMENT scaffold (not a gate): where the PANEL and CHROME phases of `painter_bridge::dispatch`
//! spend a frame, at three canvas sizes. Every call below is one the bridge makes EVERY frame the
//! Painter is active, on a document with one layer, no selection and no open shape — the case Enio
//! measured at 4096² (`dispatch p50 7.6 ms` = `panel 3.8` + `chrome 3.7`).
//!
//! Run: `cargo test -p ph2d-host-desktop --release --test measure_painter_bridge_phases -- --ignored --nocapture`

use ph2d_editor::tool::RasterEditTool;
use ph2d_tool_painter::PainterTool;

fn tool(size: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.set_brush_size_px(40.0);
    t
}

/// Median wall-clock of `n` calls, in ms.
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

#[test]
#[ignore = "measurement, not a gate"]
fn where_the_panel_and_chrome_phases_spend_a_frame() {
    for size in [1024u32, 2048, 4096] {
        let mut t = tool(size);
        // Warm: the first frame builds whatever is lazy.
        let _ = t.brush_settings();
        t.refresh_shape_source_if_changed();
        let _ = t.refresh_shape_color_preview();
        println!("\n=== canvas {size}x{size} (ms, median of 200) ===");

        // ── PANEL phase, in bridge call order ────────────────────────────────
        println!("PANEL:");
        println!(
            "  layers_revision      {:.4}",
            med(200, || t.layers_revision())
        );
        println!(
            "  layers().clone()     {:.4}",
            med(200, || t.layers().clone())
        );
        println!("  selection()          {:.4}", med(200, || t.selection()));
        println!(
            "  mask_view_grayscale  {:.4}",
            med(200, || t.mask_view_grayscale())
        );
        println!(
            "  brush_settings()     {:.4}",
            med(200, || t.brush_settings())
        );
        println!(
            "  active_paint_mode_id {:.4}",
            med(200, || t.active_paint_mode_id())
        );
        println!(
            "  eyedropper_armed     {:.4}",
            med(200, || t.eyedropper_armed())
        );
        println!(
            "  dock_shows_layers    {:.4}",
            med(200, || t.dock_shows_layers())
        );
        println!(
            "  brush_texture_image_version {:.4}",
            med(200, || t.brush_texture_image_version())
        );
        println!(
            "  brush_shape_image_version   {:.4}",
            med(200, || t.brush_shape_image_version())
        );
        println!(
            "  brush_paper_image_version   {:.4}",
            med(200, || t.brush_paper_image_version())
        );
        println!(
            "  refresh_shape_source_if_changed {:.4}",
            med(200, || t.refresh_shape_source_if_changed())
        );
        println!(
            "  refresh_shape_color_preview     {:.4}",
            med(200, || t.refresh_shape_color_preview())
        );

        // ── CHROME phase (`draw_overlays`), tool side of each of the twelve ──
        println!("CHROME (tool-side accessors):");
        println!(
            "  wet_preview_intensity {:.4}",
            med(200, || t.wet_preview_intensity())
        );
        println!(
            "  is_selection_mode     {:.4}",
            med(200, || t.is_selection_mode())
        );
        println!(
            "  canvas_size           {:.4}",
            med(200, || t.canvas_size())
        );
        println!(
            "  curve_overlay         {:.4}",
            med(200, || t.curve_overlay())
        );
        println!(
            "  ellipse_overlay       {:.4}",
            med(200, || t.ellipse_overlay())
        );
        println!(
            "  line_overlay          {:.4}",
            med(200, || t.line_overlay())
        );
        println!(
            "  polygon_overlay       {:.4}",
            med(200, || t.polygon_overlay())
        );
        println!(
            "  stroke_op_badges      {:.4}",
            med(200, || t.stroke_op_badges())
        );
        println!(
            "  selection_gizmos      {:.4}",
            med(200, || t.selection_gizmos())
        );
        println!(
            "  deform_gizmo          {:.4}",
            med(200, || t.deform_gizmo())
        );
        println!(
            "  stencil_overlay       {:.4}",
            med(200, || t.stencil_overlay())
        );
        println!("  symmetry              {:.4}", med(200, || t.symmetry()));
        println!(
            "  brush_color_srgb8     {:.4}",
            med(200, || t.brush_color_srgb8())
        );

        // ── the reads the TRAP itself adds (they are inside the PANEL window) ─
        println!("TRAP-ONLY reads:");
        println!(
            "  preview_is_trivial_stack {:.4}",
            med(200, || t.preview_is_trivial_stack())
        );
        println!(
            "  active_is_mask        {:.4}",
            med(200, || t.active_is_mask())
        );
        println!(
            "  source_size           {:.4}",
            med(200, || t.source_size())
        );
        println!(
            "  canvas_version        {:.4}",
            med(200, || t.canvas_version())
        );
    }
}
