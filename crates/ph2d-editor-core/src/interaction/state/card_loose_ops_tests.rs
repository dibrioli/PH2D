use super::*;
use crate::interaction::state::section_prefs_ops::{
    restore_section_prefs_text, section_prefs_text,
};

const PAINEL: NodeId = NodeId(7);
const CARTAO_IK: NodeId = NodeId(70);

/// ⭐ C5: o cartão solto grava-se com a ordem dos cartões e volta igual ao reabrir o app.
#[test]
fn a_loose_card_survives_reopening_the_app() {
    let mut s = WidgetStore::default();
    s.detach_card(CARTAO_IK, PAINEL, (412.5, 180.0), 270.0);
    let texto = section_prefs_text(&s);
    let mut de_novo = WidgetStore::default();
    restore_section_prefs_text(&mut de_novo, &texto);
    let c = de_novo.loose_card(CARTAO_IK).expect("o solto voltou");
    assert_eq!((c.panel, c.at, c.w), (PAINEL, (412.5, 180.0), 270.0));
    assert!(de_novo.return_loose_card(CARTAO_IK));
    assert!(!section_prefs_text(&de_novo).contains("loose."));
}

/// ⭐ C5: o rect do solto é o alvo do clique POR CIMA de tudo (o canvas não o leva), mas não
/// reserva coluna; e sai quando o painel dele deixa de pintar.
#[test]
fn a_loose_card_takes_the_click_but_reserves_no_column() {
    let mut s = WidgetStore::default();
    s.set_floating_panel_rect(PAINEL, Rect::new(1500.0, 40.0, 300.0, 600.0));
    s.detach_card(CARTAO_IK, PAINEL, (600.0, 150.0), 270.0);
    s.set_loose_card_rect(CARTAO_IK, Rect::new(600.0, 150.0, 270.0, 160.0));
    assert_eq!(s.panel_at(700.0, 200.0), Some(loose_rect_of(CARTAO_IK)));
    assert_eq!(s.docked_panel_rects().count(), 0);
    assert_eq!(s.floating_panel_rects().count(), 1);
    s.clear_panel_rect(PAINEL);
    assert_eq!(s.panel_at(700.0, 200.0), None);
}

/// ⭐ T54: com a cópia 1 presa, soltar a mesma secção cria a cópia 2 (outra instância); as duas
/// gravam-se e voltam ao reabrir, cada uma com a sua chave durável (T45).
/// *Mutação: `detach_card` chaveado pela secção ⇒ a cópia 1 é reescrita.*
#[test]
fn two_loose_copies_of_a_section_survive_reopening_with_their_own_pins() {
    let mut s = WidgetStore::default();
    s.set_card_subject(PAINEL, Some((11, "Circle".to_owned())));
    s.detach_card(CARTAO_IK, PAINEL, (400.0, 100.0), 270.0);
    s.toggle_loose_pin(CARTAO_IK);
    s.set_loose_pin_key(CARTAO_IK, (1001, "Circle".to_owned()));
    s.set_card_subject(PAINEL, Some((12, "Square".to_owned())));
    s.detach_card(CARTAO_IK, PAINEL, (700.0, 300.0), 280.0);
    let dois = loose_instance_of(CARTAO_IK, 1);
    assert_ne!(dois, CARTAO_IK);
    s.toggle_loose_pin(dois);
    s.toggle_loose_pin(dois);
    s.set_loose_pin_key(dois, (1002, "Square".to_owned()));
    assert_eq!(s.loose_card(CARTAO_IK).and_then(|c| c.pin), Some(11));
    assert_eq!(
        s.loose_card(dois).map(|c| (c.section, c.pin)),
        Some((CARTAO_IK, Some(12)))
    );
    let texto = section_prefs_text(&s);
    let mut de_novo = WidgetStore::default();
    restore_section_prefs_text(&mut de_novo, &texto);
    let um = de_novo.loose_card(CARTAO_IK).expect("a cópia 1 voltou");
    let c2 = de_novo.loose_card(dois).expect("a cópia 2 voltou");
    assert_eq!(
        (um.section, um.at, um.rope),
        (CARTAO_IK, (400.0, 100.0), false)
    );
    assert_eq!(
        (c2.section, c2.at, c2.w, c2.rope),
        (CARTAO_IK, (700.0, 300.0), 280.0, true)
    );
    assert_eq!(de_novo.loose_pin_key(CARTAO_IK), Some((1001, "Circle")));
    assert_eq!(de_novo.loose_pin_key(dois), Some((1002, "Square")));
}
