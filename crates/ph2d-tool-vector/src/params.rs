//! Vector-tool Style UI vocabulary — the snapshot the docked
//! `ph2d-panel-vector` paints, plus the Width slider ↔ px mapping shared by
//! the panel (populate/paint) and the tool (`handle_panel_event`).
//!
//! Mirrors `ph2d_tool_padding::params`: the tool owns the authoritative Style,
//! projects it into a [`VectorStyleSnapshot`] each frame (published by the
//! shell bridge → the panel reads it), and both sides agree on the affine
//! slider mapping so a drag and the tool stay in lock-step.

/// Minimum / maximum stroke width in screen pixels (inclusive range the Width
/// slider spans).
pub const WIDTH_MIN_PX: f64 = 1.0;
pub const WIDTH_MAX_PX: f64 = 20.0;

/// Affine slider mapping `display_px = track * SCALE + OFFSET` (track `0..=1`),
/// consumed by `WidgetStore::link_slider_number_mapped` so the px chip mirrors
/// the slider. `SCALE = MAX - MIN`, `OFFSET = MIN`.
pub const WIDTH_SLIDER_SCALE: f32 = (WIDTH_MAX_PX - WIDTH_MIN_PX) as f32;
pub const WIDTH_SLIDER_OFFSET: f32 = WIDTH_MIN_PX as f32;

/// Normalized slider track `0..=1` → stroke width px `MIN..=MAX`.
#[must_use]
pub fn slider_to_px(track: f32) -> f64 {
    WIDTH_MIN_PX + f64::from(track.clamp(0.0, 1.0)) * (WIDTH_MAX_PX - WIDTH_MIN_PX)
}

/// Stroke width px → normalized slider track `0..=1` (inverse of
/// [`slider_to_px`]). Used to seed the slider knob from the tool's authoritative
/// width so it renders correctly before the first drag.
#[must_use]
pub fn px_to_slider(px: f64) -> f32 {
    (((px - WIDTH_MIN_PX) / (WIDTH_MAX_PX - WIDTH_MIN_PX)) as f32).clamp(0.0, 1.0)
}

/// The canvas gesture the Vector tool performs (ADR-0108 Fase 1). `Pen` is the
/// draw + edit-anchor gesture (`PenTool`); the shape modes are drag-to-size
/// (`ShapeTool`). The tool owns the mode; the docked panel's segmented row sets
/// it and highlights the active one from the published snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DrawMode {
    /// Seta preta: seleciona e TRANSFORMA a forma pelo gizmo. Não toca a geometria.
    #[default]
    Select,
    /// Seta branca: edita âncoras e handles do path selecionado. Nunca cria um path,
    /// e o gizmo não aparece (as alças dele comeriam o clique do nó).
    Node,
    /// Caneta: cria path novo e edita os nós que ela mesma pôs. Sem gizmo.
    Pen,
    Rectangle,
    Ellipse,
    Polygon,
    Star,
    RoundRect,
    Spiral,
    Line,
    Arc,
}

/// UI-facing vertex type for the docked panel's Vertex section (mirror of
/// `ph2d_vec_scene::VertexKind`; the shell maps between them). Lives in the tool
/// crate — the panel deps this, not `ph2d-vec-scene` — alongside [`DrawMode`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VertexType {
    Corner,
    Smooth,
    Symmetric,
}

/// UI-facing line cap / join (mirror of `ph2d_vec_scene::{LineCap, LineJoin}`;
/// the shell maps between them — the tool crate doesn't dep `ph2d-vec-scene`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StrokeCap {
    #[default]
    Butt,
    Round,
    Square,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StrokeJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

/// Dash range as a **multiple of the stroke width** (`0` = solid). Width-aware:
/// the render draws dash/gap of `dash·width`, so a thick line keeps its gaps.
pub const DASH_MIN: f64 = 0.0;
pub const DASH_MAX: f64 = 8.0;
pub const DASH_SLIDER_SCALE: f32 = (DASH_MAX - DASH_MIN) as f32;
pub const DASH_SLIDER_OFFSET: f32 = DASH_MIN as f32;

/// Normalized track `0..=1` → dash multiple `MIN..=MAX`.
#[must_use]
pub fn slider_to_dash(track: f32) -> f64 {
    DASH_MIN + f64::from(track.clamp(0.0, 1.0)) * (DASH_MAX - DASH_MIN)
}
/// Dash multiple → normalized track (inverse of [`slider_to_dash`]).
#[must_use]
pub fn dash_to_slider(m: f64) -> f32 {
    ((m.clamp(DASH_MIN, DASH_MAX) - DASH_MIN) / (DASH_MAX - DASH_MIN)) as f32
}

