//! Gates do TECLADO — a forma é a do irmão gamepad, e as bordas têm de concordar com ela.

use super::*;

const A: Key = Key(0x41);
const B: Key = Key(0x42);

/// O protocolo do quadro: `begin_frame` fotografa, os eventos aplicam-se, e daí saem as bordas.
#[test]
fn the_edges_fire_once_each() {
    let mut k = KeyboardState::new();

    k.begin_frame();
    k.handle_key_down(A);
    assert!(k.pressed(A), "a borda de descida");
    assert!(k.held(A));
    assert!(!k.released(A));

    k.begin_frame();
    assert!(!k.pressed(A), "segurar nao volta a disparar a borda");
    assert!(k.held(A), "mas continua em baixo");

    k.begin_frame();
    k.handle_key_up(A);
    assert!(k.released(A), "a borda de subida");
    assert!(!k.held(A));
}

/// Uma tecla repetida (o auto-repeat do SO) não pode duplicar a entrada nem re-armar a borda.
#[test]
fn an_os_key_repeat_never_duplicates_the_key() {
    let mut k = KeyboardState::new();
    k.begin_frame();
    k.handle_key_down(A);
    k.handle_key_down(A);
    k.handle_key_down(A);

    assert_eq!(
        k.iter_held().count(),
        1,
        "a tecla entrou tres vezes na lista"
    );

    k.begin_frame();
    assert!(!k.pressed(A), "o repeat re-armou a borda");
}

/// Soltar uma tecla que nunca desceu é inócuo — a shell pode perder um par sem partir a lista.
#[test]
fn releasing_a_key_that_was_never_down_is_harmless() {
    let mut k = KeyboardState::new();
    k.handle_key_up(A);
    assert!(!k.held(A));
    assert_eq!(k.iter_held().count(), 0);
}

/// ⛔ **`release_all` é o que a janela sem foco chama** — e é a metade que esta crate pode resolver
/// do `Up` perdido que deixaria um personagem a andar sozinho para sempre.
///
/// ⚠️ **A fixtura tem de conter o fenómeno, e a primeira versão deste gate NÃO continha:** ela
/// descia e largava as teclas dentro do MESMO quadro, onde não existe borda nenhuma para ver —
/// uma tecla que nunca esteve em baixo numa fronteira de quadro não tem subida a reportar, e o
/// gate reprovava sobre produto correcto. O cenário real é o único que interessa: o jogador
/// **segura ao longo de quadros** e só então a janela perde o foco.
#[test]
fn release_all_empties_the_keyboard() {
    let mut k = KeyboardState::new();

    // Quadro 1: as duas descem.
    k.begin_frame();
    k.handle_key_down(A);
    k.handle_key_down(B);
    assert!(k.held(A) && k.held(B));

    // Quadro 2: continuam seguradas -- e' isto que poe as duas na fotografia do quadro anterior.
    k.begin_frame();
    assert!(
        k.held(A) && k.held(B),
        "a corrida atravessa a fronteira do quadro"
    );

    // Quadro 3: a janela perde o foco a meio da corrida.
    k.begin_frame();
    k.release_all();

    assert!(!k.held(A) && !k.held(B));
    assert_eq!(k.iter_held().count(), 0);
    // ⭐ E as bordas de subida APARECEM: quem largou tudo tem de poder ver que largou, senao um
    // `just_released` desaparece em silencio e quem o esperava fica a segurar para sempre.
    assert!(
        k.released(A) && k.released(B),
        "o release_all tem de produzir a borda de subida, e nao so' esvaziar a lista"
    );
}

/// A ordem de iteração é estável — não é arrumação: ela alimenta a resolução em acções, que
/// alimenta a fita determinística da física.
#[test]
fn the_held_order_is_stable_and_sorted() {
    let mut k = KeyboardState::new();
    k.handle_key_down(B);
    k.handle_key_down(A);
    let got: Vec<Key> = k.iter_held().collect();
    assert_eq!(
        got,
        vec![A, B],
        "a lista tem de sair ordenada, sempre igual"
    );
}
