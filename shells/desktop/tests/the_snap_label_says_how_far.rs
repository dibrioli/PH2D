//! **Arch-gate da ficha de distância do smart guide** (plano 25 §9, o último item da W6).
//!
//! O motor está gateado nas duas crates que o compõem — `ph2d-vec-render`
//! (`snap_labels`: quais guias, onde) e `ph2d-editor-core` (`LengthDisplay`: que
//! número, com que casas). O que só um gate de FONTE alcança é a costura da shell:
//! ela exige `TextSystem` + a `VectorScene` do frame + um `HeroScreen` vivo, e
//! nenhum teste de unidade pinta um frame.
//!
//! **Três maneiras de partir a wave deixando a suíte inteira verde:**
//!
//! 1. **ninguém desenha** — as guias continuam a aparecer, o motor continua a
//!    computar as fichas, e o artista simplesmente nunca vê um número. É a forma
//!    mais silenciosa de a feature não existir;
//! 2. **desenha ANTES do traço da guia** — a `VectorScene` tem de estar livre para
//!    o renderizador de texto (a mesma lei do overlay de dimensões do Line e do
//!    readout de joint), e o tracejado passaria por cima da ficha;
//! 3. **a unidade é um default cravado** em vez da do projeto — o artista escolhe
//!    METROS no menu Settings, a régua obedece e a ficha continua a dizer `px`:
//!    duas superfícies do mesmo frame discordando sobre a mesma distância, que é
//!    exatamente o defeito que esta wave existe para fechar.
//!
//! ⚠️ As asserções afirmam uma RELAÇÃO ou um CONTEÚDO dentro de uma janela
//! sintática, nunca uma distância em bytes.

const RENDER: &str = include_str!("../src/render_loop/mod.rs");

/// A posição da 1ª ocorrência de `needle`, ou pânico com a razão — o **controle
/// positivo**: um dono que se mudou vira falha alta, e não varredura vazia.
fn at(src: &str, needle: &str) -> usize {
    src.find(needle).unwrap_or_else(|| {
        panic!(
            "`{needle}` sumiu — se foi renomeado, atualize este gate (e confira que o número \
             ainda chega ao artista: `PH2D_BUILD_SMOKE=72`)"
        )
    })
}

/// **A ficha é desenhada, e DEPOIS do traço da guia.**
#[test]
fn the_number_is_drawn_after_the_line_it_measures() {
    let line = at(RENDER, "ph2d_vec_render::draw_snap_guides(");
    let label = at(RENDER, "vec_snap_labels::draw(");
    assert!(
        label > line,
        "a ficha é pintada ANTES do traço da guia — a cena tem de estar livre para o \
         renderizador de texto, e o tracejado passaria por cima do número"
    );
}

/// **A unidade sai do PROJETO** — a mesma porta que a régua usa.
#[test]
fn the_number_wears_the_unit_the_artist_chose() {
    let label = at(RENDER, "vec_snap_labels::draw(");
    // A janela é a chamada: os argumentos nascem aqui.
    let window = &RENDER[label..(label + 600).min(RENDER.len())];
    assert!(
        window.contains("LengthDisplay::of(&hero.project)"),
        "a ficha não lê a unidade do projeto — o artista troca para metros no menu Settings, \
         a régua obedece e a ficha continua a dizer px. Janela:\n{window}"
    );
    assert!(
        !window.contains("LengthDisplay::default()"),
        "um default cravado aqui torna o menu Settings inerte para a ficha"
    );
}

/// **O zoom sai do MESMO afim que desenhou o segmento.**
///
/// A precisão que a ficha mostra tem de ser a que aquele desenho de fato resolve;
/// uma segunda estimativa de zoom divergiria da linha que o artista está a olhar.
#[test]
fn the_precision_comes_from_the_camera_that_drew_the_segment() {
    let label = at(RENDER, "vec_snap_labels::draw(");
    let start = label.saturating_sub(400);
    let window = &RENDER[start..label];
    assert!(
        window.contains("cam_affine.as_coeffs()"),
        "o zoom da ficha não sai do afim que desenhou a guia. Janela:\n{window}"
    );
}
