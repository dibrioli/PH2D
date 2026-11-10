//! **Auditoria de fios (10/11)** — as asserções que faltavam aos claims da linha: a escuta não duplica, a do
//! Trigger não aparece na aba, os acordes e as letras passam pelo `raw_key` para o editor (A e C) e o Try it nunca
//! toma Ctrl/Alt.

use super::Mesa;
use super::lista::onze;
use ph2d_app_input_map::View;
use ph2d_app_input_map::ids::ListHit;
use ph2d_input::{Binding, Key};

fn named(n: &str) -> Key {
    Key::named_in_script(n).expect("tecla com nome")[0]
}

/// A tecla física com o texto que a shell lhe dá (o `Mesa::physical` passa sempre `None`).
fn raw(m: &mut Mesa, k: Key, text: Option<&str>) -> bool {
    let took = ph2d_app_input_map::surface_raw_key(&mut m.hero, k.0, true, text);
    m.frame();
    took
}

#[test]
fn a_escuta_da_aba_nao_duplica_uma_tecla_que_a_accao_ja_tem() {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    m.frame();
    let fire = m.hero.input_map.id("fire").expect("fire");
    let antes = m.hero.input_map.get(fire).expect("fire").bindings.clone();
    let j = Binding::Key(Key(u32::from('J')));
    assert!(antes.contains(&j), "o fire de `onze` tem J");
    m.click(m.at(ListHit::Listen(fire).id()));
    assert_eq!(m.hero.store.input_map_listening(), Some(fire));
    assert!(m.physical(Key(u32::from('J'))), "a escuta toma a tecla");
    assert_eq!(
        m.hero.input_map.get(fire).expect("fire").bindings,
        antes,
        "a mesma ligação duas vezes seriam duas linhas iguais"
    );
    assert!(m.hero.store.input_map_listening().is_none(), "desarmou");
}

#[test]
fn a_escuta_do_trigger_nao_aparece_na_aba_nem_e_desarmada_por_ela() {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    m.frame();
    let fire = m.hero.input_map.id("fire").expect("fire");
    m.hero.store.listen_for_binding_replacing(fire);
    m.frame();
    assert_eq!(
        m.tab().doc.snap.listening,
        None,
        "a escuta que TROCA é do cartão do Trigger, não desta aba"
    );
    assert!(
        m.tab().doc.list.listen_request.is_none(),
        "o quadro da aba não pede nada à escuta"
    );
    assert_eq!(
        m.hero.store.input_map_listening(),
        Some(fire),
        "o quadro da aba não a desarma"
    );
    assert!(m.hero.store.input_map_listen_replaces());
}

#[test]
fn com_a_busca_focada_a_letra_de_um_acorde_e_o_f5_sem_foco_vao_ao_editor() {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    m.frame();
    // Sem campo focado, a vista A não toma tecla nenhuma: o F5 e o Ctrl+Z são do editor.
    assert!(
        !raw(&mut m, named("f5"), None),
        "F5 sem foco é do transporte"
    );
    assert!(!raw(&mut m, named("left_ctrl"), None));
    // Com a busca focada, a letra que escreve (o Z do Ctrl+Z traz «z» no texto) não vira filtro.
    let search = m.at(ph2d_app_input_map::ids::SEARCH);
    m.click(search);
    assert!(
        !raw(&mut m, Key(u32::from('Z')), Some("z")),
        "a tecla que escreve segue para o editor (o Ctrl+Z desfaz)"
    );
    assert_eq!(m.tab().doc.list.search_key, None);
}

#[test]
fn o_try_it_deixa_passar_ctrl_alt_e_o_acorde_inteiro() {
    let mut m = Mesa::aberta(View::TryIt);
    m.hero.input_map = onze();
    m.frame();
    assert!(
        !raw(&mut m, named("left_ctrl"), None),
        "o Ctrl não é do bench"
    );
    assert!(
        !raw(&mut m, Key(u32::from('Z')), None),
        "com o Ctrl em baixo o Z é acorde: vai ao editor"
    );
    let up = ph2d_app_input_map::surface_raw_key(&mut m.hero, named("left_ctrl").0, false, None);
    m.frame();
    assert!(!up);
    assert!(
        raw(&mut m, named("up"), None),
        "solto o Ctrl, a seta volta a ser do bench"
    );
}
