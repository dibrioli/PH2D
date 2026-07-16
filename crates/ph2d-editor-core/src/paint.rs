//! [`Paint`] trait + impls — widget tree → `vello::Scene` (M11+M12).
//!
//! This is the **lowering pass** that takes the data-only widget
//! representation (zones, panels, buttons, toasts) and emits Vello
//! draw commands. The shell calls each top-level widget's `paint`
//! once per frame, then hands the resulting `VectorScene` to
//! `ph2d_render::VelloPass` which composites onto the wgpu surface.
//!
//! **Color resolution** goes through `ph2d_tokens::ColorToken::resolve(theme)`
//! so swapping Dark↔Light flips the entire UI.
//!
//! **Text rendering** uses [`paint_text`] / [`paint_text_centered`] —
//! parley layout → `vello::Scene::draw_glyphs` per parley `GlyphRun`.
//! Tabs / actions / button labels / toast messages all render visibly
//! with the system sans-serif fallback chain.

// `FloatingPanel` paint impl retired 2026-05-17 — struct itself
// remains in `crate::floating_panel` for tool event dispatch but no
// longer needs a paint helper import here.
use crate::icons::{IconId, cmd_to_path};
use crate::toast::ToastQueue;
// `paint_color_swatch`, `paint_radio_group`, `paint_slider`, and
// `paint_toggle` were used inside `impl Paint for FloatingPanel`
// (retired 2026-05-17). They still ship from `crate::widget` for
// direct consumers (Inspector, Grid Snap panel, etc.); paint.rs just
// no longer references them.
use crate::zones::{Layout, Rect, Zone};
use ph2d_text::{FontWeight, PositionedLayoutItem, TextSystem};
use ph2d_tokens::{Color as TokenColor, ColorToken, Theme};
use ph2d_vector::{
    Affine, BezPath, Brush, Circle, Color, Fill, Glyph, Rect as VelloRect, RoundedRect, Stroke,
    VectorScene,
};

/// Per-frame paint context. Built by the shell, threaded through
/// every widget's [`Paint::paint`] call.
pub struct PaintCtx<'a> {
    pub theme: Theme,
    /// Window / surface rect in device pixels. Widgets that don't
    /// own their own anchoring (toasts, panels with `Free` anchor)
    /// position themselves relative to this.
    pub viewport: Rect,
    /// Owned long-lived parley state. v1 doesn't touch this; here
    /// to keep the trait signature stable when text lands.
    pub text: &'a mut TextSystem,
}

pub trait Paint {
    fn paint(&self, scene: &mut VectorScene, ctx: &mut PaintCtx);
}

/// Convert a `ph2d_tokens::Color` (sRGB 8-bit + alpha) to Vello's
/// `peniko::Color`. Vello stores its native palette as `[r,g,b,a]`
/// f32 in linear sRGB, but `from_rgba8` does the linearization for
/// us so we keep token semantics intact.
pub fn token_to_vello(tc: TokenColor) -> Color {
    Color::from_rgba8(tc.r, tc.g, tc.b, tc.a)
}

pub fn resolve(token: ColorToken, theme: Theme) -> Color {
    token_to_vello(token.resolve(theme))
}

/// Convert our pixel `Rect` to a Vello `Rect` (which uses f64).
pub fn rect_to_vello(r: Rect) -> VelloRect {
    VelloRect::new(
        r.x as f64,
        r.y as f64,
        (r.x + r.w) as f64,
        (r.y + r.h) as f64,
    )
}

// Thread-local global radius multiplier. Lets the user pick Sharp /
// Default / Round in the topbar context menu and have every rounded
// surface (200+ sites) scale uniformly without threading the value
// through every painter signature. Set via `set_radius_scale` before
// paint, read via `radius_scale`. Defaults to 1.0.
thread_local! {
    static RADIUS_SCALE: std::cell::Cell<f32> = const { std::cell::Cell::new(1.0) };
}

/// Set the global radius scale for the current thread (paint runs).
/// Clamped to non-negative; pass `1.0` to reset.
pub fn set_radius_scale(scale: f32) {
    RADIUS_SCALE.with(|s| s.set(scale.max(0.0)));
}

