//! Single-line, ellipsized text — the counterpart to [`crate::paint::paint_text`],
//! whose `max_width` is a **wrap** budget.
//!
//! A label one pixel too wide for its column silently becomes two lines and
//! spills into the row below (the timeline track names did exactly that). Rows,
//! list items and any other fixed-height slot need text that truncates instead.
//!
//! Its own module because `paint.rs` sits at its frozen LOC cap (the gate's
//! rule is to drive those DOWN, never up).

use ph2d_text::{FontWeight, TextSystem};
use ph2d_vector::{Color, VectorScene};

use crate::paint::paint_text_weighted;

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
    paint_elided_weighted(
        text_system,
        scene,
        text,
        x,
        y,
        font_size,
        max_width,
        color,
        FontWeight::MEDIUM,
    );
}

/// ⭐ [`paint_text_elided`] em **SemiBold** — a irmã de [`paint_text_title`], para o mesmo
/// motivo pelo qual ela existe.
///
/// ⚠️ Sem ela, cortar um TÍTULO obrigava a escolher entre duas regressões silenciosas: pintar
/// o corte em `Medium` (o título muda de peso e ninguém escreveu isso) ou medir em `Medium` e
/// pintar em `SemiBold` (o prefixo escolhido transborda ~3 %, exactamente na fronteira em que o
/// corte existe para não transbordar).
#[allow(clippy::too_many_arguments)]
pub fn paint_text_title_elided(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
) {
    paint_elided_weighted(
        text_system,
        scene,
        text,
        x,
        y,
        font_size,
        max_width,
        color,
        FontWeight::SEMI_BOLD,
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_elided_weighted(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
    weight: FontWeight,
) {
    // ⚠️⚠️ **O peso ATRAVESSA, não é escolhido por um `if`.** A 1.ª redacção ramificava em
    // `weight == SEMI_BOLD`, e uma auditoria adversarial mostrou que era **um braço só**:
    // um terceiro peso seria MEDIDO nele e PINTADO em Medium, em silêncio — o defeito exacto
    // que este módulo existe para impedir. Três mutações sobreviveram a 1 100 testes por
    // causa dele. *Uma lista de pesos é uma lista que alguém esquece; um parâmetro não.*
    let paint = |ts: &mut TextSystem, sc: &mut VectorScene, t: &str| {
        paint_text_weighted(ts, sc, t, x, y, font_size, f32::INFINITY, color, weight);
    };
    if max_width <= 0.0 {
        return;
    }
    if text_system.prefix_width_weighted(text, font_size, weight) <= max_width {
        // `INFINITY`, not `max_width`: it fits, and passing the budget back would
        // let a sub-pixel measurement disagreement re-introduce the wrap.
        paint(text_system, scene, text);
        return;
    }
    let Some(elided) = elide(text_system, text, font_size, max_width, weight) else {
        return;
    };
    paint(text_system, scene, &elided);
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
    weight: FontWeight,
) -> Option<String> {
    if text_system.prefix_width_weighted(ELLIPSIS, font_size, weight) > max_width {
        return None;
    }
    let bounds: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    // Invariant: `lo` chars always fit, `hi` chars never do.
    let (mut lo, mut hi) = (0usize, bounds.len());
    while lo + 1 < hi {
        let mid = lo + (hi - lo) / 2;
        let candidate = format!("{}{ELLIPSIS}", &text[..bounds[mid]]);
        if text_system.prefix_width_weighted(&candidate, font_size, weight) <= max_width {
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
    // ⚠️⚠️ **`MEDIUM` porque é o que o CHAMADOR pinta, não porque é o default do `elide`**
    // (integração de 2026-09-04): duas linhas cruzaram-se aqui — uma deu um `weight` ao `elide`
    // (um TÍTULO cortado em `Medium` e pintado em `SemiBold` transborda ~3 %, exactamente na
    // fronteira em que o corte existe), a outra abriu esta porta para os chips do Inspector. O
    // único consumidor do `fit` é um `Button`, que pinta pelo caminho **sem peso** — o mesmo que
    // a guarda do «cabe» duas linhas acima mede (`prefix_width`), e que o `paint_text_elided`
    // mapeia para `MEDIUM`. ⛔ Medir numa espessura e pintar noutra é o defeito que aquele
    // parâmetro existe para impedir; herdá-lo sem escolher seria repeti-lo aqui.
    elide(text_system, text, font_size, max_width, FontWeight::MEDIUM)
        .unwrap_or_else(|| text.to_string())
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
        let out = elide(&mut text, long, 12.0, budget, FontWeight::MEDIUM).expect("something fits");
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
        assert_eq!(
            elide(&mut text, "Translate Y", 12.0, 0.5, FontWeight::MEDIUM),
            None
        );
    }

    /// ⭐⭐⭐ **O CORTE SEGUE O PESO EM QUE VAI SER PINTADO.**
    ///
    /// ⚠️⚠️ Nasceu de uma auditoria adversarial (2026-08-30) que achou **três mutações
    /// sobreviventes a 1 100 testes**: a `elide` a ignorar o `weight` e a medir sempre em
    /// Medium · o pintor a pintar sempre em Medium · a `paint_text_title_elided` a passar
    /// `MEDIUM`. Os três testes que existiam passavam `MEDIUM` **e usavam a medição em Medium
    /// como oráculo** — auto-consistentes, e cegos ao parâmetro novo. *Um parâmetro cujo valor
    /// novo nenhum teste exercita não tem cobertura: tem sintaxe.*
    ///
    /// A régua é a desigualdade, não um número: o SemiBold é mais largo, então para o MESMO
    /// orçamento ele nunca pode guardar MAIS texto — e em algum ponto guarda MENOS.
    #[test]
    fn the_cut_follows_the_weight_it_will_be_painted_in() {
        let mut text = TextSystem::without_system_fonts();
        let name = "Tropism Direction";
        let size = 13.0;
        let full = text.prefix_width_weighted(name, size, FontWeight::SEMI_BOLD);
        let kept = |o: &Option<String>| {
            o.as_deref()
                .map_or(0, |s| s.trim_end_matches(ELLIPSIS).chars().count())
        };
        let mut differed = 0;
        for k in 1..=120 {
            let budget = full * k as f32 / 120.0;
            let m = elide(&mut text, name, size, budget, FontWeight::MEDIUM);
            let b = elide(&mut text, name, size, budget, FontWeight::SEMI_BOLD);
            assert!(
                kept(&b) <= kept(&m),
                "o SemiBold e' MAIS largo: com o orcamento {budget} ele guardou {} contra {} \
                 do Medium",
                kept(&b),
                kept(&m)
            );
            if kept(&b) != kept(&m) {
                differed += 1;
            }
        }
        // ⚠️ **O CONTROLE, e é ele que mata a mutação**: sem esta metade, medir sempre em
        // Medium satisfaz a desigualdade acima (`<=` com igualdade em toda a parte).
        assert!(
            differed > 0,
            "o corte deu EXACTAMENTE o mesmo nos dois pesos em 120 orcamentos — o `weight` nao \
             esta' a chegar a medicao"
        );
    }

    /// ⭐⭐ **E O QUE FOI PINTADO TEM O PESO QUE FOI MEDIDO** — a outra metade, que a
    /// auditoria de 2026-08-30 também deixou sem gate (a mutação *"pinta sempre em Medium"*
    /// sobrevivia a 1 100 testes).
    ///
    /// A régua é a TINTA, lida da cena emitida. ⚠️ **E ela custou duas tentativas:** contar
    /// `n_paths` e `n_path_segments` dá **zero** nos dois (um glifo não entra na cena como
    /// caminho, entra por `draw_glyphs`), e contar os glifos dá **17 nos dois** (é a mesma
    /// string). O que separa os pesos é o **eixo normalizado da fonte VARIÁVEL** —
    /// `resources.normalized_coords` —, que é literalmente onde o peso viaja.
    /// ⛔ Não é comparar duas construções: é perguntar à saída.
    #[test]
    fn the_ink_carries_the_weight_that_was_measured() {
        let axes = |bold: bool| {
            let mut text = TextSystem::without_system_fonts();
            let mut scene = VectorScene::new();
            let name = "Tropism Direction";
            let f = if bold {
                paint_text_title_elided
            } else {
                paint_text_elided
            };
            f(
                &mut text,
                &mut scene,
                name,
                0.0,
                0.0,
                13.0,
                f32::INFINITY,
                Color::from_rgba8(255, 255, 255, 255),
            );
            scene.inner().encoding().resources.normalized_coords.clone()
        };
        assert_ne!(
            axes(true),
            axes(false),
            "as duas portas pintaram a MESMA tinta — o peso nao chega ao pintor"
        );
    }

    #[test]
    fn elide_never_cuts_a_multi_byte_char_in_half() {
        let mut text = TextSystem::without_system_fonts();
        let name = "Rotação · ângulo";
        let full = text.prefix_width(name, 12.0);
        for frac in [0.2, 0.4, 0.6, 0.8] {
            if let Some(out) = elide(&mut text, name, 12.0, full * frac, FontWeight::MEDIUM) {
                // A mid-char cut would have panicked in `elide` already; assert
                // the surviving prefix is a real prefix of the original.
                assert!(name.starts_with(out.trim_end_matches(ELLIPSIS)));
            }
        }
    }
}
