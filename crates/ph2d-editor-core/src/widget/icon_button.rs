//! Canonical icon button — the single source of truth for a clickable
//! control whose face is an icon, drawn over chrome (TopBar action pills,
//! Play/Pause/Reset transport, right-cluster icons).
//!
//! Why this exists separately from [`crate::widget::Button`]: a TopBar
//! action pill draws a *manifest-supplied* `BezPath` glyph, and
//! `ButtonKind` is `Copy + Eq` so it cannot carry a `BezPath`. The glyph
//! is therefore a paint-time parameter ([`IconGlyph`]) here, while the
//! chrome (surface / outline / tint per state) stays centralized so the
//! TopBar stops hand-rolling `fill_rounded_rect` + `stroke` + `paint_icon`
//! inline. An arch gate keeps `paint_icon_path` (the manifest-glyph draw)
//! callable only from this module.

use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, paint_icon_path, resolve, stroke_rounded_rect};
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme};
use ph2d_vector::{BezPath, VectorScene};

/// Glyph source for a canonical icon button.
pub enum IconGlyph<'a> {
    /// Built-in editor glyph from the `IconId` table.
    Builtin(IconId),
    /// Manifest-supplied 24×24 path (e.g. an Image Tools action icon).
    Path(&'a BezPath),
}

/// Visual style. The chrome differs; the glyph draw is shared. Each
/// reproduces an existing TopBar look exactly (no visual change on
/// migration) — only the call site moves from inline to canonical.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IconButtonStyle {
    /// Framed toolbar chip: `BgElev` surface + `Border` outline at
    /// `Radius::Xl`, glyph in a centered `Xl3` box tinted by state.
    /// The TopBar Image Tools action pills.
    Chip,
    /// Primary transport pill: `Danger` idle → `AccentSoft` hover →
    /// `AccentPress` pressed at `Radius::Lg`, `AccentFg` glyph filling
    /// the rect. The TopBar Play button.
    Primary,
    /// Frameless: glyph only, tinted by state. Pause / Reset / right
    /// cluster.
    Plain,
}

/// State→glyph tint for the `Chip` / `Plain` styles (the canonical
/// `icon_button_fg`): idle `Text2`, hover/focus `Text1`, pressed
/// `Accent`, disabled `TextDisabled`.
fn icon_tint(state: ButtonState) -> ColorToken {
    match state {
        ButtonState::Hovered | ButtonState::Focused => ColorToken::Text1,
        ButtonState::Pressed => ColorToken::Accent,
        ButtonState::Disabled => ColorToken::TextDisabled,
        ButtonState::Normal | ButtonState::Loading => ColorToken::Text2,
    }
}

/// Paint a canonical icon button at `rect`. Caller still registers the
/// hit-rect + AccessKit node (chrome owns those); this draws only.
pub fn paint_icon_button(
    rect: Rect,
    glyph: IconGlyph,
    style: IconButtonStyle,
    state: ButtonState,
    scene: &mut VectorScene,
    theme: Theme,
) {
    let (icon_rect, icon_color) = match style {
        IconButtonStyle::Chip => {
            fill_rounded_rect(
                scene,
                rect,
                Radius::Xl.px(),
                resolve(ColorToken::BgElev, theme),
            );
            stroke_rounded_rect(
                scene,
                rect,
                Radius::Xl.px(),
                StrokeToken::Default.px(),
                resolve(ColorToken::Border, theme),
            );
            let d = Spacing::Xl3.px();
            let ir = Rect::new(
                rect.x + (rect.w - d) * 0.5,
                rect.y + (rect.h - d) * 0.5,
                d,
                d,
            );
            (ir, resolve(icon_tint(state), theme))
        }
        IconButtonStyle::Primary => {
            let bg = match state {
                ButtonState::Pressed => ColorToken::AccentPress,
                ButtonState::Hovered => ColorToken::AccentSoft,
                _ => ColorToken::Danger,
            };
            fill_rounded_rect(scene, rect, Radius::Lg.px(), resolve(bg, theme));
            (rect, resolve(ColorToken::AccentFg, theme))
        }
        IconButtonStyle::Plain => (rect, resolve(icon_tint(state), theme)),
    };
    match glyph {
        IconGlyph::Builtin(id) => {
            paint_icon(scene, id, icon_rect, icon_color, StrokeToken::Default.px())
        }
        IconGlyph::Path(p) => {
            paint_icon_path(scene, p, icon_rect, icon_color, StrokeToken::Default.px())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn smoke(style: IconButtonStyle, state: ButtonState) {
        let mut scene = VectorScene::new();
        paint_icon_button(
            Rect::new(0.0, 0.0, 56.0, 40.0),
            IconGlyph::Builtin(IconId::Settings),
            style,
            state,
            &mut scene,
            Theme::Forge,
        );
    }

    #[test]
    fn paints_every_style_and_state_without_panic() {
        for style in [
            IconButtonStyle::Chip,
            IconButtonStyle::Primary,
            IconButtonStyle::Plain,
        ] {
            for state in [
                ButtonState::Normal,
                ButtonState::Hovered,
                ButtonState::Pressed,
                ButtonState::Disabled,
            ] {
                smoke(style, state);
            }
        }
    }

    #[test]
    fn paints_a_manifest_bezpath_glyph() {
        let mut path = BezPath::new();
        path.move_to((0.0, 0.0));
        path.line_to((24.0, 24.0));
        let mut scene = VectorScene::new();
        paint_icon_button(
            Rect::new(0.0, 0.0, 56.0, 40.0),
            IconGlyph::Path(&path),
            IconButtonStyle::Chip,
            ButtonState::Normal,
            &mut scene,
            Theme::Forge,
        );
    }

    #[test]
    fn chip_tint_follows_state() {
        assert_eq!(icon_tint(ButtonState::Normal), ColorToken::Text2);
        assert_eq!(icon_tint(ButtonState::Hovered), ColorToken::Text1);
        assert_eq!(icon_tint(ButtonState::Pressed), ColorToken::Accent);
        assert_eq!(icon_tint(ButtonState::Disabled), ColorToken::TextDisabled);
    }
}