/// Gap range as a **multiple of the stroke width**, independent of the dash
/// length — the render draws the space between dashes as `gap·width`. Same
/// width-aware model as [`slider_to_dash`]. Default `1` (Dash = 0 ⇒ solid, so
/// the gap only bites once Dash > 0).
pub const GAP_MIN: f64 = 0.0;
pub const GAP_MAX: f64 = 8.0;
pub const GAP_DEFAULT: f64 = 1.0;
pub const GAP_SLIDER_SCALE: f32 = (GAP_MAX - GAP_MIN) as f32;
pub const GAP_SLIDER_OFFSET: f32 = GAP_MIN as f32;

/// Normalized track `0..=1` → gap multiple `MIN..=MAX`.
#[must_use]
pub fn slider_to_gap(track: f32) -> f64 {
    GAP_MIN + f64::from(track.clamp(0.0, 1.0)) * (GAP_MAX - GAP_MIN)
}
/// Gap multiple → normalized track (inverse of [`slider_to_gap`]).
#[must_use]
pub fn gap_to_slider(m: f64) -> f32 {
    ((m.clamp(GAP_MIN, GAP_MAX) - GAP_MIN) / (GAP_MAX - GAP_MIN)) as f32
}

/// Minimum / maximum polygon sides (inclusive range the Sides slider spans).
pub const SIDES_MIN: u32 = 3;
pub const SIDES_MAX: u32 = 12;

/// Affine Sides-slider mapping `display_n = track * SCALE + OFFSET` (track
/// `0..=1`), consumed by `WidgetStore::link_slider_number_mapped` so the chip
/// mirrors the slider. `SCALE = MAX - MIN`, `OFFSET = MIN`.
pub const SIDES_SLIDER_SCALE: f32 = (SIDES_MAX - SIDES_MIN) as f32;
pub const SIDES_SLIDER_OFFSET: f32 = SIDES_MIN as f32;

/// Normalized slider track `0..=1` → polygon sides `MIN..=MAX` (rounded).
#[must_use]
pub fn slider_to_sides(track: f32) -> u32 {
    (SIDES_MIN as f32 + track.clamp(0.0, 1.0) * (SIDES_MAX - SIDES_MIN) as f32).round() as u32
}

/// Polygon sides → normalized slider track `0..=1` (inverse of
/// [`slider_to_sides`]); seeds the knob from the tool's authoritative sides.
#[must_use]
pub fn sides_to_slider(n: u32) -> f32 {
    ((n.clamp(SIDES_MIN, SIDES_MAX) - SIDES_MIN) as f32 / (SIDES_MAX - SIDES_MIN) as f32)
        .clamp(0.0, 1.0)
}

/// Star point count range (the Points slider spans this).
pub const STAR_POINTS_MIN: u32 = 3;
pub const STAR_POINTS_MAX: u32 = 12;
pub const STAR_POINTS_SLIDER_SCALE: f32 = (STAR_POINTS_MAX - STAR_POINTS_MIN) as f32;
pub const STAR_POINTS_SLIDER_OFFSET: f32 = STAR_POINTS_MIN as f32;

/// Normalized track `0..=1` → star points `MIN..=MAX` (rounded).
#[must_use]
pub fn slider_to_star_points(track: f32) -> u32 {
    (STAR_POINTS_MIN as f32 + track.clamp(0.0, 1.0) * STAR_POINTS_SLIDER_SCALE).round() as u32
}
/// Star points → normalized track (inverse of [`slider_to_star_points`]).
#[must_use]
pub fn star_points_to_slider(n: u32) -> f32 {
    ((n.clamp(STAR_POINTS_MIN, STAR_POINTS_MAX) - STAR_POINTS_MIN) as f32
        / STAR_POINTS_SLIDER_SCALE)
        .clamp(0.0, 1.0)
}

/// Star inner/outer radius ratio range (the Inner slider spans this).
pub const STAR_INNER_MIN: f64 = 0.1;
pub const STAR_INNER_MAX: f64 = 0.9;
pub const STAR_INNER_SLIDER_SCALE: f32 = (STAR_INNER_MAX - STAR_INNER_MIN) as f32;
pub const STAR_INNER_SLIDER_OFFSET: f32 = STAR_INNER_MIN as f32;

/// Normalized track `0..=1` → star inner ratio `MIN..=MAX`.
#[must_use]
pub fn slider_to_star_inner(track: f32) -> f64 {
    STAR_INNER_MIN + f64::from(track.clamp(0.0, 1.0)) * (STAR_INNER_MAX - STAR_INNER_MIN)
}
/// Star inner ratio → normalized track (inverse of [`slider_to_star_inner`]).
#[must_use]
pub fn star_inner_to_slider(r: f64) -> f32 {
    ((r.clamp(STAR_INNER_MIN, STAR_INNER_MAX) - STAR_INNER_MIN) / (STAR_INNER_MAX - STAR_INNER_MIN))
        as f32
}

