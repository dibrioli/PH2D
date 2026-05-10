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

use crate::floating_panel::{FloatingPanel, PanelAction, PanelControl, PanelTab};
use crate::icons::{IconId, cmd_to_path};
use crate::toast::ToastQueue;
use crate::widget::{paint_color_swatch, paint_radio_group, paint_slider, paint_toggle};
use crate::zones::{Layout, Rect, Zone};
use ph2d_text::{PositionedLayoutItem, TextSystem};
use ph2d_tokens::{Color as TokenColor, ColorToken, Theme};
use ph2d_vector::{
    Affine, BezPath, Brush, Color, Fill, Glyph, Rect as VelloRect, RoundedRect, Stroke, VectorScene,
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

/// Fill a rect with rounded corners. Pass `radius == 0` for sharp.
pub fn fill_rounded_rect(scene: &mut VectorScene, rect: Rect, radius: f32, color: Color) {
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
    const VIEWBOX: f64 = 24.0;
    let pad = 2.0_f32.min(rect.w.min(rect.h) * 0.1);
    let avail_w = (rect.w - pad * 2.0).max(0.0) as f64;
    let avail_h = (rect.h - pad * 2.0).max(0.0) as f64;
    if avail_w <= 0.0 || avail_h <= 0.0 {
        return;
    }
    let scale = (avail_w / VIEWBOX).min(avail_h / VIEWBOX);
    let drawn = VIEWBOX * scale;
    let tx = rect.x as f64 + (rect.w as f64 - drawn) * 0.5;
    let ty = rect.y as f64 + (rect.h as f64 - drawn) * 0.5;
    let transform = Affine::translate((tx, ty)) * Affine::scale(scale);
    let stroke = Stroke::new(stroke_width as f64);
    let brush = Brush::Solid(color);
    let mut path = BezPath::new();
    for cmd in icon.cmds() {
        path.extend(cmd_to_path(*cmd));
    }
    scene
        .inner_mut()
        .stroke(&stroke, transform, &brush, None, &path);
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
    let layout = text_system.layout(text, font_size, max_width);
    let inner = scene.inner_mut();
    let translate = Affine::translate((x as f64, y as f64));
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
                .brush(color)
                .transform(translate)
                .draw(
                    Fill::NonZero,
                    glyph_run.positioned_glyphs().map(|g| Glyph {
                        id: g.id,
                        x: g.x,
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
// FloatingPanel — body + tab strip + action grid (rects only, v1)
// -----------------------------------------------------------------------

impl Paint for FloatingPanel {
    fn paint(&self, scene: &mut VectorScene, ctx: &mut PaintCtx) {
        if !self.visible {
            return;
        }
        let viewport = ph2d_vector::Rect::new(
            ctx.viewport.x as f64,
            ctx.viewport.y as f64,
            (ctx.viewport.x + ctx.viewport.w) as f64,
            (ctx.viewport.y + ctx.viewport.h) as f64,
        );
        // FloatingPanel::rect takes our pixel Rect — repackage.
        let panel_rect = self.rect(Rect {
            x: viewport.x0 as f32,
            y: viewport.y0 as f32,
            w: (viewport.x1 - viewport.x0) as f32,
            h: (viewport.y1 - viewport.y0) as f32,
        });
        // Panel body — SurfaceElevated so it visually lifts off the
        // zones below.
        scene.fill_rect(
            rect_to_vello(panel_rect),
            resolve(ColorToken::BgElev, ctx.theme),
        );
        if self.collapsed {
            // Just the chip — no tabs / actions.
            return;
        }

        // Tab strip occupies the top ~40 % of the panel height.
        let tab_h = (panel_rect.h * 0.4).min(36.0);
        let tab_count = self.tabs.len().max(1) as f32;
        let tab_w = panel_rect.w / tab_count;
        for (i, tab) in self.tabs.iter().enumerate() {
            let r = Rect {
                x: panel_rect.x + tab_w * i as f32,
                y: panel_rect.y,
                w: tab_w,
                h: tab_h,
            };
            scene.fill_rect(rect_to_vello(r), tab_color(tab, ctx.theme));
            // Tab label — invert color when active so it stays readable
            // on AccentPrimary.
            let label_color = if tab.active {
                resolve(ColorToken::AccentFg, ctx.theme)
            } else {
                resolve(ColorToken::Text1, ctx.theme)
            };
            paint_text_centered(ctx.text, scene, &tab.label, r, 13.0, label_color);
        }

        // Control / action row beneath tabs. Tools using the modern
        // `controls` field (Brush, Move) render widgets; legacy panels
        // (selection_demo) fall back to the labeled action grid.
        let row_y = panel_rect.y + tab_h;
        let row_h = panel_rect.h - tab_h;
        let label_color = resolve(ColorToken::Text2, ctx.theme);
        if !self.controls.is_empty() {
            let count = self.controls.len() as f32;
            let cell_w = panel_rect.w / count;
            // Header label band ~32% of cell h, widget below.
            let label_h = (row_h * 0.32).min(14.0);
            let pad = 4.0_f32;
            for (i, ctrl) in self.controls.iter().enumerate() {
                let cell = Rect {
                    x: panel_rect.x + cell_w * i as f32 + pad,
                    y: row_y + 2.0,
                    w: cell_w - pad * 2.0,
                    h: row_h - 4.0,
                };
                let label_rect = Rect {
                    x: cell.x,
                    y: cell.y,
                    w: cell.w,
                    h: label_h,
                };
                let widget_rect = Rect {
                    x: cell.x,
                    y: cell.y + label_h,
                    w: cell.w,
                    h: cell.h - label_h,
                };
                paint_text_centered(ctx.text, scene, ctrl.label(), label_rect, 10.0, label_color);
                match ctrl {
                    PanelControl::Slider(s) => paint_slider(s, widget_rect, scene, ctx.theme),
                    PanelControl::Toggle(t) => paint_toggle(t, widget_rect, scene, ctx.theme),
                    PanelControl::RadioGroup(g) => {
                        paint_radio_group(g, widget_rect, scene, ctx.theme)
                    }
                    PanelControl::ColorSwatch(s) => {
                        paint_color_swatch(s, widget_rect, scene, ctx.theme)
                    }
                    PanelControl::Action(a) => {
                        scene.fill_rect(
                            rect_to_vello(widget_rect),
                            resolve(ColorToken::Bg1, ctx.theme),
                        );
                        paint_text_centered(
                            ctx.text,
                            scene,
                            &a.label,
                            widget_rect,
                            11.0,
                            label_color,
                        );
                    }
                }
            }
            return;
        }
        // Legacy: labels-only action grid.
        let action_count = self.actions.len().max(1) as f32;
        let action_w = panel_rect.w / action_count;
        for (i, action) in self.actions.iter().enumerate() {
            let r = Rect {
                x: panel_rect.x + action_w * i as f32 + 2.0,
                y: row_y + 4.0,
                w: action_w - 4.0,
                h: row_h - 8.0,
            };
            scene.fill_rect(rect_to_vello(r), resolve(ColorToken::Bg1, ctx.theme));
            paint_text_centered(ctx.text, scene, action_label(action), r, 11.0, label_color);
        }
    }
}

fn action_label(action: &PanelAction) -> &str {
    &action.label
}

fn tab_color(tab: &PanelTab, theme: Theme) -> Color {
    if tab.active {
        resolve(ColorToken::Accent, theme)
    } else {
        resolve(ColorToken::Bg1, theme)
    }
}

// Button paint helper now lives at crate::widget::paint_button — see
// `widget/button.rs` for the rounded-rect, kind-aware implementation.

// -----------------------------------------------------------------------
// ToastQueue — stacked at top-center
// -----------------------------------------------------------------------

impl Paint for ToastQueue {
    fn paint(&self, scene: &mut VectorScene, ctx: &mut PaintCtx) {
        use crate::toast::ToastSeverity;
        use ph2d_tokens::Radius;
        let toast_w = 360.0_f32;
        let toast_h = 44.0_f32;
        let gap = 8.0_f32;
        let top_margin = 16.0_f32;
        let center_x = ctx.viewport.x + (ctx.viewport.w - toast_w) / 2.0;
        let radius = Radius::Md.px();
        for (i, toast) in self.iter().enumerate() {
            let r = Rect {
                x: center_x,
                y: ctx.viewport.y + top_margin + (toast_h + gap) * i as f32,
                w: toast_w,
                h: toast_h,
            };
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

    /// Token → Vello color round-trips bytes accurately enough for
    /// our 8-bit palette (no banding from the sRGB linearization).
    #[test]
    fn token_to_vello_preserves_visible_difference() {
        let bg0 = ColorToken::Bg0.resolve(Theme::ForgeSdf);
        let bg1 = ColorToken::Bg1.resolve(Theme::ForgeSdf);
        // Tokens are different ⇒ vello colors must differ.
        assert_ne!(token_to_vello(bg0), token_to_vello(bg1));
    }

    /// Painting a 4-zone Layout doesn't panic and emits at least the
    /// 3 non-Center zones' rects (Center stays transparent).
    #[test]
    fn layout_paint_smoke() {
        let layout = Layout::new(1024.0, 768.0);
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        let mut ctx = PaintCtx {
            theme: Theme::ForgeSdf,
            viewport: Rect::new(0.0, 0.0, 1024.0, 768.0),
            text: &mut text,
        };
        layout.paint(&mut scene, &mut ctx);
        // Vello doesn't expose a public command count, so we settle
        // for "encoder didn't panic + reset is idempotent".
        scene.reset();
    }

    #[test]
    fn floating_panel_paint_collapsed_emits_only_chip() {
        let mut panel = crate::floating_panel::selection_demo_panel();
        panel.collapsed = true;
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        let mut ctx = PaintCtx {
            theme: Theme::ForgeSdf,
            viewport: Rect::new(0.0, 0.0, 1024.0, 768.0),
            text: &mut text,
        };
        panel.paint(&mut scene, &mut ctx);
    }

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
        let mut text = TextSystem::new();
        let mut ctx = PaintCtx {
            theme: Theme::Sunstone,
            viewport: Rect::new(0.0, 0.0, 800.0, 600.0),
            text: &mut text,
        };
        q.paint(&mut scene, &mut ctx);
    }
}
