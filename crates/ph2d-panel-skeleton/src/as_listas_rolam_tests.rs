//! ⭐⭐ **As duas listas do osso (a ACÇÃO do osso inteligente e a PONTA da curva) ROLAM de ponta a
//! ponta** — a prova partilhada sobre as funções reais do passe diferido, numa banda de painéis
//! baixa onde as dezasseis linhas do pool não cabem.

use ph2d_editor_core::ids;
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Theme;
use ph2d_ui_testkit::{MockPanelHost, prova_que_a_lista_longa_rola};

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
/// A banda de painéis por onde a lista se clampa — baixa de propósito.
const BANDA: Rect = Rect {
    x: 1200.0,
    y: 100.0,
    w: 400.0,
    h: 200.0,
};
const CHIP: Rect = Rect {
    x: 1300.0,
    y: 150.0,
    w: 200.0,
    h: 24.0,
};

fn layout() -> HeroLayout {
    let mut l = HeroLayout::for_viewport(JANELA);
    l.inspector = BANDA;
    l
}

#[test]
fn a_lista_de_accoes_do_osso_inteligente_rola() {
    let n = ids::VECTOR_BONE_SMART_CLIP_IDS.len();
    crate::state::set_current_bone_actions((0..n).map(|i| i.to_string()).collect());
    let layout = layout();
    let mut host = MockPanelHost::new();
    prova_que_a_lista_longa_rola(
        &mut host,
        crate::ids::VECTOR_BONE_SMART_CLIP,
        &ids::VECTOR_BONE_SMART_CLIP_IDS,
        &mut |h| {
            h.paint_fn(&layout, |ctx| {
                crate::section_smart::paint_action_popover(ctx, CHIP, Theme::Forge);
            })
        },
    );
    crate::state::set_current_bone_actions(Vec::new());
}

#[test]
fn a_lista_da_ponta_rola() {
    let n = ids::VECTOR_BONE_TIP_IDS.len();
    crate::state::set_current_bone_tip(Some(crate::state::TipView {
        rotulos: (0..n).map(|i| i.to_string()).collect(),
        ligado: 0,
        escondidos: 0,
    }));
    let layout = layout();
    let mut host = MockPanelHost::new();
    prova_que_a_lista_longa_rola(
        &mut host,
        crate::ids::VECTOR_BONE_TIP,
        &ids::VECTOR_BONE_TIP_IDS,
        &mut |h| {
            h.paint_fn(&layout, |ctx| {
                crate::section_tip::paint_tip_popover(ctx, CHIP, Theme::Forge);
            })
        },
    );
    crate::state::set_current_bone_tip(None);
}
