//! **Arch-gate: a régua imprime a unidade do PROJETO** (plano 25 §9, a W6).
//!
//! O motor está gateado em `src/length_tests.rs` (a porta) e em `src/ruler_tests.rs`
//! (os traços). O que só um gate de FONTE alcança é a **fiação**: `paint_rulers`
//! recebe a régua de display por argumento, e quem a monta é o pintor do hero, que
//! exige `TextSystem` + `VectorScene` + um `HeroScreen` vivo.
//!
//! **Duas maneiras de partir a wave deixando a suíte inteira verde:**
//!
//! 1. **passar uma `LengthDisplay::default()`** em vez da do projeto — as fixtures
//!    da porta continuam a passar (elas constroem a régua à mão), e o artista que
//!    escolhe METROS no menu Settings continua a ler pixels na régua, sem nada
//!    quebrar em lado nenhum;
//! 2. **ninguém a monta** — mas isso o compilador pega, e é de propósito: o
//!    argumento é obrigatório em vez de um `Option` com default, exatamente para
//!    que esquecer não seja exprimível.
//!
//! ⚠️ A asserção afirma **de onde o valor NASCE**, nunca uma distância em bytes —
//! esta linha já teve dois arch-gates apodrecerem por medirem bytes.

const HERO_PAINT: &str = include_str!("../src/screens/hero/paint.rs");

/// A posição da 1ª ocorrência, ou pânico com a razão — o **controle positivo**:
/// um dono que se mudou vira falha alta, e não uma varredura vazia que passa.
fn at(src: &str, needle: &str) -> usize {
    src.find(needle).unwrap_or_else(|| {
        panic!(
            "`{needle}` sumiu — se foi renomeado, atualize este gate (e confira que a régua \
             ainda imprime a unidade do artista: `PH2D_BUILD_SMOKE=72`)"
        )
    })
}

/// **A régua de display sai do PROJETO, não de um default.**
#[test]
fn the_ruler_is_handed_the_projects_own_display() {
    let call = at(HERO_PAINT, "crate::ruler::paint_rulers(");
    // A janela é o bloco que monta e chama — a régua nasce imediatamente antes.
    let start = call.saturating_sub(400);
    let window = &HERO_PAINT[start..call];
    assert!(
        window.contains("LengthDisplay::of(&hero.project)"),
        "a régua de display não sai do `hero.project` — o artista escolhe a unidade no menu \
         Settings e a régua do canvas ignora a escolha. Janela:\n{window}"
    );
    assert!(
        !window.contains("LengthDisplay::default()"),
        "um default cravado aqui torna o menu Settings inerte para a régua"
    );
}
