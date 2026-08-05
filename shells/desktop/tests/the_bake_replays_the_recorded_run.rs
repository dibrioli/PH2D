//! **Arch-gate: o bake do produto é entregue a FITA do app, não uma vazia.**
//!
//! ## O defeito (W16, medido antes de uma linha ser escrita)
//!
//! `bake::bake_trajectories_with_scene` dirige os players pelo `player_input`
//! **RETIDO** — o dedo do instante do clique. Medido numa corrida gravada de 90
//! tiques que leva o personagem a `x = 8,765`:
//!
//! | o bake | o que ele grava |
//! |---|---|
//! | sem fita, dedo PARADO | X **CONSTANTE** (amplitude `0,000`) |
//! | sem fita, dedo na ESQUERDA | `x = −8,765` — **o espelho exacto da corrida** |
//! | com a fita | `x = 8,765` |
//!
//! ⚠️ A primeira linha é o defeito que **não parece um defeito**: com o canal X
//! constante, *nenhuma track horizontal é escrita* e o artista recebe uma curva
//! só de Y para um personagem que andou nove metros — lê-se como *"o botão não
//! gravou nada"*, não como *"gravou a corrida errada"*.
//!
//! ## Por que um gate de TEXTO
//!
//! O comportamento tem gates próprios do outro lado da costura
//! (`ph2d-physics-ecs/tests/bake_player.rs`, quatro deles, mutação-provados).
//! O que **nenhum** deles alcança é a fiação: a chamada de produção mora dentro
//! do laço de render, que exige `hero`/`sim`/`renderer` mais uma janela.
//!
//! ⚠️ E a forma exacta do apodrecimento tem nome: os oito call-sites de TESTE do
//! `bake_selection` passam `&mut InputTape::new()` — uma fita **vazia**, que é o
//! certo lá (aquelas cenas não têm player). Escrever a mesma coisa aqui
//! compilaria, passaria os oito, e devolveria o defeito inteiro.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// Os argumentos da chamada de PRODUÇÃO ao `bake_selection`.
fn production_call() -> &'static str {
    let at = SRC
        .find("physics_bake::bake_selection(")
        .expect("o laço de render tem de chamar o bake_selection");
    let rest = &SRC[at..];
    let end = rest.find(");").expect("a chamada fecha");
    &rest[..end]
}

/// **A fita que o bake recebe é a do APP.**
///
/// ⚠️ **Mutação medida:** trocar `&mut self.player_tape` por
/// `&mut ph2d_physics_ecs::InputTape::new()` devolve o defeito inteiro — todo
/// bake de player passa a gravar o dedo do instante do clique — e **nenhum**
/// dos quatro gates de comportamento sangra, porque eles chamam a porta do
/// crate directamente.
#[test]
fn the_production_bake_is_handed_the_apps_tape() {
    let call = production_call();
    assert!(
        call.contains("&mut self.player_tape"),
        "o bake de produção não recebe a fita do app; ele grava o dedo do \
         instante do clique. Chamada:\n{call}"
    );
}

/// **E ela não é uma fita nova** — o controle positivo do gate acima.
///
/// ⚠️ Sem esta metade, um `InputTape::new()` escrito AO LADO do
/// `self.player_tape` (um argumento a mais numa refactoração) passaria: a
/// primeira asserção só diz que o texto certo está lá, não que o errado não
/// está.
#[test]
fn the_production_bake_does_not_build_a_fresh_tape() {
    let call = production_call();
    assert!(
        !call.contains("InputTape::new()"),
        "a chamada de produção constrói uma fita VAZIA — uma corrida gravada \
         seria assada como se ninguém tivesse tocado no teclado. Chamada:\n{call}"
    );
}

/// **E o scanner acha alguma coisa** — o controle que impede o gate de ficar
/// verde por não ter lido nada.
#[test]
fn the_scanner_finds_the_call() {
    let call = production_call();
    assert!(
        call.len() > 100,
        "o scanner leu {} bytes: a chamada mudou de forma e os gates acima \
         deixaram de olhar para o produto",
        call.len()
    );
}
