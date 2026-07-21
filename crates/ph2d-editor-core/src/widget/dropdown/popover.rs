//! The open option list of a [`super::Dropdown`] — see the parent for why the two halves are
//! two files.

use super::*;
use crate::widget::scrollbar;

/// Paint only the open option list (no chip). Caller is responsible
/// for invoking this AFTER any other content that might sit at the
/// same Y so the popover stays on top (the inspector calls this in
/// its second pass, after every section has painted).
///
/// No-op when `!dd.open`.
pub fn paint_dropdown_popover<T: Clone + PartialEq>(
    dd: &Dropdown<T>,
    chip_rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_dropdown_popover_in_viewport(dd, chip_rect, None, scene, text_system, theme);
}

/// Variant of [`paint_dropdown_popover`] that flips above when below
/// overflows `viewport`. Panels paint chips near the bottom of the
/// screen pass `PaintCtx::viewport` here so option lists stay on-screen.
pub fn paint_dropdown_popover_in_viewport<T: Clone + PartialEq>(
    dd: &Dropdown<T>,
    chip_rect: Rect,
    viewport: Option<Rect>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    if !dd.open {
        return;
    }
    // Floating popover panel — fully opaque. The theme tokens
    // (`BgElev`, `AccentSoft`, …) carry alpha < 255 on several themes
    // so the list let underlying widgets bleed through; the user's
    // call was "tire toda transparência da lista do dropdown".
    // `opaque(token)` forces alpha to 255 while keeping the token's
    // RGB so the color still tracks the theme.
    let panel = match viewport {
        Some(vp) => dd.popover_rect_clamped(chip_rect, vp),
        None => dd.popover_rect(chip_rect),
    };
    let panel_radius = Radius::Md.px();
    fill_rounded_rect(
        scene,
        panel,
        panel_radius,
        opaque(ColorToken::BgElev, theme),
    );
    stroke_rounded_rect(
        scene,
        panel,
        panel_radius,
        1.0,
        opaque(ColorToken::Border, theme),
    );
    let font_size = TypeToken::Base.px();
    for (i, opt) in dd.options.iter().enumerate() {
        let r = dd.option_rect_in(chip_rect, panel, i);
        let is_selected = dd.selected.as_ref() == Some(&opt.value);
        if is_selected {
            // Use the saturated `Accent` (alpha-255 by token) for
            // the selected row instead of `AccentSoft` (which is
            // intentionally semi-transparent for inline overlays).
            fill_rounded_rect(scene, r, Radius::Sm.px(), opaque(ColorToken::Accent, theme));
        }
        let fg = if is_selected {
            ColorToken::AccentFg
        } else {
            ColorToken::Text1
        };
        let (icon_rect, text_rect) = option_icon_split(r, opt.icon.is_some());
        if let Some(icon) = opt.icon {
            paint_icon(
                scene,
                icon,
                icon_rect,
                opaque(fg, theme),
                StrokeToken::Default.px(),
            );
        }
        paint_text_centered(
            text_system,
            scene,
            &opt.label,
            text_rect,
            font_size,
            opaque(fg, theme),
        );
    }
}

/// Resolve `token` against `theme` and force the alpha channel to
/// `255`. Used by the dropdown popover so the floating list never
/// shows the content behind it through token-level alpha (e.g.
/// `AccentSoft` is alpha 0x29 by design — fine for inline overlays,
/// wrong for an opaque surface).
pub fn opaque(token: ColorToken, theme: Theme) -> ph2d_vector::Color {
    let c = token.resolve(theme);
    ph2d_vector::Color::from_rgba8(c.r, c.g, c.b, 0xFF) // LITERAL-COLOR-OK: token-bridge with forced-opaque alpha (popover must occlude content behind it)
}

/// Stable hit id for the open dropdown popover's **scrollbar** track/thumb. Only one dropdown is open
/// at a time, so a single id suffices; the dispatch maps it to whichever dropdown is currently open
/// (via `WidgetStore::dropdown_popover`). Long option lists (texture Kind = 26, blend = 24…) overflow
/// the clamped popover, so the list scrolls — wheel + this draggable bar (Enio: app-wide default).
pub const DROPDOWN_SCROLLBAR_ID: NodeId = NodeId(831);