/// Set the active text-rendering strategy for the current thread.
/// Called by the shell once per frame, mirroring `set_radius_scale`.
/// Delegates to `ph2d_text::set_active_text_rendering` — the canonical
/// thread-local lives there so `TextSystem::prefix_width` can read it
/// internally (fixing the caret-position bug under CrispHeavy where
/// measurements used Medium 500 while glyphs render ExtraBold 800).
pub fn set_text_rendering(mode: ph2d_tokens::TextRendering) {
    ph2d_text::set_active_text_rendering(mode);
}

/// Read the active text-rendering strategy. Delegates to
/// `ph2d_text::active_text_rendering` — see `set_text_rendering` for
/// why the state lives in ph2d-text.
pub fn text_rendering() -> ph2d_tokens::TextRendering {
    ph2d_text::active_text_rendering()
}

/// Apply the snap strategy to a glyph X. `None` returns `x` unchanged
/// (preserves subpixel kerning); `Half` snaps to 0.5 px (preserves ~50 %
/// of kerning); `Full` snaps to 1 px integer (full pixel alignment, no
/// subpixel kerning). Trade-off documented on [`ph2d_tokens::SnapX`].
fn snap_x_apply(x: f32, snap: ph2d_tokens::SnapX) -> f32 {
    match snap {
        ph2d_tokens::SnapX::None => x,
        ph2d_tokens::SnapX::Half => (x * 2.0).round() * 0.5,
        ph2d_tokens::SnapX::Full => x.round(),
    }
}

/// Read the current global radius scale.
pub fn radius_scale() -> f32 {
    RADIUS_SCALE.with(|s| s.get())
}

fn scale_radius(r: f32) -> f32 {
    let s = radius_scale();
    if s == 1.0 {
        r
    } else {
        // Preserve perfect-circle / pill semantics — `Radius::Full`
        // (999) was chosen specifically so it always wraps to the
        // shortest axis. Scaling it would un-pill pills at scale < 1.
        if r >= 999.0 { r } else { r * s }
    }
}

/// Fill a rect with rounded corners. Pass `radius == 0` for sharp.
pub fn fill_rounded_rect(scene: &mut VectorScene, rect: Rect, radius: f32, color: Color) {
    let radius = scale_radius(radius);
    if radius <= 0.0 {
        scene.fill_rect(rect_to_vello(rect), color);
        return;
    }
    let rr = RoundedRect::new(
        rect.x as f64,
        rect.y as f64,
        (rect.x + rect.w) as f64,
        (rect.y + rect.h) as f64,
        radius as f64,
    );
    scene
        .inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, color, None, &rr);
}

/// Stroke an axis-aligned rect (sharp corners) with the given line
/// width and color. Default `Stroke` uses round joins/caps.
pub fn stroke_rect(scene: &mut VectorScene, rect: Rect, width: f32, color: Color) {
    let stroke = Stroke::new(width as f64);
    scene
        .inner_mut()
        .stroke(&stroke, Affine::IDENTITY, color, None, &rect_to_vello(rect));
}

/// Stroke a rect with rounded corners. Same defaults as
/// [`stroke_rect`]. Pass `radius == 0` to fall through to sharp.
pub fn stroke_rounded_rect(
    scene: &mut VectorScene,
    rect: Rect,
    radius: f32,
    width: f32,
    color: Color,
) {
    let radius = scale_radius(radius);
    if radius <= 0.0 {
        stroke_rect(scene, rect, width, color);
        return;
    }
    let rr = RoundedRect::new(
        rect.x as f64,
        rect.y as f64,
        (rect.x + rect.w) as f64,
        (rect.y + rect.h) as f64,
        radius as f64,
    );
    let stroke = Stroke::new(width as f64);
    scene
        .inner_mut()
        .stroke(&stroke, Affine::IDENTITY, color, None, &rr);
}

