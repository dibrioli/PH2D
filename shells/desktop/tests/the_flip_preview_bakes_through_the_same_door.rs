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

use std::fs;

#[test]
fn the_flip_preview_bakes_through_the_same_door() {
    let src = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/flip_draw.rs"))
        .expect("flip_draw.rs");

    // O corpo do preview: da assinatura até o fim da função.
    let start = src
        .find("fn flip_preview_data")
        .expect("o preview mudou de nome — reconfirme a cerca antes de renomear o gate");
    let body = &src[start..start + 2000.min(src.len() - start)];
    let body = &body[..body.find("\n    }").map_or(body.len(), |i| i)];

    assert!(
        body.contains("stroke_from_samples("),
        "o preview do traço parou de passar pela porta única (`stroke_from_samples`). \
         Se ele voltar a montar o traço por conta própria, o que o artista vê durante o \
         arrasto deixa de ser o que fica no pen-up — e o defeito só aparece quando alguém \
         mexe na tolerância da simplificação, longe daqui."
    );
    assert!(
        !body.contains("build_stroke("),
        "o preview voltou a chamar `build_stroke` direto, pulando a simplificação que o \
         bake aplica — é exatamente a divergência que esta cerca existe para impedir."
    );
}
