//! Single-line, ellipsized text — the counterpart to [`crate::paint::paint_text`],
//! whose `max_width` is a **wrap** budget.
//!
//! A label one pixel too wide for its column silently becomes two lines and
//! spills into the row below (the timeline track names did exactly that). Rows,
//! list items and any other fixed-height slot need text that truncates instead.
//!
//! Its own module because `paint.rs` sits at its frozen LOC cap (the gate's
//! rule is to drive those DOWN, never up).

use ph2d_text::TextSystem;
use ph2d_vector::{Color, VectorScene};

use crate::paint::paint_text;

/// The ellipsis appended to text that does not fit. Inside Inter's coverage
/// (U+2026 is not one of the arrow / technical blocks the tofu gate rejects).
const ELLIPSIS: &str = "\u{2026}";

/// Paint `text` on **one line**, ellipsized when it does not fit `max_width`.
///
/// [`paint_text`] treats `max_width` as a *wrap* budget, so a label one pixel
/// too wide silently becomes two lines and spills into the row below. Anything
/// that must stay on its own line — list rows, track names — belongs here.
///
/// `max_width` too small for even the ellipsis paints nothing.
#[allow(clippy::too_many_arguments)]
pub fn paint_text_elided(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
) {
    if max_width <= 0.0 {
        return;
    }
    if text_system.prefix_width(text, font_size) <= max_width {
        // `INFINITY`, not `max_width`: it fits, and passing the budget back would
        // let a sub-pixel measurement disagreement re-introduce the wrap.
        paint_text(
            text_system,
            scene,
            text,
            x,
            y,
            font_size,
            f32::INFINITY,
            color,
        );
        return;
    }
    let Some(elided) = elide(text_system, text, font_size, max_width) else {
        return;
    };
    paint_text(
        text_system,
        scene,
        &elided,
        x,
        y,
        font_size,
        f32::INFINITY,
        color,
    );
}