/// Fill a circle centered at `(cx, cy)` with radius `r` — curve-editor control-
/// point handles (W4 §3) and any round dot. `r <= 0` is a no-op.
pub fn fill_circle(scene: &mut VectorScene, cx: f32, cy: f32, r: f32, color: Color) {
    if r <= 0.0 {
        return;
    }
    let circle = Circle::new((cx as f64, cy as f64), r as f64);
    scene
        .inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, color, None, &circle);
}

/// Stroke a polyline through `points` (round joins/caps) — a smooth curve drawn
/// through its sampled output (W4 §3), replacing the dense-dot fallback. Fewer
/// than two points is a no-op.
pub fn stroke_polyline(scene: &mut VectorScene, points: &[(f32, f32)], width: f32, color: Color) {
    if points.len() < 2 {
        return;
    }
    let mut path = BezPath::new();
    path.move_to((points[0].0 as f64, points[0].1 as f64));
    for &(x, y) in &points[1..] {
        path.line_to((x as f64, y as f64));
    }
    let stroke = Stroke::new(width as f64);
    scene
        .inner_mut()
        .stroke(&stroke, Affine::IDENTITY, color, None, &path);
}

/// Render an icon centered inside `rect`. The source icons are 24x24
/// and stroked with a 1.5pt line; we scale uniformly to fit, keeping
/// 2px of padding so glyphs don't kiss the rect edge.
pub fn paint_icon(
    scene: &mut VectorScene,
    icon: IconId,
    rect: Rect,
    color: Color,
    stroke_width: f32,
) {
    let mut path = BezPath::new();
    for cmd in icon.cmds() {
        path.extend(cmd_to_path(*cmd));
    }
    paint_icon_path(scene, &path, rect, color, stroke_width);
}

/// Same as [`paint_icon`] but takes a pre-built [`BezPath`] in the
/// canonical 24×24 icon design space. Used by Wave 2 PR 11.4 chrome
/// derivation — manifest `icon_fn` returns a `BezPath` directly, so
/// the editor can paint registry-derived pills without an
/// `IconId` enum round-trip.
pub fn paint_icon_path(
    scene: &mut VectorScene,
    path: &BezPath,
    rect: Rect,
    color: Color,
    stroke_width: f32,
) {
    const VIEWBOX: f64 = 24.0;
    let pad = 2.0_f32.min(rect.w.min(rect.h) * 0.1);
    let avail_w = (rect.w - pad * 2.0).max(0.0) as f64;
    let avail_h = (rect.h - pad * 2.0).max(0.0) as f64;
    if avail_w <= 0.0 || avail_h <= 0.0 {
        return;
    }
    let scale = (avail_w / VIEWBOX).min(avail_h / VIEWBOX);
    let drawn = VIEWBOX * scale;
    // Pixel-snap the icon origin: with `stroke_width = 1.5px`, sub-pixel
    // translation makes MSAA16 stipple the stroke across two rows of
    // pixels. Rounding to the nearest integer pixel keeps strokes
    // anchored to a predictable grid position (up to 0.5 px visual
    // shift from "ideal" centering — invisible at icon sizes, big
    // crispness win on near-horizontal/vertical strokes).
    let tx = (rect.x as f64 + (rect.w as f64 - drawn) * 0.5).round();
    let ty = (rect.y as f64 + (rect.h as f64 - drawn) * 0.5).round();
    let transform = Affine::translate((tx, ty)) * Affine::scale(scale);
    let stroke = Stroke::new(stroke_width as f64);
    let brush = Brush::Solid(color);
    scene
        .inner_mut()
        .stroke(&stroke, transform, &brush, None, path);
}

/// Lay out `text` via parley + emit a glyph run for each parley
/// [`PositionedLayoutItem::GlyphRun`] at `(x, y)` (top-left origin).
/// `font_size` is in device-independent pixels; `max_width` is the
/// wrap budget (pass `f32::INFINITY` for single-line).
#[allow(clippy::too_many_arguments)]
pub fn paint_text(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
) {
    paint_text_weighted(
        text_system,
        scene,
        text,
        x,
        y,
        font_size,
        max_width,
        color,
        FontWeight::MEDIUM,
    );
}

