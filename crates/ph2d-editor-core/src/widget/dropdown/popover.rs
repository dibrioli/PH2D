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
pub(crate) fn paint_dropdown_popover<T: Clone + PartialEq>(
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
pub(crate) fn paint_dropdown_popover_in_viewport<T: Clone + PartialEq>(
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
    paint_popover_frame(scene, panel, theme);
    for (i, opt) in dd.options.iter().enumerate() {
        let row = PopoverRow::of(dd, opt, i, dd.option_rect_in(chip_rect, panel, i));
        paint_popover_row_default(scene, text_system, theme, row, opt);
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

/// ⭐⭐ **Uma linha VISÍVEL de uma lista aberta** — o que o pintor de linha recebe (o padrão
/// [`paint_popover_row_default`], ou o próprio de um painel cuja linha desenha a COISA: o glifo do
/// picker de ícone, o contorno de uma fonte).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopoverRow {
    /// O índice da opção em `dd.options`.
    pub index: usize,
    /// O rect da linha já rolado (o clip do painel trata os parciais).
    pub rect: Rect,
    /// A linha é a vigente — o realce [`paint_popover_row_highlight`] já foi pintado por baixo.
    pub selected: bool,
    pub disabled: bool,
}

impl PopoverRow {
    fn of<T: Clone + PartialEq>(
        dd: &Dropdown<T>,
        opt: &DropdownOption<T>,
        index: usize,
        rect: Rect,
    ) -> Self {
        Self {
            index,
            rect,
            selected: dd.selected.as_ref() == Some(&opt.value),
            disabled: opt.disabled,
        }
    }

    /// A tinta do conteúdo da linha: sobre o realce, apagada, ou a de repouso.
    #[must_use]
    pub fn ink(&self) -> ColorToken {
        if self.selected {
            ColorToken::AccentFg
        } else if self.disabled {
            ColorToken::TextDisabled
        } else {
            ColorToken::Text1
        }
    }
}

/// O pintor do CONTEÚDO de uma linha — `None` na porta é o [`paint_popover_row_default`].
pub type PopoverRowPainter<'a> = &'a mut dyn FnMut(&mut VectorScene, &mut TextSystem, PopoverRow);

/// **A moldura opaca de uma lista aberta** — uma só para a lista que rola e a que não rola.
fn paint_popover_frame(scene: &mut VectorScene, panel: Rect, theme: Theme) {
    let panel_radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(
        scene,
        panel,
        panel_radius,
        opaque(ColorToken::BgElev, theme),
    );
    crate::paint::stroke_frame(
        scene,
        panel,
        panel_radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        1.0,
        opaque(ColorToken::Border, theme),
    );
}

/// **O realce da linha vigente** — pintado pela lista por baixo de QUALQUER pintor de linha.
pub fn paint_popover_row_highlight(scene: &mut VectorScene, rect: Rect, theme: Theme) {
    fill_rounded_rect(
        scene,
        rect,
        crate::paint::frame_radius(theme, Radius::Sm.px()),
        opaque(ColorToken::Accent, theme),
    );
}

/// **A linha padrão**: o realce, o glifo de TIPO da opção (se tem) e o rótulo centrado.
fn paint_popover_row_default<T>(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    row: PopoverRow,
    opt: &DropdownOption<T>,
) {
    if row.selected {
        paint_popover_row_highlight(scene, row.rect, theme);
    }
    let fg = opaque(row.ink(), theme);
    let (icon_rect, text_rect) = option_icon_split(row.rect, opt.icon.is_some());
    if let Some(icon) = opt.icon {
        paint_icon(scene, icon, icon_rect, fg, StrokeToken::Default.px());
    }
    paint_text_centered(
        text_system,
        scene,
        &opt.label,
        text_rect,
        TypeToken::Base.px(),
        fg,
    );
}

/// ⭐⭐ **A lista aberta que ROLA**: moldura, linhas recortadas ao `panel` e subidas de `scroll`, e
/// a barra quando a lista transborda. `scrollbar` é o PAR visual do polegar
/// ([`crate::interaction::WidgetStore::scrollbar_visual_for`], chaveado pelo CHIP). `rows`
/// pinta o CONTEÚDO de cada linha visível por cima do realce; `None` = [`paint_popover_row_default`].
///
/// ⛔ `pub(crate)`: a store, a rolagem e os hits são da porta do `panel`
/// ([`crate::panel::paint_dropdown_list`]) — um painel que chamasse isto escreveria a 15.ª cópia
/// deles.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_dropdown_popover_scrolled<T: Clone + PartialEq>(
    dd: &Dropdown<T>,
    chip_rect: Rect,
    panel: Rect,
    scroll: f32,
    scrollbar: (crate::widget::ScrollbarState, f32),
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    mut rows: Option<PopoverRowPainter<'_>>,
) {
    if !dd.open {
        return;
    }
    paint_popover_frame(scene, panel, theme);
    // Clip the rows to the panel so scrolled-out rows don't paint past the edges.
    let clip = ph2d_vector::Rect::new(
        panel.x as f64,
        panel.y as f64,
        (panel.x + panel.w) as f64,
        (panel.y + panel.h) as f64,
    );
    scene.push_clip(&clip);
    for (i, opt) in dd.options.iter().enumerate() {
        let r = dd.option_rect_in_scrolled(chip_rect, panel, i, scroll);
        if r.y + r.h <= panel.y || r.y >= panel.y + panel.h {
            continue; // fully scrolled out (the clip handles partials)
        }
        let row = PopoverRow::of(dd, opt, i, r);
        match rows.as_mut() {
            Some(paint_row) => {
                if row.selected {
                    paint_popover_row_highlight(scene, r, theme);
                }
                paint_row(scene, text_system, row);
            }
            None => paint_popover_row_default(scene, text_system, theme, row, opt),
        }
    }
    scene.pop_layer();
    let content_h = dd.content_height(chip_rect.h);
    if scrollbar::is_needed(content_h, panel.h) {
        scrollbar::paint_scrollbar(panel, scroll, content_h, panel.h, scrollbar, scene, theme);
    }
}

/// O que uma lista ROLADA pintou neste quadro ([`crate::panel::paint_dropdown_popover_scrolled_with_hits`])
/// — o rect do painel, as duas alturas e a rolagem JÁ clampada com que pintou. É o que
/// [`crate::interaction::WidgetStore::publish_dropdown_popover`] publica: sem as alturas, um
/// popover clampado mais curto que a lista fica cortado e imóvel à roda e ao polegar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintedPopover {
    pub id: NodeId,
    pub panel: Rect,
    pub content_h: f32,
    pub visible_h: f32,
    pub scroll: f32,
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
