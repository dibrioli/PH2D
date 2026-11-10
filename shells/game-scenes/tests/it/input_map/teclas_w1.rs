//! **InputMap W1** (plano `docs/InputMap/01_plano.md` §13.1): o teclado de notebook pela costura real da aba —
//! Numpad 7 ≠ 7 na escuta e na busca por tecla, e uma tecla do sistema nunca liga.

use super::Mesa;
use super::lista::onze;
use ph2d_app_input_map::View;
use ph2d_app_input_map::ids;
use ph2d_app_input_map::view_keyboard::state::KbHit;
use ph2d_input::{Binding, InputMap, Key};

fn named(n: &str) -> Key {
    Key::named_in_script(n).expect("tecla com nome")[0]
}

fn bindings(m: &Mesa, name: &str) -> Vec<Binding> {
    let id = m.hero.input_map.id(name).expect("a acção existe");
    m.hero
        .input_map
        .get(id)
        .map(|a| a.bindings.clone())
        .unwrap_or_default()
}

/// A vista B com `reload` (sem teclas) aberta e à escuta.
fn a_escutar(map: InputMap) -> Mesa {
    let mut m = Mesa::aberta(View::Keyboard);
    m.hero.input_map = map;
    m.frame();
    let a = m.hero.input_map.id("reload").expect("reload");
    m.click(m.at(KbHit::Row(a).id()));
    m.click(m.at(KbHit::Listen.id()));
    assert_eq!(m.tab().doc.snap.listening, Some(a), "à escuta");
    m
}

/// (a) À escuta, o Numpad 7 liga o Numpad 7 — e o ladrilho diz «Numpad 7», não «7».
#[test]
fn listening_numpad_7_binds_numpad_7_not_7() {
    let mut m = a_escutar(onze());
    let kp7 = named("kp7");
    assert!(m.physical(kp7), "a escuta toma a tecla");
    assert_eq!(bindings(&m, "reload"), vec![Binding::Key(kp7)]);
    assert_ne!(kp7, named("7"));
    let t: Vec<String> = m
        .tab()
        .doc
        .census
        .texts
        .iter()
        .map(|t| t.1.clone())
        .collect();
    assert!(
        t.iter().any(|s| s == "Numpad 7"),
        "o ladrilho do Numpad 7: {t:?}"
    );
}

/// (b) A busca da lista A pela tecla Numpad 7 filtra quem tem o Numpad 7, e não quem tem o 7.
#[test]
fn the_list_search_by_numpad_7_skips_the_actions_on_7() {
    let mut map = onze();
    let (kp7, seven) = (named("kp7"), named("7"));
    let (com_kp7, com_7) = (
        map.id("reload").expect("reload"),
        map.id("jump").expect("jump"),
    );
    map.get_mut(com_kp7)
        .expect("reload")
        .bindings
        .push(Binding::Key(kp7));
    map.get_mut(com_7)
        .expect("jump")
        .bindings
        .push(Binding::Key(seven));
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = map;
    m.frame();
    m.click(m.at(ids::SEARCH));
    assert!(m.physical(kp7), "com a busca vazia, a tecla é a procurada");
    assert_eq!(m.tab().doc.list.search_key, Some(kp7));
    let shown = ph2d_app_input_map::view_list::shown(&m.tab().doc);
    assert!(shown.contains(&com_kp7), "quem tem o Numpad 7 aparece");
    assert!(!shown.contains(&com_7), "quem só tem o 7 não aparece");
}

/// (c) Uma tecla do sistema entregue à escuta não liga (o desktop nunca a dá, mas a aba recusa-a mesmo assim).
#[test]
fn a_system_key_while_listening_never_binds() {
    let mut m = a_escutar(onze());
    for k in Key::system_keys() {
        m.physical(k);
        assert_eq!(
            bindings(&m, "reload"),
            vec![],
            "{} entrou no mapa",
            k.label()
        );
    }
}