/// Paint an open dropdown popover whose option list **scrolls**: rows are clipped to `panel` and
/// shifted up by `scroll`, with a draggable scrollbar when the list overflows. `panel` is the clamped
/// popover rect, `scroll` the current offset (`>= 0`), `scrollbar_active` highlights the bar while
/// dragged. Mirrors [`paint_dropdown_popover_in_viewport`] visually; the caller owns the
/// store/scroll/hit wiring (see the panels' `paint_dropdown_popover`).
#[allow(clippy::too_many_arguments)]
pub fn paint_dropdown_popover_scrolled<T: Clone + PartialEq>(
    dd: &Dropdown<T>,
    chip_rect: Rect,
    panel: Rect,
    scroll: f32,
    scrollbar_active: bool,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    if !dd.open {
        return;
    }
    let panel_radius = Radius::Md.px();
    fill_rounded_rect(
        scene,
        panel,
        panel_radius,
        opaque(ColorToken::BgElev, theme),
    );
    stroke_rounded_rect(
        scene,
        panel,
        panel_radius,
        1.0,
        opaque(ColorToken::Border, theme),
    );
    // Clip the rows to the panel so scrolled-out rows don't paint past the edges.
    let clip = ph2d_vector::Rect::new(
        panel.x as f64,
        panel.y as f64,
        (panel.x + panel.w) as f64,
        (panel.y + panel.h) as f64,
    );
    scene.push_clip(&clip);
    let font_size = TypeToken::Base.px();
    for (i, opt) in dd.options.iter().enumerate() {
        let r = dd.option_rect_in_scrolled(chip_rect, panel, i, scroll);
        if r.y + r.h <= panel.y || r.y >= panel.y + panel.h {
            continue; // fully scrolled out (the clip handles partials)
        }
        let is_selected = dd.selected.as_ref() == Some(&opt.value);
        if is_selected {
            fill_rounded_rect(scene, r, Radius::Sm.px(), opaque(ColorToken::Accent, theme));
        }
        let fg = if is_selected {
            ColorToken::AccentFg
        } else {
            ColorToken::Text1
        };
        paint_text_centered(
            text_system,
            scene,
            &opt.label,
            r,
            font_size,
            opaque(fg, theme),
        );
    }
    scene.pop_layer();
    let content_h = dd.content_height(chip_rect.h);
    if scrollbar::is_needed(content_h, panel.h) {
        scrollbar::paint_scrollbar(
            panel,
            scroll,
            content_h,
            panel.h,
            scrollbar_active,
            scene,
            theme,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Dropdown<&'static str> {
        Dropdown::new(
            NodeId(1),
            "Tool",
            vec![
                DropdownOption::new(NodeId(2), "brush", "Brush"),
                DropdownOption::new(NodeId(3), "erase", "Erase"),
                DropdownOption::new(NodeId(4), "smudge", "Smudge"),
            ],
        )
    }

    #[test]
    fn defaults_match_spec() {
        let d = fixture();
        assert!(d.selected.is_none());
        assert!(!d.open);
        assert_eq!(d.placeholder, "Select…");
    }

    #[test]
    fn select_known_value() {
        let mut d = fixture();
        d.select("erase");
        assert_eq!(d.selected_label(), Some("Erase"));
    }

    #[test]
    fn select_unknown_value_silent() {
        let mut d = fixture();
        d.select("nope");
        assert!(d.selected.is_none());
    }

    #[test]
    fn a11y_role_is_combobox() {
        let node = fixture().build_a11y(0.0, 0.0, 200.0, 32.0);
        assert_eq!(node.role(), Role::ComboBox);
    }

    #[test]
    fn a11y_option_role_is_listbox_option() {
        let node = fixture()
            .build_option_a11y(0, 0.0, 0.0, 200.0, 32.0)
            .unwrap();
        assert_eq!(node.role(), Role::ListBoxOption);
    }

    fn smoke(d: Dropdown<&'static str>, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_dropdown(
            &d,
            Rect::new(0.0, 0.0, 200.0, 32.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn popover_rect_sits_below_chip_with_gap() {
        let d = fixture();
        let chip = Rect::new(10.0, 20.0, 200.0, 32.0);
        let pop = d.popover_rect(chip);
        assert!(pop.y > chip.y + chip.h, "popover must be below chip");
        // Gap is non-zero so the two surfaces don't fuse into one.
        assert!((pop.y - (chip.y + chip.h)) >= 1.0);
        assert_eq!(pop.x, chip.x);
        assert_eq!(pop.w, chip.w);
    }

    #[test]
    fn option_rect_inset_inside_popover() {
        let d = fixture();
        let chip = Rect::new(10.0, 20.0, 200.0, 32.0);
        let pop = d.popover_rect(chip);
        let r0 = d.option_rect(chip, 0);
        // Rows must live INSIDE the popover panel.
        assert!(r0.x > pop.x);
        assert!(r0.x + r0.w < pop.x + pop.w);
        assert!(r0.y >= pop.y);
        assert!(r0.y + r0.h <= pop.y + pop.h + 0.001);
    }

    #[test]
    fn paint_smoke_closed_empty() {
        smoke(fixture(), Theme::Forge);
    }

    #[test]
    fn paint_smoke_open_with_selection() {
        smoke(fixture().selected("erase").open(true), Theme::Sunstone);
    }

    #[test]
    fn paint_smoke_focused() {
        smoke(fixture().state(DropdownState::Focused), Theme::Blueprint);
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(fixture().state(DropdownState::Disabled), Theme::Workshop);
    }

    #[test]
    fn content_height_counts_every_row() {
        let dd = fixture(); // 3 options
        let row_h = 20.0;
        let expected = row_h * 3.0 + POPOVER_PANEL_PAD_Y * 2.0;
        assert!((dd.content_height(row_h) - expected).abs() < 0.001);
    }

    #[test]
    fn option_rect_scrolled_shifts_the_row_up_by_the_offset() {
        let dd = fixture();
        let chip = Rect::new(0.0, 0.0, 120.0, 20.0);
        let panel = dd.popover_rect(chip);
        let r0 = dd.option_rect_in_scrolled(chip, panel, 2, 0.0);
        let r1 = dd.option_rect_in_scrolled(chip, panel, 2, 15.0);
        assert!(
            (r0.y - r1.y - 15.0).abs() < 0.001,
            "scroll moves the row up by exactly the offset"
        );
        // scroll 0 matches the un-scrolled layout.
        assert!((dd.option_rect_in(chip, panel, 2).y - r0.y).abs() < 0.001);
    }
}
