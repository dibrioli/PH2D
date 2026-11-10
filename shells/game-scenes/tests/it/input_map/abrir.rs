//! O menu abre a aba; o seletor troca a vista e a vista lembrada volta no arranque seguinte.

use super::Mesa;
use ph2d_app_input_map::View;
use ph2d_editor_core::doc_surface::DocumentSurface;

#[test]
fn o_menu_abre_a_aba_activa_e_reabrir_nao_duplica() {
    let mut m = Mesa::new(View::default());
    assert!(!m.tab().is_open(), "antes do menu não há aba");
    m.menu();
    assert!(m.active(), "o menu abre a aba Input Map e põe-na à frente");
    assert_eq!(m.tab().tabs().len(), 1);
    // A Cena à frente e o menu outra vez: a MESMA aba volta à frente.
    m.hero.documents.activate(None);
    assert!(m.hero.documents.scene_active());
    m.menu();
    assert!(m.active());
    assert_eq!(m.tab().tabs().len(), 1, "uma aba só");
}

#[test]
fn cada_segmento_troca_a_vista_e_a_pref_que_a_shell_grava() {
    let mut m = Mesa::aberta(View::default());
    for (v, key) in [
        (View::List, "list"),
        (View::TryIt, "try"),
        (View::Keyboard, "keyboard"),
    ] {
        let at = m.segment(v);
        assert!(m.click(at), "o clique no segmento é da aba");
        assert_eq!(m.tab().doc.view(), v);
        assert_eq!(m.pref(), key);
    }
}

#[test]
fn reinstalar_com_a_pref_abre_nessa_vista_e_sem_pref_no_teclado() {
    assert_eq!(Mesa::aberta(View::List).tab().doc.view(), View::List);
    assert_eq!(Mesa::aberta(View::TryIt).tab().doc.view(), View::TryIt);
    let m = Mesa::aberta(View::default());
    assert_eq!(m.tab().doc.view(), View::Keyboard, "a 1.ª vez é o Keyboard");
    assert_eq!(m.pref(), "keyboard");
}

#[test]
fn a_aba_e_do_projecto_no_undo_e_o_raw_key_de_outra_familia_nao_toma() {
    let mut m = Mesa::aberta(View::default());
    assert!(m.tab().undo_is_the_projects());
    // O `raw_key` de omissão (as outras famílias) devolve `false`; esta, por ora, também.
    let (f, id) = m.hero.documents.active_surface_mut().expect("a aba activa");
    assert!(!f.raw_key(id, ph2d_input::Key(0x41), true, true));
}