/// SemiBold (600) variant of [`paint_text`] for panel titles and
/// other prominent headings. See [`TextSystem::layout_with_weight`]
/// for why titles need the extra weight: diagonals in glyphs like
/// "y" hint poorly at small sizes without LCD subpixel AA, and the
/// extra pen mass closes the perceptual gap with vertical-stem
/// letters in the same word.
#[allow(clippy::too_many_arguments)]
pub fn paint_text_title(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
) {
    paint_text_weighted(
        text_system,
        scene,
        text,
        x,
        y,
        font_size,
        max_width,
        color,
        FontWeight::SEMI_BOLD,
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_text_weighted(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
    weight: FontWeight,
) {
    let rendering = text_rendering();
    let layout = text_system.layout_for_rendering(text, font_size, max_width, weight, rendering);
    let inner = scene.inner_mut();
    // Snap the text origin to integer pixels: hinting snaps stems to the
    // glyph's local pixel grid, but if the *baseline* lands at a
    // fractional Y the snapped grid is itself offset → soft. Callers
    // routinely produce fractional Y from vertical centering math like
    // `rect.y + (rect.h - font_size) * 0.5`. Rounding here makes every
    // caller crisp without each one having to remember to align.
    let translate = Affine::translate((x.round() as f64, y.round() as f64));
    let params = rendering.params();
    let snap_x = params.snap_x;
    let hint = params.hint;
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };
            let run = glyph_run.run();
            let font = run.font();
            let run_font_size = run.font_size();
            inner
                .draw_glyphs(font)
                .font_size(run_font_size)
                // Hint per preset. `true` snapa stems ao pixel grid
                // (crisp); `false` deixa o eixo wght variable fluir
                // sem quantização (necessário para CrispHeavy ficar
                // visualmente distinto de Crisp a 11-12 px).
                .hint(hint)
                // **Critical**: forward parley's per-run variation
                // coordinates (already includes the wght axis we
                // pushed in `layout_for_rendering`). Without this,
                // Vello rasterizes glyphs with the font's default
                // axis values — Inter Variable falls back to ~Regular
                // 400 regardless of which weight stop was selected,
                // so Crisp Heavy looks identical to Crisp. The slice
                // is `&[i16]` on both sides (parley + vello typedef
                // NormalizedCoord = i16), so no conversion needed.
                .normalized_coords(run.normalized_coords())
                .brush(color)
                .transform(translate)
                .draw(
                    Fill::NonZero,
                    glyph_run.positioned_glyphs().map(|g| Glyph {
                        id: g.id,
                        // Snap glyph Y to integer to keep the baseline
                        // pixel-aligned per glyph. X snap depends on
                        // the current `TextRendering` preset's
                        // `SnapX` strategy (None / Half / Full).
                        x: snap_x_apply(g.x, snap_x),
                        y: g.y.round(),
                    }),
                );
        }
    }
}

