//! **R9b (auditoria 10/11, defeito 2)** — a acção aberta da vista B ainda sem teclas liga a tecla física que se
//! carrega, mas nunca Ctrl/Alt sozinhos, nunca um acorde (o Z do Ctrl+Z) e nunca o F5: esses seguem para o editor
//! pelo filtro partilhado `raw_keys` (o mesmo do Try it).

use super::Mesa;
use super::lista::onze;
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_keyboard::state::KbHit;
use ph2d_editor_core::doc_surface::Key as DocKey;
use ph2d_input::{Binding, Key};

fn named(n: &str) -> Key {
    Key::named_in_script(n).expect("tecla com nome")[0]
}

fn up(m: &mut Mesa, k: Key) -> bool {
    let took = ph2d_app_input_map::surface_raw_key(&mut m.hero, k.0, false, None);
    m.frame();
    took
}

#[test]
fn defeito_2_a_accao_vazia_da_vista_b_nao_toma_ctrl_nem_f5() {
    let mut m = Mesa::aberta(View::Keyboard);
    m.hero.input_map = onze();
    m.frame();
    let reload = m.hero.input_map.id("reload").expect("reload");
    m.click(m.at(KbHit::Row(reload).id()));
    assert_eq!(m.tab().doc.kb.open, Some(reload));
    let ctrl = m.physical(named("left_ctrl"));
    let f5 = m.physical(named("f5"));
    let b = m.hero.input_map.get(reload).expect("reload").bindings.clone();
    assert!(
        !ctrl && !f5 && b.is_empty(),
        "Ctrl (do Ctrl+Z/Ctrl+S) e F5 (Play) viraram teclas do reload: {b:?}"
    );
}

#[test]
fn alt_sozinho_e_o_acorde_inteiro_passam_e_solto_o_ctrl_a_tecla_liga() {
    let mut m = Mesa::aberta(View::Keyboard);
    m.hero.input_map = onze();
    m.frame();
    let reload = m.hero.input_map.id("reload").expect("reload");
    m.click(m.at(KbHit::Row(reload).id()));
    for n in ["right_ctrl", "left_alt", "right_alt"] {
        assert!(!m.physical(named(n)), "{n} sozinho é do editor");
        assert!(!up(&mut m, named(n)));
    }
    assert!(!m.physical(named("left_ctrl")));
    assert!(
        !m.physical(Key(u32::from('S'))),
        "com o Ctrl em baixo o S é o Ctrl+S"
    );
    assert!(m.hero.input_map.get(reload).expect("reload").bindings.is_empty());
    assert!(!up(&mut m, named("left_ctrl")));
    assert!(m.physical(Key(u32::from('R'))), "solto o Ctrl, a tecla liga-se");
    assert_eq!(
        m.hero.input_map.get(reload).expect("reload").bindings,
        vec![Binding::Key(Key(u32::from('R')))]
    );
}

/// O Ctrl+Z logo depois do + Add: as duas teclas vão ao editor e nada se liga — o último passo do undo continua a
/// ser o Add (que o undo por diff desfaz, `project_input_map_audit_det_tests`).
#[test]
fn o_ctrl_z_depois_do_add_vai_ao_editor_sem_ligar_nada() {
    let mut m = Mesa::aberta(View::Keyboard);
    m.hero.input_map = ph2d_input::InputMap::with_player_defaults();
    m.frame();
    m.click(m.at(KbHit::Add.id()));
    m.type_text("reload");
    m.key(DocKey::Enter, None);
    m.frame();
    let r = m.hero.input_map.id("reload").expect("criada");
    assert_eq!(m.tab().doc.kb.open, Some(r));
    assert_eq!(m.tab().doc.focus(), None, "nenhum campo focado: a tecla física é da acção vazia");
    let antes = m.hero.input_map.clone();
    assert!(!m.physical(named("left_ctrl")), "o Ctrl do Ctrl+Z passa");
    assert!(!m.physical(Key(u32::from('Z'))), "o Z do Ctrl+Z passa");
    assert_eq!(m.hero.input_map, antes, "o Ctrl+Z não ligou Left Ctrl nem Z ao reload");
}
