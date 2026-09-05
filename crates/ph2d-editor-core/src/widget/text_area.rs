//! [`TextArea`] — multiline text field.
//!
//! Same border/focus/error palette as [`super::text_input::TextInput`]
//! but reserves at least 3 rows of vertical space and renders each
//! hard-wrapped (`\n`) line separately.
//!
//! Caret + selection rendering uses the same real-measurement pattern
//! as [`super::text_input::paint_text_input_with_buffer`] (see
//! `docs/UI_Bugs/README.md` §3.3) — char-count approximations land
//! between glyphs on proportional fonts.

use crate::paint::{fill_rounded_rect, paint_text, rect_to_vello, resolve};
use crate::widget::text_input::{TextInputState, border_color, fill_token};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Clone, Debug)]
pub struct TextArea {
    pub id: NodeId,
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub state: TextInputState,
    /// Quanto do hover está presente; ver [`super::TextInput::hover_t`].
    pub hover_t: f32,
}

impl TextArea {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            value: String::new(),
            placeholder: String::new(),
            state: TextInputState::Normal,
            hover_t: crate::motion::SETTLED,
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }

    pub fn state(mut self, state: TextInputState) -> Self {
        self.state = state;
        self
    }

    /// **O par que o store publica** — `(estado, quanto do hover está presente)`.
    /// Irmão exacto do [`super::TextInput::visual`].
    #[must_use]
    pub fn visual(self, v: (TextInputState, f32)) -> Self {
        self.state(v.0).hover_t(v.1)
    }

    #[must_use]
    pub fn hover_t(mut self, t: f32) -> Self {
        self.hover_t = t.clamp(0.0, 1.0);
        self
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::MultilineTextInput)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != TextInputState::Disabled)
            .action(Action::Focus)
            .build()
    }
}

/// **Onde o texto de uma `TextArea` pousa** — a régua ÚNICA.
///
/// ⚠️ Ela existe porque a mesma regra estava escrita em **três** sítios com **três** graus de
/// fidelidade: o pintor lia os tokens VIVOS, o `min_height` reservava três alturas de FONTE (e não
/// três alturas de LINHA), e o `byte_offset_from_click_xy` do despacho copiava os números
/// (`rect.x + 12.0`, `rect.y + 8.0`, `font_size + 4.0`) sob um comentário que dizia *"matches the
/// painter"*.
///
/// ⚠️ **E a cópia não era latente, era VIVA:** `Spacing::px()` devolve o valor **autorado** desde
/// que a escala numérica virou editável, então bastava o artista mexer no `spacing.md` para o
/// caret deixar de cair onde ele clicou. Medido com `md = 20`: clicar no meio de **qualquer** linha
/// punha o caret na linha SEGUINTE — o pintor desenhava a linha 0 em `y = 220` e o despacho
/// procurava-a em `y = 208`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TextAreaMetrics {
    /// A margem esquerda do texto.
    pub inner_x: f32,
    /// O topo da PRIMEIRA linha.
    pub inner_y: f32,
    /// A largura útil.
    pub inner_w: f32,
    /// O passo vertical entre linhas — ⚠️ **maior que a fonte**, e é essa diferença que o
    /// `min_height` reservava a menos.
    pub line_h: f32,
}

/// A régua de `rect`, nos tokens VIVOS.
#[must_use]
pub fn metrics(rect: Rect) -> TextAreaMetrics {
    let pad_x = Spacing::Lg.px();
    let pad_y = Spacing::Md.px();
    TextAreaMetrics {
        inner_x: rect.x + pad_x,
        inner_y: rect.y + pad_y,
        inner_w: (rect.w - pad_x * 2.0).max(0.0),
        line_h: TypeToken::Base.px() + Spacing::Xs.px(),
    }
}

