//! **Arch-gate da costura do light table** — o passe de fantasmas recebe os pins.
//!
//! O `ph2d_flip::ghosts` sabe o que fazer com um pin (gates na própria crate), e o botão
//! **Pin** sabe guardá-lo (`flip_strip::tests`). Entre os dois há um degrau que **nenhum
//! teste alcança**: a construção do `GhostSources` em `present.rs`, dentro do caminho que
//! exige janela, GPU e superfície.
//!
//! É precisamente o degrau onde a feature morre em silêncio: passar `pinned: &[]` ali
//! deixa TODOS os gates verdes — o modelo continua correto, o botão continua guardando o
//! número — e o artista fixa um quadro que nunca aparece. Então a costura é afirmada aqui,
//! sobre o fonte: quem monta as fontes de fantasma tem de perguntar ao `pinned_keys()`.

const SRC: &str = include_str!("../src/render_loop/present.rs");

#[test]
fn the_ghost_sources_are_built_from_the_pinned_keys() {
    let sources = SRC
        .find("GhostSources {")
        .expect("o `present` monta as fontes de fantasma — se o tipo mudou de nome, atualize");
    // A janela é o próprio literal de struct, delimitado pelo `}` que o fecha: não uma
    // distância em bytes (um proxy desses expira na primeira linha que alguém acrescenta).
    let end = SRC[sources..]
        .find('}')
        .map(|o| sources + o)
        .expect("o literal de struct tem de fechar");
    let block = &SRC[sources..end];
    assert!(
        block.contains("pinned:") && block.contains("pinned_keys()"),
        "o `GhostSources` do passe tem de sair do `pinned_keys()` da tira — sem isso o light \
         table guarda o quadro e nunca o mostra, com todos os gates verdes. Bloco lido:\n{block}"
    );
    assert!(
        block.contains("selected_keys()"),
        "e a outra fonte (as chaves marcadas) continua ali — este gate não pode passar num \
         mundo onde o multiframe perdeu os fantasmas dele"
    );
}
