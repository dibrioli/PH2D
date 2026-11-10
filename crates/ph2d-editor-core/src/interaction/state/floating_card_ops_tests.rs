use super::*;

const P: NodeId = crate::ids::TIMELINE_PANEL;
const JANELA: Rect = Rect::new(0.0, 0.0, 1000.0, 800.0);
const ORIGEM: Rect = Rect::new(100.0, 500.0, 600.0, 240.0);
const MIN: (f32, f32) = (320.0, 120.0);

#[test]
fn na_origem_o_rect_e_o_do_layout() {
    let mut s = WidgetStore::default();
    assert_eq!(s.place_floating_card(P, ORIGEM, JANELA, MIN), ORIGEM);
    assert!(!s.floating_card_away(P));
}

#[test]
fn o_desvio_move_e_fica_preso_na_janela_e_grava_o_visivel() {
    let mut s = WidgetStore::default();
    s.set_floating_card_offsets(P, ((-50.0, -100.0), (0.0, 0.0)));
    let r = s.place_floating_card(P, ORIGEM, JANELA, MIN);
    assert_eq!((r.x, r.y), (50.0, 400.0));
    s.set_floating_card_offsets(P, ((5000.0, 5000.0), (0.0, 0.0)));
    let r = s.place_floating_card(P, ORIGEM, JANELA, MIN);
    assert_eq!(
        (r.x + r.w, r.y + r.h),
        (JANELA.w, JANELA.h),
        "preso no canto"
    );
    assert_eq!(
        s.floating_card_offsets(P).0,
        (r.x - ORIGEM.x, r.y - ORIGEM.y),
        "sem elástico"
    );
}

#[test]
fn a_borda_de_cima_e_a_da_esquerda_param_no_piso() {
    let mut s = WidgetStore::default();
    let base = s.floating_card_offsets(P);
    s.resize_floating_card(
        P,
        base,
        (ORIGEM.w, ORIGEM.h),
        TIMELINE_EDGE_T | TIMELINE_EDGE_L,
        (5000.0, 5000.0),
        MIN,
    );
    let r = s.place_floating_card(P, ORIGEM, JANELA, MIN);
    assert_eq!((r.w, r.h), MIN);
    assert_eq!(
        (r.x + r.w, r.y + r.h),
        (ORIGEM.x + ORIGEM.w, ORIGEM.y + ORIGEM.h),
        "o canto oposto não anda"
    );
}

#[test]
fn encaixar_apaga_o_desvio_e_o_lugar_grava_se_e_volta() {
    let mut s = WidgetStore::default();
    s.set_floating_card_offsets(P, ((-10.0, -20.0), (30.0, 40.0)));
    let texto = crate::interaction::section_prefs_text(&s);
    assert!(texto.contains("timeline_card=-10,-20,30,40"), "{texto}");
    let mut t = WidgetStore::default();
    crate::interaction::restore_section_prefs_text(&mut t, &texto);
    assert_eq!(t.floating_card_offsets(P), ((-10.0, -20.0), (30.0, 40.0)));
    s.dock_floating_card(P);
    assert!(!s.floating_card_away(P));
    assert!(s.take_section_prefs_dirty(), "encaixar é para gravar");
}
