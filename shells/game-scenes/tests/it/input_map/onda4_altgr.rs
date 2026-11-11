//! Onda 4, item 3: AltGr não é o Alt do editor. A plataforma não ergue a bandeira Alt com o AltGr (medido no
//! winit 0.30.13: Windows `get_agnostic_mods`, xkb `<RALT>` = `ISO_Level3_Shift` no `Mod5`); a escuta lê a bandeira,
//! e a lei das teclas físicas é `ph2d_input::editor_keys::chord_keys`.

use super::Mesa;
use super::lista::onze;
use ph2d_app_input_map::View;
use ph2d_input::editor_keys::{chord_keys, never_binds};
use ph2d_input::{Binding, Key};

fn tecla(n: &str) -> Key {
    Key::named_in_script(n).expect("tecla com nome")[0]
}

/// Arma a escuta de `fire`, carrega `modificador` (com a bandeira Alt da plataforma em `alt`) e depois `k`.
fn acorde(modificador: &str, alt: bool, k: Key) -> bool {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    m.frame();
    let fire = m.hero.input_map.id("fire").expect("fire");
    m.hero.store.listen_for_binding(fire);
    m.hero.store.set_alt_held(alt);
    assert!(
        !m.physical(tecla(modificador)),
        "{modificador} sozinho não liga"
    );
    m.physical(k);
    let ligou = m
        .hero
        .input_map
        .get(fire)
        .is_some_and(|a| a.bindings.contains(&Binding::Key(k)));
    assert_eq!(
        m.hero.store.input_map_listening().is_none(),
        ligou,
        "liga e desarma, ou segue armada"
    );
    ligou
}

#[test]
fn altgr_liga_a_tecla_fisica_e_o_alt_esquerdo_nao() {
    let (f, k) = (Key('F' as u32), Key('K' as u32));
    // (modificador, bandeira Alt da plataforma, tecla, liga?, caso)
    let casos = [
        (
            "left_alt",
            true,
            f,
            false,
            "Alt esquerdo + F (Alt+F do editor de scripts)",
        ),
        ("right_alt", false, f, true, "AltGr + F: liga o F"),
        (
            "left_alt",
            true,
            k,
            false,
            "Alt esquerdo + K: o acorde segue recusado",
        ),
        (
            "right_alt",
            true,
            f,
            false,
            "controlo: Alt direito que É Alt (us) é recusado",
        ),
    ];
    for (modificador, alt, key, liga, caso) in casos {
        assert_eq!(acorde(modificador, alt, key), liga, "{caso}");
    }
}

/// O controlo da lei física: sem o filtro o Alt direito em baixo é acorde; com ele, só quando a plataforma diz Alt.
#[test]
fn as_teclas_fisicas_de_acorde_tiram_o_altgr() {
    let f = Key('F' as u32);
    let ra = tecla("right_alt");
    assert!(never_binds(f, [ra]), "sem o filtro: recusado");
    assert!(!never_binds(f, chord_keys([ra], false)), "AltGr: liga");
    assert!(
        never_binds(f, chord_keys([ra], true)),
        "Alt direito que é Alt: recusado"
    );
    assert!(
        never_binds(f, chord_keys([tecla("left_alt")], false)),
        "o esquerdo é sempre Alt"
    );
}

/// A ordem da shell (winit 0.30, Wayland/xkb): o `KeyboardInput` do modificador chega com a bandeira ainda em baixo, o
/// `ModifiersChanged` ergue-a (ou não, no AltGr) e só então o `KeyboardInput` da letra, com o seu texto.
fn pela_shell(m: &mut Mesa, modificador: &str, alt: bool, k: Key, down: bool) -> bool {
    let shell = |m: &mut Mesa, key: Key, text: Option<&str>| {
        let took = ph2d_app_input_map::surface_raw_key(&mut m.hero, key.0, down, text);
        m.frame();
        took
    };
    let (first, last) = if down {
        (tecla(modificador), k)
    } else {
        (k, tecla(modificador))
    };
    let a = shell(m, first, None);
    m.hero.store.set_alt_held(alt && down);
    a | shell(m, last, down.then_some("f"))
}

/// ⛔ Defeito do dono (11/11): na Try it o Alt esquerdo + F acendia o F como o AltGr + F — o banco guardava a tecla
/// ANTES da lei, e a lei só decidia o «tomada». O F aceso é o que o desenho pinta (`held_keys`). Controlo: AltGr + F
/// (sem a bandeira) acende e é tomado; o F sozinho também.
#[test]
fn na_try_it_o_alt_esquerdo_mais_f_nao_acende_e_o_altgr_mais_f_acende() {
    let f = Key('F' as u32);
    // (modificador, bandeira Alt da plataforma, tomada e acesa?, caso)
    let casos = [
        ("left_alt", true, false, "Alt esquerdo + F: do editor"),
        ("right_alt", false, true, "AltGr + F: do banco"),
        ("right_alt", true, false, "Alt direito que É Alt (us)"),
    ];
    for (modificador, alt, banco, caso) in casos {
        let mut m = Mesa::aberta(View::TryIt);
        assert_eq!(
            pela_shell(&mut m, modificador, alt, f, true),
            banco,
            "{caso}: tomada"
        );
        let held = m.tab().doc.tryit.held_keys();
        assert_eq!(held.contains(&f), banco, "{caso}: o F aceso {held:?}");
        let _ = pela_shell(&mut m, modificador, alt, f, false);
        assert!(
            m.tab().doc.tryit.held_keys().is_empty(),
            "{caso}: soltar apaga tudo"
        );
    }
    let mut m = Mesa::aberta(View::TryIt);
    assert!(ph2d_app_input_map::surface_raw_key(
        &mut m.hero,
        f.0,
        true,
        Some("f")
    ));
    assert_eq!(
        m.tab().doc.tryit.held_keys(),
        vec![f],
        "controlo: o F sozinho acende"
    );
}

/// A mesma ordem na vista Keyboard: a acção vazia liga o F do AltGr + F e nunca o do Alt esquerdo + F.
#[test]
fn no_keyboard_o_alt_esquerdo_mais_f_nao_liga_e_o_altgr_mais_f_liga() {
    use ph2d_app_input_map::view_keyboard::state::KbHit;
    let f = Key('F' as u32);
    for (modificador, alt, liga) in [("left_alt", true, false), ("right_alt", false, true)] {
        let mut m = Mesa::aberta(View::Keyboard);
        m.hero.input_map = onze();
        m.frame();
        let reload = m.hero.input_map.id("reload").expect("reload");
        m.click(m.at(KbHit::Row(reload).id()));
        assert_eq!(
            pela_shell(&mut m, modificador, alt, f, true),
            liga,
            "{modificador}"
        );
        let b = &m.hero.input_map.get(reload).expect("reload").bindings;
        assert_eq!(b.contains(&Binding::Key(f)), liga, "{modificador}: {b:?}");
    }
}
