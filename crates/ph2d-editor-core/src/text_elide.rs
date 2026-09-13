//! **A LEI da reticência** — o texto de uma linha que CABE: ele próprio, ou `<prefixo>…`. É a
//! contraparte de [`crate::paint::paint_text`], cujo `max_width` é orçamento de QUEBRA.
//!
//! A label one pixel too wide for its column silently becomes two lines and
//! spills into the row below (the timeline track names did exactly that). Rows,
//! list items and any other fixed-height slot need text that truncates instead.
//!
//! ⚠️ **Os PINTORES moram no `paint`** ([`crate::paint::paint_text_elided`],
//! [`crate::paint::paint_text_title_elided`]), com os outros pintores de texto (auditoria A10,
//! 2026-09-12). Com eles aqui, este módulo pedia o `paint_text_weighted` ao `paint` e o `paint` pedia o
//! [`fit`] a este — um ciclo entre dois módulos da fundação. A lei fica em baixo; quem pinta, em cima.

use ph2d_text::{FontWeight, TextSystem};

/// The ellipsis appended to text that does not fit. Inside Inter's coverage
/// (U+2026 is not one of the arrow / technical blocks the tofu gate rejects).
const ELLIPSIS: &str = "\u{2026}";

/// ⛔⛔ **A LARGURA QUE [`crate::paint::paint_text_title_elided`] DE FACTO OCUPA** — medida no MESMO peso em
/// que ela pinta.
///
/// ⚠️ **Existe porque a divergência já shipou.** Report do Enio, 2026-09-05: os números dentro
/// dos cartões do Motion apareciam como `0....`, `Re...`, e mais afastado sumiam de todo. Quem
/// os desenhava media com `TextSystem::prefix_width` (peso NORMAL) e pintava com
/// `paint_text_title_elided` (**`SEMI_BOLD`**, mais largo): o texto não cabia na largura que
/// ele próprio tinha medido, e o elidor cortava-o. *Uma medição num peso e uma pintura noutro
/// são duas perguntas diferentes*, e a diferença cresce quanto menor é a fonte.
///
/// ⚠️ **A folga de meio pixel é parte da resposta**, não um remendo: a elisão decide-se por
/// comparação de floats, e um texto que mede exactamente a largura disponível fica na fronteira
/// do arredondamento.
///
/// ⇒ quem alinha à direita, reserva coluna ou decide se cabe **chama isto**, nunca o
/// `prefix_width` cru. Os dois vivem no mesmo ficheiro de propósito: o peso é um só.
pub fn title_elided_width(text_system: &mut TextSystem, text: &str, font_size: f32) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    text_system.prefix_width_weighted(text, font_size, FontWeight::SEMI_BOLD) + 0.5
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
/// guarda que o [`crate::paint::paint_text_elided`] faz antes de a chamar. *Um helper privado carrega a
/// pré-condição do único chamador que ele tinha; publicá-lo sem a publicar é publicar meia função.*
///
/// ⇒ quem precisa da STRING chama o [`fit`], que faz a guarda.
///
/// ⚠️ **`pub(crate)` só para os pintores de [`crate::paint`]** (`paint_text.rs`), que fazem a guarda
/// do «cabe» ANTES de a chamar — a mesma pré-condição do único chamador que ela sempre teve.
#[must_use]
pub(crate) fn elide(
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
/// É o [`crate::paint::paint_text_elided`] sem o pincel: quem faz a própria centragem (um `Button`) precisa da
/// string, não do desenho. ⚠️ **A guarda do «cabe» é a razão de esta porta existir** — ver o doc do
/// [`elide`], que sem ela corta sempre.
///
/// ⛔ Quando nem as reticências cabem devolve o texto **cru**: um rótulo cortado a zero é um botão
/// mudo, e é melhor transbordar visivelmente do que desaparecer.
#[must_use]
pub fn fit(text_system: &mut TextSystem, text: &str, font_size: f32, max_width: f32) -> String {
    fit_weighted(text_system, text, font_size, max_width, FontWeight::MEDIUM)
}

/// ⭐⭐⭐ **O mesmo, medindo na ESPESSURA em que o chamador vai pintar.**
///
/// Enio, 2026-09-06, com duas fotos do painel a estreitar: *«quando a palavra é grande e
/// estreitamos o painel, em vez dos três pontos (…) como no Blender, a palavra passa para baixo e
/// some»*. — `Surface Smooth` ficava `Surface`, e o resto caía fora da caixa.
///
/// ⚠️ **O peso viaja porque medir numa espessura e pintar noutra é um defeito que este ficheiro já
/// pagou** (ver o doc do [`fit`]): um rótulo cortado em `Medium` e pintado em `SemiBold`
/// transborda ~3 %, exactamente na fronteira em que o corte existe.
#[must_use]
pub fn fit_weighted(
    text_system: &mut TextSystem,
    text: &str,
    font_size: f32,
    max_width: f32,
    weight: FontWeight,
) -> String {
    if text_system.prefix_width_weighted(text, font_size, weight) <= max_width {
        return text.to_string();
    }
    elide(text_system, text, font_size, max_width, weight).unwrap_or_else(|| text.to_string())
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

#[cfg(test)]
mod title_width_tests {
    use super::*;

    /// ⭐⭐ **A LARGURA QUE A PORTA DÁ CHEGA PARA O PINTOR NÃO CORTAR** — e a condição é a
    /// **mesma linha** que o `paint_elided_weighted` avalia
    /// (`prefix_width_weighted(text, size, weight) <= max_width`), não uma reimplementação dela.
    ///
    /// FALSIFICADO por [`title_elided_width`] voltar a medir no peso normal: o texto passa a
    /// não caber na largura que ele próprio mediu, e o pintor elide.
    #[test]
    fn the_width_the_door_gives_is_enough_for_the_painter_not_to_cut() {
        let mut text = TextSystem::without_system_fonts();
        for size in [9.0_f32, 11.0, 13.0, 22.0] {
            for t in ["0.50", "1250", "Rect", "Circle", "-360", "12.34", "R"] {
                let dado = title_elided_width(&mut text, t, size);
                let preciso = text.prefix_width_weighted(t, size, FontWeight::SEMI_BOLD);
                assert!(
                    preciso <= dado,
                    "`{t}` a {size}px: a porta deu {dado} e o pintor precisa de {preciso}"
                );
            }
        }
    }

    /// ⛔⛔ **E A METADE QUE PROVA QUE O DEFEITO ERA REAL** — o peso NORMAL não chega.
    ///
    /// Report do Enio, 2026-09-05: os números dentro dos cartões do Motion liam-se `0....`,
    /// `Re...`, e ao afastar desapareciam. Sem esta metade, o gate de cima ficaria verde sobre
    /// uma porta que voltasse ao `prefix_width` cru em algum tamanho — *um gate sem
    /// contra-exemplo mede disponibilidade, não correcção*.
    #[test]
    fn measuring_at_the_wrong_weight_really_does_cut() {
        let mut text = TextSystem::without_system_fonts();
        let mut apanhados = 0usize;
        for size in [9.0_f32, 11.0, 13.0, 22.0] {
            for t in ["0.50", "1250", "Rect", "Circle", "-360", "12.34"] {
                let errado = text.prefix_width(t, size);
                let preciso = text.prefix_width_weighted(t, size, FontWeight::SEMI_BOLD);
                if preciso > errado {
                    apanhados += 1;
                }
            }
        }
        assert!(
            apanhados > 0,
            "o semi-negrito tem de ser mais largo que o normal em ALGUM caso — senao este \
             modulo esta' a defender-se de um defeito que nao existe"
        );
    }
}
