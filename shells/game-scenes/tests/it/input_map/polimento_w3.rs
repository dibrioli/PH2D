//! W3–W5 (onda 2): o inventário controlo × estado — hover, premido, desactivado, foco, cursor, movimento e
//! acessibilidade; cada célula é um teste que dirige o evento real e afirma a pintura. Plano: §13.3.

use ph2d_app_input_map::interact::{Control, ControlRole};
use ph2d_app_input_map::{TAB_ID, View, ids};
use ph2d_editor_core::doc_surface::{DocumentSurface, PointerLook};

use super::Mesa;

/// ⭐ O BUG (INFRA-1): o seletor pinta pelo `CardCtx`, que lê `hover_live(id) > 0`; a aba nunca o alimentava e um
/// id nunca visto lê `SETTLED = 1.0` — os segmentos não escolhidos pintavam-se SEMPRE em hover. Agora: frio sem
/// o ponteiro, aceso NO QUADRO em que ele entra e apagado no quadro em que sai (D32, sem fade).
#[test]
fn o_seletor_so_acende_sob_o_ponteiro_e_no_quadro() {
    let mut m = Mesa::aberta(View::Keyboard);
    for v in View::ALL {
        assert_eq!(m.tab().doc.hover_live(v.id()), 0.0, "{v:?} sem o ponteiro");
    }
    let list = m.segment(View::List);
    m.hover(list);
    assert_eq!(
        m.tab().doc.hover_live(ids::VIEW_LIST),
        1.0,
        "acende no quadro"
    );
    assert_eq!(m.tab().doc.hover_live(ids::VIEW_TRY), 0.0);
    m.hover((list.0, list.1 + 300.0)); // LITERAL-PX-OK: para baixo do seletor
    assert_eq!(
        m.tab().doc.hover_live(ids::VIEW_LIST),
        0.0,
        "apaga no quadro"
    );
}

/// O premido nativo: o Down captura e não troca a vista; arrastar para fora apaga o premido; o Up dentro dispara;
/// o Up fora cancela.
#[test]
fn o_segmento_dispara_no_up_dentro_e_cancela_fora() {
    let mut m = Mesa::aberta(View::Keyboard);
    let list = m.segment(View::List);
    let fora = (list.0, list.1 + 300.0); // LITERAL-PX-OK: para baixo do seletor
    assert!(m.down(list), "o Down é tomado");
    assert_eq!(m.tab().doc.view(), View::Keyboard, "o Down não dispara");
    assert!(m.tab().doc.is_pressed(ids::VIEW_LIST));
    m.hover(fora);
    assert!(
        !m.tab().doc.is_pressed(ids::VIEW_LIST),
        "fora apaga o premido"
    );
    m.up(fora);
    assert_eq!(m.tab().doc.view(), View::Keyboard, "o Up fora cancela");
    m.down(list);
    m.hover(fora);
    m.hover(list);
    assert!(
        m.tab().doc.is_pressed(ids::VIEW_LIST),
        "de volta, premido outra vez"
    );
    m.up(list);
    assert_eq!(m.tab().doc.view(), View::List, "o Up dentro dispara");
    assert!(m.click(m.segment(View::TryIt)));
    assert_eq!(m.tab().doc.view(), View::TryIt, "o clique inteiro troca");
}

/// O foco: Tab / Shift+Tab percorrem os segmentos pela ordem da pintura, Enter activa, Esc larga.
#[test]
fn tab_percorre_o_seletor_enter_activa_e_esc_larga() {
    let mut m = Mesa::aberta(View::Keyboard);
    let k = ph2d_editor_core::doc_surface::Key::Tab;
    assert!(m.key(k, None), "o Tab é tomado");
    assert_eq!(
        m.tab().doc.focused(),
        Some(ids::VIEW_LIST),
        "o primeiro da pintura"
    );
    m.key(k, None);
    assert_eq!(m.tab().doc.focused(), Some(ids::VIEW_KEYBOARD));
    m.physical(ph2d_input::Key(0x10)); // Shift
    m.physical(ph2d_input::Key(0x09)); // Tab
    assert_eq!(
        m.tab().doc.focused(),
        Some(ids::VIEW_LIST),
        "Shift+Tab volta"
    );
    m.key(ph2d_editor_core::doc_surface::Key::Enter, None);
    assert_eq!(m.tab().doc.view(), View::List, "Enter activa o focado");
    assert!(m.key(ph2d_editor_core::doc_surface::Key::Escape, None));
    assert_eq!(m.tab().doc.focused(), None, "Esc larga");
}