/// Rounded-rect corner radius range in **screen pixels** (the Radius slider spans this).
pub const RADIUS_MIN_PX: f64 = 0.0;
pub const RADIUS_MAX_PX: f64 = 40.0;
pub const RADIUS_SLIDER_SCALE: f32 = (RADIUS_MAX_PX - RADIUS_MIN_PX) as f32;
pub const RADIUS_SLIDER_OFFSET: f32 = RADIUS_MIN_PX as f32;

/// Normalized track `0..=1` → corner radius px `MIN..=MAX`.
#[must_use]
pub fn slider_to_radius(track: f32) -> f64 {
    RADIUS_MIN_PX + f64::from(track.clamp(0.0, 1.0)) * (RADIUS_MAX_PX - RADIUS_MIN_PX)
}
/// Corner radius px → normalized track (inverse of [`slider_to_radius`]).
#[must_use]
pub fn radius_to_slider(px: f64) -> f32 {
    ((px.clamp(RADIUS_MIN_PX, RADIUS_MAX_PX) - RADIUS_MIN_PX) / (RADIUS_MAX_PX - RADIUS_MIN_PX))
        as f32
}

/// Spiral turn count range (the Turns slider spans this).
pub const SPIRAL_TURNS_MIN: u32 = 1;
pub const SPIRAL_TURNS_MAX: u32 = 8;
pub const SPIRAL_TURNS_SLIDER_SCALE: f32 = (SPIRAL_TURNS_MAX - SPIRAL_TURNS_MIN) as f32;
pub const SPIRAL_TURNS_SLIDER_OFFSET: f32 = SPIRAL_TURNS_MIN as f32;

/// Normalized track `0..=1` → spiral turns `MIN..=MAX` (rounded).
#[must_use]
pub fn slider_to_spiral_turns(track: f32) -> u32 {
    (SPIRAL_TURNS_MIN as f32 + track.clamp(0.0, 1.0) * SPIRAL_TURNS_SLIDER_SCALE).round() as u32
}
/// Span mínimo/máximo de um arco (graus). O slider mapeia linearmente.
pub const ARC_DEGREES_MIN: f64 = 1.0;
pub const ARC_DEGREES_MAX: f64 = 360.0;
pub const ARC_DEGREES_SLIDER_SCALE: f32 = (ARC_DEGREES_MAX - ARC_DEGREES_MIN) as f32;
pub const ARC_DEGREES_SLIDER_OFFSET: f32 = ARC_DEGREES_MIN as f32;

/// Track `[0,1]` → graus `[1, 360]`.
#[must_use]
pub fn slider_to_arc_degrees(track: f32) -> f64 {
    f64::from(ARC_DEGREES_SLIDER_OFFSET + track.clamp(0.0, 1.0) * ARC_DEGREES_SLIDER_SCALE)
}

/// Graus → track `[0,1]`.
#[must_use]
pub fn arc_degrees_to_slider(deg: f64) -> f32 {
    (((deg as f32) - ARC_DEGREES_SLIDER_OFFSET) / ARC_DEGREES_SLIDER_SCALE).clamp(0.0, 1.0)
}

/// Spiral turns → normalized track (inverse of [`slider_to_spiral_turns`]).
#[must_use]
pub fn spiral_turns_to_slider(n: u32) -> f32 {
    ((n.clamp(SPIRAL_TURNS_MIN, SPIRAL_TURNS_MAX) - SPIRAL_TURNS_MIN) as f32
        / SPIRAL_TURNS_SLIDER_SCALE)
        .clamp(0.0, 1.0)
}

/// Opacity slider: track `0..=1` → alpha `0..=255`; the chip shows `0..=100`
/// (percent), so `SCALE = 100`, `OFFSET = 0`.
pub const OPACITY_SLIDER_SCALE: f32 = 100.0;
pub const OPACITY_SLIDER_OFFSET: f32 = 0.0;

/// Normalized track `0..=1` → alpha byte `0..=255` (rounded).
#[must_use]
pub fn slider_to_opacity(track: f32) -> u8 {
    (track.clamp(0.0, 1.0) * 255.0).round() as u8
}
/// Alpha byte → normalized track (inverse of [`slider_to_opacity`]).
#[must_use]
pub fn opacity_to_slider(a: u8) -> f32 {
    f32::from(a) / 255.0
}