/// Like [`paint_text`] but rotates the layout 90° counter-clockwise
/// so the text reads bottom-to-top. The anchor `(anchor_x, anchor_y)`
/// is where the rotated baseline's left edge lands — visually this is
/// the BOTTOM-left of the painted text. `max_width` constrains the
/// pre-rotation layout width (i.e. the visual HEIGHT after rotation).
///
/// Used by the LeftRail to paint per-button sub-labels in the column
/// to the left of the chips.
#[allow(clippy::too_many_arguments)]
pub fn paint_text_rotated_ccw(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    anchor_x: f32,
    anchor_y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
) {
    // Apply the same TextRendering strategy as the straight painter
    // — `layout_for_rendering` bumps the FontWeight per the preset's
    // tier and the rotated glyph loop honors snap-X (pre-rotation
    // coords; the 90° rotation is axis-aligned so post-rotation pixel
    // alignment is preserved by the snap).
    let rendering = text_rendering();
    let layout =
        text_system.layout_for_rendering(text, font_size, max_width, FontWeight::MEDIUM, rendering);
    let inner = scene.inner_mut();
    // Rotate 90° CCW around the anchor, then translate to it.
    let transform = Affine::translate((anchor_x as f64, anchor_y as f64))
        * Affine::rotate(-std::f64::consts::FRAC_PI_2);
    let params = rendering.params();
    let snap_x = params.snap_x;
    let hint = params.hint;
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };
            let run = glyph_run.run();
            let font = run.font();
            let run_font_size = run.font_size();
            inner
                .draw_glyphs(font)
                .font_size(run_font_size)
                // Hint per preset (same as the straight painter — see
                // `paint_text_weighted`). Hinting under a 90° rotation:
                // skrifa snaps to the *layout* pixel grid pre-rotation,
                // axis-aligned so post-rotation grid alignment is 1:1.
                .hint(hint)
                // Forward parley's per-run variation coords (wght +
                // opsz). See `paint_text_weighted` for why this is
                // critical — without it Vello ignores the weight stop.
                .normalized_coords(run.normalized_coords())
                .brush(color)
                .transform(transform)
                .draw(
                    Fill::NonZero,
                    glyph_run.positioned_glyphs().map(|g| Glyph {
                        id: g.id,
                        // Snap X pre-rotation in Crisp; rotation
                        // turns this into snap-Y in screen space —
                        // which aligns rotated stems to columns.
                        x: snap_x_apply(g.x, snap_x),
                        y: g.y,
                    }),
                );
        }
    }
}

/// Tool palette paint helper — for each `(rect, label, is_active)`
/// triple, draws a 36×36 icon chip in the TopRight zone:
///   - Active: AccentPrimary background + inverted text
///   - Hover (when state != Normal): SurfaceElevated
///   - Idle: Surface
///
/// Until a real icon set ships, the icon is just the first character
/// of the tool's label centered in the chip.
pub fn paint_tool_palette_icons(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    icons: &[(Rect, &str, bool)],
    theme: Theme,
) {
    for &(rect, label, is_active) in icons {
        let (bg, fg) = if is_active {
            (
                resolve(ColorToken::Accent, theme),
                resolve(ColorToken::AccentFg, theme),
            )
        } else {
            (
                resolve(ColorToken::Bg1, theme),
                resolve(ColorToken::Text1, theme),
            )
        };
        scene.fill_rect(rect_to_vello(rect), bg);
        // 1-px Border ring on inactive icons so they don't
        // disappear into the surrounding zone fill.
        if !is_active {
            let border_color = resolve(ColorToken::Border, theme);
            stroke_rect(scene, rect, 1.0, border_color);
        }
        let glyph = label.chars().next().unwrap_or('?').to_string();
        paint_text_centered(text_system, scene, &glyph, rect, 18.0, fg);
    }
}

/// Center `text` inside `rect` (horizontally + vertically).
pub fn paint_text_centered(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    rect: Rect,
    font_size: f32,
    color: Color,
) {
    let layout = text_system.layout(text, font_size, rect.w);
    let text_w = layout.width();
    let text_h = layout.height();
    let x = rect.x + (rect.w - text_w) / 2.0;
    let y = rect.y + (rect.h - text_h) / 2.0;
    paint_text(text_system, scene, text, x, y, font_size, rect.w, color);
}

// -----------------------------------------------------------------------
// Layout — 4 zones backdrop
// -----------------------------------------------------------------------

