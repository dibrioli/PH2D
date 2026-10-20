//! As peças pequenas da barra de busca e do console: fundo, rótulo, botão, campo de uma linha. Cores
//! por tokens, medidas por tokens, texto na fonte da interface.

use ph2d_code_buffer::Buffer;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{
    fill_rounded_rect, frame_radius, paint_icon, paint_text, paint_text_centered, rect_to_vello,
    resolve, stroke_frame,
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
}

/// O tamanho do texto das peças.
pub(crate) fn font() -> f32 {
    ph2d_text::displayed_font_px(TypeToken::Sm.px())
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
    ph2d_editor_core::paint::label_column_width(ts, font(), [text])
}

/// A largura de um [`text_button`] com `text`: a lei inversa do orçamento com que ele o pinta
/// (`rect_for_label`) — a folga somada à mão cortava «Word» em «W…» na barra de busca.
pub(crate) fn button_w(ts: &mut TextSystem, text: &str) -> f32 {
    ph2d_editor_core::paint::rect_for_label(label_w(ts, text))
}

/// Um rótulo centrado na vertical de `r`, na cor `color`.
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
        r.y + (r.h - f) * 0.5,
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
    paint_text_centered(
        ts,
        scene,
        text,
        r,
        font(),
        if on { ui.text1 } else { ui.text2 },
    );
}

/// O x (relativo ao início do texto) do byte `byte`.
pub(crate) fn x_of(ts: &mut TextSystem, text: &str, byte: usize) -> f32 {
    let b = byte.min(text.len());
    let b = (0..=b)
        .rev()
        .find(|&i| text.is_char_boundary(i))
        .unwrap_or(0);
    ts.prefix_width(&text[..b], font())
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
        let x = ts.prefix_width(&text[..i], font());
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
    let f = font();
    let ty = r.y + (r.h - f) * 0.5;
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
            scene.fill_rect(rect_to_vello(Rect::new(x0 + a, ty, b - a, f)), ui.selection);
        }
        paint_text(ts, scene, &text, x0, ty, f, f32::INFINITY, ui.text1);
    }
    if focused {
        let cx = x0 + x_of(ts, &text, buf.selection().primary().head);
        scene.fill_rect(
            rect_to_vello(Rect::new(cx, ty, StrokeToken::Thick.px(), f)),
            ui.caret,
        );
    }
    scene.pop_layer();
    x0
}