/// The longest `<prefix>…` of `text` that measures within `max_width`, or `None`
/// when not even the ellipsis fits. Binary search over char boundaries, so a
/// multi-byte name is never cut mid-glyph.
///
/// ⛔⛔⛔ **PRIVADA, e o contrato é «eu já sei que não cabe»** — ela **nunca** devolve o texto
/// inteiro: os limites da busca binária param no início do ÚLTIMO carácter, então até uma string
/// que caiba folgada sai com um carácter a menos e reticências.
///
/// Report do Enio com foto (2026-08-31): *«Mudei o size mas o nome do Botão não atualiza»* — os
/// dois chips liam-se `Smal …` e `Small …`, indistinguíveis, sobre valores `Small` e `Small 2` que
/// cabiam com folga num chip **medido a 88,7 px**. Eu tinha-a tornado pública e chamado sem a
/// guarda que o [`paint_text_elided`] faz três linhas acima. *Um helper privado carrega a
/// pré-condição do único chamador que ele tinha; publicá-lo sem a publicar é publicar meia função.*
///
/// ⇒ quem precisa da STRING chama o [`fit`], que faz a guarda.
#[must_use]
fn elide(
    text_system: &mut TextSystem,
    text: &str,
    font_size: f32,
    max_width: f32,
) -> Option<String> {
    if text_system.prefix_width(ELLIPSIS, font_size) > max_width {
        return None;
    }
    let bounds: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    // Invariant: `lo` chars always fit, `hi` chars never do.
    let (mut lo, mut hi) = (0usize, bounds.len());
    while lo + 1 < hi {
        let mid = lo + (hi - lo) / 2;
        let candidate = format!("{}{ELLIPSIS}", &text[..bounds[mid]]);
        if text_system.prefix_width(&candidate, font_size) <= max_width {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(format!("{}{ELLIPSIS}", &text[..bounds[lo]]))
}

/// ⭐⭐ **O texto que cabe em `max_width`** — ele próprio quando cabe, `<prefixo>…` quando não.
///
/// É o [`paint_text_elided`] sem o pincel: quem faz a própria centragem (um `Button`) precisa da
/// string, não do desenho. ⚠️ **A guarda do «cabe» é a razão de esta porta existir** — ver o doc do
/// [`elide`], que sem ela corta sempre.
///
/// ⛔ Quando nem as reticências cabem devolve o texto **cru**: um rótulo cortado a zero é um botão
/// mudo, e é melhor transbordar visivelmente do que desaparecer.
#[must_use]
pub fn fit(text_system: &mut TextSystem, text: &str, font_size: f32, max_width: f32) -> String {
    if text_system.prefix_width(text, font_size) <= max_width {
        return text.to_string();
    }
    elide(text_system, text, font_size, max_width).unwrap_or_else(|| text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⛔⛔⛔ **O que CABE sai VERBATIM** — o defeito do report de 2026-08-31.
    ///
    /// O `elide` **nunca** devolve o texto inteiro (os limites param no início do último carácter),
    /// e eu chamei-o sem guarda: `Small` saía `Smal …` e `Small 2` saía `Small …` — dois chips
    /// indistinguíveis sobre valores que cabiam com folga.
    ///
    /// ⚠️ **O gate irmão não podia apanhá-lo:** ele mede com `budget = full * 0.6`, isto é, uma
    /// fixtura em que o texto **nunca** cabe. *Uma fixtura sem o fenómeno mede silêncio.*
    #[test]
    fn what_fits_comes_back_untouched() {
        let mut text = TextSystem::without_system_fonts();
        let label = "Small 2";
        let full = text.prefix_width(label, 13.0);
        assert_eq!(fit(&mut text, label, 13.0, full + 1.0), label);
        assert_eq!(fit(&mut text, label, 13.0, full * 4.0), label);
        // ⚠️ E o que NÃO cabe continua a ser cortado — a guarda não pode desligar o corte.
        let cut = fit(&mut text, label, 13.0, full * 0.5);
        assert!(cut.ends_with(ELLIPSIS), "{cut:?}");
        assert!(label.starts_with(cut.trim_end_matches(ELLIPSIS)), "{cut:?}");
        // ⛔ E quando nem as reticências cabem, o CRU — nunca uma string vazia.
        assert_eq!(fit(&mut text, label, 13.0, 0.5), label);
    }

    #[test]
    fn elide_trims_to_the_widest_prefix_that_fits() {
        let mut text = TextSystem::without_system_fonts();
        let long = "Translate Y  #4591";
        let full = text.prefix_width(long, 12.0);
        let budget = full * 0.6;
        let out = elide(&mut text, long, 12.0, budget).expect("something fits");
        assert!(out.ends_with(ELLIPSIS), "{out:?}");
        assert!(long.starts_with(out.trim_end_matches(ELLIPSIS)), "{out:?}");
        assert!(text.prefix_width(&out, 12.0) <= budget, "{out:?} overruns");
        // ...and it is the WIDEST such prefix: one more char overflows.
        let kept = out.trim_end_matches(ELLIPSIS).chars().count();
        let more: String = long.chars().take(kept + 1).collect();
        assert!(text.prefix_width(&format!("{more}{ELLIPSIS}"), 12.0) > budget);
    }

    #[test]
    fn elide_gives_up_when_not_even_the_ellipsis_fits() {
        let mut text = TextSystem::without_system_fonts();
        assert_eq!(elide(&mut text, "Translate Y", 12.0, 0.5), None);
    }

    #[test]
    fn elide_never_cuts_a_multi_byte_char_in_half() {
        let mut text = TextSystem::without_system_fonts();
        let name = "Rotação · ângulo";
        let full = text.prefix_width(name, 12.0);
        for frac in [0.2, 0.4, 0.6, 0.8] {
            if let Some(out) = elide(&mut text, name, 12.0, full * frac) {
                // A mid-char cut would have panicked in `elide` already; assert
                // the surviving prefix is a real prefix of the original.
                assert!(name.starts_with(out.trim_end_matches(ELLIPSIS)));
            }
        }
    }
}
