//! The open option list of a [`super::Dropdown`] — see the parent for why the two halves are
//! two files.

use super::*;
use crate::paint::rect_to_vello;
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
) -> Rect {
    paint_dropdown_popover_in_viewport(dd, chip_rect, None, scene, text_system, theme)
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
) -> Rect {
    // ⭐ O rect da lista é a porta da LARGURA ([`Dropdown::popover_rect_fitted`]); devolvido para o
    // hit-test de quem chama ler o MESMO rect.
    let panel = dd.popover_rect_fitted(chip_rect, viewport, text_system);
    if !dd.open {
        return panel;
    }
    // Floating popover panel — fully opaque. The theme tokens
    // (`BgElev`, `AccentSoft`, …) carry alpha < 255 on several themes
    // so the list let underlying widgets bleed through; the user's
    // call was "tire toda transparência da lista do dropdown".
    // `opaque(token)` forces alpha to 255 while keeping the token's
    // RGB so the color still tracks the theme.
    let _veste = crate::widget::lista_flutuante::veste_fora_de_cartao(theme);
    crate::widget::camada_das_listas::por_cima(scene, |scene| {
        paint_popover_frame(scene, panel, theme);
        for (i, opt) in dd.options.iter().enumerate() {
            let r = dd.option_rect_in(chip_rect, panel, i);
            if opt.header {
                paint_popover_header_row(scene, text_system, theme, r, i, &opt.label);
                continue;
            }
            let row = PopoverRow::of(dd, opt, i, r);
            paint_popover_row_default(scene, text_system, theme, row, opt);
        }
    });
    panel
}

/// Resolve `token` against `theme` and force the alpha channel to
/// `255`. Used by the dropdown popover so the floating list never
/// shows the content behind it through token-level alpha (e.g.
/// `AccentSoft` is alpha 0x29 by design — fine for inline overlays,
/// wrong for an opaque surface).
pub fn opaque(token: ColorToken, theme: Theme) -> ph2d_vector::Color {
    if let Some(ink) = crate::widget::card_ink::card_body() {
        let c = ink.color(crate::widget::card_ink::papel_no_popover(token));
        return c.with_alpha(1.0);
    }
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
    if let Some(ink) = crate::widget::card_ink::card_body() {
        return crate::widget::lista_flutuante::moldura_de_cartao(scene, panel, ink);
    }
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
    let raio = crate::widget::card_ink::card_body().map_or_else(
        || crate::paint::frame_radius(theme, Radius::Sm.px()),
        |_| card_chip::RAIO_DA_LINHA,
    );
    fill_rounded_rect(scene, rect, raio, opaque(ColorToken::Accent, theme));
}

/// **A linha com o FOCO do teclado, sem estar carregada** — uma tinta neutra um degrau acima do
/// popover (`Bg3` sobre `BgElev`) e o anel de foco fino por dentro dela. É a 1.ª acção do Alt+Enter
/// do editor de código: o bloco cheio do [`paint_popover_row_highlight`] lia-se como já carregada
/// (dono, 18/10). O VS Code 1.142 pinta a escolhida pelo teclado assim (`#313233` sobre `#202122`,
/// anel `#3994BC` de 1 px por dentro); medição em
/// `docs/CodeEditor/ferramentas/medicoes_onda2/I8b_botoes_no_balao.md`.
pub fn paint_popover_row_focus(scene: &mut VectorScene, rect: Rect, theme: Theme) {
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    fill_rounded_rect(scene, rect, radius, opaque(ColorToken::Bg3, theme));
    let w = StrokeToken::Thin.px();
    let ring = Rect::new(
        rect.x + w * 0.5,
        rect.y + w * 0.5,
        (rect.w - w).max(0.0),
        (rect.h - w).max(0.0),
    );
    // FRAME-RAW-OK: o ANEL DE FOCO — vê-se em qualquer pele, como o do `paint_button`.
    crate::paint::stroke_rounded_rect(
        scene,
        ring,
        (radius - w * 0.5).max(0.0),
        w,
        opaque(ColorToken::BorderEmph, theme),
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
    if crate::widget::card_ink::card_body().is_some() {
        return card_chip::texto_da_linha(scene, text_system, text_rect, &opt.label, fg);
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

/// A linha `opt` na mão, em `at`: a lei do voo do cartão do Kanban, por cima da lista.
pub(crate) fn paint_popover_row_flying<T>(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    opt: &DropdownOption<T>,
    at: Rect,
) {
    use crate::paint_flight::{FLIGHT_TILT_DEG, flight_shadow, fly};
    let _veste = crate::widget::lista_flutuante::veste_fora_de_cartao(theme);
    let raio = crate::paint::frame_radius(theme, Radius::Sm.px());
    let local_r = Rect::new(0.0, 0.0, at.w, at.h);
    let mut local = VectorScene::new();
    fill_rounded_rect(&mut local, local_r, raio, opaque(ColorToken::BgElev, theme));
    let row = PopoverRow {
        index: 0,
        rect: local_r,
        selected: false,
        disabled: false,
    };
    paint_popover_row_default(&mut local, text_system, theme, row, opt);
    crate::widget::camada_das_listas::por_cima(scene, |scene| {
        flight_shadow(scene, theme, at, raio);
        fly(scene, &local, at, FLIGHT_TILT_DEG);
    });
}

/// A marca de acento no topo de `vao`, a linha onde a que está na mão vai cair.
pub(crate) fn paint_popover_drop_mark(scene: &mut VectorScene, theme: Theme, vao: Rect) {
    let h = StrokeToken::Thick.px();
    let raio = crate::paint::frame_radius(theme, Radius::Sm.px());
    let marca = Rect::new(vao.x, vao.y - h * 0.5, vao.w, h);
    let cor = crate::paint::resolve(ColorToken::Accent, theme);
    crate::widget::camada_das_listas::por_cima(scene, |scene| {
        fill_rounded_rect(scene, marca, raio, cor);
    });
}

/// ⭐ **Um TÍTULO de categoria** ([`DropdownOption::header`]) — o rótulo de secção do design system:
/// caixa alta, `Xs`, `Text3`, à esquerda e assente no fundo da linha, com um separador `Thin` em
/// `Border` por cima (salvo a 1.ª linha). Nunca leva realce: não se escolhe.
fn paint_popover_header_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    rect: Rect,
    index: usize,
    label: &str,
) {
    let pad = Spacing::Sm.px();
    if index > 0 {
        let line = Rect::new(
            rect.x + pad,
            rect.y,
            (rect.w - pad * 2.0).max(0.0),
            StrokeToken::Thin.px(),
        );
        scene.fill_rect(rect_to_vello(line), opaque(ColorToken::Border, theme));
    }
    let font = TypeToken::Xs.px();
    paint_text(
        text_system,
        scene,
        &label.to_uppercase(),
        rect.x + pad,
        rect.y + rect.h - font - Spacing::Xs.px(),
        font,
        (rect.w - pad * 2.0).max(0.0),
        opaque(ColorToken::Text3, theme),
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
    let _veste = crate::widget::lista_flutuante::veste_fora_de_cartao(theme);
    crate::widget::camada_das_listas::por_cima(scene, |scene| {
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
            if opt.header {
                paint_popover_header_row(scene, text_system, theme, r, i, &opt.label);
                continue;
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
    });
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
#[path = "popover_tests.rs"]
mod tests;