impl Paint for Layout {
    fn paint(&self, scene: &mut VectorScene, ctx: &mut PaintCtx) {
        // Center is canvas — the sprite layer underneath shows through;
        // we deliberately do NOT paint Center.
        let surface = resolve(ColorToken::Bg1, ctx.theme);
        let border = resolve(ColorToken::Border, ctx.theme);
        let border_emphasis = resolve(ColorToken::BorderEmph, ctx.theme);
        let label_color = resolve(ColorToken::Text2, ctx.theme);

        for zone in [Zone::TopLeft, Zone::TopRight, Zone::Sidebar] {
            let r = self.rect(zone);
            if r.area() <= 0.0 {
                continue; // Zen mode — zone collapsed.
            }
            // Surface fill.
            scene.fill_rect(rect_to_vello(r), surface);

            // Zone label — top-left/right have CREATE/EDIT per ADR-0023
            // §3, sidebar gets MOD label oriented vertically (we just
            // render text horizontally for now — vertical text needs
            // vello transform handling that's a follow-up).
            let label = match zone {
                Zone::TopLeft => Some("EDIT"),
                Zone::TopRight => Some("CREATE"),
                Zone::Sidebar => Some("MOD"),
                Zone::Center => None,
            };
            if let Some(label) = label {
                // Inset 12 px from the seam-facing edge so the label
                // doesn't kiss the divider line.
                let label_rect = match zone {
                    Zone::Sidebar => Rect {
                        x: r.x,
                        y: r.y + 12.0,
                        w: r.w,
                        h: 14.0,
                    },
                    _ => Rect {
                        x: r.x + 16.0,
                        y: r.y,
                        w: r.w - 32.0,
                        h: r.h,
                    },
                };
                paint_text_centered(ctx.text, scene, label, label_rect, 11.0, label_color);
            }

            // 1-px divider along the seam toward Center (BorderEmphasis
            // when sidebar; Border for top zones — quieter line above
            // the canvas).
            let divider = match zone {
                Zone::TopLeft | Zone::TopRight => Rect {
                    x: r.x,
                    y: r.y + r.h - 1.0,
                    w: r.w,
                    h: 1.0,
                },
                Zone::Sidebar => match self.sidebar_side {
                    crate::zones::SidebarSide::Right => Rect {
                        x: r.x,
                        y: r.y,
                        w: 1.0,
                        h: r.h,
                    },
                    crate::zones::SidebarSide::Left => Rect {
                        x: r.x + r.w - 1.0,
                        y: r.y,
                        w: 1.0,
                        h: r.h,
                    },
                },
                Zone::Center => continue,
            };
            let divider_color = if matches!(zone, Zone::Sidebar) {
                border_emphasis
            } else {
                border
            };
            scene.fill_rect(rect_to_vello(divider), divider_color);
        }

        // Mirror-side toggle button on the sidebar — small chip at top
        // edge. Click target exposed via [`Layout::mirror_button_rect`]
        // so the shell can hit-test it.
        if let Some(btn) = self.mirror_button_rect() {
            scene.fill_rect(rect_to_vello(btn), resolve(ColorToken::BgElev, ctx.theme));
            let glyph = match self.sidebar_side {
                crate::zones::SidebarSide::Right => "<",
                crate::zones::SidebarSide::Left => ">",
            };
            paint_text_centered(ctx.text, scene, glyph, btn, 14.0, label_color);
        }
    }
}

// -----------------------------------------------------------------------
// FloatingPanel paint retired 2026-05-17.
//
// The legacy `impl Paint for FloatingPanel` rendered a Procreate-style
// drawer with an Accent-colored tab strip + Accent-tinted control row.
// That decoration predated the dark-glass canonical surface used by
// Inspector / Hierarchy / Widget Gallery and stuck out visually (pink
// Transform panel hovering over a dark editor). It was the last
// residual old-decoration site flagged in the 2026-05-17 UI audit.
//
// `Tool::build_panel()` and the `FloatingPanel` struct itself are
// kept — they still encode the tool's intended controls + tab model,
// useful as input to a future panel re-paint that uses
// `paint_panel_surface` (the canonical dark-glass painter). Removing
// only the `Paint` trait impl (and its `tab_color` / `action_label`
// helpers) cuts the visual without touching the data model.
// -----------------------------------------------------------------------

// Button paint helper now lives at crate::widget::paint_button — see
// `widget/button.rs` for the rounded-rect, kind-aware implementation.

// -----------------------------------------------------------------------
// ToastQueue — stacked at top-center
// -----------------------------------------------------------------------