/// Mode + shape parameters the shell mirrors from the tool each frame to route
/// canvas gestures (pen vs shape) and drive the [`ShapeTool`] without a downcast.
///
/// [`ShapeTool`]: ph2d_vec_edit::ShapeTool
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VectorDrawConfig {
    pub mode: DrawMode,
    pub polygon_sides: u32,
    pub star_points: u32,
    pub star_inner_ratio: f64,
    pub corner_radius_px: f64,
    pub spiral_turns: u32,
    pub arc_degrees: f64,
}

impl Default for VectorDrawConfig {
    fn default() -> Self {
        Self {
            mode: DrawMode::Select,
            polygon_sides: super::tool::DEFAULT_POLYGON_SIDES,
            star_points: super::tool::DEFAULT_STAR_POINTS,
            star_inner_ratio: super::tool::DEFAULT_STAR_INNER,
            corner_radius_px: super::tool::DEFAULT_CORNER_RADIUS_PX,
            spiral_turns: super::tool::DEFAULT_SPIRAL_TURNS,
            arc_degrees: super::tool::DEFAULT_ARC_DEGREES,
        }
    }
}

/// Per-frame projection of the tool's Style, published by the shell bridge for
/// the docked panel to paint. `stroke` / `fill` are sRGB8; `fill[3] == 0` ⇒ no
/// fill ("None"). `mode` / `polygon_sides` drive the draw-mode segmented row +
/// the Sides slider.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VectorStyleSnapshot {
    pub stroke: [u8; 4],
    pub fill: [u8; 4],
    pub stroke_width_px: f64,
    pub mode: DrawMode,
    pub polygon_sides: u32,
    pub star_points: u32,
    pub star_inner_ratio: f64,
    pub corner_radius_px: f64,
    pub cap: StrokeCap,
    pub join: StrokeJoin,
    /// Dash as a multiple of stroke width (`0` = solid).
    pub dash: f64,
    /// Gap between dashes as a multiple of stroke width.
    pub gap: f64,
    /// Turn count for `DrawMode::Spiral`.
    pub spiral_turns: u32,
    /// Span in degrees for `DrawMode::Arc`.
    pub arc_degrees: f64,
}

impl Default for VectorStyleSnapshot {
    fn default() -> Self {
        Self {
            stroke: [240, 240, 245, 255],
            fill: [90, 150, 230, 255],
            stroke_width_px: super::tool::DEFAULT_STROKE_WIDTH_PX,
            mode: DrawMode::Pen,
            polygon_sides: super::tool::DEFAULT_POLYGON_SIDES,
            star_points: super::tool::DEFAULT_STAR_POINTS,
            star_inner_ratio: super::tool::DEFAULT_STAR_INNER,
            corner_radius_px: super::tool::DEFAULT_CORNER_RADIUS_PX,
            cap: StrokeCap::Butt,
            join: StrokeJoin::Miter,
            dash: 0.0,
            gap: GAP_DEFAULT,
            spiral_turns: super::tool::DEFAULT_SPIRAL_TURNS,
            arc_degrees: super::tool::DEFAULT_ARC_DEGREES,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_px_round_trip_endpoints() {
        assert_eq!(slider_to_px(0.0), WIDTH_MIN_PX);
        assert_eq!(slider_to_px(1.0), WIDTH_MAX_PX);
        assert!((px_to_slider(WIDTH_MIN_PX) - 0.0).abs() < 1e-6);
        assert!((px_to_slider(WIDTH_MAX_PX) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn slider_mapping_matches_affine_consts() {
        // The panel's chip display uses `track * SCALE + OFFSET`; it must equal
        // the tool's `slider_to_px` for the chip to mirror the slider exactly.
        for &t in &[0.0_f32, 0.25, 0.5, 0.75, 1.0] {
            let via_affine = f64::from(t * WIDTH_SLIDER_SCALE + WIDTH_SLIDER_OFFSET);
            assert!((via_affine - slider_to_px(t)).abs() < 1e-6);
        }
    }

    #[test]
    fn sides_slider_round_trip_endpoints() {
        assert_eq!(slider_to_sides(0.0), SIDES_MIN);
        assert_eq!(slider_to_sides(1.0), SIDES_MAX);
        assert!((sides_to_slider(SIDES_MIN) - 0.0).abs() < 1e-6);
        assert!((sides_to_slider(SIDES_MAX) - 1.0).abs() < 1e-6);
        // Mid-track rounds to the nearest integer side count.
        assert_eq!(slider_to_sides(0.5), (SIDES_MIN + SIDES_MAX) / 2 + 1);
    }

    /// ADR-0112: a ferramenta abre na SELEÇÃO (seta preta), como qualquer editor
    /// vetorial. A caneta é um modo, não o ponto de partida.
    #[test]
    fn draw_mode_defaults_to_select() {
        assert_eq!(DrawMode::default(), DrawMode::Select);
        assert_eq!(VectorDrawConfig::default().mode, DrawMode::Select);
    }
}
