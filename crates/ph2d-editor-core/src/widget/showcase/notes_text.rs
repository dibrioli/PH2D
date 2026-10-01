//! ⭐⭐ **O TEXTO de uma nota — a QUEBRA de linha automática e os dois pintores editáveis**
//! (2026-10-01, ordem do dono: *«O card das notas não cresce com o aumento do número de palavras e
//! linhas e não há quebra automática de linha. Corrija tudo»*).
//!
//! Até aqui o corpo de uma nota tinha **três linhas fixas** e cada `\n` era uma linha: uma frase
//! comprida saía cortada com reticências e a quarta linha era desenhada FORA do cartão.
//!
//! ⭐ **Uma lei, três leitores:** [`linhas_visuais`] parte o texto em linhas que CABEM na largura
//! (por palavras; uma palavra mais comprida que a linha parte-se por caractere), e é ela que
//! (1) o pintor desenha, (2) a altura do cartão conta e (3) o despacho do clique usa para pôr o
//! caret onde o rato está ([`crate::interaction`], `text_ops`). Três contas punham o caret numa
//! linha e o texto noutra.
//!
//! ⚠️ **As setas ↑/↓ continuam a andar por linhas LÓGICAS** (`\n`): o despacho de teclado não tem
//! o sistema de texto, e medir ali seria a segunda cópia desta lei. Declarado.

use crate::paint::{fill_rounded_rect, paint_text};
use crate::widget::TextInputState;
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, StrokeToken};
use ph2d_vector::VectorScene;

/// O que um pintor de caixa lê dela — `(estado, texto, caret, âncora)`, o que
/// [`super::read_text_input`] devolve. ⚠️ Os pintores recebem os VALORES e não o `WidgetStore`
/// (2026-10-01): este ficheiro não conhece o estado de widget, que é a aresta `widget → interaction`
/// que a catraca da fundação quer ver encolher.
pub(super) type CaixaLida<'a> = (TextInputState, &'a str, usize, Option<usize>);

/// ⭐ **As linhas visuais de `texto` numa largura `largura`** — `(início, fim)` em bytes, uma
/// PARTIÇÃO de cada linha lógica: a linha `k` acaba onde a `k+1` começa (os espaços de uma quebra
/// ficam no fim da linha de cima), e a última de cada linha lógica acaba no `\n` dela.
///
/// `mede` devolve a largura de um pedaço de texto. Uma linha lógica vazia dá uma linha vazia.
pub fn linhas_visuais(
    texto: &str,
    largura: f32,
    mut mede: impl FnMut(&str) -> f32,
) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut ini_logica = 0_usize;
    for logica in texto.split('\n') {
        let fim_logica = ini_logica + logica.len();
        let mut a = ini_logica;
        loop {
            if a >= fim_logica || mede(&texto[a..fim_logica]) <= largura {
                out.push((a, fim_logica));
                break;
            }
            // A última quebra DEPOIS de um espaço que ainda cabe.
            let mut melhor: Option<usize> = None;
            for (k, c) in texto[a..fim_logica].char_indices() {
                if c == ' ' {
                    let b = a + k + 1;
                    if mede(texto[a..b].trim_end()) <= largura {
                        melhor = Some(b);
                    } else {
                        break;
                    }
                }
            }
            let b = melhor.filter(|&b| b < fim_logica).unwrap_or_else(|| {
                // Uma palavra mais comprida que a linha: parte-se no último caractere que cabe
                // (pelo menos um, senão o laço não andava).
                let mut fim = a;
                for (k, c) in texto[a..fim_logica].char_indices() {
                    let e = a + k + c.len_utf8();
                    if e > a && mede(&texto[a..e]) > largura && fim > a {
                        break;
                    }
                    fim = e;
                }
                fim
            });
            out.push((a, b));
            a = b;
        }
        ini_logica = fim_logica + 1;
    }
    out
}

/// ⭐ **A linha visual onde está o byte `c`** — a ÚLTIMA cujo início é `≤ c`: numa quebra suave o
/// caret no ponto da quebra vai para o início da linha de baixo; no fim de uma linha lógica fica no
/// fim dela (a seguinte começa depois do `\n`).
#[must_use]
pub fn linha_do_byte(linhas: &[(usize, usize)], c: usize) -> usize {
    linhas.iter().rposition(|&(a, _)| a <= c).unwrap_or(0)
}

