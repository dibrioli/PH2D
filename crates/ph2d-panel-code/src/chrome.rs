//! As peças pequenas da barra de busca e do console: fundo, rótulo, botão, campo de uma linha. Cores
//! por tokens, medidas por tokens, texto na fonte da interface.

use crate::peso::paint_text;
use ph2d_code_buffer::Buffer;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{
    fill_rounded_rect, folga_do_texto, frame_radius, paint_icon, paint_text_centered,
    rect_to_vello, resolve, stroke_frame,
};
use ph2d_editor_core::widget::section_cards::CardDepth;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::visuals::Feel;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Color, VectorScene};

/// Um retângulo vazio (o estado antes do primeiro quadro).
pub(crate) const NO_RECT: Rect = Rect::new(0.0, 0.0, 0.0, 0.0);

/// As cores das peças, resolvidas uma vez pelo tema.
pub(crate) struct Ui {
    pub bg: Color,
    pub border: Color,
    pub field: Color,
    pub field_edge: Color,
    pub focus: Color,
    pub button: Color,
    pub button_on: Color,
    pub text1: Color,
    pub text2: Color,
    pub text3: Color,
    pub selection: Color,
    pub caret: Color,
    pub error: Color,
    pub warn: Color,
    pub hover: Color,
    /// O tema — as quinas e a moldura perguntam-lhe.
    pub theme: Theme,
}

impl Ui {
    pub fn new(theme: Theme) -> Self {
        if ph2d_editor_core::panel::cards::ferramenta::cards_on() {
            return Self::do_cartao(theme);
        }
        let c = |t: ColorToken| resolve(t, theme);
        Self {
            // A barra assenta na raiz do documento, não dentro de um cartão: é a superfície de
            // nível de cartão, e o tom vem da porta.
            bg: CardDepth::Section.fill(theme),
            border: c(ColorToken::Border),
            field: c(ColorToken::CodeBg),
            field_edge: c(ColorToken::Border),
            focus: c(ColorToken::Accent),
            button: c(ColorToken::Bg3),
            button_on: c(ColorToken::AccentSoft),
            text1: c(ColorToken::Text1),
            text2: c(ColorToken::Text2),
            text3: c(ColorToken::Text3),
            selection: c(ColorToken::Selection),
            caret: c(ColorToken::Text1),
            error: c(ColorToken::Danger),
            warn: c(ColorToken::Warn),
            hover: c(ColorToken::Bg2),
            theme,
        }
    }

    /// A barra, a busca, o console e a lista na tinta do cartão. As cores que dizem um estado (erro, seleção,
    /// cursor, foco) ficam: o texto do editor vive ao lado delas.
    fn do_cartao(theme: Theme) -> Self {
        let c = |t: ColorToken| resolve(t, theme);
        Self {
            // As regiões da aba são cartões (dono 08/11): o fundo é o do cartão, o chão é a lista.
            bg: c(ColorToken::KanbanCard),
            border: c(ColorToken::KanbanSlot),
            field: c(ColorToken::CardField),
            field_edge: c(ColorToken::CardField),
            focus: c(ColorToken::Accent),
            button: c(ColorToken::CardField),
            button_on: c(ColorToken::KanbanCardRaised),
            text1: c(ColorToken::KanbanCardFg),
            text2: c(ColorToken::CardLabel),
            text3: c(ColorToken::CardLabel),
            selection: c(ColorToken::Selection),
            caret: c(ColorToken::Text1),
            error: c(ColorToken::Danger),
            warn: c(ColorToken::Warn),
            hover: c(ColorToken::CardFieldHover),
            theme,
        }
    }
}

/// O tamanho NOMINAL do texto das peças: o `TextSystem` aplica o tamanho da letra da interface.
pub(crate) fn font() -> f32 {
    TypeToken::Sm.px()
}

/// O tamanho em que [`font`] sai desenhado — para alturas de linha, nunca para um pintor.
pub(crate) fn font_shown() -> f32 {
    ph2d_text::displayed_font_px(font())
}

/// O fundo de uma faixa, com a linha de borda em cima (`top`) ou em baixo.
pub(crate) fn panel_bg(scene: &mut VectorScene, area: Rect, ui: &Ui, edge_below: bool) {
    scene.fill_rect(rect_to_vello(area), ui.bg);
    let t = StrokeToken::Thin.px();
    let y = if edge_below {
        area.y + area.h - t
    } else {
        area.y
    };
    scene.fill_rect(rect_to_vello(Rect::new(area.x, y, area.w, t)), ui.border);
}

pub(crate) fn label_w(ts: &mut TextSystem, text: &str) -> f32 {
    crate::peso::label_column_width(ts, font(), [text])
}