/// A acessibilidade e o cursor: um nó `Tab` por segmento (o da vista escolhida, seleccionado) e a mão sobre eles.
#[test]
fn o_seletor_tem_nos_de_tab_e_a_mao() {
    let m = Mesa::aberta(View::Keyboard);
    let nodes = m.tab().a11y(TAB_ID);
    assert_eq!(nodes[0].1.role(), ph2d_a11y::Role::Document);
    for v in View::ALL {
        let (_, n) = nodes
            .iter()
            .find(|(id, _)| *id == v.id())
            .expect("o nó do segmento");
        assert_eq!(n.role(), ph2d_a11y::Role::Tab, "{v:?}");
        assert_eq!(n.label(), Some(ph2d_i18n::tr(v.label_key())));
        assert_eq!(
            n.is_selected(),
            (v == View::Keyboard).then_some(true),
            "{v:?}"
        );
    }
    let (x, y) = m.segment(View::List);
    assert_eq!(m.tab().pointer_look(TAB_ID, x, y), Some(PointerLook::Hand));
    assert!(!m.tab().wants_frame(), "em repouso, nenhum quadro pedido");
}

/// Um controlo desligado: o Down não captura, o Up não dispara, o cursor é a seta.
#[test]
fn um_controlo_desligado_nao_dispara_e_mostra_a_seta() {
    let mut m = Mesa::aberta(View::Keyboard);
    let r = m.tab().doc.segment_rect(View::List).expect("pintado");
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let off = Control::new(ids::VIEW_LIST, r, ControlRole::Segment, "List")
        .disabled(Some("input_map_tab.view.list_tip"));
    let doc = &mut m.tab_mut().doc;
    doc.control(off);
    assert!(doc.press(x, y), "dentro da aba");
    assert!(!doc.is_pressed(ids::VIEW_LIST), "desligado não captura");
    doc.release_at(x, y);
    assert_eq!(doc.view(), View::Keyboard, "nem dispara");
    assert_eq!(doc.look_at(x, y), Some(PointerLook::Arrow));
}

/// O relógio do substrato: o caret pisca pelo `UiMotion::elapsed` e fica fixo com reduced motion; o pulso é
/// `Role::Decoration` — ausente em Discreto e no reduced motion, presente no Expressivo (e pede quadro).
#[test]
fn caret_e_pulso_seguem_o_relogio_e_o_reduced_motion() {
    let mut m = Mesa::aberta(View::Keyboard);
    let id = ids::SEARCH;
    m.hero
        .motion
        .set_character(ph2d_editor_core::motion::UiCharacter::Discrete);
    m.frame();
    assert_eq!(m.tab_mut().doc.pulse(id), None, "Discreto: sem enfeite");
    m.hero
        .motion
        .set_character(ph2d_editor_core::motion::UiCharacter::Expressive);
    m.frame();
    assert!(
        m.tab_mut().doc.pulse(id).is_some(),
        "Expressivo: o pulso existe"
    );
    assert!(m.tab().doc.wants_frame(), "um pulso vivo pede quadro");
    let a = m.tab_mut().doc.caret_on(id);
    m.hero.motion.advance(0.6);
    m.frame();
    assert_ne!(
        m.tab_mut().doc.caret_on(id),
        a,
        "meio período depois, o caret trocou"
    );
    m.hero.motion.set_reduced_motion(true);
    m.frame();
    assert_eq!(m.tab_mut().doc.pulse(id), None, "reduced: sem pulso");
    assert!(m.tab_mut().doc.caret_on(id), "reduced: caret fixo, aceso");
}
