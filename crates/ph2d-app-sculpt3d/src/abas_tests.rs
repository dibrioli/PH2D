//! Os gates das abas da peça — ver o [`super`].

use super::decide;
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::ids::{PAINTER_LAYERS_PANEL, SCULPT3D_PANEL};

fn topo(hero: &HeroScreen) -> Option<ph2d_editor_core::NodeId> {
    hero.store
        .panel_z_order()
        .iter()
        .rev()
        .find(|n| **n == PAINTER_LAYERS_PANEL || **n == SCULPT3D_PANEL)
        .copied()
}

fn hero() -> HeroScreen {
    HeroScreen::new(ph2d_editor_core::NodeId(1))
}

/// ⭐⭐ **GATE — com a peça no ecrã a aba do Painter existe fora da mão, e nasce ATRÁS.** Sem ela o
/// «vice-versa» do dono não tinha onde clicar — e SEM o IMG ligado (o modo é a porta, spec/06 F3).
/// ⚠️ O CONTROLO: sem a peça ela não é forçada.
#[test]
fn a_aba_do_painter_fica_atras_com_a_peca_no_ecra() {
    let mut h = hero();
    decide(&mut h, true, false, false);
    assert_eq!(h.panel_visibility.get("painter_layers"), Some(&true));
    assert_eq!(
        topo(&h),
        Some(SCULPT3D_PANEL),
        "a aba do Painter nasceu À FRENTE da escultura"
    );

    let mut sem_peca = hero();
    decide(&mut sem_peca, false, false, false);
    assert_ne!(
        sem_peca.panel_visibility.get("painter_layers"),
        Some(&true),
        "o CONTROLO: sem peça"
    );
}

/// ⭐⭐ **GATE — o Painter sai da mão ⇒ a aba da escultura vem à frente.** ⚠️ O CONTROLO: com ele
/// na mão a frente fica com quem lá estava.
#[test]
fn o_painter_sai_da_mao_e_a_escultura_vem_a_frente() {
    let mut h = hero();
    h.store.bump_panel_z(SCULPT3D_PANEL);
    h.store.bump_panel_z(PAINTER_LAYERS_PANEL);
    decide(&mut h, true, true, true);
    assert_eq!(
        topo(&h),
        Some(PAINTER_LAYERS_PANEL),
        "o CONTROLO: com o Painter na mão"
    );
    decide(&mut h, true, false, true);
    assert_eq!(
        topo(&h),
        Some(SCULPT3D_PANEL),
        "o Painter saiu e a aba dele ficou à frente"
    );
}
