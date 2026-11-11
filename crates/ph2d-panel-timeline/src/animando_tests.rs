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
