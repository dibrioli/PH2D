//! ⭐⭐⭐ **O GLIFO DA ABA CHEGA A PIXEL — e o nome espremido NÃO.**
//!
//! ⛔⛔ **Os gates irmãos medem RECTS, e um rect não é tinta.** Eles vivem em
//! `screens/hero/slot_tabs_tests.rs` e afirmam onde o glifo fica e onde o nome começa; apagar a
//! chamada que os DESENHA deixa-os **verdes sobre um ecrã em branco**. É a lição que este repo já
//! pagou (*«faixa reservada ≠ faixa pintada»*), e é por ela que este ficheiro existe.
//!
//! # As duas metades, e as duas são novas
//!
//! 1. **O glifo chega:** ao piso, onde não há nome nenhum, a cena tem de ganhar caminhos — e um
//!    caminho ali só pode ser o ícone.
//! 2. **O nome NÃO chega quando não cabe:** ao piso a contagem de GLIFOS é **zero**. Sem esta
//!    metade, pintar o `…` que o dono reportou passaria despercebido — *a régua da metade 1 conta
//!    caminhos, e o Vello encaminha texto por `draw_glyphs`, que não entra nessa contagem.*

use ph2d_editor_core::screens::hero::slot_tabs_face;
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Theme};
use ph2d_vector::VectorScene;

const TITLE: &str = "Inspector";

/// `(caminhos, glifos)` que uma aba de largura `w` deixa numa cena vazia.
fn painted(w: f32) -> (u32, usize) {
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    slot_tabs_face::paint(
        &mut scene,
        &mut text,
        Rect::new(0.0, 0.0, w, ROW_H_PX),
        ph2d_editor_core::icons::IconId::Inspector,
        TITLE,
        ColorToken::Text1,
        Theme::Dark,
    );
    let enc = scene.inner().encoding();
    (enc.n_paths, enc.resources.glyphs.len())
}

/// ⭐⭐⭐ **Ao piso: o desenho do painel sim, o nome não.**
#[test]
fn a_squeezed_tab_paints_its_glyph_and_no_letters() {
    let (paths, glyphs) = painted(ROW_H_PX);
    assert!(
        paths > 0,
        "a aba no piso não deixou caminho nenhum na cena — o glifo não é pintado, e ali não há \
         mais nada que o artista possa ler"
    );
    assert_eq!(
        glyphs, 0,
        "a aba no piso pintou {glyphs} glifo(s): é o `…` do report de 2026-09-08 a voltar, ou \
         texto a transbordar por cima da aba vizinha"
    );
}

/// ⭐⭐ **Com espaço: as DUAS coisas chegam.**
///
/// ⚠️ **É também o controlo de vacuidade do teste acima** — ele prova que este arnês *sabe* ver
/// letras. Sem ele, um observador de glifos partido leria `0` em toda a parte e o gate do piso
/// passaria por não medir nada.
#[test]
fn a_roomy_tab_paints_the_glyph_and_the_letters() {
    let mut text = TextSystem::without_system_fonts();
    let w = slot_tabs_face::natural_w(TITLE, &mut text);
    let (paths, glyphs) = painted(w);
    assert!(paths > 0, "a aba larga não pintou o glifo");
    assert_eq!(
        glyphs,
        TITLE.chars().count(),
        "na largura que mede o próprio nome ele tem de sair INTEIRO"
    );
}