/// O texto que uma linha visual DESENHA — sem os espaços da quebra no fim.
#[must_use]
pub fn texto_da_linha(texto: &str, (a, b): (usize, usize)) -> &str {
    texto[a..b].trim_end_matches(' ')
}

/// ⭐ **Quantas linhas o corpo de uma nota ocupa** — as visuais do texto (ou do texto de espera
/// quando ele está vazio), e nunca menos de `3`: um corpo novo convida a escrever.
#[must_use]
pub fn linhas_do_corpo(
    text_system: &mut TextSystem,
    texto: &str,
    largura: f32,
    fonte: f32,
) -> usize {
    let n = if texto.is_empty() {
        1
    } else {
        linhas_visuais(texto, largura, |s| text_system.prefix_width(s, fonte)).len()
    };
    n.max(LINHAS_MINIMAS)
}

/// O corpo de uma nota nunca tem menos linhas do que isto.
pub const LINHAS_MINIMAS: usize = 3;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_note_editable_multiline(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    caixa: CaixaLida<'_>,
    rect: Rect,
    font_size: f32,
    fg: ph2d_vector::Color,
    placeholder: &str,
) {
    let (state, text, caret, anchor) = caixa;
    let focused = state == TextInputState::Focused;
    // ⚠️ A régua é a do `TextArea` — a mesma que o despacho do clique usa para a nota.
    let m = crate::widget::text_area_metrics(rect);
    let (text_x, text_y0, text_w, line_h) = (m.inner_x, m.inner_y, m.inner_w, m.line_h);
    if text.is_empty() && !focused {
        paint_text(
            text_system,
            scene,
            placeholder,
            text_x,
            text_y0,
            font_size,
            text_w,
            ph2d_vector::Color::from_rgba8(0x21, 0x21, 0x21, 0x80), // LITERAL-COLOR-OK: note-placeholder multiline
        );
        return;
    }
    let linhas = linhas_visuais(text, text_w, |s| text_system.prefix_width(s, font_size));
    if focused
        && let Some(a) = anchor
        && a != caret
    {
        let (s, e) = if a < caret { (a, caret) } else { (caret, a) };
        let s = s.min(text.len());
        let e = e.min(text.len());
        let sel_color = ph2d_vector::Color::from_rgba8(0x21, 0x21, 0x21, 0x33); // LITERAL-COLOR-OK: note-selection multiline
        for (i, &(la, lb)) in linhas.iter().enumerate() {
            let seg_s = s.max(la);
            let seg_e = e.min(lb);
            if seg_s < seg_e {
                let prefix_w = text_system.prefix_width(&text[la..seg_s], font_size);
                let mid_w = text_system.prefix_width(&text[seg_s..seg_e], font_size);
                let sel_x = text_x + prefix_w;
                let sel_w = mid_w.min(text_x + text_w - sel_x).max(0.0);
                if sel_w > 0.0 {
                    let sel = Rect::new(sel_x, text_y0 + i as f32 * line_h, sel_w, line_h);
                    fill_rounded_rect(scene, sel, 1.0, sel_color);
                }
            }
        }
    }
    for (i, &l) in linhas.iter().enumerate() {
        paint_text(
            text_system,
            scene,
            texto_da_linha(text, l),
            text_x,
            text_y0 + i as f32 * line_h,
            font_size,
            text_w,
            fg,
        );
    }
    if focused {
        let caret_byte = caret.min(text.len());
        let k = linha_do_byte(&linhas, caret_byte);
        let la = linhas.get(k).map_or(0, |l| l.0);
        let prefix_w = text_system.prefix_width(&text[la..caret_byte.max(la)], font_size);
        let caret_rect = Rect::new(
            (text_x + prefix_w).min(text_x + text_w),
            text_y0 + k as f32 * line_h,
            StrokeToken::Default.px(),
            (line_h - 2.0).max(2.0),
        );
        fill_rounded_rect(scene, caret_rect, 0.75, fg); // LITERAL-PX-OK: caret half-width radius
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_note_editable_line(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    caixa: CaixaLida<'_>,
    rect: Rect,
    font_size: f32,
    fg: ph2d_vector::Color,
    placeholder: &str,
) {
    let (state, text, caret, anchor) = caixa;
    let focused = state == TextInputState::Focused;
    let text_x = rect.x + super::notes::note_text_pad_x();
    let text_w = (rect.w - super::notes::note_text_pad_x()).max(0.0);
    let text_y = rect.y + (rect.h - font_size) * 0.5;
    if focused
        && !text.is_empty()
        && let Some(a) = anchor
        && a != caret
    {
        let (s, e) = if a < caret { (a, caret) } else { (caret, a) };
        let s = s.min(text.len());
        let e = e.min(text.len());
        let prefix_w = text_system.prefix_width(&text[..s], font_size);
        let mid_w = if s == e {
            0.0
        } else {
            text_system.prefix_width(&text[s..e], font_size)
        };
        let sel_x = text_x + prefix_w;
        let sel_w = mid_w.min(text_x + text_w - sel_x).max(0.0);
        if sel_w > 0.0 {
            let sel = Rect::new(
                sel_x,
                rect.y + 2.0,
                sel_w,
                (rect.h - Spacing::Xs.px()).max(2.0),
            );
            let sel_color = ph2d_vector::Color::from_rgba8(0x21, 0x21, 0x21, 0x33); // LITERAL-COLOR-OK: note-selection
            fill_rounded_rect(scene, sel, 1.0, sel_color);
        }
    }
    let displayed: &str = if text.is_empty() && !focused {
        placeholder
    } else {
        text
    };
    let display_color = if text.is_empty() && !focused {
        ph2d_vector::Color::from_rgba8(0x21, 0x21, 0x21, 0x80) // LITERAL-COLOR-OK: note-placeholder
    } else {
        fg
    };
    paint_text(
        text_system,
        scene,
        displayed,
        text_x,
        text_y,
        font_size,
        text_w,
        display_color,
    );
    if focused {
        let caret_byte = caret.min(text.len());
        let prefix_w = text_system.prefix_width(&text[..caret_byte], font_size);
        let caret_rect = Rect::new(
            (text_x + prefix_w).min(text_x + text_w),
            rect.y + 2.0,
            StrokeToken::Default.px(),
            (rect.h - Spacing::Xs.px()).max(2.0),
        );
        fill_rounded_rect(scene, caret_rect, 0.75, fg); // LITERAL-PX-OK: caret half-width radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Largura de teste: um caractere = 1.
    fn chars(s: &str) -> f32 {
        s.chars().count() as f32
    }

    /// ⭐⭐ **Quebra por palavras e a partição cobre o texto inteiro.** *Mutação: devolver a linha
    /// lógica inteira ⇒ uma linha.*
    #[test]
    fn quebra_por_palavras_e_cobre_o_texto() {
        let t = "uma frase bem comprida aqui";
        let l = linhas_visuais(t, 10.0, chars);
        let vistas: Vec<&str> = l.iter().map(|&x| texto_da_linha(t, x)).collect();
        assert_eq!(vistas, vec!["uma frase", "bem", "comprida", "aqui"]);
        // partição: cada uma acaba onde a seguinte começa, e a última no fim
        for w in l.windows(2) {
            assert_eq!(w[0].1, w[1].0);
        }
        assert_eq!(l.last().unwrap().1, t.len());
        for &(a, b) in &l {
            assert!(chars(texto_da_linha(t, (a, b))) <= 10.0);
        }
    }

    /// Uma palavra mais comprida que a linha parte-se por caractere; o `\n` é sempre uma quebra.
    #[test]
    fn palavra_comprida_e_linha_logica() {
        let t = "abcdefghij\nxy";
        let l = linhas_visuais(t, 4.0, chars);
        let vistas: Vec<&str> = l.iter().map(|&x| texto_da_linha(t, x)).collect();
        assert_eq!(vistas, vec!["abcd", "efgh", "ij", "xy"]);
        assert_eq!(linhas_visuais("", 4.0, chars), vec![(0, 0)]);
        assert_eq!(linhas_visuais("a\n\nb", 4.0, chars).len(), 3);
    }

    /// ⭐ O caret numa quebra suave vai para a linha de BAIXO; no fim de uma linha lógica fica nela.
    #[test]
    fn o_caret_na_quebra() {
        let t = "uma frase bem";
        let l = linhas_visuais(t, 10.0, chars); // "uma frase " | "bem"
        assert_eq!(
            linha_do_byte(&l, 10),
            1,
            "no ponto da quebra suave: a de baixo"
        );
        assert_eq!(linha_do_byte(&l, 9), 0);
        let t2 = "ab\ncd";
        let l2 = linhas_visuais(t2, 10.0, chars);
        assert_eq!(linha_do_byte(&l2, 2), 0, "antes do \\n: a de cima");
        assert_eq!(linha_do_byte(&l2, 3), 1);
    }
}
