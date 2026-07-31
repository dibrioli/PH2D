//! **Arch-gate: o preview ao vivo do traço e o bake passam pela MESMA função.**
//!
//! É a cerca de 2026-07-11 (Enio: *"o desenho em tempo real está mais suave que o traço
//! cosido após mouse up"*), e ela precisa ser afirmada **sobre o código**, não sobre um
//! valor.
//!
//! Até 2026-07-18 o invariante era mantido por CALIBRAÇÃO: o preview repetia só o
//! `active_smooth`, o bake acrescentava um RDP, e os dois coincidiam porque esse RDP
//! estava ajustado para não fazer nada (tolerância de 0,05 px). Isso quebra em silêncio na
//! primeira vez que alguém mexe na tolerância — e o motivo para mexer existia (o Enio
//! pediu menos vértices no MESMO smoke que criou a cerca).
//!
//! Um teste de unidade não alcança isto: o preview vive num método do `App` que exige GPU
//! e janela. O que se pode afirmar sem o app vivo é que **os dois sítios chamam a mesma
//! porta** — e é exatamente o que quebraria.
//!
//! ⚠️ **2026-07-31 — a porta ganhou um andar, e o gate ganhou a metade que faltava.** O preview
//! passou a levar o cache do ajuste ([`FitCache`]), então ele chama `stroke_from_samples_cached`
//! enquanto o bake chama `stroke_from_samples`. **Nomes diferentes não são portas diferentes** — o
//! segundo DELEGA ao primeiro com um cache recém-nascido (vazio ⇒ percurso completo), e é essa
//! delegação que mantém a cerca valendo por CONSTRUÇÃO. Por isso o gate agora afirma as duas
//! metades: *o preview chega à porta de baixo* **e** *a de cima não tem pipeline próprio*. A
//! primeira sozinha ficaria verde sobre um bake que reimplementasse o traço.

use std::fs;

/// O corpo de uma função, da assinatura até o `}` que fecha na coluna 0.
fn corpo<'a>(src: &'a str, assinatura: &str) -> &'a str {
    let ini = src
        .find(assinatura)
        .unwrap_or_else(|| panic!("`{assinatura}` sumiu — reconfirme a cerca antes de renomear"));
    let resto = &src[ini..];
    &resto[..resto.find("\n}").map_or(resto.len(), |i| i)]
}

#[test]
fn the_flip_preview_bakes_through_the_same_door() {
    let src = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/flip_draw.rs"))
        .expect("flip_draw.rs");

    let preview = corpo(&src, "fn flip_preview_data");
    assert!(
        preview.contains("stroke_from_samples_cached("),
        "o preview do traço parou de passar pela porta única. \
         Se ele voltar a montar o traço por conta própria, o que o artista vê durante o \
         arrasto deixa de ser o que fica no pen-up — e o defeito só aparece quando alguém \
         mexe na tolerância da simplificação, longe daqui."
    );
    assert!(
        !preview.contains("build_stroke("),
        "o preview voltou a chamar `build_stroke` direto, pulando a simplificação que o \
         bake aplica — é exatamente a divergência que esta cerca existe para impedir."
    );

    // A metade nova: a porta de cima é uma DELEGAÇÃO, não um segundo pipeline.
    let bake = corpo(&src, "pub(crate) fn stroke_from_samples(");
    assert!(
        bake.contains("stroke_from_samples_cached("),
        "`stroke_from_samples` deixou de delegar à porta de baixo. Com dois corpos, o traço \
         do bake e o do preview passam a ser duas respostas para a mesma pergunta — e elas \
         divergem no dia em que alguém mexer só num deles."
    );
    for proprio in ["active_smooth(", "resample_smooth(", "build_stroke("] {
        assert!(
            !bake.contains(proprio),
            "`stroke_from_samples` voltou a montar o traço por conta própria (`{proprio}`) \
             em vez de delegar — é o segundo pipeline que esta cerca existe para impedir."
        );
    }
}
