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
