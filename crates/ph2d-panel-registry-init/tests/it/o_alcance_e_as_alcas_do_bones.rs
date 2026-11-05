//! ⭐ Bones, cartão Bone: Reach Strength e Curve handles são irmãos curtos, na mesma linha (lei da
//! compactação, `docs/CartoesUI/05_ESPECIFICACAO_UI_COMPLETA.md` §4), e cada um responde ao gesto.
//! O segmento é registado por dentro da pílula (`SEG_PAD`): compara-se o CENTRO, nunca o topo.
#![cfg(feature = "panel-skeleton")]

use super::o_cartao_solto_no_canvas::{
    arrasta, centro, clica, hero_com_o_bones, osso_em_foco, pinta,
};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::card_field::handle_rect;
use ph2d_panel_skeleton as skel;

/// Larga o que a thread publicou, mesmo quando o teste cai.
struct Limpa;
impl Drop for Limpa {
    fn drop(&mut self) {
        skel::state::set_current_bone(None);
        skel::state::set_current_bone_handles(None);
        skel::state::set_current_envelope_manda(true);
    }
}

fn osso_curvo() -> (HeroScreen, Limpa) {
    let mut h = hero_com_o_bones();
    let limpa = Limpa;
    osso_em_foco(7, "Bone", 0.5);
    let mut osso = ph2d_skeleton::bend::BoneSpec::straight(20.0, 1.0);
    osso.segments = 4;
    skel::state::set_current_bone(Some(osso));
    skel::state::set_current_bone_handles(Some(0));
    skel::state::set_current_envelope_manda(true);
    pinta(&mut h, 60);
    (h, limpa)
}

/// Na largura de fábrica os dois dividem a linha, nas colunas de Length e Segments (uma grelha por
/// cartão), e o texto dos segmentos cabe. *Mutação: «From Chain» de volta, 5 px maior que a coluna ⇒
/// a linha desce.*
#[test]
fn reach_and_handles_share_a_row_on_the_grid_of_the_card() {
    let (h, _l) = osso_curvo();
    let rect = |id| h.hit_index.rect_for(id).expect("pintado");
    let reach = rect(ids::VECTOR_BONE_STRENGTH);
    let manual = rect(ids::VECTOR_BONE_HANDLES_AUTHORED);
    let auto = rect(ids::VECTOR_BONE_HANDLES_AUTO);
    let (comp, seg) = (
        rect(ids::VECTOR_BONE_LENGTH),
        rect(ids::VECTOR_BONE_SEGMENTS),
    );
    for s in [manual, auto] {
        assert_eq!(
            centro(reach).1,
            centro(s).1,
            "Reach {reach:?}, segmento {s:?}"
        );
    }
    assert_eq!(
        (reach.x, reach.w),
        (comp.x, comp.w),
        "Reach fora da coluna de Length"
    );
    assert!(
        manual.x >= seg.x && auto.x + auto.w <= seg.x + seg.w,
        "a escolha saiu da coluna de Segments: {manual:?} {auto:?} {seg:?}"
    );
}

/// Clicar em cada segmento leva o clique ao barramento; a pega de Reach escreve o número.
#[test]
fn reach_and_handles_answer_the_gesture_on_their_shared_row() {
    let (mut h, _l) = osso_curvo();
    for id in [
        ids::VECTOR_BONE_HANDLES_AUTO,
        ids::VECTOR_BONE_HANDLES_AUTHORED,
    ] {
        let _ = h.bus.drain().count();
        clica(&mut h, id);
        assert!(
            h.bus.drain().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id
            )),
            "o clique em {id:?} não chegou ao barramento"
        );
    }
    let _ = h.bus.drain().count();
    let pega = centro(handle_rect(
        h.hit_index
            .rect_for(ids::VECTOR_BONE_STRENGTH)
            .expect("Reach pintado"),
    ));
    arrasta(&mut h, pega, (pega.0 + 40.0, pega.1));
    assert!(
        h.bus.drain().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(c, v))
                if c == ids::VECTOR_BONE_STRENGTH && v > 1.0
        )),
        "arrastar a pega de Reach não escreveu um número maior"
    );
}