/// A largura do título de um cartão da coluna (MAIÚSCULAS espaçadas em cartões, o rótulo sem
/// eles), medida como se pinta: a parte do título no mínimo do divisor.
pub(crate) fn card_title_w(ts: &mut TextSystem, title: &str) -> f32 {
    if ph2d_editor_core::panel::cards::ferramenta::cards_on() {
        use ph2d_editor_core::panel::cards::banda::metrics as m;
        let t = title.to_uppercase();
        ph2d_editor_core::paint::title_tracked_width(
            ts,
            &t,
            m::LIST_TITLE_PX,
            m::LIST_TITLE_TRACKING_EM,
        )
    } else {
        label_w(ts, title)
    }
}

/// A largura de um [`text_button`] com `text`: a lei inversa do orçamento com que ele o pinta
/// (`rect_for_label`) — a folga somada à mão cortava «Word» em «W…» na barra de busca.
pub(crate) fn button_w(ts: &mut TextSystem, text: &str) -> f32 {
    let w = crate::peso::largura_em(ts, text, font(), crate::peso::BOTAO);
    ph2d_editor_core::paint::rect_for_label(w)
}

/// Um rótulo centrado na vertical de `r` pelas maiúsculas ([`folga_do_texto`]), na cor `color`.
pub(crate) fn label_in(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    text: &str,
    r: Rect,
    color: Color,
) {
    let f = font();
    paint_text(
        ts,
        scene,
        text,
        r.x,
        r.y + folga_do_texto(r.h, f),
        f,
        r.w.max(0.0),
        color,
    );
}

pub(crate) fn label(scene: &mut VectorScene, ts: &mut TextSystem, text: &str, r: Rect, ui: &Ui) {
    label_in(scene, ts, text, r, ui.text2);
}

pub(crate) fn icon_button(scene: &mut VectorScene, icon: IconId, r: Rect, ui: &Ui) {
    let s = ph2d_tokens::INLINE_ICON_PX.min(r.h);
    let ir = Rect::new(r.x + (r.w - s) * 0.5, r.y + (r.h - s) * 0.5, s, s);
    paint_icon(scene, icon, ir, ui.text2, StrokeToken::Default.px());
}

/// Um botão de texto; `on` = ligado (o realce de um interruptor).
pub(crate) fn text_button(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    text: &str,
    r: Rect,
    on: bool,
    ui: &Ui,
) {
    fill_rounded_rect(
        scene,
        r,
        frame_radius(ui.theme, Radius::Sm.px()),
        if on { ui.button_on } else { ui.button },
    );
    let ink = if on { ui.text1 } else { ui.text2 };
    // O `.btn` da maquete 14 é `500`, o `Medium` do pintor, sem o reforço do preset.
    ph2d_text::com_peso_exato(|| paint_text_centered(ts, scene, text, r, font(), ink));
}

/// O x (relativo ao início do texto) do byte `byte`.
pub(crate) fn x_of(ts: &mut TextSystem, text: &str, byte: usize) -> f32 {
    let b = byte.min(text.len());
    let b = (0..=b)
        .rev()
        .find(|&i| text.is_char_boundary(i))
        .unwrap_or(0);
    crate::peso::largura(ts, &text[..b], font())
}

/// O byte mais próximo de `dx` (relativo ao início do texto): o meio de cada carácter decide.
pub(crate) fn byte_at_x(ts: &mut TextSystem, text: &str, dx: f32) -> usize {
    let mut prev = (0usize, 0.0f32);
    let bounds = text
        .char_indices()
        .map(|(i, _)| i)
        .skip(1)
        .chain(std::iter::once(text.len()));
    for i in bounds {
        let x = crate::peso::largura(ts, &text[..i], font());
        if dx < (prev.1 + x) * 0.5 {
            return prev.0;
        }
        prev = (i, x);
    }
    text.len()
}

