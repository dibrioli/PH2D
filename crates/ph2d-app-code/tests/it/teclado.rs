//! ⭐ **E5 — com a aba de código activa, as teclas da CENA são do editor.** Pela porta por onde a shell
//! entrega TODA tecla com um documento activo (`board_keys::key`, chamada por
//! `keyboard_board.rs::board_owns_the_keyboard` ANTES de todo atalho da cena): tomada = o despacho da
//! shell pára aí, e o Espaço nunca chega ao play/pause.

use crate::{Mesa, ctrl};
use ph2d_editor_core::screens::hero::board_keys::{self, BoardKey};
use ph2d_host::Modifiers;

const TEXTO: &str = "local a = 1\n";

fn tecla(m: &mut Mesa, k: BoardKey, mods: Modifiers, text: Option<&str>) -> bool {
    let mut ts = std::mem::replace(&mut m.ts, ph2d_text::TextSystem::without_system_fonts());
    let out = board_keys::key(&mut m.hero, &mut ts, k, mods, true, text, None);
    m.ts = ts;
    out.consumed
}

#[test]
fn espaco_p_w_delete_e_ctrl_z_sao_do_editor_e_mudam_o_texto() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", TEXTO)]);
    m.pede(ids[0]);
    let mut antes = m.docs().text_of(ids[0]).expect("aberto");
    for (k, mods, text) in [
        (BoardKey::Space, Modifiers::default(), Some(" ")),
        (BoardKey::Char('p'), Modifiers::default(), Some("p")),
        (BoardKey::Char('w'), Modifiers::default(), Some("w")),
        (BoardKey::Left, Modifiers::default(), None),
        (BoardKey::Delete, Modifiers::default(), None),
        (BoardKey::Char('z'), ctrl(), Some("z")),
    ] {
        assert!(
            tecla(&mut m, k, mods, text),
            "{k:?} não foi tomada pelo editor — a cena vê-la-ia"
        );
        let agora = m.docs().text_of(ids[0]).expect("aberto");
        if k != BoardKey::Left {
            assert_ne!(agora, antes, "{k:?} foi tomada e não mudou o texto");
        }
        antes = agora;
    }
    // Espaço, p, w, ← (o cursor entre o p e o w), Delete (sai o w), Ctrl+Z (o w volta).
    assert_eq!(antes, " pwlocal a = 1\n");
}

/// Controlo: com a CENA activa a mesma porta NÃO toma o Espaço — o despacho segue para o play/pause.
#[test]
fn com_a_cena_activa_o_espaco_volta_a_ser_da_cena() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", TEXTO)]);
    m.pede(ids[0]);
    m.hero.documents.activate(None);
    assert!(m.hero.documents.scene_active());
    for (k, text) in [
        (BoardKey::Space, Some(" ")),
        (BoardKey::Char('p'), Some("p")),
        (BoardKey::Delete, None),
    ] {
        assert!(
            !tecla(&mut m, k, Modifiers::default(), text),
            "{k:?} foi tomada com a Cena activa"
        );
    }
    assert_eq!(m.docs().text_of(ids[0]).as_deref(), Some(TEXTO));
}

/// Teclas de edição: Enter indenta depois de quem abre um bloco, Tab/Shift+Tab, setas com Shift.
#[test]
fn enter_tab_e_setas() {
    use ph2d_editor_core::doc_surface::Key;
    let (mut m, ids) = Mesa::new(&[("bob.luau", "")]);
    m.pede(ids[0]);
    m.escreve(ids[0], "function f()");
    m.tecla(ids[0], Key::Enter, Modifiers::default(), None);
    m.escreve(ids[0], "x");
    assert_eq!(
        m.docs().text_of(ids[0]).as_deref(),
        Some("function f()\n  x"),
        "Enter indenta depois de quem abre um bloco"
    );
    // Shift+Home selecciona até ao 1.º não-espaço; Tab indenta a linha seleccionada; Shift+Tab recua.
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    m.tecla(ids[0], Key::Home, shift, None);
    m.tecla(ids[0], Key::Tab, Modifiers::default(), None);
    assert_eq!(
        m.docs().text_of(ids[0]).as_deref(),
        Some("function f()\n    x")
    );
    m.tecla(ids[0], Key::Tab, shift, None);
    m.tecla(ids[0], Key::Tab, shift, None);
    assert_eq!(m.docs().text_of(ids[0]).as_deref(), Some("function f()\nx"));
    // Ctrl+A e escrever troca tudo.
    m.tecla(ids[0], Key::Char('a'), ctrl(), Some("a"));
    m.escreve(ids[0], "y");
    assert_eq!(m.docs().text_of(ids[0]).as_deref(), Some("y"));
}