impl Paint for ToastQueue {
    fn paint(&self, scene: &mut VectorScene, ctx: &mut PaintCtx) {
        use crate::toast::ToastSeverity;
        use ph2d_tokens::Radius;
        // The toast stream owns the TOP of the top-center column and the job bars
        // (`progress::JobQueue`) stack under it — a toast lives three seconds and gets one
        // chance to be read, so its slot must not move because some background job happens to
        // be running. `column_row` is the shared ruler for both tenants; it lives over there
        // because this file is at its frozen LOC ceiling (see the workspace LOC-cap gate).
        let radius = Radius::Md.px();
        for (i, toast) in self.iter().enumerate() {
            let r = crate::progress::column_row(ctx.viewport, i);
            // Body uses BgElev so the toast lifts off the canvas
            // independently of its severity tint; the severity color
            // is reserved for the icon + accent stripe on the left.
            fill_rounded_rect(scene, r, radius, resolve(ColorToken::BgElev, ctx.theme));
            stroke_rounded_rect(
                scene,
                r,
                radius,
                1.0,
                resolve(ColorToken::Border, ctx.theme),
            );

            let (severity_token, icon) = match toast.severity {
                ToastSeverity::Info => (ColorToken::Info, crate::icons::IconId::Info),
                ToastSeverity::Success => (ColorToken::Success, crate::icons::IconId::Check),
                ToastSeverity::Warning => (ColorToken::Warn, crate::icons::IconId::Warning),
                ToastSeverity::ErrorState => (ColorToken::Danger, crate::icons::IconId::Error),
            };
            let severity_color = resolve(severity_token, ctx.theme);

            // Left accent stripe — 4 px wide, full height, severity-tinted.
            let stripe = Rect::new(r.x + 1.0, r.y + 1.0, 4.0, r.h - 2.0);
            scene.fill_rect(rect_to_vello(stripe), severity_color);

            // Severity icon, centered in a 28x28 chip after the stripe.
            let icon_w = 24.0;
            let icon_rect = Rect::new(r.x + 16.0, r.y + (r.h - icon_w) * 0.5, icon_w, icon_w);
            paint_icon(scene, icon, icon_rect, severity_color, 1.5);

            // Message text fills the rest, left-aligned with padding.
            let text_x = icon_rect.x + icon_w + 12.0;
            let text_rect = Rect {
                x: text_x,
                y: r.y,
                w: (r.x + r.w - text_x - 16.0).max(0.0),
                h: r.h,
            };
            paint_text_centered(
                ctx.text,
                scene,
                &toast.message,
                text_rect,
                13.0,
                resolve(ColorToken::Text1, ctx.theme),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `radius_scale` is a thread-local — paint_rounded_rect helpers
    /// scale by it, but `Radius::Full` (999) is preserved exactly so
    /// pill / circle shapes stay perfect even at scale < 1.
    #[test]
    fn radius_scale_preserves_full_pill() {
        set_radius_scale(0.2);
        assert!((scale_radius(999.0) - 999.0).abs() < f32::EPSILON);
        assert!((scale_radius(12.0) - 2.4).abs() < 1e-4);
        set_radius_scale(1.6);
        assert!((scale_radius(999.0) - 999.0).abs() < f32::EPSILON);
        assert!((scale_radius(8.0) - 12.8).abs() < 1e-4);
        set_radius_scale(1.0);
    }

    /// `text_rendering` thread-local round-trip + default.
    #[test]
    fn text_rendering_thread_local_roundtrip() {
        // Start from a known state; not testing default-on-fresh-thread
        // (other tests in this binary may have set the cell).
        set_text_rendering(ph2d_tokens::TextRendering::Default);
        assert_eq!(text_rendering(), ph2d_tokens::TextRendering::Default);
        set_text_rendering(ph2d_tokens::TextRendering::CrispHeavy);
        assert_eq!(text_rendering(), ph2d_tokens::TextRendering::CrispHeavy);
        // Reset so we don't leak state into sibling tests.
        set_text_rendering(ph2d_tokens::TextRendering::Default);
    }

    /// Token → Vello color round-trips bytes accurately enough for
    /// our 8-bit palette (no banding from the sRGB linearization).
    #[test]
    fn token_to_vello_preserves_visible_difference() {
        let bg0 = ColorToken::Bg0.resolve(Theme::Forge);
        let bg1 = ColorToken::Bg1.resolve(Theme::Forge);
        // Tokens are different ⇒ vello colors must differ.
        assert_ne!(token_to_vello(bg0), token_to_vello(bg1));
    }

    /// Painting a 4-zone Layout doesn't panic and emits at least the
    /// 3 non-Center zones' rects (Center stays transparent).
    #[test]
    fn layout_paint_smoke() {
        let layout = Layout::new(1024.0, 768.0);
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let mut ctx = PaintCtx {
            theme: Theme::Forge,
            viewport: Rect::new(0.0, 0.0, 1024.0, 768.0),
            text: &mut text,
        };
        layout.paint(&mut scene, &mut ctx);
        // Vello doesn't expose a public command count, so we settle
        // for "encoder didn't panic + reset is idempotent".
        scene.reset();
    }

    // `floating_panel_paint_collapsed_emits_only_chip` removed
    // alongside the `impl Paint for FloatingPanel` retirement
    // (2026-05-17). The struct itself stays for `Tool::build_panel`
    // event dispatch; only the legacy decoration was dropped.

    #[test]
    fn fill_rounded_rect_smoke() {
        let mut scene = VectorScene::new();
        let color = Color::WHITE;
        // sharp fall-through
        fill_rounded_rect(&mut scene, Rect::new(0.0, 0.0, 10.0, 10.0), 0.0, color);
        // rounded
        fill_rounded_rect(&mut scene, Rect::new(0.0, 0.0, 24.0, 24.0), 6.0, color);
    }

    #[test]
    fn stroke_rect_smoke() {
        let mut scene = VectorScene::new();
        stroke_rect(
            &mut scene,
            Rect::new(0.0, 0.0, 100.0, 50.0),
            1.0,
            Color::WHITE,
        );
        stroke_rect(
            &mut scene,
            Rect::new(0.0, 0.0, 100.0, 50.0),
            2.0,
            Color::BLACK,
        );
    }

    #[test]
    fn stroke_rounded_rect_smoke() {
        let mut scene = VectorScene::new();
        stroke_rounded_rect(
            &mut scene,
            Rect::new(0.0, 0.0, 80.0, 40.0),
            8.0,
            1.5,
            Color::WHITE,
        );
        // sharp fall-through
        stroke_rounded_rect(
            &mut scene,
            Rect::new(0.0, 0.0, 80.0, 40.0),
            0.0,
            1.0,
            Color::WHITE,
        );
    }

    #[test]
    fn paint_icon_smoke() {
        let mut scene = VectorScene::new();
        for icon in [
            IconId::Add,
            IconId::Check,
            IconId::Save,
            IconId::Settings,
            IconId::Sprite,
        ] {
            paint_icon(
                &mut scene,
                icon,
                Rect::new(10.0, 10.0, 36.0, 36.0),
                Color::WHITE,
                1.5,
            );
        }
    }

    #[test]
    fn paint_icon_zero_size_does_nothing() {
        let mut scene = VectorScene::new();
        // Zero/negative size must not panic and must early-return.
        paint_icon(
            &mut scene,
            IconId::Add,
            Rect::new(0.0, 0.0, 0.0, 0.0),
            Color::WHITE,
            1.5,
        );
        paint_icon(
            &mut scene,
            IconId::Add,
            Rect::new(0.0, 0.0, 1.0, 1.0),
            Color::WHITE,
            1.5,
        );
    }

    #[test]
    fn toast_queue_paint_with_three_severities() {
        use crate::toast::Toast;
        let mut q = ToastQueue::new();
        q.push(Toast::info("info"));
        q.push(Toast::success("success"));
        q.push(Toast::warning("warn"));
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let mut ctx = PaintCtx {
            theme: Theme::Sunstone,
            viewport: Rect::new(0.0, 0.0, 800.0, 600.0),
            text: &mut text,
        };
        q.paint(&mut scene, &mut ctx);
    }
}