/// Um campo de uma linha: o texto (ou a pista apagada), a seleção e o cursor quando tem o teclado.
/// Devolve o x onde o texto começa.
#[allow(clippy::too_many_arguments)]
pub(crate) fn text_field(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    buf: &Buffer,
    placeholder: &str,
    r: Rect,
    focused: bool,
    ui: &Ui,
) -> f32 {
    let radius = frame_radius(ui.theme, Radius::Sm.px());
    fill_rounded_rect(scene, r, radius, ui.field);
    let (feel, edge) = if focused {
        (Feel::Focused, ui.focus)
    } else {
        (Feel::Rest, ui.field_edge)
    };
    stroke_frame(
        scene,
        r,
        radius,
        ui.theme,
        feel,
        StrokeToken::Thin.px(),
        edge,
    );
    let pad = Spacing::Sm.px();
    let x0 = r.x + pad;
    let (f, fs) = (font(), font_shown());
    let ty = r.y + folga_do_texto(r.h, f);
    let text = buf.text().to_string();
    scene.push_clip(&rect_to_vello(r));
    if text.is_empty() {
        paint_text(
            ts,
            scene,
            placeholder,
            x0,
            ty,
            f,
            (r.w - pad * 2.0).max(0.0),
            ui.text3,
        );
    } else {
        let p = buf.selection().primary();
        if focused && !p.is_empty() {
            let (a, b) = (x_of(ts, &text, p.from()), x_of(ts, &text, p.to()));
            scene.fill_rect(
                rect_to_vello(Rect::new(x0 + a, ty, b - a, fs)),
                ui.selection,
            );
        }
        paint_text(ts, scene, &text, x0, ty, f, f32::INFINITY, ui.text1);
    }
    if focused {
        let cx = x0 + x_of(ts, &text, buf.selection().primary().head);
        scene.fill_rect(
            rect_to_vello(Rect::new(cx, ty, StrokeToken::Thick.px(), fs)),
            ui.caret,
        );
    }
    scene.pop_layer();
    x0
}

#[cfg(test)]
mod cromo_tests {
    use super::*;
    use ph2d_editor_core::panel::cards::ferramenta::force_cards_on_this_thread;

    /// ⭐ Nos cartões a barra sai da tinta da lista; com `PH2D_CARDS=0`, os tokens de sempre, e o estado
    /// (erro, foco, seleção) não muda. *Mutação: tirar o ramo de `Ui::new` ⇒ vermelho.*
    #[test]
    fn a_barra_muda_de_tinta_so_nos_cartoes() {
        let tema = Theme::Dark;
        force_cards_on_this_thread(Some(false));
        let antes = Ui::new(tema);
        force_cards_on_this_thread(Some(true));
        let novo = Ui::new(tema);
        force_cards_on_this_thread(None);
        assert_eq!(antes.field, resolve(ColorToken::CodeBg, tema));
        assert_eq!(antes.button, resolve(ColorToken::Bg3, tema));
        assert_eq!(novo.bg, resolve(ColorToken::KanbanCard, tema));
        assert_eq!(novo.field, resolve(ColorToken::CardField, tema));
        assert_eq!(novo.button, resolve(ColorToken::CardField, tema));
        assert_eq!(novo.text2, resolve(ColorToken::CardLabel, tema));
        assert_eq!(novo.hover, resolve(ColorToken::CardFieldHover, tema));
        assert_eq!(novo.error, antes.error);
        assert_eq!(novo.focus, antes.focus);
        assert_eq!(novo.selection, antes.selection);
    }

    /// O cursor e a seleção do campo têm a altura da letra desenhada (Small, Large), não a nominal.
    #[test]
    fn o_cursor_do_campo_tem_a_altura_da_letra_mostrada() {
        use ph2d_tokens::{UiTextSize, UiTextStyle};
        use ph2d_vector::DrawOp;
        let antes = ph2d_text::active_text_style();
        let mut falhas = Vec::new();
        for size in [UiTextSize::Small, UiTextSize::Large] {
            ph2d_text::set_active_text_style(UiTextStyle { size, ..antes });
            let ui = Ui::new(Theme::Dark);
            let mut buf = Buffer::new("speed");
            buf.set_selection(ph2d_code_buffer::Selection::single(
                ph2d_code_buffer::Range::new(0, 2),
            ));
            let mut ts = TextSystem::without_system_fonts();
            let mut scene = VectorScene::new();
            scene.record();
            let r = Rect::new(0.0, 0.0, 200.0, ph2d_tokens::ROW_H_PX);
            let _ = text_field(&mut scene, &mut ts, &buf, "", r, true, &ui);
            let quer = font_shown();
            let alturas: Vec<_> = scene
                .take_record()
                .into_iter()
                .filter_map(|op| match op {
                    DrawOp::Fill { brush, path, .. }
                        if brush == ui.caret.into() || brush == ui.selection.into() =>
                    {
                        let ys = path.elements().iter().filter_map(|e| e.end_point());
                        let (lo, hi) =
                            ys.fold((f64::MAX, f64::MIN), |(a, b), p| (a.min(p.y), b.max(p.y)));
                        Some((hi - lo) as f32)
                    }
                    _ => None,
                })
                .collect();
            if alturas.len() != 2 || alturas.iter().any(|h| (h - quer).abs() > 0.5) {
                falhas.push(format!("{size:?}: {alturas:?} contra {quer}"));
            }
        }
        ph2d_text::set_active_text_style(antes);
        assert!(falhas.is_empty(), "{falhas:#?}");
    }
}
