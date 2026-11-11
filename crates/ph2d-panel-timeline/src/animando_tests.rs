//! A pílula «Animating · Done» (doc 17 item 9): quem a publica e quando a sessão pára.

use super::*;

/// Fora da aba Keys não há sessão, e a paragem do Done não sobrevive à troca de aba.
/// *Mutação: `publica` sem o `keys` ⇒ a sessão anima no Arrange.*
#[test]
fn only_the_keys_tab_animates_and_leaving_it_clears_done() {
    larga();
    publica(false);
    assert!(!animating(), "o Arrange não é uma sessão de animar");
    done();
    publica(false);
    assert!(!parked(), "sair da aba Keys desfaz o Done");
}

/// O Done pára a sessão; o `resume` (um gesto na Timeline) devolve-a. Nos cartões só.
/// *Mutação: `publica` sem o `!parked()` ⇒ o Done não pára nada.*
#[test]
fn done_parks_and_resume_animates_again() {
    larga();
    let cartoes = ph2d_editor_core::panel::cards::ferramenta::cards_on();
    publica(true);
    assert_eq!(animating(), cartoes);
    done();
    publica(true);
    assert!(!animating(), "o Done pára a sessão");
    assert_eq!(parked(), cartoes);
    resume();
    publica(true);
    assert_eq!(animating(), cartoes);
    larga();
    assert!(!animating() && !parked(), "o painel escondido larga tudo");
}

/// O Done avisa a shell UMA vez (ela desliga o Auto Key); sem clique, nada.
/// *Mutação: `done` sem o `FEITO` ⇒ o Auto Key fica ligado depois do Done.*
#[test]
fn done_tells_the_shell_once_to_turn_auto_key_off() {
    let _ = take_done(|| ());
    assert!(
        !take_done(|| panic!("sem clique não se pausa")),
        "sem clique não há aviso"
    );
    done();
    let mut pausou = false;
    assert!(take_done(|| pausou = true), "o Done avisa a shell");
    assert!(pausou, "e pausa a agulha");
    assert!(!take_done(|| ()), "e só uma vez");
    larga();
}

/// O Done com a agulha a TOCAR (dono 11/11): pausada antes da `sessao`, ela não é gesto e o Done fica.
/// *Mutação: `take_done` sem chamar `para` ⇒ a agulha continua, o gesto desfaz o Done.*
#[test]
fn done_while_playing_stays_done() {
    larga();
    let cartoes = ph2d_editor_core::panel::cards::ferramenta::cards_on();
    publica(true);
    let mut tocando = true;
    assert_eq!(sessao(true, tocando).0, cartoes);
    done();
    publica(true);
    let _ = take_done(|| tocando = false);
    assert!(
        !sessao(true, tocando).0,
        "o Done pára a sessão mesmo a tocar"
    );
    assert_eq!(parked(), cartoes);
    larga();
    let _ = take_ended();
}
