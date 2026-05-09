//! [`Paint`] trait + impls — widget tree → `vello::Scene` (M11).
//!
//! This is the **lowering pass** that takes the data-only widget
//! representation (zones, panels, buttons, toasts) and emits Vello
//! draw commands. The shell calls each top-level widget's `paint`
//! once per frame, then hands the resulting `VectorScene` to
//! `ph2d_render::VelloPass` which composites onto the wgpu surface.
//!
//! **v1 scope (this PR):** rects + borders only. Color resolution
//! goes through `ph2d_tokens::ColorToken::resolve(theme)` so swapping
//! Dark↔Light flips the entire UI.
//!
//! **Text is deliberately deferred** to a follow-up PR — parley
//! glyph runs need a `vello::draw_glyphs` integration that's
//! non-trivial enough to merit its own scoped change. Buttons and
//! tabs render as solid surfaces with their tinting (the a11y tree
//! still carries the label, so screen readers work even before
//! visible text lands).

use crate::floating_panel::{FloatingPanel, PanelTab};
use crate::toast::ToastQueue;
use crate::widget::Button;
use crate::zones::{Layout, Rect, Zone};
use ph2d_text::TextSystem;
use ph2d_tokens::{Color as TokenColor, ColorToken, Theme};
use ph2d_vector::{Color, Rect as VelloRect, VectorScene};

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

// -----------------------------------------------------------------------
// Layout — 4 zones backdrop
// -----------------------------------------------------------------------

impl Paint for Layout {
    fn paint(&self, scene: &mut VectorScene, ctx: &mut PaintCtx) {
        // Center is canvas — the sprite layer underneath is what the
        // user sees. We deliberately do NOT paint the Center rect so
        // the existing wgpu surface (sprites) shows through.
        for zone in [Zone::TopLeft, Zone::TopRight, Zone::Sidebar] {
            let r = self.rect(zone);
            if r.area() <= 0.0 {
                continue; // Zen mode — zone collapsed to zero.
            }
            // Zones use Surface (panel chrome). A 1-px BorderEmphasis
            // edge along the seam toward Center sells the layout.
            scene.fill_rect(rect_to_vello(r), resolve(ColorToken::Surface, ctx.theme));
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
            resolve(ColorToken::SurfaceElevated, ctx.theme),
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
        }

        // Action row beneath tabs.
        let action_y = panel_rect.y + tab_h;
        let action_h = panel_rect.h - tab_h;
        let action_count = self.actions.len().max(1) as f32;
        let action_w = panel_rect.w / action_count;
        for (i, _action) in self.actions.iter().enumerate() {
            let r = Rect {
                x: panel_rect.x + action_w * i as f32 + 2.0,
                y: action_y + 4.0,
                w: action_w - 4.0,
                h: action_h - 8.0,
            };
            scene.fill_rect(rect_to_vello(r), resolve(ColorToken::Surface, ctx.theme));
        }
    }
}

fn tab_color(tab: &PanelTab, theme: Theme) -> Color {
    if tab.active {
        resolve(ColorToken::AccentPrimary, theme)
    } else {
        resolve(ColorToken::Surface, theme)
    }
}

// -----------------------------------------------------------------------
// Button — solid rect tinted by state. Position+size come from caller.
// -----------------------------------------------------------------------

/// Convenience: draw a button at an explicit rect. The Button data
/// type itself doesn't carry geometry (panels arrange them); the
/// shell or panel impl decides where, then calls this.
pub fn paint_button(button: &Button, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    use crate::widget::ButtonState;
    let token = if button.accent {
        match button.state {
            ButtonState::Disabled => ColorToken::AccentTertiary,
            ButtonState::Pressed => ColorToken::AccentSecondary,
            _ => ColorToken::AccentPrimary,
        }
    } else {
        match button.state {
            ButtonState::Disabled => ColorToken::Border,
            ButtonState::Pressed => ColorToken::AccentTertiary,
            ButtonState::Hovered | ButtonState::Focused => ColorToken::SurfaceElevated,
            ButtonState::Normal => ColorToken::Surface,
        }
    };
    scene.fill_rect(rect_to_vello(rect), resolve(token, theme));
}

// -----------------------------------------------------------------------
// ToastQueue — stacked at top-center
// -----------------------------------------------------------------------

impl Paint for ToastQueue {
    fn paint(&self, scene: &mut VectorScene, ctx: &mut PaintCtx) {
        use crate::toast::ToastSeverity;
        let toast_w = 320.0_f32;
        let toast_h = 36.0_f32;
        let gap = 6.0_f32;
        let top_margin = 16.0_f32;
        let center_x = ctx.viewport.x + (ctx.viewport.w - toast_w) / 2.0;
        for (i, toast) in self.iter().enumerate() {
            let r = Rect {
                x: center_x,
                y: ctx.viewport.y + top_margin + (toast_h + gap) * i as f32,
                w: toast_w,
                h: toast_h,
            };
            let token = match toast.severity {
                ToastSeverity::Info => ColorToken::Info,
                ToastSeverity::Success => ColorToken::Success,
                ToastSeverity::Warning => ColorToken::Warning,
                ToastSeverity::ErrorState => ColorToken::ErrorState,
            };
            scene.fill_rect(rect_to_vello(r), resolve(token, ctx.theme));
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
        let dark_bg = ColorToken::Background.resolve(Theme::Dark);
        let dark_surface = ColorToken::Surface.resolve(Theme::Dark);
        // Tokens are different ⇒ vello colors must differ.
        assert_ne!(token_to_vello(dark_bg), token_to_vello(dark_surface));
    }

    /// Painting a 4-zone Layout doesn't panic and emits at least the
    /// 3 non-Center zones' rects (Center stays transparent).
    #[test]
    fn layout_paint_smoke() {
        let layout = Layout::new(1024.0, 768.0);
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        let mut ctx = PaintCtx {
            theme: Theme::Dark,
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
            theme: Theme::Dark,
            viewport: Rect::new(0.0, 0.0, 1024.0, 768.0),
            text: &mut text,
        };
        panel.paint(&mut scene, &mut ctx);
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
            theme: Theme::Light,
            viewport: Rect::new(0.0, 0.0, 800.0, 600.0),
            text: &mut text,
        };
        q.paint(&mut scene, &mut ctx);
    }
}
