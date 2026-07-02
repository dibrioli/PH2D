//! Fill (Bucket) ColorDrop cursor overlay — the small paint-colour swatch drawn at the pointer while a
//! colour is being dragged from the Fill rail button onto the canvas. Split from `painter_bridge_overlays`
//! for the HR-18 file-LOC cap. Pure draw: reads the armed drag state + the paint colour and writes one
//! disc into the overlay `VectorScene`; it mutates no tool or model state. Called once per frame by
//! `painter_bridge_overlays::draw_overlays` while the Painter is active.

use ph2d_editor::HeroScreen;
use ph2d_vector::VectorScene;

/// Radius (screen px) of the Fill (Bucket) ColorDrop cursor swatch.
const FILL_CURSOR_R: f64 = 8.0;

/// While a Fill ColorDrop drag is armed, draw a small filled disc of the current paint colour at the
/// cursor — the "colour being dragged onto the canvas" affordance — with a subtle contrast ring so it
/// reads on a same-coloured canvas. Shown until the drop lands (then the Fill-adjust modal opens).
pub(super) fn draw_fill_cursor(
    hero: &HeroScreen,
    vector_scene: &mut VectorScene,
    cursor: (f32, f32),
) {
    if !crate::input_dispatch::fill_drag::fill_drag_armed() {
        return;
    }
    use ph2d_vector::{Affine, Brush, Circle, Color, Fill, Point, Stroke};
    let rgba = hero
        .store
        .widget_color(ph2d_editor::ids::PAINTER_COLOR_THUMB)
        .unwrap_or([0x88, 0x88, 0x88, 0xFF]); // LITERAL-COLOR-OK: neutral default before a colour is set
    let color = Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]);
    let center = Point::new(f64::from(cursor.0), f64::from(cursor.1));
    let disc = Circle::new(center, FILL_CURSOR_R);
    let scene = vector_scene.inner_mut();
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(color),
        None,
        &disc,
    );
    let ring = Color::new([0.1, 0.1, 0.1, 0.6]); // LITERAL-COLOR-OK: subtle contrast ring around the swatch
    scene.stroke(
        &Stroke::new(1.0),
        Affine::IDENTITY,
        &Brush::Solid(ring),
        None,
        &disc,
    );
}
