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

/// O Done avisa a shell UMA vez (ela desliga o Auto Key), pára a agulha e leva-a ao início (dono
/// 12/11); sem clique, a agulha fica onde está.
/// *Mutação: `done` sem o `FEITO` ⇒ o Auto Key fica ligado; `take_done` sem o `rewind` ⇒ a agulha fica.*
#[test]
fn done_tells_the_shell_once_to_turn_auto_key_off() {
    let mut agulha = ph2d_core::Playhead::default();
    let _ = take_done(&mut agulha);
    agulha.seek(1.5);
    agulha.play();
    assert!(!take_done(&mut agulha), "sem clique não há aviso");
    assert!(
        agulha.is_playing() && agulha.time() == 1.5,
        "sem clique a agulha segue"
    );
    done();
    assert!(take_done(&mut agulha), "o Done avisa a shell");
    assert!(!agulha.is_playing(), "e pausa a agulha");
    assert_eq!(agulha.time(), 0.0, "e leva-a ao início");
    assert!(!take_done(&mut agulha), "e só uma vez");
    larga();
}

/// O Done com a agulha a TOCAR (dono 11/11): pausada antes da `sessao`, ela não é gesto e o Done fica.
/// *Mutação: `take_done` sem o `pause` ⇒ a agulha continua, o gesto desfaz o Done.*
#[test]
fn done_while_playing_stays_done() {
    larga();
    let cartoes = ph2d_editor_core::panel::cards::ferramenta::cards_on();
    publica(true);
    let mut agulha = ph2d_core::Playhead::default();
    agulha.play();
    assert_eq!(sessao(true, agulha.is_playing()).0, cartoes);
    done();
    publica(true);
    let _ = take_done(&mut agulha);
    assert!(
        !sessao(true, agulha.is_playing()).0,
        "o Done pára a sessão mesmo a tocar"
    );
    assert_eq!(parked(), cartoes);
    larga();
    let _ = take_ended();
}