/// **Altura mínima para as três linhas que este widget PROMETE.**
///
/// ⚠️ **Não recebe mais a fonte, e o parâmetro era uma mentira:** o pintor desenha sempre em
/// `TypeToken::Base`, então um chamador que passasse outro tamanho recebia a altura de uma fonte
/// que o widget não usa. A fonte é do WIDGET.
///
/// ⚠️ E ela reservava três alturas de **FONTE** onde o pintor gasta três alturas de **LINHA** —
/// medido, 55 px contra os 67 que ele precisa: a terceira linha caía 12 px para fora da caixa.
#[must_use]
pub fn min_height() -> f32 {
    let m = metrics(Rect::new(0.0, 0.0, 0.0, 0.0));
    m.line_h * 3.0 + Spacing::Md.px() * 2.0 // LITERAL-PX-OK: 3 = as tres linhas que o widget promete (contagem)
}

pub fn paint_text_area(
    area: &TextArea,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    paint_text_area_with_state(area, None, None, rect, scene, text_system, theme);
}

/// Like [`paint_text_area`] but draws a caret line and (optionally) a
/// selection range when focused. Pass `caret = None` /
/// `selection_anchor = None` for the unfocused case.
///
/// Multi-line aware: the painter splits `value` on `\n`, walks each
/// line, and uses the real text-system layout width of the prefix
/// inside the caret's line to position the caret between glyphs (not
/// on top of them — see `docs/UI_Bugs/README.md` §3.3).
#[allow(clippy::too_many_arguments)]
pub fn paint_text_area_with_state(
    area: &TextArea,
    caret: Option<usize>,
    selection_anchor: Option<usize>,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    // ⭐ Raio e moldura pela porta do TEMA (ver `number_input`).
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    fill_rounded_rect(scene, rect, radius, resolve(fill_token(area.state), theme));
    let stroke_w = if area.state == TextInputState::Focused {
        2.0
    } else {
        1.0
    };
    crate::paint::stroke_frame(
        scene,
        rect,
        radius,
        theme,
        crate::widget::text_input::feel_of(area.state),
        stroke_w,
        border_color(area.state, area.hover_t, theme),
    );

    // ⚠️ Pela porta ÚNICA, e não por uma segunda cópia da regra: o despacho de clique lê a MESMA,
    // e é isso que faz o caret cair onde o artista clicou depois de a escala ser autorada.
    let TextAreaMetrics {
        inner_x,
        inner_y,
        inner_w,
        line_h,
    } = metrics(rect);
    let inner_right = inner_x + inner_w;
    let font_size = TypeToken::Base.px();

    // ⚠️ **O RECORTE, e o irmão de uma linha já o fazia** — a razão está escrita nele, num report
    // do Enio (2026-07-16): *"renaming a clip to a long name drew a SECOND line that ran out of the
    // box and over the buttons below it"*. O `TextInput` ganhou-o naquele dia; este, que por
    // construção guarda MAIS texto, ficou sem — e ao contrário dele **não scrolla**, então tudo o
    // que não coubesse era pintado por cima do que estivesse por baixo.
    //
    // ⚠️ **A moldura é a do irmão, verbatim:** horizontalmente o box INTERNO (um glifo não pisa a
    // borda nem o enchimento), verticalmente o rect INTEIRO — o enchimento de baixo existe para a
    // última linha ter onde descer, e cortá-lo ali decepava as descidas das letras.
    //
    // ⚠️ E o que **não** vem com ele está nomeado: uma `TextArea` cheia demais deixa de mostrar o
    // fim do texto, em vez de o mostrar fora da caixa. Rolar é uma FEATURE (o irmão a tem porque
    // uma linha só precisa de perseguir o caret na horizontal), e inventá-la aqui de passagem seria
    // contrabandear uma decisão de produto dentro de uma correção de desenho.
    scene.push_clip(&rect_to_vello(Rect::new(inner_x, rect.y, inner_w, rect.h)));

    let focused = area.state == TextInputState::Focused;
    let displayed = area.value.as_str();

    // Selection background per affected line — drawn before the
    // text so glyphs sit on top. For multi-line selections the rect
    // extends to the right edge on intermediate lines (standard
    // text-editor convention).
    if focused
        && !displayed.is_empty()
        && let Some(anchor) = selection_anchor
        && let Some(c) = caret
        && anchor != c
    {
        let (mut s, mut e) = if anchor < c { (anchor, c) } else { (c, anchor) };
        s = s.min(displayed.len());
        e = e.min(displayed.len());

        let mut line_start: usize = 0;
        for (line_idx, line) in displayed.split('\n').enumerate() {
            let line_end = line_start + line.len();
            if e >= line_start && s <= line_end {
                let local_s = s.saturating_sub(line_start).min(line.len());
                let local_e = (e - line_start).min(line.len());
                let prefix_w = text_system.prefix_width(&line[..local_s], font_size);
                let mid_w = if local_s == local_e {
                    if e > line_end {
                        inner_right - (inner_x + prefix_w)
                    } else {
                        0.0
                    }
                } else {
                    let measured = text_system.prefix_width(&line[local_s..local_e], font_size);
                    if e > line_end {
                        (inner_right - (inner_x + prefix_w)).max(measured)
                    } else {
                        measured
                    }
                };
                let sel_x = (inner_x + prefix_w).min(inner_right);
                let sel_w = mid_w.min(inner_right - sel_x);
                if sel_w > 0.0 {
                    let sel_y = inner_y + line_idx as f32 * line_h;
                    let sel_rect = Rect::new(sel_x, sel_y, sel_w, line_h);
                    fill_rounded_rect(scene, sel_rect, 1.0, resolve(ColorToken::AccentSoft, theme));
                }
            }
            line_start = line_end + 1; // +1 for the '\n'
        }
    }

    if displayed.is_empty() && !area.placeholder.is_empty() {
        paint_text(
            text_system,
            scene,
            &area.placeholder,
            inner_x,
            inner_y,
            font_size,
            inner_w,
            resolve(ColorToken::Text3, theme),
        );
    } else if !displayed.is_empty() {
        let color = if area.state == TextInputState::Disabled {
            ColorToken::TextDisabled
        } else {
            ColorToken::Text1
        };
        // Paint each line individually so the caret/selection math
        // (which uses `\n`-split offsets) stays in lockstep with the
        // visible glyph layout. Using a single paint_text call would
        // let parley insert its own line breaks for long lines,
        // desyncing the caret.
        for (i, line) in displayed.split('\n').enumerate() {
            paint_text(
                text_system,
                scene,
                line,
                inner_x,
                inner_y + i as f32 * line_h,
                font_size,
                inner_w,
                resolve(color, theme),
            );
        }
    }

    if focused {
        let caret_byte = caret.unwrap_or(0).min(displayed.len());
        let mut line_start: usize = 0;
        let mut line_idx: usize = 0;
        let mut line_text: &str = "";
        for line in displayed.split('\n') {
            let line_end = line_start + line.len();
            if caret_byte <= line_end {
                line_text = line;
                break;
            }
            line_start = line_end + 1;
            line_idx += 1;
        }
        let local = caret_byte.saturating_sub(line_start).min(line_text.len());
        let prefix = &line_text[..local];
        let prefix_w = if prefix.is_empty() {
            0.0
        } else {
            text_system.prefix_width(prefix, font_size)
        };
        let caret_x = (inner_x + prefix_w).min(inner_right);
        let caret_y = inner_y + line_idx as f32 * line_h;
        let caret_rect = Rect::new(
            caret_x,
            caret_y,
            StrokeToken::Default.px(),
            (line_h - 2.0).max(2.0),
        );
        scene.fill_rect(
            rect_to_vello(caret_rect),
            resolve(ColorToken::Accent, theme),
        );
    }

    // ⚠️ Fecha o recorte aberto no topo. Uma camada deixada aberta não corta só este widget: ela
    // corta **tudo o que for pintado depois dele** — o gate afirma `n_open_clips == 0` por isso.
    scene.pop_layer();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let a = TextArea::new(NodeId(1), "Notes");
        assert_eq!(a.value, "");
        assert_eq!(a.state, TextInputState::Normal);
    }

    /// **A terceira linha CABE numa caixa da altura reservada.**
    ///
    /// ⚠️ O `min_height` reservava tres alturas de FONTE onde o pintor gasta tres alturas de
    /// LINHA — medido, 55 px contra os 67 que ele precisa. A terceira linha caia 12 px para fora
    /// da caixa que o widget diz bastar.
    ///
    /// ⚠️ **O oraculo e' o fundo da terceira linha contra o fundo da CAIXA**, e nao a formula do
    /// `min_height` reescrita aqui: a primeira versao deste gate afirmava `min_height() >=
    /// line_h * 3.0` — verdadeira TAMBEM na versao quebrada (55 >= 51), porque esquecia o
    /// enchimento que a propria caixa declara. Ela ficou VERDE sobre a mutacao que repoe o
    /// defeito, e foi a mutacao quem a denunciou.
    #[test]
    fn the_third_line_fits_inside_a_box_of_the_reserved_height() {
        let rect = Rect::new(0.0, 0.0, 200.0, min_height());
        let m = metrics(rect);
        let fundo_da_terceira = m.inner_y + m.line_h * 3.0;
        assert!(
            fundo_da_terceira <= rect.y + rect.h,
            "a terceira linha acaba em {fundo_da_terceira} e a caixa acaba em {} — ela e'              desenhada para fora do que o widget diz reservar",
            rect.y + rect.h
        );
    }

    #[test]
    fn a11y_role_is_multiline_text_input() {
        let node = TextArea::new(NodeId(1), "x").build_a11y(0.0, 0.0, 200.0, 80.0);
        assert_eq!(node.role(), Role::MultilineTextInput);
    }

    fn smoke(area: TextArea, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_text_area(
            &area,
            Rect::new(0.0, 0.0, 240.0, 96.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    /// **O texto que passa da caixa e' CORTADO, nao desenhado por cima do que houver ali.**
    ///
    /// ⚠️ **O irmao de UMA linha ja' faz isto, e a razao esta' escrita nele — um report do Enio
    /// (2026-07-16):** *"renaming a clip to a long name drew a SECOND line that ran out of the box
    /// and over the buttons below it"*. O `TextInput` ganhou o recorte naquele dia; o `TextArea`,
    /// que por construcao guarda MAIS texto, ficou sem — e ele nem sequer scrolla, entao tudo o que
    /// nao cabe era pintado para fora.
    ///
    /// ⚠️ O oraculo e' o MESMO do irmao (`n_clips` / `n_open_clips` da codificacao), e nao uma
    /// segunda maneira de perguntar *"isto ficou dentro da caixa?"*.
    #[test]
    fn text_past_the_box_is_clipped_instead_of_spilling_over_what_is_below() {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        // Seis linhas numa caixa dimensionada para as tres que o widget promete.
        let a = TextArea::new(NodeId(1), "Notes")
            .value("um\ndois\ntres\nquatro\ncinco\nseis")
            .state(TextInputState::Focused);
        paint_text_area_with_state(
            &a,
            Some(0),
            None,
            Rect::new(0.0, 0.0, 240.0, min_height()),
            &mut scene,
            &mut text,
            Theme::Forge,
        );
        let enc = scene.inner().encoding();
        assert!(
            enc.n_clips >= 1,
            "a caixa nao recorta o proprio texto — a quarta linha em diante e' desenhada por cima              do que estiver por baixo dela"
        );
        assert_eq!(enc.n_open_clips, 0, "o recorte ficou aberto");
    }

    #[test]
    fn paint_smoke_empty_with_placeholder() {
        smoke(
            TextArea::new(NodeId(1), "x").placeholder("Notes…"),
            Theme::Forge,
        );
    }

    #[test]
    fn paint_smoke_filled_focused() {
        smoke(
            TextArea::new(NodeId(1), "x")
                .value("multi\nline\nvalue")
                .state(TextInputState::Focused),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(
            TextArea::new(NodeId(1), "x")
                .value("read-only")
                .state(TextInputState::Disabled),
            Theme::Blueprint,
        );
    }
}
